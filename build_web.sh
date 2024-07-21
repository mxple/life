cargo build --release --features bevy/shader_format_glsl --target wasm32-unknown-unknown  &&
wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "life" ./target/wasm32-unknown-unknown/release/life.wasm
