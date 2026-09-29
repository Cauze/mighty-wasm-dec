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
int32_t intops(int32_t p0, int32_t p1);
int64_t llops(int64_t p0, int64_t p1);
int32_t __original_main();
void _start();
void dummy();
void libc_exit_fini();
void exit(int32_t p0);
void _Exit(int32_t p0);
void _emscripten_stack_restore(int32_t p0);
int32_t emscripten_stack_get_current();


void __wasm_call_ctors() {
}

int32_t intops(int32_t p0, int32_t p1) {
  int32_t l0 = p0;
  int32_t l1 = p1;
  int32_t l2 = 0;
  int32_t l3 = 0;
  l2 = 0;
  do { /* block B1 */
    if ((!l1)) goto __end_B1;
    l2 = (l0 / l1);
  } while (0);
  __end_B1: ;
  l2 = 0;
  do { /* block B2 */
    if ((!l1)) goto __end_B2;
    l2 = (l0 % l1);
  } while (0);
  __end_B2: ;
  return ((((((((i32.rotl(l0, 3) + i32.clz(l0)) + ((i32)l0)) + i32.ctz(l1)) + (l1 ^ l0)) + ((i32)l1)) + i32.popcnt((l1 + l0))) + l2) + l2);
}

int64_t llops(int64_t p0, int64_t p1) {
  int64_t l0 = p0;
  int64_t l1 = p1;
  do { /* block B1 */
    do { /* block B2 */
      if (l1) goto __end_B2;
      l1 = 0ll;
      goto __end_B1; /* break */
    } while (0);
    __end_B2: ;
    l1 = (l0 / l1);
  } while (0);
  __end_B1: ;
  return ((((l0 >> 7ll) + (l0 << 40ll)) + (l0 & 281474976710655ll)) + l1);
}

int32_t __original_main() {
  return (intops(100, 7) + ((i32)llops(1000000ll, 3ll)));
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
