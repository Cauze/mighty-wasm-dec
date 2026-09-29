#include <stdint.h>
#include <stddef.h>
#include <math.h>
typedef unsigned __int128 __v128_u __attribute__((vector_size(16)));
typedef __v128_u __v128;
static uint8_t *mem = 0; /* wasm linear memory base (host provides) */
static uint32_t memory_size(uint32_t m) { (void)m; return 0; }
static uint32_t memgrow(uint32_t d) { (void)d; return (uint32_t)-1; }

/* imports (host-provided) */
void a_a(int32_t a0, int32_t a1, int32_t a2); /* a.a_a */
void a_b(); /* a.a_b */
int32_t a_c(int32_t a0); /* a.a_c */

/* globals */
static int32_t /*mut*/ g0 = 0;

int32_t f3(int32_t p0, int32_t p1, S_3_l2* p2);
void g(int32_t p0);
int32_t f5(int32_t p0);
int32_t f(S_6_l0* p0);
void f7(int32_t p0);
int32_t f8(int32_t p0);
void f9();
int32_t f10(int32_t p0);
int32_t f11(int32_t p0);
void f12();
void f13(int32_t p0);
int32_t f14(S_14_l0* p0);
void f15(S_15_l0* p0, int32_t p1, int32_t p2, int32_t p3);
void f16(S_16_l0* p0, int32_t p1, int32_t p2);
void f17(int32_t p0);
int32_t f18(int32_t p0);
int32_t f19(int32_t p0);
int32_t f20(int32_t p0);
int32_t f21(int32_t p0);
void f22(int32_t p0, int32_t p1, int32_t p2, int32_t p3, int32_t p4, int32_t p5);
void f23(S_23_l0* p0, int32_t p1, int32_t p2, int32_t p3, int32_t p4, int32_t p5);
int32_t h(S_24_l0* p0, S_24_l1* p1, int32_t p2);
void f25(int32_t p0, S_25_l1* p1, int32_t p2, int32_t p3, int32_t p4);
void f26(S_26_l0* p0, S_26_l1* p1, int32_t p2, int32_t p3, int32_t p4);
void f27(S_27_l0* p0, int32_t p1, int32_t p2, int32_t p3);
void f28(int32_t p0, int32_t p1, int32_t p2, int32_t p3);
int32_t f29(int32_t p0, int32_t p1, int32_t p2);
void e();

/* recovered from f3 via base l2 */
typedef struct S_3_l2 {
  int8_t f_off_0; /* +0 w1 x1 */
  int8_t f_off_1; /* +1 w1 x1 */
} S_3_l2;
/* S_3_l3 (f3:l3) aliases S_3_l2 */
typedef S_3_l2 S_3_l3;
/* recovered from g via base l1 */
typedef struct S_4_l1 {
  int32_t f_off_8; /* +8 w4 x4 */
  int32_t f_off_16; /* +16 w4 x4 */
  int32_t f_off_20; /* +20 w4 x4 */
  int32_t f_off_24; /* +24 w4 x2 */
} S_4_l1;
/* recovered from g via base l2 */
typedef struct S_4_l2 {
  int32_t f_off_0; /* +0 w4 x4 */
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_8; /* +8 w4 x4 */
  int32_t f_off_12; /* +12 w4 x4 */
  int32_t f_off_24; /* +24 w4 x4 */
} S_4_l2;
/* recovered from g via base l3 */
typedef struct S_4_l3 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x5 */
  int32_t f_off_8; /* +8 w4 x3 */
  int32_t f_off_12; /* +12 w4 x3 */
  int64_t f_off_16; /* +16 w8 x3 */
  int32_t f_off_20; /* +20 w4 x2 */
  int32_t f_off_24; /* +24 w4 x1 */
  int32_t f_off_28; /* +28 w4 x2 */
} S_4_l3;
/* recovered from g via base l4 */
typedef struct S_4_l4 {
  int32_t f_off_0; /* +0 w4 x2 */
  int32_t f_off_12; /* +12 w4 x1 */
} S_4_l4;
/* S_4_l5 (f4:l5) aliases S_4_l3 */
typedef S_4_l3 S_4_l5;
/* recovered from g via base l6 */
typedef struct S_4_l6 {
  int32_t f_off_0; /* +0 w4 x2 */
  int32_t f_off_16; /* +16 w4 x2 */
} S_4_l6;
/* recovered from g via base l7 */
typedef struct S_4_l7 {
  int32_t f_off_16; /* +16 w4 x2 */
  int32_t f_off_20; /* +20 w4 x1 */
} S_4_l7;
/* S_4_l8 (f4:l8) aliases S_4_l7 */
typedef S_4_l7 S_4_l8;
/* recovered from f via base l0 */
typedef struct S_6_l0 {
  int32_t f_off_0; /* +0 w4 x9 */
  int32_t f_off_4; /* +4 w4 x20 */
  int32_t f_off_8; /* +8 w4 x18 */
  int32_t f_off_12; /* +12 w4 x5 */
  int32_t f_off_16; /* +16 w4 x5 */
  int32_t f_off_20; /* +20 w4 x6 */
  int32_t f_off_24; /* +24 w4 x4 */
} S_6_l0;
/* S_6_l1 (f6:l1) aliases S_6_l0 */
typedef S_6_l0 S_6_l1;
/* S_6_l2 (f6:l2) aliases S_4_l3 */
typedef S_4_l3 S_6_l2;
/* recovered from f via base l3 */
typedef struct S_6_l3 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x4 */
  int32_t f_off_8; /* +8 w4 x4 */
  int32_t f_off_12; /* +12 w4 x4 */
  int64_t f_off_16; /* +16 w8 x2 */
  int32_t f_off_24; /* +24 w4 x3 */
  int32_t f_off_28; /* +28 w4 x2 */
} S_6_l3;
/* S_6_l4 (f6:l4) aliases S_4_l3 */
typedef S_4_l3 S_6_l4;
/* S_6_l5 (f6:l5) aliases S_4_l3 */
typedef S_4_l3 S_6_l5;
/* S_6_l6 (f6:l6) aliases S_4_l7 */
typedef S_4_l7 S_6_l6;
/* recovered from f via base l7 */
typedef struct S_6_l7 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_16; /* +16 w4 x2 */
} S_6_l7;
/* recovered from f via base l8 */
typedef struct S_6_l8 {
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_16; /* +16 w4 x2 */
  int32_t f_off_20; /* +20 w4 x1 */
} S_6_l8;
/* S_6_l9 (f6:l9) aliases S_4_l7 */
typedef S_4_l7 S_6_l9;
/* recovered from f9 via base l0 */
typedef struct S_9_l0 {
  int32_t f_off_0; /* +0 w4 x3 */
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_8; /* +8 w4 x1 */
} S_9_l0;
/* recovered from f9 via base l3 */
typedef struct S_9_l3 {
  int32_t f_off_0; /* +0 w4 x2 */
  int32_t f_off_4; /* +4 w4 x1 */
} S_9_l3;
/* S_14_l0 (f14:l0) aliases S_9_l3 */
typedef S_9_l3 S_14_l0;
/* recovered from f15 via base l0 */
typedef struct S_15_l0 {
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_16; /* +16 w4 x2 */
  int32_t f_off_24; /* +24 w4 x3 */
  int32_t f_off_36; /* +36 w4 x3 */
  int32_t f_off_48; /* +48 w4 x2 */
  int8_t f_off_52; /* +52 w1 x1 */
  int8_t f_off_53; /* +53 w1 x1 */
  int8_t f_off_54; /* +54 w1 x1 */
} S_15_l0;
/* recovered from f16 via base l0 */
typedef struct S_16_l0 {
  int32_t f_off_16; /* +16 w4 x2 */
  int32_t f_off_20; /* +20 w4 x2 */
  int32_t f_off_24; /* +24 w4 x4 */
  int32_t f_off_36; /* +36 w4 x3 */
  int8_t f_off_54; /* +54 w1 x1 */
  int32_t f_off_56; /* +56 w4 x2 */
} S_16_l0;
/* recovered from f23 via base l0 */
typedef struct S_23_l0 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_8; /* +8 w4 x1 */
} S_23_l0;
/* recovered from h via base l0 */
typedef struct S_24_l0 {
  int32_t f_off_0; /* +0 w4 x6 */
  int16_t f_off_6; /* +6 w2 x2 */
  int32_t f_off_12; /* +12 w4 x1 */
  int32_t f_off_16; /* +16 w4 x1 */
  int32_t f_off_20; /* +20 w4 x3 */
  int8_t f_off_36; /* +36 w1 x1 */
  int32_t f_off_60; /* +60 w4 x1 */
} S_24_l0;
/* recovered from h via base l1 */
typedef struct S_24_l1 {
  int32_t f_off_12; /* +12 w4 x1 */
  int32_t f_off_16; /* +16 w4 x1 */
  int32_t f_off_20; /* +20 w4 x1 */
} S_24_l1;
/* recovered from h via base l10 */
typedef struct S_24_l10 {
  int16_t f_off_0; /* +0 w2 x10 */
} S_24_l10;
/* recovered from h via base l11 */
typedef struct S_24_l11 {
  int32_t f_off_4; /* +4 w4 x2 */
  int32_t f_off_8; /* +8 w4 x2 */
  int32_t f_off_12; /* +12 w4 x3 */
  int32_t f_off_16; /* +16 w4 x3 */
  int32_t f_off_20; /* +20 w4 x2 */
  int32_t f_off_24; /* +24 w4 x4 */
  int64_t f_off_28; /* +28 w8 x6 */
  int32_t f_off_32; /* +32 w4 x3 */
  int32_t f_off_36; /* +36 w4 x3 */
  int64_t f_off_40; /* +40 w8 x4 */
  int32_t f_off_44; /* +44 w4 x2 */
  int32_t f_off_48; /* +48 w4 x4 */
  int64_t f_off_52; /* +52 w8 x4 */
  int32_t f_off_56; /* +56 w4 x3 */
  int32_t f_off_60; /* +60 w4 x3 */
} S_24_l11;
/* S_24_l12 (f24:l12) aliases S_24_l10 */
typedef S_24_l10 S_24_l12;
/* S_24_l21 (f24:l21) aliases S_24_l10 */
typedef S_24_l10 S_24_l21;
/* S_24_l24 (f24:l24) aliases S_24_l10 */
typedef S_24_l10 S_24_l24;
/* recovered from h via base l3 */
typedef struct S_24_l3 {
  int32_t f_off_0; /* +0 w4 x20 */
  int32_t f_off_4; /* +4 w4 x1 */
  int16_t f_off_6; /* +6 w2 x1 */
  int64_t f_off_8; /* +8 w8 x2 */
  int32_t f_off_12; /* +12 w4 x1 */
  int64_t f_off_16; /* +16 w8 x3 */
  int32_t f_off_20; /* +20 w4 x2 */
  int64_t f_off_48; /* +48 w8 x1 */
  int32_t f_off_52; /* +52 w4 x1 */
  int64_t f_off_56; /* +56 w8 x2 */
  int32_t f_off_60; /* +60 w4 x3 */
  int32_t f_off_80; /* +80 w4 x1 */
  int64_t f_off_88; /* +88 w8 x1 */
  int64_t f_off_96; /* +96 w8 x1 */
  int64_t f_off_128; /* +128 w8 x1 */
  int64_t f_off_136; /* +136 w8 x1 */
} S_24_l3;
/* S_24_l5 (f24:l5) aliases S_24_l10 */
typedef S_24_l10 S_24_l5;
/* recovered from h via base l8 */
typedef struct S_24_l8 {
  int16_t f_off_0; /* +0 w2 x7 */
  int16_t f_off_2; /* +2 w2 x7 */
} S_24_l8;
/* recovered from f25 via base l1 */
typedef struct S_25_l1 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_8; /* +8 w4 x1 */
  int32_t f_off_16; /* +16 w4 x1 */
  int32_t f_off_20; /* +20 w4 x2 */
  int32_t f_off_24; /* +24 w4 x1 */
  int32_t f_off_28; /* +28 w4 x2 */
  int32_t f_off_32; /* +32 w4 x2 */
  int32_t f_off_36; /* +36 w4 x1 */
  int32_t f_off_40; /* +40 w4 x2 */
  int32_t f_off_44; /* +44 w4 x1 */
  int8_t f_off_54; /* +54 w1 x1 */
} S_25_l1;
/* S_26_l0 (f26:l0) aliases S_23_l0 */
typedef S_23_l0 S_26_l0;
/* recovered from f26 via base l1 */
typedef struct S_26_l1 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_8; /* +8 w4 x1 */
  int32_t f_off_16; /* +16 w4 x1 */
  int32_t f_off_20; /* +20 w4 x2 */
  int32_t f_off_24; /* +24 w4 x1 */
  int32_t f_off_28; /* +28 w4 x2 */
  int32_t f_off_32; /* +32 w4 x2 */
  int32_t f_off_36; /* +36 w4 x1 */
  int32_t f_off_40; /* +40 w4 x2 */
  int32_t f_off_44; /* +44 w4 x3 */
  int16_t f_off_52; /* +52 w2 x2 */
  int8_t f_off_53; /* +53 w1 x1 */
  int8_t f_off_54; /* +54 w1 x1 */
} S_26_l1;
/* S_27_l0 (f27:l0) aliases S_23_l0 */
typedef S_23_l0 S_27_l0;
/* recovered from f29 via base l3 */
typedef struct S_29_l3 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x2 */
  int32_t f_off_8; /* +8 w4 x2 */
  int32_t f_off_12; /* +12 w4 x2 */
  int32_t f_off_16; /* +16 w4 x2 */
  int64_t f_off_20; /* +20 w8 x2 */
  int32_t f_off_24; /* +24 w4 x1 */
  int64_t f_off_28; /* +28 w8 x3 */
  int32_t f_off_32; /* +32 w4 x2 */
  int64_t f_off_36; /* +36 w8 x3 */
  int32_t f_off_40; /* +40 w4 x1 */
  int64_t f_off_44; /* +44 w8 x3 */
  int64_t f_off_52; /* +52 w8 x1 */
  int8_t f_off_59; /* +59 w1 x1 */
  int32_t f_off_60; /* +60 w4 x2 */
} S_29_l3;
/* recovered from f29 via base l4 */
typedef struct S_29_l4 {
  int32_t f_off_0; /* +0 w4 x1 */
  int32_t f_off_4; /* +4 w4 x1 */
  int32_t f_off_8; /* +8 w4 x1 */
  int32_t f_off_20; /* +20 w4 x1 */
  int32_t f_off_28; /* +28 w4 x1 */
  int32_t f_off_32; /* +32 w4 x1 */
  int32_t f_off_36; /* +36 w4 x1 */
  int32_t f_off_44; /* +44 w4 x1 */
  int32_t f_off_68; /* +68 w4 x1 */
  int8_t f_off_75; /* +75 w1 x1 */
} S_29_l4;
/* recovered from f29 via base l6 */
typedef struct S_29_l6 {
  int32_t f_off_4; /* +4 w4 x2 */
  int32_t f_off_8; /* +8 w4 x2 */
  int32_t f_off_12; /* +12 w4 x2 */
} S_29_l6;

int32_t f3(int32_t p0, int32_t p1, S_3_l2* p2) {
  int32_t l0 = p0;
  int32_t l1 = p1;
  S_3_l2* l2 = p2;
  int32_t l3 = 0;
  /* hint: l2 is S_3_l2* (2 fields) */
  /* hint: l3 is S_3_l3* (2 fields) */
  if ((!l2)) {
    return (*(int32_t*)(mem + (l0) + 4) == *(int32_t*)(mem + (l1) + 4));
  }
  if ((l0 == l1)) {
    return 1;
  }
  l2 = *(int32_t*)(mem + (l1) + 4);
  l1 = *(int8_t*)(mem + (l2));
  do { /* block B3 */
    l3 = *(int32_t*)(mem + (l0) + 4);
    l0 = *(int8_t*)(mem + (l3));
    if ((!l0)) goto __end_B3;
    if ((l0 != l1)) goto __end_B3;
    __head_L4: while (1) { /* loop L4 */
      l1 = *(int8_t*)(mem + (l2) + 1);
      l0 = *(int8_t*)(mem + (l3) + 1);
      if ((!l0)) goto __end_B3;
      l2 = (l2 + 1);
      l3 = (l3 + 1);
      if ((l0 == l1)) goto __head_L4;
    }
    __end_L4: ;
  } while (0);
  __end_B3: ;
  return (l0 == l1);
}

