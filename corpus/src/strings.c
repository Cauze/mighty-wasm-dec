static const char greeting[] = "auth-ok";

__attribute__((noinline)) int get_byte(int i) {
    return greeting[i];
}

int main(void) {
    int s = 0;
    for (int i = 0; i < 7; i++)
        s += get_byte(i);
    return s;
}
