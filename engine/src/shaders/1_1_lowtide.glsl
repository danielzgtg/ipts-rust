#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer B {
    uint b[64][44];
};

layout(set = 0, binding = 1) writeonly restrict buffer A {
    uint a[64][44];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    uint n = b[x][y];
    if (x != 0u) {
        n |= b[x - 1u][y];
    }
    if (x != 63u) {
        n |= b[x + 1u][y];
    }
    if (y != 0u) {
        n |= b[x][y - 1u];
    }
    if (y != 43u) {
        n |= b[x][y + 1u];
    }

    a[x][y] = n;
}