void g(int32_t p0) {
  int32_t l0 = p0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  int32_t l3 = 0;
  int32_t l4 = 0;
  int32_t l5 = 0;
  int32_t l6 = 0;
  int32_t l7 = 0;
  int32_t l8 = 0;
  int32_t t0 = 0; /* synthetic (type TODO) */
  int32_t t1 = 0; /* synthetic (type TODO) */
  /* hint: l1 is S_4_l1* (4 fields) */
  /* hint: l2 is S_4_l2* (5 fields) */
  /* hint: l3 is S_4_l3* (8 fields) */
  /* hint: l4 is S_4_l4* (2 fields) */
  /* hint: l5 is S_4_l5* (8 fields) */
  /* hint: l6 is S_4_l6* (2 fields) */
  /* hint: l7 is S_4_l7* (2 fields) */
  /* hint: l8 is S_4_l8* (2 fields) */
  do { /* block B1 */
    if ((!l0)) goto __end_B1;
    l3 = (l0 - 8);
    l2 = *(int32_t*)(mem + ((l0 - 4)));
    l0 = (l2 & -8);
    l5 = (l3 + l0);
    do { /* block B2 */
      if ((l2 & 1)) goto __end_B2;
      if ((!(l2 & 2))) goto __end_B1;
      l4 = *(int32_t*)(mem + (l3));
      l3 = (l3 - l4);
      if ((l3 < *(int32_t*)(mem + 1720))) goto __end_B1;
      l0 = (l0 + l4);
      do { /* block B3 */
        do { /* block B4 */
          do { /* block B5 */
            if ((*(int32_t*)(mem + 1724) != l3)) {
              l1 = *(int32_t*)(mem + (l3) + 12);
              if ((l4 <= 255)) {
                l2 = *(int32_t*)(mem + (l3) + 8);
                if ((l1 != l2)) goto __end_B5;
                *(int32_t*)(mem + 1704) = (*(int32_t*)(mem + 1704) & i32.rotl(-2, (l4 >> 3)));
                goto __end_B2; /* break */
              }
              l7 = *(int32_t*)(mem + (l3) + 24);
              if ((l1 != l3)) {
                l2 = *(int32_t*)(mem + (l3) + 8);
                ((S_4_l2*)l2)->f_off_12 = l1;
                ((S_4_l1*)l1)->f_off_8 = l2;
                goto __end_B3; /* break */
              }
              l2 = *(int32_t*)(mem + (l3) + 20);
              if (l2) {
                t0 = (l3 + 20);
              } else {
                l2 = *(int32_t*)(mem + (l3) + 16);
                if ((!l2)) goto __end_B4;
                t0 = (l3 + 16);
              }
              l4 = t0;
              __head_L10: while (1) { /* loop L10 */
                l6 = l4;
                l1 = l2;
                l4 = (l1 + 20);
                l2 = *(int32_t*)(mem + (l1) + 20);
                if (l2) goto __head_L10;
                l4 = (l1 + 16);
                l2 = *(int32_t*)(mem + (l1) + 16);
                if (l2) goto __head_L10;
              }
              __end_L10: ;
              ((S_4_l6*)l6)->f_off_0 = 0;
              goto __end_B3; /* break */
            }
            l2 = *(int32_t*)(mem + (l5) + 4);
            if (((l2 & 3) != 3)) goto __end_B2;
            *(int32_t*)(mem + 1712) = l0;
            ((S_4_l5*)l5)->f_off_4 = (l2 & -2);
            ((S_4_l3*)l3)->f_off_4 = (l0 | 1);
            ((S_4_l5*)l5)->f_off_0 = l0;
            return;
          } while (0);
          __end_B5: ;
          ((S_4_l2*)l2)->f_off_12 = l1;
          ((S_4_l1*)l1)->f_off_8 = l2;
          goto __end_B2; /* break */
        } while (0);
        __end_B4: ;
        l1 = 0;
      } while (0);
      __end_B3: ;
      if ((!l7)) goto __end_B2;
      do { /* block B11 */
        l4 = *(int32_t*)(mem + (l3) + 28);
        l2 = ((l4 << 2) + 2008);
        if ((*(int32_t*)(mem + (l2)) == l3)) {
          ((S_4_l2*)l2)->f_off_0 = l1;
          if (l1) goto __end_B11;
          *(int32_t*)(mem + 1708) = (*(int32_t*)(mem + 1708) & i32.rotl(-2, l4));
          goto __end_B2; /* break */
        }
        do { /* block B13 */
          if ((l3 == *(int32_t*)(mem + (l7) + 16))) {
            ((S_4_l7*)l7)->f_off_16 = l1;
            goto __end_B13; /* break */
          }
          ((S_4_l7*)l7)->f_off_20 = l1;
        } while (0);
        __end_B13: ;
        if ((!l1)) goto __end_B2;
      } while (0);
      __end_B11: ;
      ((S_4_l1*)l1)->f_off_24 = l7;
      l2 = *(int32_t*)(mem + (l3) + 16);
      if (l2) {
        ((S_4_l1*)l1)->f_off_16 = l2;
        ((S_4_l2*)l2)->f_off_24 = l1;
      }
      l2 = *(int32_t*)(mem + (l3) + 20);
      if ((!l2)) goto __end_B2;
      ((S_4_l1*)l1)->f_off_20 = l2;
      ((S_4_l2*)l2)->f_off_24 = l1;
    } while (0);
    __end_B2: ;
    if ((l3 >= l5)) goto __end_B1;
    l4 = *(int32_t*)(mem + (l5) + 4);
    if ((!(l4 & 1))) goto __end_B1;
    do { /* block B16 */
      do { /* block B17 */
        do { /* block B18 */
          do { /* block B19 */
            if ((!(l4 & 2))) {
              if ((*(int32_t*)(mem + 1728) == l5)) {
                *(int32_t*)(mem + 1728) = l3;
                l0 = (*(int32_t*)(mem + 1716) + l0);
                *(int32_t*)(mem + 1716) = l0;
                ((S_4_l3*)l3)->f_off_4 = (l0 | 1);
                if ((l3 != *(int32_t*)(mem + 1724))) goto __end_B1;
                *(int32_t*)(mem + 1712) = 0;
                *(int32_t*)(mem + 1724) = 0;
                return;
              }
              l7 = *(int32_t*)(mem + 1724);
              if ((l7 == l5)) {
                *(int32_t*)(mem + 1724) = l3;
                l0 = (*(int32_t*)(mem + 1712) + l0);
                *(int32_t*)(mem + 1712) = l0;
                ((S_4_l3*)l3)->f_off_4 = (l0 | 1);
                *(int32_t*)(mem + ((l0 + l3))) = l0;
                return;
              }
              l0 = ((l4 & -8) + l0);
              l1 = *(int32_t*)(mem + (l5) + 12);
              if ((l4 <= 255)) {
                l2 = *(int32_t*)(mem + (l5) + 8);
                if ((l2 == l1)) {
                  *(int32_t*)(mem + 1704) = (*(int32_t*)(mem + 1704) & i32.rotl(-2, (l4 >> 3)));
                  goto __end_B17; /* break */
                }
                ((S_4_l2*)l2)->f_off_12 = l1;
                ((S_4_l1*)l1)->f_off_8 = l2;
                goto __end_B17; /* break */
              }
              l8 = *(int32_t*)(mem + (l5) + 24);
              if ((l1 != l5)) {
                l2 = *(int32_t*)(mem + (l5) + 8);
                ((S_4_l2*)l2)->f_off_12 = l1;
                ((S_4_l1*)l1)->f_off_8 = l2;
                goto __end_B18; /* break */
              }
              l2 = *(int32_t*)(mem + (l5) + 20);
              if (l2) {
                t1 = (l5 + 20);
              } else {
                l2 = *(int32_t*)(mem + (l5) + 16);
                if ((!l2)) goto __end_B19;
                t1 = (l5 + 16);
              }
              l4 = t1;
              __head_L27: while (1) { /* loop L27 */
                l6 = l4;
                l1 = l2;
                l4 = (l1 + 20);
                l2 = *(int32_t*)(mem + (l1) + 20);
                if (l2) goto __head_L27;
                l4 = (l1 + 16);
                l2 = *(int32_t*)(mem + (l1) + 16);
                if (l2) goto __head_L27;
              }
              __end_L27: ;
              ((S_4_l6*)l6)->f_off_0 = 0;
              goto __end_B18; /* break */
            }
            ((S_4_l5*)l5)->f_off_4 = (l4 & -2);
            ((S_4_l3*)l3)->f_off_4 = (l0 | 1);
            *(int32_t*)(mem + ((l0 + l3))) = l0;
            goto __end_B16; /* break */
          } while (0);
          __end_B19: ;
          l1 = 0;
        } while (0);
        __end_B18: ;
        if ((!l8)) goto __end_B17;
        do { /* block B28 */
          l4 = *(int32_t*)(mem + (l5) + 28);
          l2 = ((l4 << 2) + 2008);
          if ((*(int32_t*)(mem + (l2)) == l5)) {
            ((S_4_l2*)l2)->f_off_0 = l1;
            if (l1) goto __end_B28;
            *(int32_t*)(mem + 1708) = (*(int32_t*)(mem + 1708) & i32.rotl(-2, l4));
            goto __end_B17; /* break */
          }
          do { /* block B30 */
            if ((l5 == *(int32_t*)(mem + (l8) + 16))) {
              ((S_4_l8*)l8)->f_off_16 = l1;
              goto __end_B30; /* break */
            }
            ((S_4_l8*)l8)->f_off_20 = l1;
          } while (0);
          __end_B30: ;
          if ((!l1)) goto __end_B17;
        } while (0);
        __end_B28: ;
        ((S_4_l1*)l1)->f_off_24 = l8;
        l2 = *(int32_t*)(mem + (l5) + 16);
        if (l2) {
          ((S_4_l1*)l1)->f_off_16 = l2;
          ((S_4_l2*)l2)->f_off_24 = l1;
        }
        l2 = *(int32_t*)(mem + (l5) + 20);
        if ((!l2)) goto __end_B17;
        ((S_4_l1*)l1)->f_off_20 = l2;
        ((S_4_l2*)l2)->f_off_24 = l1;
      } while (0);
      __end_B17: ;
      ((S_4_l3*)l3)->f_off_4 = (l0 | 1);
      *(int32_t*)(mem + ((l0 + l3))) = l0;
      if ((l3 != l7)) goto __end_B16;
      *(int32_t*)(mem + 1712) = l0;
      return;
    } while (0);
    __end_B16: ;
    if ((l0 <= 255)) {
      l2 = ((l0 & -8) + 1744);
      do { /* block B34 */
        l4 = *(int32_t*)(mem + 1704);
        l0 = (1 << (l0 >> 3));
        if ((!(l4 & l0))) {
          *(int32_t*)(mem + 1704) = (l0 | l4);
          goto __end_B34; /* break */
        }
      } while (0);
      __end_B34: ;
      l0 = *(int32_t*)(mem + (l2) + 8);
      ((S_4_l2*)l2)->f_off_8 = l3;
      *(int32_t*)(mem + (l0) + 12) = l3;
      ((S_4_l3*)l3)->f_off_12 = l2;
      ((S_4_l3*)l3)->f_off_8 = l0;
      return;
    }
    l1 = 31;
    if ((l0 <= 16777215)) {
      l2 = i32.clz((l0 >> 8));
      l1 = ((((l0 >> (38 - l2)) & 1) - (l2 << 1)) + 62);
    }
    ((S_4_l3*)l3)->f_off_28 = l1;
    /* split i64 init 0x0 into 2xi32 LE */
    ((S_4_l3*)l3)->f_off_16 = 0;
    ((S_4_l3*)l3)->f_off_20 = 0;
    l4 = ((l1 << 2) + 2008);
    do { /* block B37 */
      do { /* block B38 */
        do { /* block B39 */
          l6 = *(int32_t*)(mem + 1708);
          l2 = (1 << l1);
          if ((!(l6 & l2))) {
            *(int32_t*)(mem + 1708) = (l2 | l6);
            ((S_4_l4*)l4)->f_off_0 = l3;
            l1 = 24;
            goto __end_B39; /* break */
          }
          l1 = (l0 << ((l1 != 31) ? (25 - (l1 >> 1)) : 0));
          l4 = *(int32_t*)(mem + (l4));
          __head_L41: while (1) { /* loop L41 */
            l2 = l4;
            if (((*(int32_t*)(mem + (l2) + 4) & -8) == l0)) goto __end_B38;
            l4 = (l1 >> 29);
            l1 = (l1 << 1);
            l6 = (l2 + (l4 & 4));
            l4 = *(int32_t*)(mem + (l6) + 16);
            if (l4) goto __head_L41;
          }
          __end_L41: ;
          ((S_4_l6*)l6)->f_off_16 = l3;
          l1 = 24;
          l4 = l2;
        } while (0);
        __end_B39: ;
        l0 = 8;
        l2 = l3;
        goto __end_B37; /* break */
      } while (0);
      __end_B38: ;
      l4 = *(int32_t*)(mem + (l2) + 8);
      ((S_4_l4*)l4)->f_off_12 = l3;
      ((S_4_l2*)l2)->f_off_8 = l3;
      l0 = 24;
      l1 = 8;
    } while (0);
    __end_B37: ;
    l6 = 0;
    *(int32_t*)(mem + ((l1 + l3))) = l4;
    ((S_4_l3*)l3)->f_off_12 = l2;
    *(int32_t*)(mem + ((l0 + l3))) = l6;
    l0 = (*(int32_t*)(mem + 1736) - 1);
    *(int32_t*)(mem + 1736) = (l0 ? l0 : -1);
  } while (0);
  __end_B1: ;
}

int32_t f5(int32_t p0) {
  int32_t l0 = p0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  l1 = *(int32_t*)(mem + 1696); /* data1+0 */
  l2 = ((l0 + 7) & -8);
  l0 = (l1 + l2);
  do { /* block B1 */
    if ((!((l0 <= l1) ? l2 : 0))) {
      if ((l0 <= (memory_size(0) << 16))) goto __end_B1;
      if (a_c(l0)) goto __end_B1;
    }
    *(int32_t*)(mem + 1700) = 48;
    return -1;
  } while (0);
  __end_B1: ;
  *(int32_t*)(mem + 1696) = l0; /* data1+0 */
  return l1;
}

