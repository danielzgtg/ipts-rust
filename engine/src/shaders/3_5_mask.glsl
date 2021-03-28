#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer I {
    uint i[10];
};

layout(set = 0, binding = 1) restrict buffer A {
    uint a[64][44];
};

layout(set = 0, binding = 2) readonly restrict buffer B {
    uint b[64][44];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    uint me = a[x][y];
    if (me == ~0u) return;

    me -= 1u;
    uint m_x = me & 63u;
    uint m_y = me >> 6u;

    uint result = uint(i[0u] == me);
    result |= uint(i[1u] == me) * 2u;
    result |= uint(i[2u] == me) * 3u;
    result |= uint(i[3u] == me) * 4u;
    result |= uint(i[4u] == me) * 5u;
    result |= uint(i[5u] == me) * 6u;
    result |= uint(i[6u] == me) * 7u;
    result |= uint(i[7u] == me) * 8u;
    result |= uint(i[8u] == me) * 9u;
    result |= uint(i[9u] == me) * 10u;
    result *= b[m_x][m_y];

    a[x][y] = result;
}
