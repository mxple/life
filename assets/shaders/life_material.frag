#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform texture2D tex;
layout(set = 2, binding = 1) uniform sampler samp;

layout(set = 2, binding = 2) uniform vec4 info; // for drawing

layout(set = 2, binding = 3) uniform vec4 color; // for drawing

// layout(set = 2, binding = 4) uniform vec2 viewport;

const float cutoff = 0.5;

const int k = 0xDECAF;

int hash() {
    int x = floatBitsToInt(gl_FragCoord.x) ^ floatBitsToInt(gl_FragCoord.y) * (floatBitsToInt(info.z) + k);

    x = ((x >> 8) ^ x) * k;
    x = ((x >> 8) ^ x) * k;

    return x;
}

vec4 v(float x, float y) {
    vec4 col = texture(sampler2D(tex, samp), vec2(x, y) / vec2(800., 600.));

    return (col.a >= cutoff) ? vec4(col.rgb, 1.) : vec4(0.);
}

vec4 neighborSum(float x, float y) {
    float t = mod(y + 1, 600.);
    float b = mod(y - 1, 600.);
    float l = mod(x - 1, 800.);
    float r = mod(x + 1, 800.);

    vec4 sum = v(l, t) + v(x, t) + v(r, t)
             + v(l, y) /*     */ + v(r, y)
             + v(l, b) + v(x, b) + v(r, b);

    return sum;
}

void main() {
    float x = gl_FragCoord.x;
    float y = gl_FragCoord.y;

    vec4 col = texture(sampler2D(tex, samp), vec2(x, y) / vec2(800., 600.));

    vec4 curr = v(x, y);
    bool alive = curr.a == 1.;

    vec4 rgb_sum = neighborSum(x, y);

    bool keep = alive && ( rgb_sum.a == 2. || rgb_sum.a == 7. );
    bool born = rgb_sum.a == 3.;

    if (born) {
        o_Target.rgb = rgb_sum.rgb / rgb_sum.a;
        o_Target.a = 1.;
    }
    else if (keep) {
        o_Target = col;
        o_Target.a = cutoff + 0.01;
    }
    else {
        o_Target.rgb = col.rgb;
        o_Target.a = clamp(col.a - 0.005, 0.01, cutoff - 0.01);
    }


    // draw
    if (distance(vec2(x, y), info.xy) < info.w) {
        if (hash() > 0.0) {
            o_Target.rgb = color.rgb;
            o_Target.a = 1.0;
        }
    }
    
    o_Target.rg = fract(gl_FragCoord.xy);
    o_Target.b = 0.;
    o_Target.a = 1.0;
}