int32_t f(S_6_l0* p0) {
  S_6_l0* l0 = p0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  int32_t l3 = 0;
  int32_t l4 = 0;
  int32_t l5 = 0;
  int32_t l6 = 0;
  int32_t l7 = 0;
  int32_t l8 = 0;
  int32_t l9 = 0;
  int32_t l10 = 0;
  int32_t l11 = 0;
  int32_t t0 = 0; /* synthetic (type TODO) */
  int32_t t1 = 0; /* synthetic (type TODO) */
  int32_t t2 = 0; /* synthetic (type TODO) */
  int32_t t9 = 0; /* synthetic (type TODO) */
  /* hint: l0 is S_6_l0* (7 fields) */
  /* hint: l1 is S_6_l1* (7 fields) */
  /* hint: l2 is S_6_l2* (8 fields) */
  /* hint: l3 is S_6_l3* (7 fields) */
  /* hint: l4 is S_6_l4* (8 fields) */
  /* hint: l5 is S_6_l5* (8 fields) */
  /* hint: l6 is S_6_l6* (2 fields) */
  /* hint: l7 is S_6_l7* (3 fields) */
  /* hint: l8 is S_6_l8* (3 fields) */
  /* hint: l9 is S_6_l9* (2 fields) */
  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */
  l10 = (g0 - 16);
  g0 = l10;
  do { /* block B1 */
    do { /* block B2 */
      do { /* block B3 */
        do { /* block B4 */
          do { /* block B5 */
            do { /* block B6 */
              do { /* block B7 */
                do { /* block B8 */
                  do { /* block B9 */
                    do { /* block B10 */
                      if ((l0 <= 244)) {
                        l4 = *(int32_t*)(mem + 1704);
                        l6 = ((l0 < 11) ? 16 : ((l0 + 11) & 504));
                        l0 = (l6 >> 3);
                        l1 = (l4 >> l0);
                        if ((l1 & 3)) {
                          do { /* block B13 */
                            l2 = (((l1 ^ -1) & 1) + l0);
                            l1 = (l2 << 3);
                            l0 = (l1 + 1744);
                            l1 = *(int32_t*)(mem + ((l1 + 1752)));
                            l5 = *(int32_t*)(mem + (l1) + 8);
                            if ((l0 == l5)) {
                              *(int32_t*)(mem + 1704) = (l4 & i32.rotl(-2, l2));
                              goto __end_B13; /* break */
                            }
                            ((S_6_l5*)l5)->f_off_12 = l0;
                            ((S_6_l0*)l0)->f_off_8 = l5;
                          } while (0);
                          __end_B13: ;
                          l0 = (l1 + 8);
                          l2 = (l2 << 3);
                          ((S_6_l1*)l1)->f_off_4 = (l2 | 3);
                          l1 = (l1 + l2);
                          ((S_6_l1*)l1)->f_off_4 = (*(int32_t*)(mem + (l1) + 4) | 1);
                          goto __end_B1; /* break */
                        }
                        l8 = *(int32_t*)(mem + 1712);
                        if ((l6 <= l8)) goto __end_B10;
                        if (l1) {
                          do { /* block B16 */
                            l2 = (2 << l0);
                            l1 = i32.ctz(((l2 | (0 - l2)) & (l1 << l0)));
                            l0 = (l1 << 3);
                            l2 = (l0 + 1744);
                            l0 = *(int32_t*)(mem + ((l0 + 1752)));
                            l5 = *(int32_t*)(mem + (l0) + 8);
                            if ((l2 == l5)) {
                              l4 = (l4 & i32.rotl(-2, l1));
                              *(int32_t*)(mem + 1704) = l4;
                              goto __end_B16; /* break */
                            }
                            ((S_6_l5*)l5)->f_off_12 = l2;
                            ((S_6_l2*)l2)->f_off_8 = l5;
                          } while (0);
                          __end_B16: ;
                          ((S_6_l0*)l0)->f_off_4 = (l6 | 3);
                          l7 = (l0 + l6);
                          l1 = (l1 << 3);
                          l5 = (l1 - l6);
                          ((S_6_l7*)l7)->f_off_4 = (l5 | 1);
                          *(int32_t*)(mem + ((l0 + l1))) = l5;
                          if (l8) {
                            l1 = ((l8 & -8) + 1744);
                            l2 = *(int32_t*)(mem + 1724);
                            do { /* block B19 */
                              l3 = (1 << (l8 >> 3));
                              if ((!(l4 & l3))) {
                                *(int32_t*)(mem + 1704) = (l3 | l4);
                                goto __end_B19; /* break */
                              }
                            } while (0);
                            __end_B19: ;
                            l3 = *(int32_t*)(mem + (l1) + 8);
                            ((S_6_l1*)l1)->f_off_8 = l2;
                            ((S_6_l3*)l3)->f_off_12 = l2;
                            ((S_6_l2*)l2)->f_off_12 = l1;
                            ((S_6_l2*)l2)->f_off_8 = l3;
                          }
                          l0 = (l0 + 8);
                          *(int32_t*)(mem + 1724) = l7;
                          *(int32_t*)(mem + 1712) = l5;
                          goto __end_B1; /* break */
                        }
                        l11 = *(int32_t*)(mem + 1708);
                        if ((!l11)) goto __end_B10;
                        l2 = *(int32_t*)(mem + (((i32.ctz(l11) << 2) + 2008)));
                        l3 = ((*(int32_t*)(mem + (l2) + 4) & -8) - l6);
                        l1 = l2;
                        __head_L21: while (1) { /* loop L21 */
                          do { /* block B22 */
                            l0 = *(int32_t*)(mem + (l1) + 16);
                            if ((!l0)) {
                              l0 = *(int32_t*)(mem + (l1) + 20);
                              if ((!l0)) goto __end_B22;
                            }
                            l1 = ((*(int32_t*)(mem + (l0) + 4) & -8) - l6);
                            l1 = (l1 < l3);
                            l3 = (l1 ? l1 : l3);
                            l2 = (l1 ? l0 : l2);
                            l1 = l0;
                            goto __head_L21; /* continue */
                          } while (0);
                          __end_B22: ;
                        }
                        __end_L21: ;
                        l9 = *(int32_t*)(mem + (l2) + 24);
                        l0 = *(int32_t*)(mem + (l2) + 12);
                        if ((l2 != l0)) {
                          l1 = *(int32_t*)(mem + (l2) + 8);
                          ((S_6_l1*)l1)->f_off_12 = l0;
                          ((S_6_l0*)l0)->f_off_8 = l1;
                          goto __end_B2; /* break */
                        }
                        l1 = *(int32_t*)(mem + (l2) + 20);
                        if (l1) {
                          t0 = (l2 + 20);
                        } else {
                          l1 = *(int32_t*)(mem + (l2) + 16);
                          if ((!l1)) goto __end_B9;
                          t0 = (l2 + 16);
                        }
                        l5 = t0;
                        __head_L26: while (1) { /* loop L26 */
                          l7 = l5;
                          l0 = l1;
                          l5 = (l0 + 20);
                          l1 = *(int32_t*)(mem + (l0) + 20);
                          if (l1) goto __head_L26;
                          l5 = (l0 + 16);
                          l1 = *(int32_t*)(mem + (l0) + 16);
                          if (l1) goto __head_L26;
                        }
                        __end_L26: ;
                        ((S_6_l7*)l7)->f_off_0 = 0;
                        goto __end_B2; /* break */
                      }
                      l6 = -1;
                      if ((l0 > -65)) goto __end_B10;
                      l1 = (l0 + 11);
                      l6 = (l1 & -8);
                      l7 = *(int32_t*)(mem + 1708);
                      if ((!l7)) goto __end_B10;
                      l8 = 31;
                      l3 = (0 - l6);
                      if ((l0 <= 16777204)) {
                        l0 = i32.clz((l1 >> 8));
                        l8 = ((((l6 >> (38 - l0)) & 1) - (l0 << 1)) + 62);
                      }
                      do { /* block B28 */
                        do { /* block B29 */
                          do { /* block B30 */
                            l1 = *(int32_t*)(mem + (((l8 << 2) + 2008)));
                            if ((!l1)) {
                              l0 = 0;
                              goto __end_B30; /* break */
                            }
                            l0 = 0;
                            l2 = (l6 << ((l8 != 31) ? (25 - (l8 >> 1)) : 0));
                            __head_L32: while (1) { /* loop L32 */
                              do { /* block B33 */
                                l4 = ((*(int32_t*)(mem + (l1) + 4) & -8) - l6);
                                if ((l4 >= l3)) goto __end_B33;
                                l5 = l1;
                                l3 = l4;
                                if (l3) goto __end_B33;
                                l3 = 0;
                                l0 = l1;
                                goto __end_B29; /* break */
                              } while (0);
                              __end_B33: ;
                              l4 = *(int32_t*)(mem + (l1) + 20);
                              l1 = *(int32_t*)(mem + ((l1 + ((l2 >> 29) & 4))) + 16);
                              l0 = (l4 ? ((l4 == l1) ? l0 : l4) : l0);
                              l2 = (l2 << 1);
                              if (l1) goto __head_L32;
                            }
                            __end_L32: ;
                          } while (0);
                          __end_B30: ;
                          if ((!(l0 | l5))) {
                            l5 = 0;
                            l0 = (2 << l8);
                            l0 = ((l0 | (0 - l0)) & l7);
                            if ((!l0)) goto __end_B10;
                            l0 = *(int32_t*)(mem + (((i32.ctz(l0) << 2) + 2008)));
                          }
                          if ((!l0)) goto __end_B28;
                        } while (0);
                        __end_B29: ;
                        __head_L35: while (1) { /* loop L35 */
                          l2 = ((*(int32_t*)(mem + (l0) + 4) & -8) - l6);
                          l1 = (l2 < l3);
                          l3 = (l1 ? l2 : l3);
                          l5 = (l1 ? l0 : l5);
                          l1 = *(int32_t*)(mem + (l0) + 16);
                          if (l1) {
                            t1 = l1;
                          } else {
                            t1 = *(int32_t*)(mem + (l0) + 20);
                          }
                          l0 = t1;
                          if (l0) goto __head_L35;
                        }
                        __end_L35: ;
                      } while (0);
                      __end_B28: ;
                      if ((!l5)) goto __end_B10;
                      if ((l3 >= (*(int32_t*)(mem + 1712) - l6))) goto __end_B10;
                      l8 = *(int32_t*)(mem + (l5) + 24);
                      l0 = *(int32_t*)(mem + (l5) + 12);
                      if ((l5 != l0)) {
                        l1 = *(int32_t*)(mem + (l5) + 8);
                        ((S_6_l1*)l1)->f_off_12 = l0;
                        ((S_6_l0*)l0)->f_off_8 = l1;
                        goto __end_B3; /* break */
                      }
                      l1 = *(int32_t*)(mem + (l5) + 20);
                      if (l1) {
                        t2 = (l5 + 20);
                      } else {
                        l1 = *(int32_t*)(mem + (l5) + 16);
                        if ((!l1)) goto __end_B8;
                        t2 = (l5 + 16);
                      }
                      l2 = t2;
                      __head_L39: while (1) { /* loop L39 */
                        l4 = l2;
                        l0 = l1;
                        l2 = (l0 + 20);
                        l1 = *(int32_t*)(mem + (l0) + 20);
                        if (l1) goto __head_L39;
                        l2 = (l0 + 16);
                        l1 = *(int32_t*)(mem + (l0) + 16);
                        if (l1) goto __head_L39;
                      }
                      __end_L39: ;
                      ((S_6_l4*)l4)->f_off_0 = 0;
                      goto __end_B3; /* break */
                    } while (0);
                    __end_B10: ;
                    l5 = *(int32_t*)(mem + 1712);
                    if ((l6 <= l5)) {
                      l0 = *(int32_t*)(mem + 1724);
                      do { /* block B41 */
                        l1 = (l5 - l6);
                        if ((l1 >= 16)) {
                          l2 = (l0 + l6);
                          ((S_6_l2*)l2)->f_off_4 = (l1 | 1);
                          *(int32_t*)(mem + ((l0 + l5))) = l1;
                          ((S_6_l0*)l0)->f_off_4 = (l6 | 3);
                          goto __end_B41; /* break */
                        }
                        ((S_6_l0*)l0)->f_off_4 = (l5 | 3);
                        l1 = (l0 + l5);
                        ((S_6_l1*)l1)->f_off_4 = (*(int32_t*)(mem + (l1) + 4) | 1);
                        l2 = 0;
                        l1 = 0;
                      } while (0);
                      __end_B41: ;
                      *(int32_t*)(mem + 1712) = l1;
                      *(int32_t*)(mem + 1724) = l2;
                      l0 = (l0 + 8);
                      goto __end_B1; /* break */
                    }
                    l2 = *(int32_t*)(mem + 1716);
                    if ((l6 < l2)) {
                      l1 = (l2 - l6);
                      *(int32_t*)(mem + 1716) = l1;
                      l0 = *(int32_t*)(mem + 1728);
                      l2 = (l0 + l6);
                      *(int32_t*)(mem + 1728) = l2;
                      ((S_6_l2*)l2)->f_off_4 = (l1 | 1);
                      ((S_6_l0*)l0)->f_off_4 = (l6 | 3);
                      l0 = (l0 + 8);
                      goto __end_B1; /* break */
                    }
                    l0 = 0;
                    l3 = (l6 + 47);
                    do { /* block B44 */
                      if (*(int32_t*)(mem + 2176)) {
                        goto __end_B44; /* break */
                      }
                      *(int64_t*)(mem + 2188) = -1ll;
                      *(int64_t*)(mem + 2180) = 17592186048512ll;
                      *(int32_t*)(mem + 2176) = (((l10 + 12) & -16) ^ 1431655768);
                      *(int32_t*)(mem + 2196) = 0;
                      *(int32_t*)(mem + 2148) = 0;
                    } while (0);
                    __end_B44: ;
                    l1 = 4096;
                    l4 = (l3 + l1);
                    l7 = (0 - l1);
                    l1 = (l4 & l7);
                    if ((l1 <= l6)) goto __end_B1;
                    l5 = *(int32_t*)(mem + 2144);
                    if (l5) {
                      l8 = *(int32_t*)(mem + 2136);
                      l9 = (l8 + l1);
                      if ((l9 <= l8)) goto __end_B1;
                      if ((l5 < l9)) goto __end_B1;
                    }
                    do { /* block B47 */
                      if ((!(*(int8_t*)(mem + 2148) & 4))) {
                        do { /* block B49 */
                          do { /* block B50 */
                            do { /* block B51 */
                              do { /* block B52 */
                                l5 = *(int32_t*)(mem + 1728);
                                if (l5) {
                                  l0 = 2152;
                                  __head_L54: while (1) { /* loop L54 */
                                    l8 = *(int32_t*)(mem + (l0));
                                    if ((l8 <= l5)) {
                                      if ((l5 < (l8 + *(int32_t*)(mem + (l0) + 4)))) goto __end_B52;
                                    }
                                    l0 = *(int32_t*)(mem + (l0) + 8);
                                    if (l0) goto __head_L54;
                                  }
                                  __end_L54: ;
                                }
                                l2 = f5(0);
                                if ((l2 == -1)) goto __end_B49;
                                l4 = l1;
                                l0 = *(int32_t*)(mem + 2180);
                                l5 = (l0 - 1);
                                if ((l5 & l2)) {
                                  l4 = ((l1 - l2) + ((l2 + l5) & (0 - l0)));
                                }
                                if ((l4 <= l6)) goto __end_B49;
                                l0 = *(int32_t*)(mem + 2144);
                                if (l0) {
                                  l5 = *(int32_t*)(mem + 2136);
                                  l7 = (l5 + l4);
                                  if ((l7 <= l5)) goto __end_B49;
                                  if ((l0 < l7)) goto __end_B49;
                                }
                                l0 = f5(l4);
                                if ((l0 != l2)) goto __end_B51;
                                goto __end_B47; /* break */
                              } while (0);
                              __end_B52: ;
                              l4 = ((l4 - l2) & l7);
                              l2 = f5(l4);
                              if ((l2 == (*(int32_t*)(mem + (l0)) + *(int32_t*)(mem + (l0) + 4)))) goto __end_B50;
                              l0 = l2;
                            } while (0);
                            __end_B51: ;
                            if ((l0 == -1)) goto __end_B49;
                            if (((l6 + 48) <= l4)) {
                              l2 = l0;
                              goto __end_B47; /* break */
                            }
                            l2 = *(int32_t*)(mem + 2184);
                            l2 = ((l2 + (l3 - l4)) & (0 - l2));
                            if ((f5(l2) == -1)) goto __end_B49;
                            l4 = (l2 + l4);
                            l2 = l0;
                            goto __end_B47; /* break */
                          } while (0);
                          __end_B50: ;
                          if ((l2 != -1)) goto __end_B47;
                        } while (0);
                        __end_B49: ;
                        *(int32_t*)(mem + 2148) = (*(int32_t*)(mem + 2148) | 4);
                      }
                      l2 = f5(l1);
                      l0 = f5(0);
                      if ((l2 == -1)) goto __end_B5;
                      if ((l0 == -1)) goto __end_B5;
                      if ((l0 <= l2)) goto __end_B5;
                      l4 = (l0 - l2);
                      if ((l4 <= (l6 + 40))) goto __end_B5;
                    } while (0);
                    __end_B47: ;
                    l0 = (*(int32_t*)(mem + 2136) + l4);
                    *(int32_t*)(mem + 2136) = l0;
                    if ((*(int32_t*)(mem + 2140) < l0)) {
                      *(int32_t*)(mem + 2140) = l0;
                    }
                    do { /* block B60 */
                      l3 = *(int32_t*)(mem + 1728);
                      if (l3) {
                        l0 = 2152;
                        __head_L62: while (1) { /* loop L62 */
                          l1 = *(int32_t*)(mem + (l0));
                          l5 = *(int32_t*)(mem + (l0) + 4);
                          if ((l2 == (l1 + l5))) goto __end_B60;
                          l0 = *(int32_t*)(mem + (l0) + 8);
                          if (l0) goto __head_L62;
                        }
                        __end_L62: ;
                        goto __end_B7; /* break */
                      }
                      l0 = *(int32_t*)(mem + 1720);
                      if ((!((l0 <= l2) ? l0 : 0))) {
                        *(int32_t*)(mem + 1720) = l2;
                      }
                      l0 = 0;
                      *(int32_t*)(mem + 2156) = l4;
                      *(int32_t*)(mem + 2152) = l2;
                      *(int32_t*)(mem + 1736) = -1;
                      *(int32_t*)(mem + 1740) = *(int32_t*)(mem + 2176);
                      *(int32_t*)(mem + 2164) = 0;
                      __head_L64: while (1) { /* loop L64 */
                        l1 = (l0 << 3);
                        l5 = (l1 + 1744);
                        *(int32_t*)(mem + ((l1 + 1752))) = l5;
                        *(int32_t*)(mem + ((l1 + 1756))) = l5;
                        l0 = (l0 + 1);
                        if ((l0 != 32)) goto __head_L64;
                      }
                      __end_L64: ;
                      l0 = (l4 - 40);
                      l1 = ((-8 - l2) & 7);
                      l5 = (l0 - l1);
                      *(int32_t*)(mem + 1716) = l5;
                      l1 = (l1 + l2);
                      *(int32_t*)(mem + 1728) = l1;
                      ((S_6_l1*)l1)->f_off_4 = (l5 | 1);
                      *(int32_t*)(mem + ((l0 + l2)) + 4) = 40;
                      *(int32_t*)(mem + 1732) = *(int32_t*)(mem + 2192);
                      goto __end_B6; /* break */
                    } while (0);
                    __end_B60: ;
                    if ((l2 <= l3)) goto __end_B7;
                    if ((l1 > l3)) goto __end_B7;
                    if ((*(int32_t*)(mem + (l0) + 12) & 8)) goto __end_B7;
                    ((S_6_l0*)l0)->f_off_4 = (l4 + l5);
                    l0 = ((-8 - l3) & 7);
                    l1 = (l3 + l0);
                    *(int32_t*)(mem + 1728) = l1;
                    l2 = (*(int32_t*)(mem + 1716) + l4);
                    l0 = (l2 - l0);
                    *(int32_t*)(mem + 1716) = l0;
                    ((S_6_l1*)l1)->f_off_4 = (l0 | 1);
                    *(int32_t*)(mem + ((l2 + l3)) + 4) = 40;
                    *(int32_t*)(mem + 1732) = *(int32_t*)(mem + 2192);
                    goto __end_B6; /* break */
                  } while (0);
                  __end_B9: ;
                  l0 = 0;
                  goto __end_B2; /* break */
                } while (0);
                __end_B8: ;
                l0 = 0;
                goto __end_B3; /* break */
              } while (0);
              __end_B7: ;
              if ((*(int32_t*)(mem + 1720) > l2)) {
                *(int32_t*)(mem + 1720) = l2;
              }
              l5 = (l2 + l4);
              l0 = 2152;
              do { /* block B66 */
                __head_L67: while (1) { /* loop L67 */
                  l1 = *(int32_t*)(mem + (l0));
                  if ((l5 != l1)) {
                    l0 = *(int32_t*)(mem + (l0) + 8);
                    if (l0) goto __head_L67;
                    goto __end_B66; /* break */
                  }
                }
                __end_L67: ;
                if ((!(*(int8_t*)(mem + (l0) + 12) & 8))) goto __end_B4;
              } while (0);
              __end_B66: ;
              l0 = 2152;
              __head_L69: while (1) { /* loop L69 */
                do { /* block B70 */
                  l1 = *(int32_t*)(mem + (l0));
                  if ((l1 <= l3)) {
                    l5 = (l1 + *(int32_t*)(mem + (l0) + 4));
                    if ((l3 < l5)) goto __end_B70;
                  }
                  l0 = *(int32_t*)(mem + (l0) + 8);
                  goto __head_L69; /* continue */
                } while (0);
                __end_B70: ;
              }
              __end_L69: ;
              l0 = (l4 - 40);
              l1 = ((-8 - l2) & 7);
              l7 = (l0 - l1);
              *(int32_t*)(mem + 1716) = l7;
              l1 = (l1 + l2);
              *(int32_t*)(mem + 1728) = l1;
              ((S_6_l1*)l1)->f_off_4 = (l7 | 1);
              *(int32_t*)(mem + ((l0 + l2)) + 4) = 40;
              *(int32_t*)(mem + 1732) = *(int32_t*)(mem + 2192);
              l0 = ((l5 + ((39 - l5) & 7)) - 47);
              l1 = ((l0 < (l3 + 16)) ? l3 : l0);
              ((S_6_l1*)l1)->f_off_4 = 27;
              ((S_6_l1*)l1)->f_off_16 = *(int64_t*)(mem + 2160);
              ((S_6_l1*)l1)->f_off_8 = *(int64_t*)(mem + 2152);
              *(int32_t*)(mem + 2160) = (l1 + 8);
              *(int32_t*)(mem + 2156) = l4;
              *(int32_t*)(mem + 2152) = l2;
              *(int32_t*)(mem + 2164) = 0;
              l0 = (l1 + 24);
              __head_L72: while (1) { /* loop L72 */
                ((S_6_l0*)l0)->f_off_4 = 7;
                l0 = (l0 + 4);
                if (((l0 + 8) < l5)) goto __head_L72;
              }
              __end_L72: ;
              if ((l1 == l3)) goto __end_B6;
              ((S_6_l1*)l1)->f_off_4 = (*(int32_t*)(mem + (l1) + 4) & -2);
              l2 = (l1 - l3);
              ((S_6_l3*)l3)->f_off_4 = (l2 | 1);
              ((S_6_l1*)l1)->f_off_0 = l2;
              do { /* block B73 */
                if ((l2 <= 255)) {
                  l0 = ((l2 & -8) + 1744);
                  do { /* block B75 */
                    l1 = *(int32_t*)(mem + 1704);
                    l2 = (1 << (l2 >> 3));
                    if ((!(l1 & l2))) {
                      *(int32_t*)(mem + 1704) = (l1 | l2);
                      goto __end_B75; /* break */
                    }
                  } while (0);
                  __end_B75: ;
                  l1 = *(int32_t*)(mem + (l0) + 8);
                  ((S_6_l0*)l0)->f_off_8 = l3;
                  ((S_6_l1*)l1)->f_off_12 = l3;
                  l2 = 12;
                  goto __end_B73; /* break */
                }
                l0 = 31;
                if ((l2 <= 16777215)) {
                  l0 = i32.clz((l2 >> 8));
                  l0 = ((((l2 >> (38 - l0)) & 1) - (l0 << 1)) + 62);
                }
                ((S_6_l3*)l3)->f_off_28 = l0;
                ((S_6_l3*)l3)->f_off_16 = 0ll;
                l1 = ((l0 << 2) + 2008);
                do { /* block B78 */
                  do { /* block B79 */
                    l5 = *(int32_t*)(mem + 1708);
                    l4 = (1 << l0);
                    if ((!(l5 & l4))) {
                      *(int32_t*)(mem + 1708) = (l4 | l5);
                      ((S_6_l1*)l1)->f_off_0 = l3;
                      goto __end_B79; /* break */
                    }
                    l0 = (l2 << ((l0 != 31) ? (25 - (l0 >> 1)) : 0));
                    l5 = *(int32_t*)(mem + (l1));
                    __head_L81: while (1) { /* loop L81 */
                      l1 = l5;
                      if (((*(int32_t*)(mem + (l1) + 4) & -8) == l2)) goto __end_B78;
                      l5 = (l0 >> 29);
                      l0 = (l0 << 1);
                      l4 = (l1 + (l5 & 4));
                      l5 = *(int32_t*)(mem + (l4) + 16);
                      if (l5) goto __head_L81;
                    }
                    __end_L81: ;
                    ((S_6_l4*)l4)->f_off_16 = l3;
                  } while (0);
                  __end_B79: ;
                  ((S_6_l3*)l3)->f_off_24 = l1;
                  l2 = 8;
                  l1 = l3;
                  l0 = l1;
                  goto __end_B73; /* break */
                } while (0);
                __end_B78: ;
                l0 = *(int32_t*)(mem + (l1) + 8);
                ((S_6_l0*)l0)->f_off_12 = l3;
                ((S_6_l1*)l1)->f_off_8 = l3;
                ((S_6_l3*)l3)->f_off_8 = l0;
                l0 = 0;
                l2 = 24;
              } while (0);
              __end_B73: ;
              *(int32_t*)(mem + ((12 + l3))) = l1;
              *(int32_t*)(mem + ((l2 + l3))) = l0;
            } while (0);
            __end_B6: ;
            l0 = *(int32_t*)(mem + 1716);
            if ((l0 <= l6)) goto __end_B5;
            l1 = (l0 - l6);
            *(int32_t*)(mem + 1716) = l1;
            l0 = *(int32_t*)(mem + 1728);
            l2 = (l0 + l6);
            *(int32_t*)(mem + 1728) = l2;
            ((S_6_l2*)l2)->f_off_4 = (l1 | 1);
            ((S_6_l0*)l0)->f_off_4 = (l6 | 3);
            l0 = (l0 + 8);
            goto __end_B1; /* break */
          } while (0);
          __end_B5: ;
          *(int32_t*)(mem + 1700) = 48;
          l0 = 0;
          goto __end_B1; /* break */
        } while (0);
        __end_B4: ;
        ((S_6_l0*)l0)->f_off_0 = l2;
        ((S_6_l0*)l0)->f_off_4 = (*(int32_t*)(mem + (l0) + 4) + l4);
        l8 = (l2 + ((-8 - l2) & 7));
        ((S_6_l8*)l8)->f_off_4 = (l6 | 3);
        l4 = (l1 + ((-8 - l1) & 7));
        l3 = (l6 + l8);
        l7 = (l4 - l3);
        do { /* block B82 */
          if ((*(int32_t*)(mem + 1728) == l4)) {
            *(int32_t*)(mem + 1728) = l3;
            l0 = (*(int32_t*)(mem + 1716) + l7);
            *(int32_t*)(mem + 1716) = l0;
            ((S_6_l3*)l3)->f_off_4 = (l0 | 1);
            goto __end_B82; /* break */
          }
          if ((*(int32_t*)(mem + 1724) == l4)) {
            *(int32_t*)(mem + 1724) = l3;
            l0 = (*(int32_t*)(mem + 1712) + l7);
            *(int32_t*)(mem + 1712) = l0;
            ((S_6_l3*)l3)->f_off_4 = (l0 | 1);
            *(int32_t*)(mem + ((l0 + l3))) = l0;
            goto __end_B82; /* break */
          }
          l0 = *(int32_t*)(mem + (l4) + 4);
          if (((l0 & 3) == 1)) {
            l9 = (l0 & -8);
            l2 = *(int32_t*)(mem + (l4) + 12);
            do { /* block B86 */
              if ((l0 <= 255)) {
                l1 = *(int32_t*)(mem + (l4) + 8);
                if ((l1 == l2)) {
                  *(int32_t*)(mem + 1704) = (*(int32_t*)(mem + 1704) & i32.rotl(-2, (l0 >> 3)));
                  goto __end_B86; /* break */
                }
                ((S_6_l1*)l1)->f_off_12 = l2;
                ((S_6_l2*)l2)->f_off_8 = l1;
                goto __end_B86; /* break */
              }
              l6 = *(int32_t*)(mem + (l4) + 24);
              do { /* block B89 */
                if ((l2 != l4)) {
                  l0 = *(int32_t*)(mem + (l4) + 8);
                  ((S_6_l0*)l0)->f_off_12 = l2;
                  ((S_6_l2*)l2)->f_off_8 = l0;
                  goto __end_B89; /* break */
                }
                do { /* block B91 */
                  l0 = *(int32_t*)(mem + (l4) + 20);
                  if (l0) {
                    t9 = (l4 + 20);
                  } else {
                    l0 = *(int32_t*)(mem + (l4) + 16);
                    if ((!l0)) goto __end_B91;
                    t9 = (l4 + 16);
                  }
                  l1 = t9;
                  __head_L93: while (1) { /* loop L93 */
                    l5 = l1;
                    l2 = l0;
                    l1 = (l2 + 20);
                    l0 = *(int32_t*)(mem + (l0) + 20);
                    if (l0) goto __head_L93;
                    l1 = (l2 + 16);
                    l0 = *(int32_t*)(mem + (l2) + 16);
                    if (l0) goto __head_L93;
                  }
                  __end_L93: ;
                  ((S_6_l5*)l5)->f_off_0 = 0;
                  goto __end_B89; /* break */
                } while (0);
                __end_B91: ;
                l2 = 0;
              } while (0);
              __end_B89: ;
              if ((!l6)) goto __end_B86;
              do { /* block B94 */
                l0 = *(int32_t*)(mem + (l4) + 28);
                l1 = ((l0 << 2) + 2008);
                if ((*(int32_t*)(mem + (l1)) == l4)) {
                  ((S_6_l1*)l1)->f_off_0 = l2;
                  if (l2) goto __end_B94;
                  *(int32_t*)(mem + 1708) = (*(int32_t*)(mem + 1708) & i32.rotl(-2, l0));
                  goto __end_B86; /* break */
                }
                do { /* block B96 */
                  if ((l4 == *(int32_t*)(mem + (l6) + 16))) {
                    ((S_6_l6*)l6)->f_off_16 = l2;
                    goto __end_B96; /* break */
                  }
                  ((S_6_l6*)l6)->f_off_20 = l2;
                } while (0);
                __end_B96: ;
                if ((!l2)) goto __end_B86;
              } while (0);
              __end_B94: ;
              ((S_6_l2*)l2)->f_off_24 = l6;
              l0 = *(int32_t*)(mem + (l4) + 16);
              if (l0) {
                ((S_6_l2*)l2)->f_off_16 = l0;
                ((S_6_l0*)l0)->f_off_24 = l2;
              }
              l0 = *(int32_t*)(mem + (l4) + 20);
              if ((!l0)) goto __end_B86;
              ((S_6_l2*)l2)->f_off_20 = l0;
              ((S_6_l0*)l0)->f_off_24 = l2;
            } while (0);
            __end_B86: ;
            l7 = (l7 + l9);
            l4 = (l4 + l9);
            l0 = *(int32_t*)(mem + (l4) + 4);
          }
          ((S_6_l4*)l4)->f_off_4 = (l0 & -2);
          ((S_6_l3*)l3)->f_off_4 = (l7 | 1);
          *(int32_t*)(mem + ((l3 + l7))) = l7;
          if ((l7 <= 255)) {
            l0 = ((l7 & -8) + 1744);
            do { /* block B100 */
              l1 = *(int32_t*)(mem + 1704);
              l2 = (1 << (l7 >> 3));
              if ((!(l1 & l2))) {
                *(int32_t*)(mem + 1704) = (l1 | l2);
                goto __end_B100; /* break */
              }
            } while (0);
            __end_B100: ;
            l1 = *(int32_t*)(mem + (l0) + 8);
            ((S_6_l0*)l0)->f_off_8 = l3;
            ((S_6_l1*)l1)->f_off_12 = l3;
            ((S_6_l3*)l3)->f_off_12 = l0;
            ((S_6_l3*)l3)->f_off_8 = l1;
            goto __end_B82; /* break */
          }
          l2 = 31;
          if ((l7 <= 16777215)) {
            l0 = i32.clz((l7 >> 8));
            l2 = ((((l7 >> (38 - l0)) & 1) - (l0 << 1)) + 62);
          }
          ((S_6_l3*)l3)->f_off_28 = l2;
          ((S_6_l3*)l3)->f_off_16 = 0ll;
          l0 = ((l2 << 2) + 2008);
          do { /* block B103 */
            do { /* block B104 */
              l1 = *(int32_t*)(mem + 1708);
              l5 = (1 << l2);
              if ((!(l1 & l5))) {
                *(int32_t*)(mem + 1708) = (l1 | l5);
                ((S_6_l0*)l0)->f_off_0 = l3;
                goto __end_B104; /* break */
              }
              l2 = (l7 << ((l2 != 31) ? (25 - (l2 >> 1)) : 0));
              l1 = *(int32_t*)(mem + (l0));
              __head_L106: while (1) { /* loop L106 */
                l0 = l1;
                if (((*(int32_t*)(mem + (l0) + 4) & -8) == l7)) goto __end_B103;
                l1 = (l2 >> 29);
                l2 = (l2 << 1);
                l5 = (l0 + (l1 & 4));
                l1 = *(int32_t*)(mem + (l5) + 16);
                if (l1) goto __head_L106;
              }
              __end_L106: ;
              ((S_6_l5*)l5)->f_off_16 = l3;
            } while (0);
            __end_B104: ;
            ((S_6_l3*)l3)->f_off_24 = l0;
            ((S_6_l3*)l3)->f_off_12 = l3;
            ((S_6_l3*)l3)->f_off_8 = l3;
            goto __end_B82; /* break */
          } while (0);
          __end_B103: ;
          l1 = *(int32_t*)(mem + (l0) + 8);
          ((S_6_l1*)l1)->f_off_12 = l3;
          ((S_6_l0*)l0)->f_off_8 = l3;
          ((S_6_l3*)l3)->f_off_24 = 0;
          ((S_6_l3*)l3)->f_off_12 = l0;
          ((S_6_l3*)l3)->f_off_8 = l1;
        } while (0);
        __end_B82: ;
        l0 = (l8 + 8);
        goto __end_B1; /* break */
      } while (0);
      __end_B3: ;
      do { /* block B107 */
        if ((!l8)) goto __end_B107;
        do { /* block B108 */
          l1 = *(int32_t*)(mem + (l5) + 28);
          l2 = ((l1 << 2) + 2008);
          if ((*(int32_t*)(mem + (l2)) == l5)) {
            ((S_6_l2*)l2)->f_off_0 = l0;
            if (l0) goto __end_B108;
            l7 = (l7 & i32.rotl(-2, l1));
            *(int32_t*)(mem + 1708) = l7;
            goto __end_B107; /* break */
          }
          do { /* block B110 */
            if ((l5 == *(int32_t*)(mem + (l8) + 16))) {
              ((S_6_l8*)l8)->f_off_16 = l0;
              goto __end_B110; /* break */
            }
            ((S_6_l8*)l8)->f_off_20 = l0;
          } while (0);
          __end_B110: ;
          if ((!l0)) goto __end_B107;
        } while (0);
        __end_B108: ;
        ((S_6_l0*)l0)->f_off_24 = l8;
        l1 = *(int32_t*)(mem + (l5) + 16);
        if (l1) {
          ((S_6_l0*)l0)->f_off_16 = l1;
          ((S_6_l1*)l1)->f_off_24 = l0;
        }
        l1 = *(int32_t*)(mem + (l5) + 20);
        if ((!l1)) goto __end_B107;
        ((S_6_l0*)l0)->f_off_20 = l1;
        ((S_6_l1*)l1)->f_off_24 = l0;
      } while (0);
      __end_B107: ;
      do { /* block B113 */
        if ((l3 <= 15)) {
          l0 = (l3 + l6);
          ((S_6_l5*)l5)->f_off_4 = (l0 | 3);
          l0 = (l0 + l5);
          ((S_6_l0*)l0)->f_off_4 = (*(int32_t*)(mem + (l0) + 4) | 1);
          goto __end_B113; /* break */
        }
        ((S_6_l5*)l5)->f_off_4 = (l6 | 3);
        l4 = (l5 + l6);
        ((S_6_l4*)l4)->f_off_4 = (l3 | 1);
        *(int32_t*)(mem + ((l3 + l4))) = l3;
        if ((l3 <= 255)) {
          l0 = ((l3 & -8) + 1744);
          do { /* block B116 */
            l1 = *(int32_t*)(mem + 1704);
            l2 = (1 << (l3 >> 3));
            if ((!(l1 & l2))) {
              *(int32_t*)(mem + 1704) = (l1 | l2);
              goto __end_B116; /* break */
            }
          } while (0);
          __end_B116: ;
          l1 = *(int32_t*)(mem + (l0) + 8);
          ((S_6_l0*)l0)->f_off_8 = l4;
          ((S_6_l1*)l1)->f_off_12 = l4;
          ((S_6_l4*)l4)->f_off_12 = l0;
          ((S_6_l4*)l4)->f_off_8 = l1;
          goto __end_B113; /* break */
        }
        l0 = 31;
        if ((l3 <= 16777215)) {
          l0 = i32.clz((l3 >> 8));
          l0 = ((((l3 >> (38 - l0)) & 1) - (l0 << 1)) + 62);
        }
        ((S_6_l4*)l4)->f_off_28 = l0;
        /* split i64 init 0x0 into 2xi32 LE */
        ((S_6_l4*)l4)->f_off_16 = 0;
        ((S_6_l4*)l4)->f_off_20 = 0;
        l1 = ((l0 << 2) + 2008);
        do { /* block B119 */
          do { /* block B120 */
            l2 = (1 << l0);
            if ((!(l7 & l2))) {
              *(int32_t*)(mem + 1708) = (l2 | l7);
              ((S_6_l1*)l1)->f_off_0 = l4;
              ((S_6_l4*)l4)->f_off_24 = l1;
              goto __end_B120; /* break */
            }
            l0 = (l3 << ((l0 != 31) ? (25 - (l0 >> 1)) : 0));
            l1 = *(int32_t*)(mem + (l1));
            __head_L122: while (1) { /* loop L122 */
              l2 = l1;
              if (((*(int32_t*)(mem + (l2) + 4) & -8) == l3)) goto __end_B119;
              l1 = (l0 >> 29);
              l0 = (l0 << 1);
              l7 = (l2 + (l1 & 4));
              l1 = *(int32_t*)(mem + (l7) + 16);
              if (l1) goto __head_L122;
            }
            __end_L122: ;
            ((S_6_l7*)l7)->f_off_16 = l4;
            ((S_6_l4*)l4)->f_off_24 = l2;
          } while (0);
          __end_B120: ;
          ((S_6_l4*)l4)->f_off_12 = l4;
          ((S_6_l4*)l4)->f_off_8 = l4;
          goto __end_B113; /* break */
        } while (0);
        __end_B119: ;
        l0 = *(int32_t*)(mem + (l2) + 8);
        ((S_6_l0*)l0)->f_off_12 = l4;
        ((S_6_l2*)l2)->f_off_8 = l4;
        ((S_6_l4*)l4)->f_off_24 = 0;
        ((S_6_l4*)l4)->f_off_12 = l2;
        ((S_6_l4*)l4)->f_off_8 = l0;
      } while (0);
      __end_B113: ;
      l0 = (l5 + 8);
      goto __end_B1; /* break */
    } while (0);
    __end_B2: ;
    do { /* block B123 */
      if ((!l9)) goto __end_B123;
      do { /* block B124 */
        l1 = *(int32_t*)(mem + (l2) + 28);
        l5 = ((l1 << 2) + 2008);
        if ((*(int32_t*)(mem + (l5)) == l2)) {
          ((S_6_l5*)l5)->f_off_0 = l0;
          if (l0) goto __end_B124;
          *(int32_t*)(mem + 1708) = (l11 & i32.rotl(-2, l1));
          goto __end_B123; /* break */
        }
        do { /* block B126 */
          if ((l2 == *(int32_t*)(mem + (l9) + 16))) {
            ((S_6_l9*)l9)->f_off_16 = l0;
            goto __end_B126; /* break */
          }
          ((S_6_l9*)l9)->f_off_20 = l0;
        } while (0);
        __end_B126: ;
        if ((!l0)) goto __end_B123;
      } while (0);
      __end_B124: ;
      ((S_6_l0*)l0)->f_off_24 = l9;
      l1 = *(int32_t*)(mem + (l2) + 16);
      if (l1) {
        ((S_6_l0*)l0)->f_off_16 = l1;
        ((S_6_l1*)l1)->f_off_24 = l0;
      }
      l1 = *(int32_t*)(mem + (l2) + 20);
      if ((!l1)) goto __end_B123;
      ((S_6_l0*)l0)->f_off_20 = l1;
      ((S_6_l1*)l1)->f_off_24 = l0;
    } while (0);
    __end_B123: ;
    do { /* block B129 */
      if ((l3 <= 15)) {
        l0 = (l3 + l6);
        ((S_6_l2*)l2)->f_off_4 = (l0 | 3);
        l0 = (l0 + l2);
        ((S_6_l0*)l0)->f_off_4 = (*(int32_t*)(mem + (l0) + 4) | 1);
        goto __end_B129; /* break */
      }
      ((S_6_l2*)l2)->f_off_4 = (l6 | 3);
      l5 = (l2 + l6);
      ((S_6_l5*)l5)->f_off_4 = (l3 | 1);
      *(int32_t*)(mem + ((l3 + l5))) = l3;
      if (l8) {
        l0 = ((l8 & -8) + 1744);
        l1 = *(int32_t*)(mem + 1724);
        do { /* block B132 */
          l7 = (1 << (l8 >> 3));
          if ((!(l7 & l4))) {
            *(int32_t*)(mem + 1704) = (l4 | l7);
            goto __end_B132; /* break */
          }
        } while (0);
        __end_B132: ;
        l4 = *(int32_t*)(mem + (l0) + 8);
        ((S_6_l0*)l0)->f_off_8 = l1;
        ((S_6_l4*)l4)->f_off_12 = l1;
        ((S_6_l1*)l1)->f_off_12 = l0;
        ((S_6_l1*)l1)->f_off_8 = l4;
      }
      *(int32_t*)(mem + 1724) = l5;
      *(int32_t*)(mem + 1712) = l3;
    } while (0);
    __end_B129: ;
    l0 = (l2 + 8);
  } while (0);
  __end_B1: ;
  g0 = (l10 + 16);
  return l0;
}

