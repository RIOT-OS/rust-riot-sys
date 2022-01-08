//! # C2Rust transpiled header contents (static inline functions
//!
//! Types in here are distinct from those created in the main module (using bindgen); unifying
//! those will be part of [bindgen's #1334], but it's a long way there.
//!
//! [bindgen's #1334]: https://github.com/rust-lang/rust-bindgen/issues/1344
//!
//! Use these functions through the re-export in the main module, for the C headers may flip-flop
//! between static inline and linked.
//!
//! ---
//!
//! Some special treatment has been applied in the course of the transpilation process:
//!
//! * All functions were made `pub`
//! * All functions have their `extern "C"` removed. Any C component would already use it via their
//!   original definitions, there is no need to re-export them or to restrain their ABI (as they
//!   are here for efficient inlining into Rust code only).
//! * For C const initializers (eg. `#define MUTEX_INIT { { NULL } }`), there is no way for a
//!   transpiler to recognize which type this is actually for. That information is tracked manually
//!   in `build.rs` as a list of known initializers. They get turned into const functions in the
//!   style of `fn init_MUTEX_INIT() -> mutex_t`.
//!
// While it'd be tempting to clean them all up in RIOT by a large constification haul, now is not
// the time for that
#![allow(unused_mut)]
// Probably __attribute__((used)) doesn't get translated
#![allow(unused)]
// Would be nice if we could only do that for `llvm_asm`
#![allow(deprecated)]

use cty as libc;

use c2rust_bitfields::*;

mod f128 {
    extern "C" {
        pub type f128;
    }
}

include!(concat!(env!("OUT_DIR"), "/riot_c2rust_replaced.rs"));
