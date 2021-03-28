#version 450

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) readonly restrict buffer C {
    uint c[64][44];
};

layout(set = 0, binding = 1) uniform VP {
    vec2 viewport;
};

void main() {
    vec2 uv = vec2(1.0f, 1.0f) - (gl_FragCoord.xy / viewport);
    float color = float(c[uint((uv.x * 64.0f) + 0.0f)][uint((uv.y * 44.0f) + 0.0f)]) / 255.0f;
    f_color = vec4(color, color, 0.0f, 1.0f);
}
