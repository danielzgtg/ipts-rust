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
    uint me = a[x][y];
    if (me == 0u) return;

    uint left = 0u;
    uint right = 0u;
    uint top = 0u;
    uint bottom = 0u;
    if (x != 0u) {
        left = a[x - 1u][y];
    }
    if (left == 0u) {
        left = me;
    }
    if (x != 63u) {
        right = a[x + 1u][y];
    }
    if (right == 0u) {
        right = me;
    }
    if (y != 0u) {
        top = a[x][y - 1];
    }
    if (top == 0u) {
        top = me;
    }
    if (y != 43u) {
        bottom = a[x][y + 1u];
    }
    if (bottom == 0u) {
        bottom = me;
    }
    if (me == left && me == right && me == top && me == bottom) return;

    me -= 1u;
    x = me & 63u;
    y = me >> 6u;
    b[x][y] = 0u;
}
