// Structured control: nested loops, continue, early break,
// select (ternary) in both i32 and i64 flavors.
__attribute__((noinline)) int control(int n) {
    int total = 0;
    for (int i = 0; i < n; i++) {
        if (i % 2 == 0)
            continue;
        int inner = 0;
        for (int j = i; j > 0; j--) {
            inner += j;
            if (inner > 50)
                break;
        }
        total += inner > 25 ? inner : -inner;
    }
    return total;
}

int main(void) {
    return control(12);
}
