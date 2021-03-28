#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 10) in;

layout(set = 0, binding = 0) readonly restrict buffer T {
    uint t[64][20];
};

layout(set = 0, binding = 1) writeonly restrict buffer P {
    uint p[10];
};

void main() {
    uint z = gl_GlobalInvocationID.z << 1u;
    uint x_sum = 0u;
    uint y_sum = 0u;
    uint w_sum = 0u;
    for (uint x = 0u; x < 64u; x++) {
        uint y_w = t[x][z];
        uint w = t[x][z + 1u];
        x_sum += (63u - x) * w;
        y_sum += y_w;
        w_sum += w;
    }
    float w_float = float(w_sum);
    uint screen_x = uint((float(x_sum) * 43.42857142857143f  / w_float));
    uint screen_y = uint((float(y_sum) * 42.41860465116279f / w_sum));
    p[z >> 1u] = (screen_x << 16u) | screen_y;
}
