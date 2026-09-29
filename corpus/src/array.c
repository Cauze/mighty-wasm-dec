__attribute__((noinline)) int sum_array(int *a, int n) {
    int s = 0;
    for (int i = 0; i < n; i++)
        s += a[i];
    return s;
}

int main(void) {
    int vals[4] = {10, 20, 30, 40};
    return sum_array(vals, 4);
}
