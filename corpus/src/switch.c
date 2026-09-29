__attribute__((noinline)) int grade(int score) {
    switch (score / 10) {
    case 10:
    case 9:
        return 'A';
    case 8:
        return 'B';
    case 7:
        return 'C';
    default:
        return 'F';
    }
}

int main(void) {
    return grade(85) + grade(42);
}
