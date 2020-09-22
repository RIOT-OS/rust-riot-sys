// When GCC preprocesses the sources on native, it puts a __float128 into the
// max_align_t which clang does not understand.
#define __float128 long double

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

#include "riot-headers.h"
