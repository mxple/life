use std::mem;

use bevy::{
    asset::AssetMetaCheck,
    color::palettes::css::WHITE,
    core::FrameCount,
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{vec2, vec3, vec4},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, BlendState, Extent3d, RenderPipelineDescriptor, SamplerDescriptor,
            ShaderRef, SpecializedMeshPipelineError, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages,
        },
        texture::{ImageSampler, ImageSamplerDescriptor},
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle},
    window::{PresentMode, WindowResized},
};

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 900;

#[derive(Resource, Default)]
struct Cursor {
    pos: Vec2,
    size: f32,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(Cursor {
        size: 10.,
        ..default()
    });

    app.insert_resource(Msaa::Off);

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,
                    fit_canvas_to_parent: true,
                    canvas: Some("#mygame-canvas".into()),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        Material2dPlugin::<LifeMaterial>::default(),
        Material2dPlugin::<RenderMaterial>::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (draw_cursor, draw_life).chain());

    app.run();
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut life_materials: ResMut<Assets<LifeMaterial>>,
    mut render_materials: ResMut<Assets<RenderMaterial>>,
) {
    let size = Extent3d {
        width: WIDTH,
        height: HEIGHT,
        ..default()
    };

    let rect_handle = meshes.add(Rectangle::from_size((WIDTH as f32, HEIGHT as f32).into()));

    let image_handle = images.add(Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler: ImageSampler::nearest(),
        data: [0u8; (HEIGHT * WIDTH * 4) as usize].to_vec(),
        ..default()
    });

    let iter_pass_layer = RenderLayers::layer(1);

    // iterate world
    cmd.spawn((
        MaterialMesh2dBundle {
            mesh: rect_handle.clone().into(),
            material: life_materials.add(LifeMaterial {
                texture: image_handle.clone().into(),
                ..default()
            }),
            ..default()
        },
        iter_pass_layer.clone(),
    ));

    cmd.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::None,
                order: -1,
                target: image_handle.clone().into(),
                hdr: false,
                ..default()
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
        iter_pass_layer,
    ));

    // render world to screen
    cmd.spawn((MaterialMesh2dBundle {
        mesh: rect_handle.into(),
        material: render_materials.add(RenderMaterial {
            texture: image_handle,
        }),
        ..default()
    },));

    cmd.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        CursorTracker::default(),
    ));

    // render cursor
    cmd.spawn((Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    },));
}

#[derive(Component, Default)]
struct CursorTracker;

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform, &CursorTracker)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    mut cursor: ResMut<Cursor>,
) {
    let (camera, camera_transform, _) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    gizmos.circle_2d(point, 10.0, WHITE);
    cursor.pos = point;
}

fn draw_life(
    query: Query<&Handle<LifeMaterial>>,
    mut materials: ResMut<Assets<LifeMaterial>>,
    cursor: Res<Cursor>,
    frame: Res<FrameCount>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let handle = query.single();
    let Some(mat) = materials.get_mut(handle) else {
        println!("Material not found");
        return;
    };

    mat.info.x = cursor.pos.x;
    mat.info.y = cursor.pos.y;

    mat.info.x += (WIDTH / 2) as f32;
    mat.info.y *= -1.;
    mat.info.y += (HEIGHT / 2) as f32;

    mat.info.w = if mouse.pressed(MouseButton::Left) {
        cursor.size
    } else {
        0.
    };

    mat.info.z = unsafe { mem::transmute_copy::<u32, f32>(&frame.0) };

    let gradient = colorous::RAINBOW;
    let col = gradient.eval_rational((frame.0 % 1000) as usize, 1000);
    mat.draw_color = vec4(
        col.r as f32 / 255.,
        col.g as f32 / 255.,
        col.b as f32 / 255.,
        0.,
    );
}

fn update_viewport_size(
    query: Query<&Handle<LifeMaterial>>,
    mut materials: ResMut<Assets<LifeMaterial>>,
    mut resize_reader: EventReader<WindowResized>,
) {
    let Ok(handle) = query.get_single() else {
        return;
    };

    let Some(mat) = materials.get_mut(handle) else {
        println!("Material not found");
        return;
    };

    for e in resize_reader.read() {
        mat.viewport = vec2(e.width, e.height);
    }
}

/// Material to iterate life using fragment shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct LifeMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,

    #[uniform(2)]
    info: Vec4,

    #[uniform(3)]
    draw_color: Vec4,

    #[uniform(4)]
    viewport: Vec2,
}

impl Material2d for LifeMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/default.vert".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/life_material.frag".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();

        if let Some(fragment) = &mut descriptor.fragment {
            for target_state in &mut fragment.targets.iter_mut().flatten() {
                target_state.blend = Some(BlendState::REPLACE);
            }
        }
        Ok(())
    }
}

/// Material to render life onto the screen
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RenderMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl Material2d for RenderMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/default.vert".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/render_material.frag".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}
