__attribute__((noinline)) double poly(double x) {
    return 2.5 * x * x + 0.5 * x + 1.0;
}

int main(void) {
    double v = poly(3.0);
    return (int)v;
}
