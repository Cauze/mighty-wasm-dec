#include <stdint.h>
#include <stddef.h>
#include <math.h>
typedef unsigned __int128 __v128_u __attribute__((vector_size(16)));
typedef __v128_u __v128;
static uint8_t *mem = 0; /* wasm linear memory base (host provides) */
static uint32_t memory_size(uint32_t m) { (void)m; return 0; }
static uint32_t memgrow(uint32_t d) { (void)d; return (uint32_t)-1; }

/* imports (host-provided) */
void wasi_snapshot_preview1_proc_exit(int32_t a0); /* wasi_snapshot_preview1.wasi_snapshot_preview1_proc_exit */

/* globals */
static int32_t /*mut*/ g0 = 0;

void __wasm_call_ctors();
void fill_blob(S_2_l0* p0, int32_t p1);
void copy_blob(S_3_l0* p0, S_3_l1* p1);
int32_t __original_main();
void _start();
void dummy();
void libc_exit_fini();
void exit(int32_t p0);
void _Exit(int32_t p0);
void _emscripten_stack_restore(int32_t p0);
int32_t emscripten_stack_get_current();

/* recovered from fill_blob via base l0 */
typedef struct S_2_l0 {
  int64_t f_off_0; /* +0 w8 x4 */
  int64_t f_off_4; /* +4 w8 x1 */
} S_2_l0;
/* recovered from copy_blob via base l0 */
typedef struct S_3_l0 {
  int64_t f_off_0; /* +0 w8 x5 */
} S_3_l0;
/* S_3_l1 (f3:l1) aliases S_3_l0 */
typedef S_3_l0 S_3_l1;
/* recovered from __original_main via base l0 */
typedef struct S_4_l0 {
  int32_t f_off_8; /* +8 w4 x1 */
  int32_t f_off_40; /* +40 w4 x1 */
} S_4_l0;

void __wasm_call_ctors() {
}

void fill_blob(S_2_l0* p0, int32_t p1) {
  S_2_l0* l0 = p0;
  int32_t l1 = p1;
  int64_t l2 = 0;
  /* hint: l0 is S_2_l0* (2 fields) */
  ((S_2_l0*)l0)->f_off_0 = l1;
  l2 = ((((i64)l1) & 255ll) * 72340172838076673ll);
  *(int64_t*)(mem + ((l0 + 28))) = l2;
  *(int64_t*)(mem + ((l0 + 20))) = l2;
  *(int64_t*)(mem + ((l0 + 12))) = l2;
  ((S_2_l0*)l0)->f_off_4 = l2;
}

void copy_blob(S_3_l0* p0, S_3_l1* p1) {
  S_3_l0* l0 = p0;
  S_3_l1* l1 = p1;
  /* hint: l0 is S_3_l0* (1 fields) */
  /* hint: l1 is S_3_l1* (1 fields) */
  ((S_3_l0*)l0)->f_off_0 = *(int64_t*)(mem + (l1));
  *(int32_t*)(mem + ((l0 + 32))) = *(int32_t*)(mem + ((l1 + 32)));
  *(int64_t*)(mem + ((l0 + 24))) = *(int64_t*)(mem + ((l1 + 24)));
  *(int64_t*)(mem + ((l0 + 16))) = *(int64_t*)(mem + ((l1 + 16)));
  *(int64_t*)(mem + ((l0 + 8))) = *(int64_t*)(mem + ((l1 + 8)));
}

int32_t __original_main() {
  int32_t l0 = 0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  /* hint: l0 is S_4_l0* (2 fields) */
  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */
  l0 = (g0 - 80);
  g0 = l0;
  fill_blob((l0 + 44), 3);
  copy_blob((l0 + 8), (l0 + 44));
  g0 = (l0 + 80);
  return (*(int32_t*)(mem + (l0) + 40) + *(int32_t*)(mem + (l0) + 8));
}

void _start() {
  /* block */ {
    __wasm_call_ctors();
  }
  exit(__original_main());
  /* unreachable (trap) */
  __builtin_trap();
}

void dummy() {
}

void libc_exit_fini() {
  int32_t l0 = 0;
  l0 = 0;
  do { /* block B1 */
    goto __end_B1; /* break */
    __head_L2: while (1) { /* loop L2 */
      l0 = (l0 + -4);
      table_call(*(int32_t*)(mem + (l0)))();
      if ((l0 > 0)) goto __head_L2;
    }
    __end_L2: ;
  } while (0);
  __end_B1: ;
  dummy();
}

void exit(int32_t p0) {
  int32_t l0 = p0;
  dummy();
  libc_exit_fini();
  dummy();
  _Exit(l0);
  /* unreachable (trap) */
  __builtin_trap();
}

void _Exit(int32_t p0) {
  int32_t l0 = p0;
  wasi_snapshot_preview1_proc_exit(l0);
  /* unreachable (trap) */
  __builtin_trap();
}

void _emscripten_stack_restore(int32_t p0) {
  int32_t l0 = p0;
  g0 = l0;
}

int32_t emscripten_stack_get_current() {
  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */
  return g0;
}

/* tables */
/* table0: [NULL, __wasm_call_ctors] */
