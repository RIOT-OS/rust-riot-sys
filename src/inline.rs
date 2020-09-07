// While it'd be tempting to clean them all up in RIOT by a large constification haul, now is not
// the time for that
#![allow(unused_mut)]
// Probably __attribute__((used)) doesn't get translated
#![allow(unused)]

/// Contains header code converted to Rust by C2Rust
///
/// Types in here are distinct from those created in the main module (using bindgen); unifying
/// those will be part of [bindgen's #1334], but it's a long way there.
///
/// [bindgen's #1334]: https://github.com/rust-lang/rust-bindgen/issues/1344
///
/// Use these functions through the re-export in the main module, for the C headers may flip-flop
/// between static inline and linked.

use crate::libc;

include!(concat!(env!("OUT_DIR"), "/riot_c2rust_replaced.rs"));
