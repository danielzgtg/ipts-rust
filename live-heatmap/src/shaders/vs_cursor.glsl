#version 450

layout(location = 0) in vec2 vertex_position;
layout(location = 1) in uint[3] instance_data;
layout(location = 0) out uint instance;

void main() {
    //
    //Some((i, ((*x as f32 + 0.5) / 1368.0) - 1.0, ((*y as f32 + 0.5) / 912.0) - 1.0)),
    float x = ((float(instance_data[0u]) + 0.5f) / 1368.0) - 1.0f;
    float y = ((float(instance_data[1u]) + 0.5f) / 912.0) - 1.0f;
    gl_Position = vec4(vertex_position.xy + vec2(x, y), 0.0f, 1.0f);
    instance = instance_data[2u];
}
