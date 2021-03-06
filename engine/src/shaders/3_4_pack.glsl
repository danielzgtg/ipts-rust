#version 450

layout(local_size_x = 1, local_size_y = 44, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer B {
    uint b[64][44];
};

layout(set = 0, binding = 1) writeonly restrict buffer R {
    uint r[704];
};

void main() {
    uint x = (gl_GlobalInvocationID.x << 2u) + 3u;
    uint y = gl_GlobalInvocationID.y;
    uint result = uint(b[x][y] != 0u);
    result <<= 8u;
    x -= 1u;
    result |= uint(b[x][y] != 0u);
    result <<= 8u;
    x -= 1u;
    result |= uint(b[x][y] != 0);
    result <<= 8u;
    x -= 1u;
    result |= uint(b[x][y] != 0u);
    r[(y << 4u) | (x >> 2u)] = result;
}
