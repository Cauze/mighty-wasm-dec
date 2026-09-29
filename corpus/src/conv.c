// Conversions: trunc/extend/float convert, saturating float->int.
__attribute__((noinline)) int to_int(float f, double d) {
    int a = (int)f;
    int b = (int)d;
    long long c = (long long)(f * 2.0f);
    float back = (float)(a + b) + (float)c;
    return a + b + (int)back;
}

__attribute__((noinline)) double to_double(int x, long long y) {
    return (double)x * 1.5 + (double)(unsigned long long)y;
}

int main(void) {
    return to_int(3.7f, 9.25) + (int)to_double(-4, 8LL);
}
