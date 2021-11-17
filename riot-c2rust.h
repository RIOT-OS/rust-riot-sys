
// When GCC preprocesses the sources on native, it puts a __float128 into the
// max_align_t which clang does not understand.
#define __float128 long double

// On native, the stdlib inclusion (needed for abort) would make things trip;
// the ones used with the embedded boards is tamer there.
#ifndef BOARD_NATIVE
// Workaround for https://github.com/immunant/c2rust/issues/345
//
// As these are not really in the call tree of any public RIOT function,
// aborting is probably enough.
//
// Their names are changed around in preprocessor because otherwise they'd
// cause a failure at the translation stage already ("Unimplemented builtin
// __builtin_arm_get_fpscr"); this way the error can be delayed and the
// function redirected.
#include <stdlib.h>
#define __builtin_arm_get_fpscr __masked_builtin_arm_get_fpscr
#define __builtin_arm_set_fpscr __masked_builtin_arm_set_fpscr
static inline int __masked_builtin_arm_get_fpscr(void) {
	abort();
}
static inline void __masked_builtin_arm_set_fpscr(int fpscr){
	(void)fpscr;
	abort();
}
#endif

// This is currently the only relevant user of stdatomic.h. As it doesn't
// access its relevant atomic field from static inlines (and thus from built
// Rust) and forbids users from touching it themselves, we can work around
// C2Rust's current inability to do atomics here
//
// Proper fix: resolve https://github.com/immunant/c2rust/issues/293
#define __CLANG_STDATOMIC_H // for clang
#define _STDATOMIC_H // for GCC
#define _STDATOMIC_H_ // for newlib
#define ATOMIC_VAR_INIT(x) x
#define atomic_int_least16_t int_least16_t // FIXME is it?
#include <rmutex.h>
#undef __CLANG_STDATOMIC_H
#undef _STDATOMIC_H_
#undef _STDATOMIC_H
#undef ATOMIC_VAR_INIT
#undef atomic_int_least16_t

// Allow header files that pull in lots of odd stuff but don't depend on
// inlines -- like nimble's host/ble_gap.h -- to opt out of C2Rust altogether
#define IS_C2RUST

#include "riot-headers.h"
