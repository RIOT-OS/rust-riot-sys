// Taken from the no_std tests of bindgen (tests/headers/no-std.h).
//
// For a more correct approach, I'd probably need to take the C compiler used for RIOT into
// account.

#![allow(non_camel_case_types)]

pub use cty::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort,
    // Not even loading size_t and ssize_t as they don't fit with bindgen's mapping anyway
};

// Used to be a dedicated type, pub-used to avoid breaking the API
pub use core::ffi::c_void;

pub use cstr_core::{CStr, FromBytesWithNulError};
