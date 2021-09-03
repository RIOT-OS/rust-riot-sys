// If this is not included, components start disagreeingon whether timspec is
// defined or not
#include <fcntl.h>

// Copied over from newlib's stdatomic.h -- not sure why it's not included, but
// that's currently needed to get mutexes to build.
#include <stdint.h>
typedef _Atomic(int_least16_t)      atomic_int_least16_t;

// Workarounds for https://github.com/rust-lang/rust-bindgen/issues/1636
// (only needed when building for cortex using toolchain=llvm)
#define UINT16_MAX 0xffff
#define UINT32_MAX 0xffffffff

#include "riot-headers.h"
