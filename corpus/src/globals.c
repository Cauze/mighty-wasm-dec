static int counter = 0;

__attribute__((noinline)) void tick(int n) {
    counter += n;
}

__attribute__((noinline)) int read_counter(void) {
    return counter;
}

int main(void) {
    tick(5);
    tick(7);
    return read_counter();
}
