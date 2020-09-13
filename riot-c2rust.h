// FIXME: This loses the atomic properties of inlined code, which is obviously bad.
//
// Proper fix: resolve https://github.com/immunant/c2rust/issues/293
//
// Until that's done, used inline functions need to be checked manually for
// whether they do atomic stuff.
#define _Atomic(x) x

#include "riot-headers.h"
