#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly restrict buffer A {
    uint a[64][44];
};

layout(set = 0, binding = 1) readonly restrict buffer C {
    uint c[64][44];
};

layout(set = 0, binding = 2) restrict buffer T {
    uint t[64][20];
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    for (uint y = 0u; y < 44u; y++) {
        uint m = a[x][y];
        if (m != 0u) {
            m -= 1u;
            m <<= 1u;
            uint w = c[x][y];
            t[x][m] += (43u - y) * w;
            m += 1u;
            t[x][m] += w;
        }
    }
}
