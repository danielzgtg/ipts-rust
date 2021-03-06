#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer A {
    uint a[64][44];
};

layout(set = 0, binding = 1) writeonly restrict buffer B {
    uint b[64][44];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    b[x][y] = uint(a[x][y] == ((x | (y << 6u)) + 1u));
}
