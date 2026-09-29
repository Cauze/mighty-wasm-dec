// SIMD via wasm_simd128.h (built with -msimd128).
#include <wasm_simd128.h>

__attribute__((noinline)) v128_t vec_add4(v128_t a, v128_t b) {
    return wasm_i32x4_add(a, b);
}

__attribute__((noinline)) int vec_sum4(v128_t v) {
    return wasm_i32x4_extract_lane(v, 0) + wasm_i32x4_extract_lane(v, 1) +
           wasm_i32x4_extract_lane(v, 2) + wasm_i32x4_extract_lane(v, 3);
}

int main(void) {
    v128_t a = wasm_i32x4_make(1, 2, 3, 4);
    v128_t b = wasm_i32x4_splat(10);
    return vec_sum4(vec_add4(a, b));
}
