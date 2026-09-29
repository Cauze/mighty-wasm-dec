// Recursion + mutual recursion: call graph with back edges.
__attribute__((noinline)) int fib(int n) {
    if (n < 2)
        return n;
    return fib(n - 1) + fib(n - 2);
}

__attribute__((noinline)) int is_even(int n);

__attribute__((noinline)) int is_odd(int n) {
    if (n == 0)
        return 0;
    return is_even(n - 1);
}

__attribute__((noinline)) int is_even(int n) {
    if (n == 0)
        return 1;
    return is_odd(n - 1);
}

int main(void) {
    return fib(10) + is_even(8);
}
