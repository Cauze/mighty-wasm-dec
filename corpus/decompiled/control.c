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
int32_t control(int32_t p0);
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

int32_t control(int32_t p0) {
  int32_t l0 = p0;
  int32_t l1 = 0;
  int32_t l2 = 0;
  int32_t l3 = 0;
  int32_t l4 = 0;
  int32_t l5 = 0;
  int32_t l6 = 0;
  do { /* block B1 */
    do { /* block B2 */
      if ((l0 > 0)) goto __end_B2;
      l1 = 0;
      goto __end_B1; /* break */
    } while (0);
    __end_B2: ;
    l2 = 0;
    l3 = 0;
    __head_L3: while (1) { /* loop L3 */
      l4 = l3;
      do { /* block B4 */
        do { /* block B5 */
          l5 = l2;
          if ((l5 & 1)) goto __end_B5;
          l2 = l4;
          goto __end_B4; /* break */
        } while (0);
        __end_B5: ;
        l6 = l5;
        l1 = 0;
        l2 = 0;
        do { /* block B6 */
          if ((!l5)) goto __end_B6;
          __head_L7: while (1) { /* loop L7 */
            do { /* block B8 */
              l3 = l6;
              l2 = (l3 + l1);
              if ((l2 <= 50)) goto __end_B8;
              goto __end_B6; /* break */
            } while (0);
            __end_B8: ;
            l6 = (l3 + -1);
            l1 = l2;
            if ((l3 > 1)) goto __head_L7;
          }
          __end_L7: ;
        } while (0);
        __end_B6: ;
        l2 = (((l2 > 25) ? l2 : (0 - l2)) + l4);
      } while (0);
      __end_B4: ;
      l3 = l2;
      l1 = l3;
      l6 = (l5 + 1);
      l2 = l6;
      if ((l6 != l0)) goto __head_L3;
    }
    __end_L3: ;
  } while (0);
  __end_B1: ;
  return l1;
}

int32_t __original_main() {
  return control(12);
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
