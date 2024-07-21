#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform texture2D tex;
layout(set = 2, binding = 1) uniform sampler samp;

// includle resolutionn

layout(set = 2, binding = 2) uniform vec4 info;



const int k = 0xDECAFE;

int hash() {
	int x = floatBitsToInt(gl_FragCoord.x) ^ floatBitsToInt(gl_FragCoord.y) * (floatBitsToInt(info.z) + k);

    x = ((x>>8)^x)*k;
    x = ((x>>8)^x)*k;

    return x;
}

int v(float xrel, float yrel) {
	vec2 xy;
	xy.x = mod(gl_FragCoord.x + xrel, 800.);
	xy.y = mod(gl_FragCoord.y + yrel, 600.);
	
	return texture(sampler2D(tex, samp), xy/vec2(800.,600.)).r > 0.5 ? 1 : 0;
}

int neighborSum() {
	float a=-1.,b=0.,c=1.;
	return v(a,a)+v(a,b)+v(a,c)+v(b,a)+v(b,c)+v(c,a)+v(c,b)+v(c,c);
}

void main() {
	bool alive = v(0.,0.) == 1.;
	int sum = neighborSum();
	bool b = sum == 3 || (alive && (sum == 2));
	float a = float(b);
	o_Target = vec4(a, a, a, 1.0);
	
	// draw
	if (distance(gl_FragCoord.xy, info.xy) < info.w) {
		if (hash() > 0.0) {
			o_Target.r = 1.0;
		}
	}
}