void f7(int32_t p0) {
  int32_t l0 = p0;
  g(l0);
}

int32_t f8(int32_t p0) {
  int32_t l0 = p0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  l1 = ((l0 <= 1) ? 1 : l0);
  __head_L1: while (1) { /* loop L1 */
    do { /* block B2 */
      l0 = f(l1);
      if (l0) goto __end_B2;
      l2 = *(int32_t*)(mem + 2200);
      if ((!l2)) goto __end_B2;
      table_call(l2)();
      goto __head_L1; /* continue */
    } while (0);
    __end_B2: ;
  }
  __end_L1: ;
  if ((!l0)) {
    f12();
    /* unreachable (trap) */
    __builtin_trap();
  }
  return l0;
}

void f9() {
  int32_t l0 = 0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  int32_t l3 = 0;
  /* hint: l0 is S_9_l0* (3 fields) */
  /* hint: l3 is S_9_l3* (2 fields) */
  l0 = f11(8);
  ((S_9_l0*)l0)->f_off_0 = 1468;
  l3 = l0;
  ((S_9_l3*)l3)->f_off_0 = 1580;
  do { /* block B1 */
    do { /* block B2 */
      do { /* block B3 */
        l1 = 1034;
        if ((!(l1 & 3))) goto __end_B3;
        if ((!*(int8_t*)(mem + 1034))) goto __end_B1;
        __head_L4: while (1) { /* loop L4 */
          l1 = (l1 + 1);
          if ((!(l1 & 3))) goto __end_B3;
          if (*(int8_t*)(mem + (l1))) goto __head_L4;
        }
        __end_L4: ;
        goto __end_B2; /* break */
      } while (0);
      __end_B3: ;
      __head_L5: while (1) { /* loop L5 */
        l0 = l1;
        l1 = (l0 + 4);
        l2 = *(int32_t*)(mem + (l0));
        if (((((16843008 - l2) | l2) & -2139062144) == -2139062144)) goto __head_L5;
      }
      __end_L5: ;
      __head_L6: while (1) { /* loop L6 */
        l1 = l0;
        l0 = (l1 + 1);
        if (*(int8_t*)(mem + (l1))) goto __head_L6;
      }
      __end_L6: ;
    } while (0);
    __end_B2: ;
  } while (0);
  __end_B1: ;
  l2 = (l1 - 1034);
  l0 = f8((l2 + 13));
  ((S_9_l0*)l0)->f_off_8 = 0;
  ((S_9_l0*)l0)->f_off_4 = l2;
  ((S_9_l0*)l0)->f_off_0 = l2;
  l1 = (l0 + 12);
  l0 = (l2 + 1);
  if (l0) {
    memory.copy(l1, 1034, l0);
  }
  ((S_9_l3*)l3)->f_off_4 = l1;
  ((S_9_l3*)l3)->f_off_0 = 1628;
  a_a(l3, 1640, 2);
  /* unreachable (trap) */
  __builtin_trap();
}

