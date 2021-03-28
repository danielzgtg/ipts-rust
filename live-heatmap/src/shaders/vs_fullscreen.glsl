#version 450

void main() {
    float x = float((uint(gl_VertexIndex) & 2u) << 1u) - 1.0f;
    float y = float((uint(gl_VertexIndex) & 1u) << 2u) - 1.0f;
    gl_Position = vec4(x, y, 0.0f, 1.0f);
}
