#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer R {
    uint r[704];
};

layout(set = 0, binding = 1) writeonly restrict buffer A {
    uint a[64][44];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    a[x][y] = (r[(y << 4u) | ((x >> 2u) & 15u)] >> ((x & 3u) << 3u)) & 255u;
}