int32_t f10(int32_t p0) {
  int32_t l0 = p0;
  return l0;
}

int32_t f11(int32_t p0) {
  int32_t l0 = p0;
  return (f((l0 + 80)) + 80);
}

void f12() {
  a_b();
  /* unreachable (trap) */
  __builtin_trap();
}

void f13(int32_t p0) {
  int32_t l0 = p0;
  f14(l0);
  g(l0);
}

int32_t f14(S_14_l0* p0) {
  S_14_l0* l0 = p0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  /* hint: l0 is S_14_l0* (2 fields) */
  ((S_14_l0*)l0)->f_off_0 = 1580;
  l1 = (*(int32_t*)(mem + (l0) + 4) - 12);
  l2 = (*(int32_t*)(mem + (l1) + 8) - 1);
  *(int32_t*)(mem + (l1) + 8) = l2;
  if ((l2 < 0)) {
    g(l1);
  }
  return l0;
}

void f15(S_15_l0* p0, int32_t p1, int32_t p2, int32_t p3) {
  S_15_l0* l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  /* hint: l0 is S_15_l0* (8 fields) */
  ((S_15_l0*)l0)->f_off_53 = 1;
  do { /* block B1 */
    if ((l2 != *(int32_t*)(mem + (l0) + 4))) goto __end_B1;
    ((S_15_l0*)l0)->f_off_52 = 1;
    do { /* block B2 */
      l2 = *(int32_t*)(mem + (l0) + 16);
      if ((!l2)) {
        ((S_15_l0*)l0)->f_off_36 = 1;
        ((S_15_l0*)l0)->f_off_24 = l3;
        ((S_15_l0*)l0)->f_off_16 = l1;
        if ((l3 != 1)) goto __end_B1;
        if ((*(int32_t*)(mem + (l0) + 48) == 1)) goto __end_B2;
        goto __end_B1; /* break */
      }
      if ((l1 == l2)) {
        l2 = *(int32_t*)(mem + (l0) + 24);
        if ((l2 == 2)) {
          ((S_15_l0*)l0)->f_off_24 = l3;
          l2 = l3;
        }
        if ((*(int32_t*)(mem + (l0) + 48) != 1)) goto __end_B1;
        if ((l2 == 1)) goto __end_B2;
        goto __end_B1; /* break */
      }
      ((S_15_l0*)l0)->f_off_36 = (*(int32_t*)(mem + (l0) + 36) + 1);
    } while (0);
    __end_B2: ;
    ((S_15_l0*)l0)->f_off_54 = 1;
  } while (0);
  __end_B1: ;
}

void f16(S_16_l0* p0, int32_t p1, int32_t p2) {
  S_16_l0* l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = 0;
  /* hint: l0 is S_16_l0* (6 fields) */
  l3 = *(int32_t*)(mem + (l0) + 36);
  if ((!l3)) {
    ((S_16_l0*)l0)->f_off_24 = l2;
    ((S_16_l0*)l0)->f_off_16 = l1;
    ((S_16_l0*)l0)->f_off_36 = 1;
    ((S_16_l0*)l0)->f_off_20 = *(int32_t*)(mem + (l0) + 56);
    return;
  }
  do { /* block B2 */
    do { /* block B3 */
      if ((*(int32_t*)(mem + (l0) + 20) != *(int32_t*)(mem + (l0) + 56))) goto __end_B3;
      if ((*(int32_t*)(mem + (l0) + 16) != l1)) goto __end_B3;
      if ((*(int32_t*)(mem + (l0) + 24) != 2)) goto __end_B2;
      ((S_16_l0*)l0)->f_off_24 = l2;
      return;
    } while (0);
    __end_B3: ;
    ((S_16_l0*)l0)->f_off_54 = 1;
    ((S_16_l0*)l0)->f_off_24 = 2;
    ((S_16_l0*)l0)->f_off_36 = (l3 + 1);
  } while (0);
  __end_B2: ;
}

void f17(int32_t p0) {
  int32_t l0 = p0;
}

int32_t f18(int32_t p0) {
  int32_t l0 = p0;
  return *(int32_t*)(mem + (l0) + 4);
}

int32_t f19(int32_t p0) {
  int32_t l0 = p0;
  return 1120;
}

int32_t f20(int32_t p0) {
  int32_t l0 = p0;
  return 1141;
}

int32_t f21(int32_t p0) {
  int32_t l0 = p0;
  return 1105;
}

void f22(int32_t p0, int32_t p1, int32_t p2, int32_t p3, int32_t p4, int32_t p5) {
  int32_t l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  int32_t l4 = p4;
  int32_t l5 = p5;
  if (f3(l0, *(int32_t*)(mem + (l1) + 8), l5)) {
    f15(l1, l2, l3, l4);
  }
}

void f23(S_23_l0* p0 /* table index */, int32_t p1, int32_t p2, int32_t p3, int32_t p4, int32_t p5) {
  S_23_l0* l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  int32_t l4 = p4;
  int32_t l5 = p5;
  /* hint: l0 is S_23_l0* (2 fields) */
  if (f3(l0, *(int32_t*)(mem + (l1) + 8), l5)) {
    f15(l1, l2, l3, l4);
    return;
  }
  l0 = *(int32_t*)(mem + (l0) + 8);
  table_call(l0)(l1, l2, l3, l4, l5, *(int32_t*)(mem + (*(int32_t*)(mem + (l0))) + 20));
}

