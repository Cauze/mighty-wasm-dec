struct Point {
    int x;
    int y;
};

__attribute__((noinline)) int move_x(struct Point *p, int dx) {
    p->x += dx;
    p->y += 1;
    return p->x + p->y;
}

int main(void) {
    struct Point p;
    p.x = 3;
    p.y = 4;
    return move_x(&p, 10);
}
