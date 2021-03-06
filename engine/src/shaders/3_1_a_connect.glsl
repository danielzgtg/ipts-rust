#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer B {
    uint b[64][44];
};

layout(set = 0, binding = 1) writeonly restrict buffer A {
    uint a[64][44];
};

uint min_neighbor_b() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    uint m = 0xFFFFu;
    uint n = 0u;
    if (x != 0u) {
        n = b[x - 1u][y];
    }
    n |= uint(n == 0u) * 0xFFFFu;
    m = min(m, n);
    if (x != 63u) {
        n = b[x + 1u][y];
    }
    n |= uint(n == 0u) * 0xFFFFu;
    m = min(m, n);
    if (y != 0u) {
        n = b[x][y - 1u];
    }
    n |= uint(n == 0u) * 0xFFFFu;
    m = min(m, n);
    if (y != 43u) {
        n = b[x][y + 1u];
    }
    n |= uint(n == 0u) * 0xFFFFu;
    m = min(m, n);
    return m;
}

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    a[x][y] = min(b[x][y], min_neighbor_b());
}