int32_t h(S_24_l0* p0, S_24_l1* p1, int32_t p2) {
  S_24_l0* l0 = p0;
  S_24_l1* l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = 0;
  int32_t l4 = 0;
  int32_t l5 = 0;
  int32_t l6 = 0;
  int32_t l7 = 0;
  int32_t l8 = 0;
  int32_t l9 = 0;
  int32_t l10 = 0;
  int32_t l11 = 0;
  int32_t l12 = 0;
  int32_t l13 = 0;
  int32_t l14 = 0;
  int32_t l15 = 0;
  int32_t l16 = 0;
  int32_t l17 = 0;
  int32_t l18 = 0;
  int32_t l19 = 0;
  int32_t l20 = 0;
  int32_t l21 = 0;
  int32_t l22 = 0;
  int32_t l23 = 0;
  int32_t l24 = 0;
  int32_t l25 = 0;
  int32_t l26 = 0;
  int32_t l27 = 0;
  int32_t l28 = 0;
  int32_t l29 = 0;
  int32_t l30 = 0;
  int32_t l31 = 0;
  int32_t l32 = 0;
  int32_t l33 = 0;
  int32_t l34 = 0;
  int32_t l35 = 0;
  int32_t l36 = 0;
  int32_t l37 = 0;
  int32_t l38 = 0;
  int32_t l39 = 0;
  int32_t l40 = 0;
  int32_t l41 = 0;
  int32_t l42 = 0;
  int32_t t2 = 0; /* synthetic (type TODO) */
  int32_t t5 = 0; /* synthetic (type TODO) */
  int32_t t6 = 0; /* synthetic (type TODO) */
  int32_t t7 = 0; /* synthetic (type TODO) */
  int32_t t8 = 0; /* synthetic (type TODO) */
  int32_t t9 = 0; /* synthetic (type TODO) */
  int32_t t10 = 0; /* synthetic (type TODO) */
  int32_t t11 = 0; /* synthetic (type TODO) */
  int32_t t12 = 0; /* synthetic (type TODO) */
  int32_t t13 = 0; /* synthetic (type TODO) */
  int32_t t14 = 0; /* synthetic (type TODO) */
  int32_t t15 = 0; /* synthetic (type TODO) */
  int32_t t16 = 0; /* synthetic (type TODO) */
  int32_t t17 = 0; /* synthetic (type TODO) */
  /* hint: l0 is S_24_l0* (7 fields) */
  /* hint: l1 is S_24_l1* (3 fields) */
  /* hint: l10 is S_24_l10* (1 fields) */
  /* hint: l11 is S_24_l11* (15 fields) */
  /* hint: l12 is S_24_l12* (1 fields) */
  /* hint: l21 is S_24_l21* (1 fields) */
  /* hint: l24 is S_24_l24* (1 fields) */
  /* hint: l3 is S_24_l3* (16 fields) */
  /* hint: l5 is S_24_l5* (1 fields) */
  /* hint: l8 is S_24_l8* (2 fields) */
  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */
  l11 = (g0 + -64);
  g0 = l11;
  ((S_24_l11*)l11)->f_off_60 = 0;
  /* split i64 init 0x0 into 2xi32 LE */
  ((S_24_l11*)l11)->f_off_52 = 0;
  ((S_24_l11*)l11)->f_off_56 = 0;
  do { /* block B1 */
    do { /* block B2 */
      do { /* block B3 */
        do { /* block B4 */
          do { /* block B5 */
            if (l1) {
              if ((l1 < 0)) goto __end_B5;
              l3 = f8(l1);
              ((S_24_l11*)l11)->f_off_52 = l3;
              l9 = (l1 + l3);
              ((S_24_l11*)l11)->f_off_60 = l9;
              if (l1) {
                memory.copy(l3, l0, l1);
              }
              ((S_24_l11*)l11)->f_off_56 = l9;
            }
            l0 = (l3 + *(int32_t*)(mem + (l3) + 60));
            l7 = *(int16_t*)(mem + (l0) + 6);
            l1 = *(int16_t*)(mem + (l0) + 20);
            ((S_24_l11*)l11)->f_off_48 = 0;
            /* split i64 init 0x0 into 2xi32 LE */
            ((S_24_l11*)l11)->f_off_40 = 0;
            ((S_24_l11*)l11)->f_off_44 = 0;
            if ((!l7)) {
              l20 = l3;
              goto __end_B1; /* break */
            }
            l6 = ((l0 + l1) + 24);
            l1 = 0;
            __head_L9: while (1) { /* loop L9 */
              do { /* block B10 */
                l0 = (l6 + (l1 * 40));
                if (*(int32_t*)(mem + (l0) + 16)) goto __end_B10;
                if (*(int32_t*)(mem + (l0) + 20)) goto __end_B10;
                if ((*(int8_t*)(mem + (l0) + 36) & 128)) goto __end_B10;
                l15 = *(int32_t*)(mem + (l0) + 12);
                do { /* block B11 */
                  l14 = *(int32_t*)(mem + (l11) + 48);
                  if ((l14 > l5)) {
                    ((S_24_l5*)l5)->f_off_0 = ((((i64)l15) << 32ll) | 4294967295ll);
                    l5 = (l5 + 8);
                    goto __end_B11; /* break */
                  }
                  l0 = *(int32_t*)(mem + (l11) + 40);
                  l9 = (l5 - l0);
                  l20 = (l9 >> 3);
                  l5 = (l20 + 1);
                  if ((l5 >= 536870912)) goto __end_B4;
                  l14 = (l14 - l0);
                  l4 = (l14 >> 2);
                  l5 = ((l14 >= 2147483640) ? 536870911 : ((l4 > l5) ? l4 : l5));
                  if (l5) {
                    if ((l5 >= 536870912)) goto __end_B3;
                    t2 = f8((l5 << 3));
                  } else {
                    t2 = 0;
                  }
                  l4 = t2;
                  l14 = (l9 + l4);
                  *(int64_t*)(mem + (l14)) = ((((i64)l15) << 32ll) | 4294967295ll);
                  l15 = (l14 - (l20 << 3));
                  if (l9) {
                    memory.copy(l15, l0, l9);
                  }
                  ((S_24_l11*)l11)->f_off_48 = (l4 + (l5 << 3));
                  ((S_24_l11*)l11)->f_off_40 = l15;
                  l5 = (l14 + 8);
                  if ((!l0)) goto __end_B11;
                  g(l0);
                } while (0);
                __end_B11: ;
                ((S_24_l11*)l11)->f_off_44 = l5;
              } while (0);
              __end_B10: ;
              l1 = (l1 + 1);
              if ((l7 != l1)) goto __head_L9;
            }
            __end_L9: ;
            goto __end_B2; /* break */
          } while (0);
          __end_B5: ;
          f9();
          /* unreachable (trap) */
          __builtin_trap();
        } while (0);
        __end_B4: ;
        f9();
        /* unreachable (trap) */
        __builtin_trap();
      } while (0);
      __end_B3: ;
      l0 = f11(4);
      ((S_24_l0*)l0)->f_off_0 = 1468;
      ((S_24_l0*)l0)->f_off_0 = 1428;
      ((S_24_l0*)l0)->f_off_0 = 1448;
      a_a(l0, 1532, 1);
      /* unreachable (trap) */
      __builtin_trap();
    } while (0);
    __end_B2: ;
    l9 = *(int32_t*)(mem + (l11) + 56);
    l20 = *(int32_t*)(mem + (l11) + 52);
  } while (0);
  __end_B1: ;
  l22 = *(int32_t*)(mem + (l11) + 40);
  l1 = 0;
  do { /* block B15 */
    do { /* block B16 */
      do { /* block B17 */
        l7 = (l5 - l22);
        l0 = (l9 - l20);
        if ((l7 > l0)) goto __end_B17;
        if (((l7 - 1) >= l0)) goto __end_B17;
        l15 = (l7 + 1);
        l9 = 0;
        __head_L18: while (1) { /* loop L18 */
          l25 = (l9 + l20);
          l1 = 0;
          __head_L19: while (1) { /* loop L19 */
            do { /* block B20 */
              do { /* block B21 */
                l14 = *(int8_t*)(mem + ((l1 + l22)));
                if ((l14 == 255)) goto __end_B21;
                if ((*(int8_t*)(mem + ((l1 + l25))) == l14)) goto __end_B21;
                if ((l1 == l7)) goto __end_B20;
                l1 = (l9 + l15);
                l9 = (l9 + 1);
                if ((l0 >= l1)) goto __head_L18;
                l1 = 0;
                goto __end_B16; /* break */
              } while (0);
              __end_B21: ;
              l1 = (l1 + 1);
              if ((l1 != l7)) goto __head_L19;
            } while (0);
            __end_B20: ;
          }
          __end_L19: ;
        }
        __end_L18: ;
        if ((!l25)) {
          l1 = 0;
          goto __end_B17; /* break */
        }
        l0 = 0;
        l3 = (l3 + *(int32_t*)(mem + (l3) + 60));
        l15 = *(int16_t*)(mem + (l3) + 6);
        l9 = *(int16_t*)(mem + (l3) + 20);
        ((S_24_l11*)l11)->f_off_36 = 0;
        /* split i64 init 0x0 into 2xi32 LE */
        ((S_24_l11*)l11)->f_off_28 = 0;
        ((S_24_l11*)l11)->f_off_32 = 0;
        do { /* block B23 */
          do { /* block B24 */
            l1 = *(int32_t*)(mem + (l3) + 80);
            if (l1) {
              if ((l1 < 0)) goto __end_B24;
              l0 = f8(l1);
              if (l1) {
                memory.fill(l0, 0, l1);
              }
              l1 = (l0 + l1);
              ((S_24_l11*)l11)->f_off_36 = l1;
              ((S_24_l11*)l11)->f_off_32 = l1;
              ((S_24_l11*)l11)->f_off_28 = l0;
            }
            l14 = *(int32_t*)(mem + (l11) + 52);
            l6 = ((l3 + l9) + 24);
            l1 = *(int32_t*)(mem + (l6) + 12);
            if (l1) {
              memory.copy(l0, l14, l1);
            }
            do { /* block B28 */
              if ((!l15)) goto __end_B28;
              l1 = 0;
              if ((l15 != 1)) {
                l8 = (l15 & 65534);
                l9 = 0;
                __head_L30: while (1) { /* loop L30 */
                  do { /* block B31 */
                    l3 = (l6 + (l1 * 40));
                    l4 = *(int32_t*)(mem + (l3) + 16);
                    if ((!l4)) goto __end_B31;
                    if ((!l4)) goto __end_B31;
                    memory.copy((l0 + *(int32_t*)(mem + (l3) + 12)), (l14 + *(int32_t*)(mem + (l3) + 20)), l4);
                  } while (0);
                  __end_B31: ;
                  do { /* block B32 */
                    l4 = *(int32_t*)(mem + (l3) + 56);
                    if ((!l4)) goto __end_B32;
                    if ((!l4)) goto __end_B32;
                    memory.copy((l0 + *(int32_t*)(mem + (l3) + 52)), (l14 + *(int32_t*)(mem + (l3) + 60)), l4);
                  } while (0);
                  __end_B32: ;
                  l1 = (l1 + 2);
                  l9 = (l9 + 2);
                  if ((l9 != l8)) goto __head_L30;
                }
                __end_L30: ;
              }
              if ((!(l15 & 1))) goto __end_B28;
              l1 = (l6 + (l1 * 40));
              l3 = *(int32_t*)(mem + (l1) + 16);
              if ((!l3)) goto __end_B28;
              if ((!l3)) goto __end_B28;
              memory.copy((l0 + *(int32_t*)(mem + (l1) + 12)), (l14 + *(int32_t*)(mem + (l1) + 20)), l3);
            } while (0);
            __end_B28: ;
            do { /* block B33 */
              l0 = (l0 + *(int32_t*)(mem + (l0) + 60));
              l15 = *(int16_t*)(mem + (l0) + 6);
              if ((!l15)) goto __end_B33;
              l14 = ((l0 + *(int16_t*)(mem + (l0) + 20)) + 24);
              l9 = 0;
              l1 = 0;
              if ((l15 >= 4)) {
                l6 = (l15 & 65532);
                l0 = 0;
                __head_L35: while (1) { /* loop L35 */
                  l3 = (l14 + (l1 * 40));
                  ((S_24_l3*)l3)->f_off_16 = *(int64_t*)(mem + (l3) + 8);
                  ((S_24_l3*)l3)->f_off_56 = *(int64_t*)(mem + (l3) + 48);
                  ((S_24_l3*)l3)->f_off_96 = *(int64_t*)(mem + (l3) + 88);
                  ((S_24_l3*)l3)->f_off_136 = *(int64_t*)(mem + (l3) + 128);
                  l1 = (l1 + 4);
                  l0 = (l0 + 4);
                  if ((l0 != l6)) goto __head_L35;
                }
                __end_L35: ;
              }
              l0 = (l15 & 3);
              if ((!l0)) goto __end_B33;
              __head_L36: while (1) { /* loop L36 */
                l3 = (l14 + (l1 * 40));
                ((S_24_l3*)l3)->f_off_16 = *(int64_t*)(mem + (l3) + 8);
                l1 = (l1 + 1);
                l9 = (l9 + 1);
                if ((l9 != l0)) goto __head_L36;
              }
              __end_L36: ;
            } while (0);
            __end_B33: ;
            goto __end_B23; /* break */
          } while (0);
          __end_B24: ;
          f9();
          /* unreachable (trap) */
          __builtin_trap();
        } while (0);
        __end_B23: ;
        l1 = 0;
        l0 = (*(int32_t*)(mem + (l11) + 28) + *(int32_t*)(mem + ((l25 - 8))));
        l3 = 1;
        do { /* block B37 */
          if ((*(int32_t*)(mem + ((l25 - 4))) < 5)) goto __end_B37;
          l0 = *(int8_t*)(mem + (l0));
          if ((l0 > 224)) goto __end_B37;
          l3 = 0;
          if ((l0 >= 45)) {
            l0 = (l0 - 45);
            l9 = ((l0 & 255) / 45);
            l0 = (l0 - (l9 * 45));
            t5 = (l9 + 1);
          } else {
            t5 = 0;
          }
          ((S_24_l11*)l11)->f_off_20 = t5;
          if (((l0 & 255) >= 9)) {
            l0 = (l0 - 9);
            l9 = ((l0 & 255) / 9);
            l0 = (l0 - (l9 * 9));
            t6 = (l9 + 1);
          } else {
            t6 = 0;
          }
          ((S_24_l11*)l11)->f_off_16 = t6;
          ((S_24_l11*)l11)->f_off_12 = (l0 & 255);
        } while (0);
        __end_B37: ;
        if ((!l3)) {
          t7 = f(((1536 << (*(int32_t*)(mem + (l11) + 16) + *(int32_t*)(mem + (l11) + 12))) + 3692));
          l9 = t7;
          ((S_24_l11*)l11)->f_off_24 = l9;
          do { /* block B41 */
            if ((l5 != l22)) {
              do { /* block B43 */
                __head_L44: while (1) { /* loop L44 */
                  do { /* block B45 */
                    l0 = *(int32_t*)(mem + (l11) + 28);
                    l3 = (l25 + (l1 << 3));
                    l26 = (l0 + *(int32_t*)(mem + (l3)));
                    l23 = (l0 + *(int32_t*)(mem + (l3) + 4));
                    l4 = 0;
                    l8 = 0;
                    l16 = 0;
                    l17 = 0;
                    l5 = *(int32_t*)(mem + (l11) + 20);
                    l27 = *(int32_t*)(mem + (l11) + 12);
                    l9 = *(int32_t*)(mem + (l11) + 16);
                    l18 = *(int32_t*)(mem + (l11) + 24);
                    ((S_24_l11*)l11)->f_off_8 = 0;
                    ((S_24_l11*)l11)->f_off_4 = 0;
                    l3 = (l18 + 14);
                    l7 = (l18 + 12);
                    l15 = (l18 + 10);
                    l14 = (l18 + 8);
                    l6 = (l18 + 6);
                    l13 = (l18 + 4);
                    l10 = (l18 + 2);
                    l12 = ((768 << (l9 + l27)) + 1832);
                    __head_L46: while (1) { /* loop L46 */
                      l0 = (l4 << 1);
                      *(int16_t*)(mem + ((l18 + l0))) = 1024;
                      *(int16_t*)(mem + ((l0 + l10))) = 1024;
                      *(int16_t*)(mem + ((l0 + l13))) = 1024;
                      *(int16_t*)(mem + ((l0 + l6))) = 1024;
                      *(int16_t*)(mem + ((l0 + l14))) = 1024;
                      *(int16_t*)(mem + ((l0 + l15))) = 1024;
                      *(int16_t*)(mem + ((l0 + l7))) = 1024;
                      *(int16_t*)(mem + ((l0 + l3))) = 1024;
                      l4 = (l4 + 8);
                      l16 = (l16 + 8);
                      if ((l12 != l16)) goto __head_L46;
                    }
                    __end_L46: ;
                    __head_L47: while (1) { /* loop L47 */
                      *(int16_t*)(mem + ((l18 + (l4 << 1)))) = 1024;
                      l4 = (l4 + 1);
                      l8 = (l8 + 1);
                      if ((l8 != 6)) goto __head_L47;
                    }
                    __end_L47: ;
                    l3 = (l26 + 5);
                    l13 = (l26 - 1);
                    l29 = ((-1 << l9) ^ -1);
                    l30 = ((-1 << l5) ^ -1);
                    l0 = *(int32_t*)(mem + (l26) + 1);
                    l4 = (((l0 << 24) | ((l0 & 65280) << 8)) | (((l0 >> 8) & 65280) | (l0 >> 24)));
                    l31 = (8 - l27);
                    l32 = (l18 + 3692);
                    l34 = (l18 + 1604);
                    l35 = (l18 + 864);
                    l37 = (l18 + 2664);
                    l38 = (l18 + 480);
                    l39 = (l18 + 456);
                    l40 = (l18 + 432);
                    l41 = (l18 + 408);
                    l42 = (l18 + 384);
                    l16 = 0;
                    l8 = 0;
                    l7 = 1;
                    l14 = 1;
                    l15 = 1;
                    l0 = 1;
                    l6 = -1;
                    do { /* block B48 */
                      __head_L49: while (1) { /* loop L49 */
                        do { /* block B50 */
                          if ((l6 > 16777215)) {
                            l5 = l3;
                            goto __end_B50; /* break */
                          }
                          if ((l3 == l13)) goto __end_B48;
                          l5 = (l3 + 1);
                          l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                        } while (0);
                        __end_B50: ;
                        l9 = (l6 << 8);
                        do { /* block B52 */
                          do { /* block B53 */
                            do { /* block B54 */
                              do { /* block B55 */
                                do { /* block B56 */
                                  l10 = (l16 & l30);
                                  l19 = (l10 << 1);
                                  l12 = (l19 + (l18 + (l17 << 5)));
                                  l3 = *(int16_t*)(mem + (l12));
                                  l6 = (l3 * (l9 >> 11));
                                  if ((l6 > l4)) {
                                    ((S_24_l12*)l12)->f_off_0 = (l3 + ((2048 - l3) >> 5));
                                    l12 = (l32 + ((((l16 & l29) << l27) + ((l8 & 255) >> l31)) * 1536));
                                    l8 = 1;
                                    if ((l17 < 7)) goto __end_B55;
                                    l21 = (l12 + 512);
                                    l10 = *(int8_t*)(mem + ((l23 + (l16 - l7))));
                                    __head_L58: while (1) { /* loop L58 */
                                      if ((l6 > 16777215)) {
                                        t8 = l6;
                                      } else {
                                        if ((l5 == l13)) goto __end_B48;
                                        l4 = (*(int8_t*)(mem + (l5)) | (l4 << 8));
                                        l5 = (l5 + 1);
                                        t8 = (l6 << 8);
                                      }
                                      l3 = t8;
                                      do { /* block B60 */
                                        l8 = (l8 << 1);
                                        l10 = (l10 << 1);
                                        l19 = (l10 & 256);
                                        l24 = (l8 + (l21 + (l19 << 1)));
                                        l9 = *(int16_t*)(mem + (l24));
                                        l6 = (l9 * (l3 >> 11));
                                        if ((l6 > l4)) {
                                          ((S_24_l24*)l24)->f_off_0 = (l9 + ((2048 - l9) >> 5));
                                          if ((!l19)) goto __end_B60;
                                          goto __end_B56; /* break */
                                        }
                                        ((S_24_l24*)l24)->f_off_0 = (l9 - (l9 >> 5));
                                        l8 = (l8 | 1);
                                        l4 = (l4 - l6);
                                        l6 = (l3 - l6);
                                        if ((!l19)) goto __end_B56;
                                      } while (0);
                                      __end_B60: ;
                                      if ((l8 < 256)) goto __head_L58;
                                    }
                                    __end_L58: ;
                                    goto __end_B54; /* break */
                                  }
                                  ((S_24_l12*)l12)->f_off_0 = (l3 - (l3 >> 5));
                                  l4 = (l4 - l6);
                                  l8 = (l9 - l6);
                                  if ((l8 > 16777215)) {
                                    t9 = l5;
                                  } else {
                                    if ((l5 == l13)) goto __end_B48;
                                    l8 = (l8 << 8);
                                    l4 = (*(int8_t*)(mem + (l5)) | (l4 << 8));
                                    t9 = (l5 + 1);
                                  }
                                  l3 = t9;
                                  do { /* block B63 */
                                    l5 = (l17 << 1);
                                    l12 = (l42 + l5);
                                    l9 = *(int16_t*)(mem + (l12));
                                    l6 = (l9 * (l8 >> 11));
                                    if ((l6 > l4)) {
                                      ((S_24_l12*)l12)->f_off_0 = (l9 + ((2048 - l9) >> 5));
                                      l17 = ((l17 >= 7) ? 3 : 0);
                                      l9 = l15;
                                      l15 = l14;
                                      l14 = l7;
                                      l0 = l14;
                                      goto __end_B63; /* break */
                                    }
                                    ((S_24_l12*)l12)->f_off_0 = (l9 - (l9 >> 5));
                                    l4 = (l4 - l6);
                                    l6 = (l8 - l6);
                                    if ((l6 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l6 = (l6 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B66 */
                                      l12 = (l5 + l41);
                                      l9 = *(int16_t*)(mem + (l12));
                                      l8 = (l9 * (l6 >> 11));
                                      if ((l8 > l4)) {
                                        ((S_24_l12*)l12)->f_off_0 = (l9 + ((2048 - l9) >> 5));
                                        if ((l8 <= 16777215)) {
                                          if ((l3 == l13)) goto __end_B48;
                                          l8 = (l8 << 8);
                                          l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                          l3 = (l3 + 1);
                                        }
                                        l5 = ((l38 + (l17 << 5)) + l19);
                                        l9 = *(int16_t*)(mem + (l5));
                                        l6 = (l9 * (l8 >> 11));
                                        if ((l6 > l4)) {
                                          ((S_24_l5*)l5)->f_off_0 = (l9 + ((2048 - l9) >> 5));
                                          if ((!l16)) goto __end_B48;
                                          l8 = *(int8_t*)(mem + ((l23 + (l16 - l7))));
                                          *(int8_t*)(mem + ((l16 + l23))) = l8;
                                          l17 = ((l17 < 7) ? 9 : 11);
                                          l16 = (l16 + 1);
                                          goto __end_B53; /* break */
                                        }
                                        ((S_24_l5*)l5)->f_off_0 = (l9 - (l9 >> 5));
                                        l4 = (l4 - l6);
                                        l6 = (l8 - l6);
                                        l9 = l0;
                                        l0 = l7;
                                        goto __end_B66; /* break */
                                      }
                                      ((S_24_l12*)l12)->f_off_0 = (l9 - (l9 >> 5));
                                      l4 = (l4 - l8);
                                      l8 = (l6 - l8);
                                      if ((l8 <= 16777215)) {
                                        if ((l3 == l13)) goto __end_B48;
                                        l8 = (l8 << 8);
                                        l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                        l3 = (l3 + 1);
                                      }
                                      l12 = (l5 + l40);
                                      l9 = *(int16_t*)(mem + (l12));
                                      l6 = (l9 * (l8 >> 11));
                                      if ((l6 > l4)) {
                                        ((S_24_l12*)l12)->f_off_0 = (l9 + ((2048 - l9) >> 5));
                                        l9 = l0;
                                        l0 = l14;
                                        goto __end_B66; /* break */
                                      }
                                      ((S_24_l12*)l12)->f_off_0 = (l9 - (l9 >> 5));
                                      l4 = (l4 - l6);
                                      l8 = (l8 - l6);
                                      if ((l8 <= 16777215)) {
                                        if ((l3 == l13)) goto __end_B48;
                                        l8 = (l8 << 8);
                                        l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                        l3 = (l3 + 1);
                                      }
                                      do { /* block B73 */
                                        l12 = (l5 + l39);
                                        l5 = *(int16_t*)(mem + (l12));
                                        l6 = (l5 * (l8 >> 11));
                                        if ((l6 > l4)) {
                                          l9 = l0;
                                          l0 = l15;
                                          goto __end_B73; /* break */
                                        }
                                        l4 = (l4 - l6);
                                        l6 = (l8 - l6);
                                        l9 = l15;
                                      } while (0);
                                      __end_B73: ;
                                      l5 = (l5 - (l5 >> 5));
                                      ((S_24_l12*)l12)->f_off_0 = l5;
                                      l15 = l14;
                                    } while (0);
                                    __end_B66: ;
                                    l14 = l7;
                                    l17 = ((l17 < 7) ? 8 : 11);
                                  } while (0);
                                  __end_B63: ;
                                  l8 = l37;
                                  if ((l6 > 16777215)) {
                                    t10 = l3;
                                  } else {
                                    if ((l3 == l13)) goto __end_B48;
                                    l6 = (l6 << 8);
                                    l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                    t10 = (l3 + 1);
                                  }
                                  l7 = t10;
                                  do { /* block B76 */
                                    l3 = *(int16_t*)(mem + (l8));
                                    l5 = (l3 * (l6 >> 11));
                                    if ((l5 > l4)) {
                                      ((S_24_l8*)l8)->f_off_0 = (l3 + ((2048 - l3) >> 5));
                                      l8 = ((l8 + (l10 << 4)) + 4);
                                      l19 = 3;
                                      l10 = 1;
                                      goto __end_B76; /* break */
                                    }
                                    ((S_24_l8*)l8)->f_off_0 = (l3 - (l3 >> 5));
                                    l4 = (l4 - l5);
                                    l6 = (l6 - l5);
                                    if ((l6 <= 16777215)) {
                                      if ((l7 == l13)) goto __end_B48;
                                      l4 = (*(int8_t*)(mem + (l7)) | (l4 << 8));
                                      l6 = (l6 << 8);
                                      l7 = (l7 + 1);
                                    }
                                    do { /* block B79 */
                                      l3 = *(int16_t*)(mem + (l8) + 2);
                                      l5 = (l3 * (l6 >> 11));
                                      if ((l5 > l4)) {
                                        ((S_24_l8*)l8)->f_off_2 = (l3 + ((2048 - l3) >> 5));
                                        l8 = ((l8 + (l10 << 4)) + 260);
                                        l19 = 3;
                                        l10 = 1;
                                        goto __end_B79; /* break */
                                      }
                                      ((S_24_l8*)l8)->f_off_2 = (l3 - (l3 >> 5));
                                      l8 = (l8 + 516);
                                      l4 = (l4 - l5);
                                      l5 = (l6 - l5);
                                      l19 = 8;
                                      l10 = 0;
                                    } while (0);
                                    __end_B79: ;
                                  } while (0);
                                  __end_B76: ;
                                  if ((l5 > 16777215)) {
                                    t11 = l7;
                                  } else {
                                    if ((l7 == l13)) goto __end_B48;
                                    l5 = (l5 << 8);
                                    l4 = (*(int8_t*)(mem + (l7)) | (l4 << 8));
                                    t11 = (l7 + 1);
                                  }
                                  l3 = t11;
                                  do { /* block B82 */
                                    l7 = *(int16_t*)(mem + (l8) + 2);
                                    l6 = (l7 * (l5 >> 11));
                                    if ((l6 > l4)) {
                                      l5 = (l7 + ((2048 - l7) >> 5));
                                      goto __end_B82; /* break */
                                    }
                                    l4 = (l4 - l6);
                                    l6 = (l5 - l6);
                                    l5 = (l7 - (l7 >> 5));
                                  } while (0);
                                  __end_B82: ;
                                  l7 = 3;
                                  ((S_24_l8*)l8)->f_off_2 = l5;
                                  if ((l6 <= 16777215)) {
                                    if ((l3 == l13)) goto __end_B48;
                                    l6 = (l6 << 8);
                                    l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                    l3 = (l3 + 1);
                                  }
                                  do { /* block B85 */
                                    l7 = (l7 << 1);
                                    l21 = (l8 + l7);
                                    l12 = *(int16_t*)(mem + (l21));
                                    l5 = (l12 * (l6 >> 11));
                                    if ((l5 <= l4)) {
                                      l7 = (l7 | 1);
                                      l4 = (l4 - l5);
                                      l5 = (l6 - l5);
                                      goto __end_B85; /* break */
                                    }
                                  } while (0);
                                  __end_B85: ;
                                  l6 = (l12 + ((2048 - l12) >> 5));
                                  ((S_24_l21*)l21)->f_off_0 = l6;
                                  if ((l5 <= 16777215)) {
                                    if ((l3 == l13)) goto __end_B48;
                                    l5 = (l5 << 8);
                                    l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                    l3 = (l3 + 1);
                                  }
                                  do { /* block B88 */
                                    l7 = (l7 << 1);
                                    l21 = (l8 + l7);
                                    l12 = *(int16_t*)(mem + (l21));
                                    l6 = (l12 * (l5 >> 11));
                                    if ((l6 <= l4)) {
                                      l7 = (l7 | 1);
                                      l4 = (l4 - l6);
                                      l6 = (l5 - l6);
                                      goto __end_B88; /* break */
                                    }
                                  } while (0);
                                  __end_B88: ;
                                  l5 = (l12 + ((2048 - l12) >> 5));
                                  ((S_24_l21*)l21)->f_off_0 = l5;
                                  if ((!l10)) {
                                    if ((l6 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l6 = (l6 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B92 */
                                      l7 = (l7 << 1);
                                      l12 = (l8 + l7);
                                      l10 = *(int16_t*)(mem + (l12));
                                      l5 = (l10 * (l6 >> 11));
                                      if ((l5 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l5);
                                        l5 = (l6 - l5);
                                        goto __end_B92; /* break */
                                      }
                                    } while (0);
                                    __end_B92: ;
                                    l6 = (l10 + ((2048 - l10) >> 5));
                                    ((S_24_l12*)l12)->f_off_0 = l6;
                                    if ((l5 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l5 = (l5 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B95 */
                                      l7 = (l7 << 1);
                                      l12 = (l8 + l7);
                                      l10 = *(int16_t*)(mem + (l12));
                                      l6 = (l10 * (l5 >> 11));
                                      if ((l6 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l6);
                                        l6 = (l5 - l6);
                                        goto __end_B95; /* break */
                                      }
                                    } while (0);
                                    __end_B95: ;
                                    l5 = (l10 + ((2048 - l10) >> 5));
                                    ((S_24_l12*)l12)->f_off_0 = l5;
                                    if ((l6 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l6 = (l6 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B98 */
                                      l7 = (l7 << 1);
                                      l12 = (l8 + l7);
                                      l10 = *(int16_t*)(mem + (l12));
                                      l5 = (l10 * (l6 >> 11));
                                      if ((l5 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l5);
                                        l5 = (l6 - l5);
                                        goto __end_B98; /* break */
                                      }
                                    } while (0);
                                    __end_B98: ;
                                    l6 = (l10 + ((2048 - l10) >> 5));
                                    ((S_24_l12*)l12)->f_off_0 = l6;
                                    if ((l5 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l5 = (l5 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B101 */
                                      l7 = (l7 << 1);
                                      l12 = (l8 + l7);
                                      l6 = *(int16_t*)(mem + (l12));
                                      l10 = (l6 * (l5 >> 11));
                                      if ((l10 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l10);
                                        l10 = (l5 - l10);
                                        goto __end_B101; /* break */
                                      }
                                    } while (0);
                                    __end_B101: ;
                                    l5 = (l6 + ((2048 - l6) >> 5));
                                    ((S_24_l12*)l12)->f_off_0 = l5;
                                    if ((l10 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l10 = (l10 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B104 */
                                      l7 = (l7 << 1);
                                      l8 = (l8 + l7);
                                      l5 = *(int16_t*)(mem + (l8));
                                      l6 = (l5 * (l10 >> 11));
                                      if ((l6 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l6);
                                        l6 = (l10 - l6);
                                        goto __end_B104; /* break */
                                      }
                                    } while (0);
                                    __end_B104: ;
                                    l5 = (l5 + ((2048 - l5) >> 5));
                                    ((S_24_l8*)l8)->f_off_0 = l5;
                                  }
                                  l19 = ((16 + (-1 << l19)) + l7);
                                  if ((l17 <= 3)) {
                                    if ((l6 > 16777215)) {
                                      t12 = l3;
                                    } else {
                                      if ((l3 == l13)) goto __end_B48;
                                      l6 = (l6 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      t12 = (l3 + 1);
                                    }
                                    l0 = t12;
                                    do { /* block B108 */
                                      l8 = (l35 + (((l19 >= 3) ? 3 : l19) << 7));
                                      l7 = *(int16_t*)(mem + (l8) + 2);
                                      l5 = (l7 * (l6 >> 11));
                                      if ((l5 > l4)) {
                                        l3 = 2;
                                        goto __end_B108; /* break */
                                      }
                                      l4 = (l4 - l5);
                                      l5 = (l6 - l5);
                                      l3 = 3;
                                    } while (0);
                                    __end_B108: ;
                                    l7 = (l7 - (l7 >> 5));
                                    ((S_24_l8*)l8)->f_off_2 = l7;
                                    if ((l5 <= 16777215)) {
                                      if ((l0 == l13)) goto __end_B48;
                                      l5 = (l5 << 8);
                                      l4 = (*(int8_t*)(mem + (l0)) | (l4 << 8));
                                      l0 = (l0 + 1);
                                    }
                                    do { /* block B111 */
                                      l7 = (l3 << 1);
                                      l10 = (l8 + l7);
                                      l3 = *(int16_t*)(mem + (l10));
                                      l6 = (l3 * (l5 >> 11));
                                      if ((l6 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l6);
                                        l6 = (l5 - l6);
                                        goto __end_B111; /* break */
                                      }
                                    } while (0);
                                    __end_B111: ;
                                    l3 = (l3 + ((2048 - l3) >> 5));
                                    ((S_24_l10*)l10)->f_off_0 = l3;
                                    if ((l6 > 16777215)) {
                                      t13 = l0;
                                    } else {
                                      if ((l0 == l13)) goto __end_B48;
                                      l6 = (l6 << 8);
                                      l4 = (*(int8_t*)(mem + (l0)) | (l4 << 8));
                                      t13 = (l0 + 1);
                                    }
                                    l3 = t13;
                                    do { /* block B114 */
                                      l7 = (l7 << 1);
                                      l10 = (l8 + l7);
                                      l0 = *(int16_t*)(mem + (l10));
                                      l5 = (l0 * (l6 >> 11));
                                      if ((l5 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l5);
                                        l5 = (l6 - l5);
                                        goto __end_B114; /* break */
                                      }
                                    } while (0);
                                    __end_B114: ;
                                    l0 = (l0 + ((2048 - l0) >> 5));
                                    ((S_24_l10*)l10)->f_off_0 = l0;
                                    if ((l5 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l5 = (l5 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B117 */
                                      l7 = (l7 << 1);
                                      l10 = (l8 + l7);
                                      l0 = *(int16_t*)(mem + (l10));
                                      l6 = (l0 * (l5 >> 11));
                                      if ((l6 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l6);
                                        l6 = (l5 - l6);
                                        goto __end_B117; /* break */
                                      }
                                    } while (0);
                                    __end_B117: ;
                                    l0 = (l0 + ((2048 - l0) >> 5));
                                    ((S_24_l10*)l10)->f_off_0 = l0;
                                    if ((l6 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l6 = (l6 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B120 */
                                      l7 = (l7 << 1);
                                      l10 = (l8 + l7);
                                      l0 = *(int16_t*)(mem + (l10));
                                      l5 = (l0 * (l6 >> 11));
                                      if ((l5 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l5);
                                        l5 = (l6 - l5);
                                        goto __end_B120; /* break */
                                      }
                                    } while (0);
                                    __end_B120: ;
                                    l0 = (l0 + ((2048 - l0) >> 5));
                                    ((S_24_l10*)l10)->f_off_0 = l0;
                                    if ((l5 <= 16777215)) {
                                      if ((l3 == l13)) goto __end_B48;
                                      l5 = (l5 << 8);
                                      l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                      l3 = (l3 + 1);
                                    }
                                    do { /* block B123 */
                                      l7 = (l7 << 1);
                                      l8 = (l8 + l7);
                                      l0 = *(int16_t*)(mem + (l8));
                                      l6 = (l0 * (l5 >> 11));
                                      if ((l6 <= l4)) {
                                        l7 = (l7 | 1);
                                        l4 = (l4 - l6);
                                        l6 = (l5 - l6);
                                        goto __end_B123; /* break */
                                      }
                                    } while (0);
                                    __end_B123: ;
                                    l0 = (l0 + ((2048 - l0) >> 5));
                                    ((S_24_l8*)l8)->f_off_0 = l0;
                                    l10 = (l7 + -64);
                                    if ((l7 >= 68)) {
                                      l0 = (l10 >> 1);
                                      l8 = ((l7 & 1) | 2);
                                      do { /* block B126 */
                                        if ((l10 <= 13)) {
                                          l0 = (l0 - 1);
                                          l10 = (l8 << l0);
                                          goto __end_B126; /* break */
                                        }
                                        l5 = (l0 - 5);
                                        __head_L128: while (1) { /* loop L128 */
                                          if ((l6 <= 16777215)) {
                                            if ((l3 == l13)) goto __end_B48;
                                            l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                            l3 = (l3 + 1);
                                            t14 = (l6 << 8);
                                          } else {
                                            t14 = l6;
                                          }
                                          l6 = (t14 >> 1);
                                          l0 = (l6 <= l4);
                                          l8 = (l0 | (l8 << 1));
                                          l4 = (l4 - (l0 ? l6 : 0));
                                          l5 = (l5 - 1);
                                          if (l5) goto __head_L128;
                                        }
                                        __end_L128: ;
                                        l0 = 4;
                                        l10 = (l8 << 4);
                                      } while (0);
                                      __end_B126: ;
                                      l24 = l34;
                                      l7 = 1;
                                      l8 = 1;
                                      __head_L130: while (1) { /* loop L130 */
                                        if ((l6 > 16777215)) {
                                          t15 = l6;
                                        } else {
                                          if ((l3 == l13)) goto __end_B48;
                                          l4 = (*(int8_t*)(mem + (l3)) | (l4 << 8));
                                          l3 = (l3 + 1);
                                          t15 = (l6 << 8);
                                        }
                                        l5 = t15;
                                        do { /* block B132 */
                                          l8 = (l8 << 1);
                                          l21 = (l24 + l8);
                                          l12 = *(int16_t*)(mem + (l21));
                                          l6 = (l12 * (l5 >> 11));
                                          if ((l6 > l4)) {
                                            goto __end_B132; /* break */
                                          }
                                          l10 = (l7 | l10);
                                          l8 = (l8 | 1);
                                          l4 = (l4 - l6);
                                          l6 = (l5 - l6);
                                        } while (0);
                                        __end_B132: ;
                                        l5 = (l12 - (l12 >> 5));
                                        ((S_24_l21*)l21)->f_off_0 = l5;
                                        l7 = (l7 << 1);
                                        l0 = (l0 - 1);
                                        if (l0) goto __head_L130;
                                      }
                                      __end_L130: ;
                                    }
                                    l0 = (l10 + 1);
                                    if ((!l0)) goto __end_B52;
                                    l17 = (l17 + 7);
                                  }
                                  if ((l0 > l16)) goto __end_B48;
                                  l5 = (l19 + 2);
                                  __head_L134: while (1) { /* loop L134 */
                                    do { /* block B135 */
                                      l8 = *(int8_t*)(mem + ((l23 + (l16 - l0))));
                                      *(int8_t*)(mem + ((l16 + l23))) = l8;
                                      l16 = (l16 + 1);
                                      l5 = (l5 - 1);
                                      if ((!l5)) goto __end_B135;
                                      if ((l16 != -1)) goto __head_L134;
                                    } while (0);
                                    __end_B135: ;
                                  }
                                  __end_L134: ;
                                  l7 = l0;
                                  l0 = l9;
                                  goto __end_B53; /* break */
                                } while (0);
                                __end_B56: ;
                                if ((l8 > 255)) goto __end_B54;
                              } while (0);
                              __end_B55: ;
                              __head_L136: while (1) { /* loop L136 */
                                if ((l6 > 16777215)) {
                                  t16 = l6;
                                } else {
                                  if ((l5 == l13)) goto __end_B48;
                                  l4 = (*(int8_t*)(mem + (l5)) | (l4 << 8));
                                  l5 = (l5 + 1);
                                  t16 = (l6 << 8);
                                }
                                l3 = t16;
                                do { /* block B138 */
                                  l8 = (l8 << 1);
                                  l10 = (l12 + l8);
                                  l9 = *(int16_t*)(mem + (l10));
                                  l6 = (l9 * (l3 >> 11));
                                  if ((l6 > l4)) {
                                    goto __end_B138; /* break */
                                  }
                                  l8 = (l8 | 1);
                                  l4 = (l4 - l6);
                                  l6 = (l3 - l6);
                                } while (0);
                                __end_B138: ;
                                l3 = (l9 - (l9 >> 5));
                                ((S_24_l10*)l10)->f_off_0 = l3;
                                if ((l8 < 256)) goto __head_L136;
                              }
                              __end_L136: ;
                            } while (0);
                            __end_B54: ;
                            *(int8_t*)(mem + ((l16 + l23))) = l8;
                            l16 = (l16 + 1);
                            if ((l17 < 4)) {
                              l17 = 0;
                              l3 = l5;
                              goto __end_B53; /* break */
                            }
                            if ((l17 <= 9)) {
                              l17 = (l17 - 3);
                              l3 = l5;
                              goto __end_B53; /* break */
                            }
                            l17 = (l17 - 6);
                            l3 = l5;
                          } while (0);
                          __end_B53: ;
                          if ((l16 != -1)) goto __head_L49;
                        } while (0);
                        __end_B52: ;
                      }
                      __end_L49: ;
                      if ((l6 <= 16777215)) {
                        if ((l3 == l13)) goto __end_B48;
                        t17 = (l3 + 1);
                      } else {
                        t17 = l3;
                      }
                      ((S_24_l11*)l11)->f_off_8 = (t17 - l26);
                      ((S_24_l11*)l11)->f_off_4 = l16;
                      goto __end_B45; /* break */
                    } while (0);
                    __end_B48: ;
                  } while (0);
                  __end_B45: ;
                }
                __end_L44: ;
                l1 = 0;
                l9 = *(int32_t*)(mem + (l11) + 24);
                goto __end_B41; /* break */
              } while (0);
              __end_B43: ;
              l9 = *(int32_t*)(mem + (l11) + 24);
            }
            l3 = *(int32_t*)(mem + (l11) + 28);
            l0 = (*(int32_t*)(mem + (l11) + 32) - l3);
            l1 = f(l0);
            *(int32_t*)(mem + (l2)) = l0;
            if ((!l0)) goto __end_B41;
            memory.copy(l1, l3, l0);
          } while (0);
          __end_B41: ;
          g(l9);
        }
        l0 = *(int32_t*)(mem + (l11) + 28);
        if ((!l0)) goto __end_B17;
        ((S_24_l11*)l11)->f_off_32 = l0;
        g(l0);
      } while (0);
      __end_B17: ;
      if ((!l22)) goto __end_B15;
    } while (0);
    __end_B16: ;
    ((S_24_l11*)l11)->f_off_44 = l22;
    g(l22);
  } while (0);
  __end_B15: ;
  if (l20) {
    ((S_24_l11*)l11)->f_off_56 = l20;
    g(l20);
  }
  g0 = (l11 - -64);
  return l1;
}

void f25(int32_t p0, S_25_l1* p1, int32_t p2, int32_t p3, int32_t p4) {
  int32_t l0 = p0;
  S_25_l1* l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  int32_t l4 = p4;
  /* hint: l1 is S_25_l1* (12 fields) */
  if (f3(l0, *(int32_t*)(mem + (l1) + 8), l4)) {
    do { /* block B2 */
      if ((l2 != *(int32_t*)(mem + (l1) + 4))) goto __end_B2;
      if ((*(int32_t*)(mem + (l1) + 28) == 1)) goto __end_B2;
      ((S_25_l1*)l1)->f_off_28 = l3;
    } while (0);
    __end_B2: ;
    return;
  }
  do { /* block B3 */
    if ((!f3(l0, *(int32_t*)(mem + (l1)), l4))) goto __end_B3;
    do { /* block B4 */
      if ((*(int32_t*)(mem + (l1) + 16) != l2)) {
        if ((l2 != *(int32_t*)(mem + (l1) + 20))) goto __end_B4;
      }
      if ((l3 != 1)) goto __end_B3;
      ((S_25_l1*)l1)->f_off_32 = 1;
      return;
    } while (0);
    __end_B4: ;
    ((S_25_l1*)l1)->f_off_20 = l2;
    ((S_25_l1*)l1)->f_off_32 = l3;
    ((S_25_l1*)l1)->f_off_40 = (*(int32_t*)(mem + (l1) + 40) + 1);
    do { /* block B6 */
      if ((*(int32_t*)(mem + (l1) + 36) != 1)) goto __end_B6;
      if ((*(int32_t*)(mem + (l1) + 24) != 2)) goto __end_B6;
      ((S_25_l1*)l1)->f_off_54 = 1;
    } while (0);
    __end_B6: ;
    ((S_25_l1*)l1)->f_off_44 = 4;
  } while (0);
  __end_B3: ;
}

void f26(S_26_l0* p0 /* table index */, S_26_l1* p1, int32_t p2, int32_t p3, int32_t p4) {
  S_26_l0* l0 = p0;
  S_26_l1* l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  int32_t l4 = p4;
  /* hint: l0 is S_26_l0* (2 fields) */
  /* hint: l1 is S_26_l1* (14 fields) */
  if (f3(l0, *(int32_t*)(mem + (l1) + 8), l4)) {
    do { /* block B2 */
      if ((l2 != *(int32_t*)(mem + (l1) + 4))) goto __end_B2;
      if ((*(int32_t*)(mem + (l1) + 28) == 1)) goto __end_B2;
      ((S_26_l1*)l1)->f_off_28 = l3;
    } while (0);
    __end_B2: ;
    return;
  }
  do { /* block B3 */
    if (f3(l0, *(int32_t*)(mem + (l1)), l4)) {
      do { /* block B5 */
        if ((*(int32_t*)(mem + (l1) + 16) != l2)) {
          if ((l2 != *(int32_t*)(mem + (l1) + 20))) goto __end_B5;
        }
        if ((l3 != 1)) goto __end_B3;
        ((S_26_l1*)l1)->f_off_32 = 1;
        return;
      } while (0);
      __end_B5: ;
      ((S_26_l1*)l1)->f_off_32 = l3;
      do { /* block B7 */
        if ((*(int32_t*)(mem + (l1) + 44) == 4)) goto __end_B7;
        ((S_26_l1*)l1)->f_off_52 = 0;
        l0 = *(int32_t*)(mem + (l0) + 8);
        table_call(l0)(l1, l2, l2, 1, l4, *(int32_t*)(mem + (*(int32_t*)(mem + (l0))) + 20));
        if ((*(int8_t*)(mem + (l1) + 53) == 1)) {
          ((S_26_l1*)l1)->f_off_44 = 3;
          if ((!*(int8_t*)(mem + (l1) + 52))) goto __end_B7;
          goto __end_B3; /* break */
        }
        ((S_26_l1*)l1)->f_off_44 = 4;
      } while (0);
      __end_B7: ;
      ((S_26_l1*)l1)->f_off_20 = l2;
      ((S_26_l1*)l1)->f_off_40 = (*(int32_t*)(mem + (l1) + 40) + 1);
      if ((*(int32_t*)(mem + (l1) + 36) != 1)) goto __end_B3;
      if ((*(int32_t*)(mem + (l1) + 24) != 2)) goto __end_B3;
      ((S_26_l1*)l1)->f_off_54 = 1;
      return;
    }
    l0 = *(int32_t*)(mem + (l0) + 8);
    table_call(l0)(l1, l2, l3, l4, *(int32_t*)(mem + (*(int32_t*)(mem + (l0))) + 24));
  } while (0);
  __end_B3: ;
}

void f27(S_27_l0* p0 /* table index */, int32_t p1, int32_t p2, int32_t p3) {
  S_27_l0* l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  /* hint: l0 is S_27_l0* (2 fields) */
  if (f3(l0, *(int32_t*)(mem + (l1) + 8), 0)) {
    f16(l1, l2, l3);
    return;
  }
  l0 = *(int32_t*)(mem + (l0) + 8);
  table_call(l0)(l1, l2, l3, *(int32_t*)(mem + (*(int32_t*)(mem + (l0))) + 28));
}

void f28(int32_t p0, int32_t p1, int32_t p2, int32_t p3) {
  int32_t l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = p3;
  if (f3(l0, *(int32_t*)(mem + (l1) + 8), 0)) {
    f16(l1, l2, l3);
  }
}

int32_t f29(int32_t p0, int32_t p1, int32_t p2) {
  int32_t l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = p2;
  int32_t l3 = 0;
  int32_t l4 = 0;
  int32_t l5 = 0;
  int32_t l6 = 0;
  int32_t l7 = 0;
  int32_t l8 = 0;
  /* hint: l3 is S_29_l3* (15 fields) */
  /* hint: l4 is S_29_l4* (10 fields) */
  /* hint: l6 is S_29_l6* (3 fields) */
  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */
  l4 = (g0 - 80);
  g0 = l4;
  do { /* block B1 */
    do { /* block B2 */
      if (f3(l0, l1, 0)) goto __end_B2;
      if ((!l1)) goto __end_B2;
      l6 = (g0 - 16);
      g0 = l6;
      l3 = *(int32_t*)(mem + (l1));
      l5 = *(int32_t*)(mem + ((l3 - 8)));
      ((S_29_l6*)l6)->f_off_12 = l5;
      ((S_29_l6*)l6)->f_off_4 = (l1 + l5);
      ((S_29_l6*)l6)->f_off_8 = *(int32_t*)(mem + ((l3 - 4)));
      l3 = *(int32_t*)(mem + (l6) + 8);
      l5 = f3(l3, 1240, 0);
      l7 = *(int32_t*)(mem + (l6) + 4);
      do { /* block B3 */
        if (l5) {
          l1 = *(int32_t*)(mem + (l6) + 12);
          l3 = (g0 + -64);
          g0 = l3;
          g0 = (l3 - -64);
          l3 = (l1 ? 0 : l7);
          goto __end_B3; /* break */
        }
        l5 = l3;
        l3 = (g0 + -64);
        g0 = l3;
        if ((l1 >= l7)) {
          /* split i64 init 0x0 into 2xi32 LE */
          ((S_29_l3*)l3)->f_off_28 = 0;
          ((S_29_l3*)l3)->f_off_32 = 0;
          /* split i64 init 0x0 into 2xi32 LE */
          ((S_29_l3*)l3)->f_off_36 = 0;
          ((S_29_l3*)l3)->f_off_40 = 0;
          ((S_29_l3*)l3)->f_off_44 = 0ll;
          /* split i64 init 0x0 into 2xi32 LE */
          ((S_29_l3*)l3)->f_off_20 = 0;
          ((S_29_l3*)l3)->f_off_24 = 0;
          ((S_29_l3*)l3)->f_off_16 = 0;
          ((S_29_l3*)l3)->f_off_12 = 1240;
          ((S_29_l3*)l3)->f_off_4 = l5;
          ((S_29_l3*)l3)->f_off_60 = 0;
          ((S_29_l3*)l3)->f_off_52 = 72057594037927937ll;
          ((S_29_l3*)l3)->f_off_8 = l1;
          table_call(l5)((l3 + 4), l7, l7, 1, 0, *(int32_t*)(mem + (*(int32_t*)(mem + (l5))) + 20));
          l8 = (*(int32_t*)(mem + (l3) + 28) ? l1 : 0);
        }
        g0 = (l3 - -64);
        l3 = l8;
        if (l3) goto __end_B3;
        l3 = (g0 + -64);
        g0 = l3;
        ((S_29_l3*)l3)->f_off_16 = 0;
        ((S_29_l3*)l3)->f_off_12 = 1192;
        ((S_29_l3*)l3)->f_off_8 = l1;
        ((S_29_l3*)l3)->f_off_4 = 1240;
        l1 = 0;
        memory.fill((l3 + 20), 0, 39);
        ((S_29_l3*)l3)->f_off_60 = 0;
        ((S_29_l3*)l3)->f_off_59 = 1;
        table_call(l5)((l3 + 4), l7, 1, 0, *(int32_t*)(mem + (*(int32_t*)(mem + (l5))) + 24));
        do { /* block B6 */
          do { /* block B7 */
            do { /* block B8 */
              switch (*(int32_t*)(mem + (l3) + 40)) {
                case 0: goto __end_B8;
                case 1: goto __end_B7;
                default: goto __end_B6;
              }
            } while (0);
            __end_B8: ;
            l1 = ((*(int32_t*)(mem + (l3) + 44) == 1) ? ((*(int32_t*)(mem + (l3) + 32) == 1) ? ((*(int32_t*)(mem + (l3) + 36) == 1) ? *(int32_t*)(mem + (l3) + 24) : 0) : 0) : 0);
            goto __end_B6; /* break */
          } while (0);
          __end_B7: ;
          if ((*(int32_t*)(mem + (l3) + 28) != 1)) {
            if (*(int32_t*)(mem + (l3) + 44)) goto __end_B6;
            if ((*(int32_t*)(mem + (l3) + 32) != 1)) goto __end_B6;
            if ((*(int32_t*)(mem + (l3) + 36) != 1)) goto __end_B6;
          }
          l1 = *(int32_t*)(mem + (l3) + 20);
        } while (0);
        __end_B6: ;
        g0 = (l3 - -64);
        l3 = l1;
      } while (0);
      __end_B3: ;
      g0 = (l6 + 16);
      if ((!l3)) goto __end_B2;
      l1 = *(int32_t*)(mem + (l2));
      if ((!l1)) goto __end_B1;
      memory.fill((l4 + 24), 0, 56);
      ((S_29_l4*)l4)->f_off_75 = 1;
      ((S_29_l4*)l4)->f_off_32 = -1;
      ((S_29_l4*)l4)->f_off_28 = l0;
      ((S_29_l4*)l4)->f_off_20 = l3;
      ((S_29_l4*)l4)->f_off_68 = 1;
      table_call(l3)((l4 + 20), l1, 1, *(int32_t*)(mem + (*(int32_t*)(mem + (l3))) + 28));
      l0 = *(int32_t*)(mem + (l4) + 44);
      if ((l0 == 1)) {
        *(int32_t*)(mem + (l2)) = *(int32_t*)(mem + (l4) + 36);
      }
    } while (0);
    __end_B2: ;
    g0 = (l4 + 80);
    return (l0 == 1);
  } while (0);
  __end_B1: ;
  ((S_29_l4*)l4)->f_off_8 = 1156;
  ((S_29_l4*)l4)->f_off_4 = 485;
  ((S_29_l4*)l4)->f_off_0 = 1041;
  f12();
  /* unreachable (trap) */
  __builtin_trap();
  return 0; /* TODO: missing result int32_t */
}

void e() {
}

/* data segments */
/* data0 @Some(1024) (668 bytes, binary) */
/* data1 @Some(1696) (3 bytes, binary) */
/* tables */
/* table0: [NULL, f10, f14, f10, f7, f17, f17, f29, f22, f25, f28, f7, f23, f26, f27, f7, f20, f7, f19, f7, f21, f13, f18, f13] */
