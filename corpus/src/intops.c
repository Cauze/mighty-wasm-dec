// Integer ops: division, remainder, bit ops, clz/ctz/popcnt, rotates,
// sign extension. Exercises the full i32/i64 ALU incl. trapping div.
__attribute__((noinline)) int intops(int a, int b) {
    int q = b == 0 ? 0 : a / b;
    int r = b == 0 ? 0 : a % b;
    int e = (a ^ b) & (a | b);
    e += __builtin_clz((unsigned)a) + __builtin_ctz((unsigned)b);
    e += __builtin_popcount((unsigned)(a + b));
    e += (int)(((unsigned)a << 3) | ((unsigned)a >> 29));
    e += (signed char)a + (short)b;
    return q + r + e;
}

__attribute__((noinline)) long long llops(long long x, long long y) {
    long long q = y == 0 ? 0 : x / y;
    return q + (x << 40) + (x >> 7) + (x & 0xffffffffffffLL);
}

int main(void) {
    return intops(100, 7) + (int)llops(1000000LL, 3LL);
}
