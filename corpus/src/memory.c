// Bulk memory: memcpy/memset/memmove lower to memory.init/fill/copy.
struct Blob {
    int tag;
    int vals[8];
};

__attribute__((noinline)) void fill_blob(struct Blob *b, int v) {
    __builtin_memset(b->vals, v, sizeof(b->vals));
    b->tag = v;
}

__attribute__((noinline)) void copy_blob(struct Blob *dst, struct Blob *src) {
    __builtin_memcpy(dst, src, sizeof(struct Blob));
}

int main(void) {
    struct Blob a, b;
    fill_blob(&a, 3);
    copy_blob(&b, &a);
    return b.tag + b.vals[7];
}
