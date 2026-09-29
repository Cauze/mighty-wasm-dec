__attribute__((noinline)) int add(int a, int b) {
    return a + b;
}

__attribute__((noinline)) int pick(int x) {
    if (x > 10)
        return add(x, 1);
    else
        return add(x, -1);
}

int main(void) {
    return pick(20) + pick(5);
}
