#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) restrict buffer B {
    uint b[64][44];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    b[x][y] = uint(b[x][y] != 0u) * ((x | (y << 6u)) + 1u);
}
