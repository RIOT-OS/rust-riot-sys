#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod libc;

pub mod inline;
pub use inline::*;

// This is not moved into a dedicated linked module that'd be reexported in analogy to the inline,
// for that'd need explicit `pub use linked::mutex_t` etc for every type that's present in both and
// thus not imported for either. As long as this is inlined here, linked types (which are
// predominantly used so far) take precedence automatically.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
