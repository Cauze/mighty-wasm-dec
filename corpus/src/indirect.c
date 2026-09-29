typedef int (*op_fn)(int);

__attribute__((noinline)) int op_add(int x) {
    return x + 1;
}

__attribute__((noinline)) int op_mul(int x) {
    return x * 2;
}

__attribute__((noinline)) int apply(op_fn f, int v) {
    return f(v);
}

int main(void) {
    op_fn table[2] = {op_add, op_mul};
    return apply(table[0], 21) + apply(table[1], 21);
}
