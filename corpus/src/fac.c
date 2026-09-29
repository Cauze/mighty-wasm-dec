__attribute__((noinline)) int fac(int n) {
    int r = 1;
    for (int i = 2; i <= n; i++)
        r *= i;
    return r;
}

int main(void) {
    return fac(6);
}
