// add.cu
extern "C" __global__ void sum(const float* x, const float* y, float* out, int count) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    // Each thread processes one or more elements (grid-stride loop):
    for (int i = idx; i < count; i += blockDim.x * gridDim.x) {
        out[i] = x[i] + y[i];
    }
}
