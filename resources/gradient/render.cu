
#include <stdint.h> 

extern "C" __global__ 
void gradient(uint16_t *fb, int max_x, int max_y)
{
    int i = threadIdx.x + blockIdx.x * blockDim.x;
    int j = threadIdx.y + blockIdx.y * blockDim.y;
    if ((i >= max_x) || (j >= max_y)) return;

    int idx = (j * max_x + i) * 3;

    fb[idx + 0] = (uint16_t)( (float(i) / max_x) * 65535.0f );
    fb[idx + 1] = (uint16_t)( (float(j) / max_y) * 65535.0f );
    fb[idx + 2] = (uint16_t)( 0.2 * 65535.0f );
}