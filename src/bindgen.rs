//! The bindgen generated definitions
//!
//! These are kept in a separate module (rather than including the bindings right in the lib.rs)
//! because bindgen is a bit sensitive to some of its assignments being defined differently

use crate::libc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
