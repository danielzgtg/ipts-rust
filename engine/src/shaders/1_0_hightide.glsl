#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer C {
    uint c[64][44];
};

layout(set = 0, binding = 1) writeonly restrict buffer B {
    uint b[64][44];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    uint n = uint(c[x][y] != 0u);
    if (x != 0u) {
        n &= uint(c[x - 1u][y] != 0u);
    }
    if (x != 63u) {
        n &= uint(c[x + 1u][y] != 0u);
    }
    if (y != 0u) {
        n &= uint(c[x][y - 1u] != 0u);
    }
    if (y != 43u) {
        n &= uint(c[x][y + 1u] != 0u);
    }

    b[x][y] = n;
}
