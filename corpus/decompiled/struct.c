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
int32_t move_x(S_2_l0* p0, int32_t p1);
int32_t __original_main();
void _start();
void dummy();
void libc_exit_fini();
void exit(int32_t p0);
void _Exit(int32_t p0);
void _emscripten_stack_restore(int32_t p0);
int32_t emscripten_stack_get_current();

/* recovered from move_x via base l0 */
typedef struct S_2_l0 {
  int32_t f_off_0; /* +0 w4 x2 */
  int32_t f_off_4; /* +4 w4 x2 */
} S_2_l0;

void __wasm_call_ctors() {
}

int32_t move_x(S_2_l0* p0, int32_t p1) {
  S_2_l0* l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = 0;
  /* hint: l0 is S_2_l0* (2 fields) */
  l1 = (*(int32_t*)(mem + (l0)) + l1);
  ((S_2_l0*)l0)->f_off_0 = l1;
  l2 = (*(int32_t*)(mem + (l0) + 4) + 1);
  ((S_2_l0*)l0)->f_off_4 = l2;
  return (l2 + l1);
}

int32_t __original_main() {
  int32_t l0 = 0;
  int32_t l1 = 0;
  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */
  l0 = (g0 - 16);
  g0 = l0;
  *(int64_t*)(mem + (l0) + 8) = 17179869187ll;
  g0 = (l0 + 16);
  return move_x((l0 + 8), 10);
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
