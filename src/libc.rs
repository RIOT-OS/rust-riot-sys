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

/// This is a limited copy of the std::ffi:c_str::CStr struct.
///
/// When https://github.com/Amanieu/cstr_core/issues/18 is resolved, this may switch over to pub
/// using cstr_core instead.
pub struct CStr {
    inner: [c_char],
}

fn strlen(ptr: *const c_char) -> usize {
    let mut len = 0;
    while unsafe { ::core::slice::from_raw_parts(ptr, len + 1) }[len] != 0 {
        len = len + 1;
    }
    len
}

use core::str;
impl CStr {
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a CStr {
        let len = strlen(ptr);
        let ptr = ptr as *const u8;
        CStr::from_bytes_with_nul_unchecked(::core::slice::from_raw_parts(ptr, len as usize + 1))
    }

    pub const unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &CStr {
        &*(bytes as *const [u8] as *const CStr)
    }

    /// Check in advance whether a given byte array is safe to put through
    /// from_bytes_with_nul_unchecked.
    ///
    /// The function may not be overly efficient at run time (the equivalent core_cstr check around
    /// from_bytes_with_nul uses memchr, this here may even have bounds checks), but is const and
    /// can thus become part of the assert in the [`cstr!`] macro, which allows the compiler to do
    /// all checks at build time.
    ///
    /// The result type is kept vague to not interfere with a later port to a pub export from
    /// cstr_core where it's likely to return that crate's error type.
    #[inline]
    pub const fn bytes_are_valid(bytes: &[u8]) -> Result<(), impl core::any::Any> {
        if bytes.len() == 0 || bytes[bytes.len() - 1] != 0 {
            return Err(());
        }
        let mut index = 0;
        // No for loops yet in const functions
        while index < bytes.len() - 1 {
            if bytes[index] == 0 {
                return Err(());
            }
            index += 1;
        }
        Ok(())
    }

    pub const fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    pub fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.inner as *const [c_char] as *const [u8]) }
    }

    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.to_bytes())
    }
}

// End of plain CStr imitation

impl CStr {
    /// This is an experimental variation on from_ptr which allows passing in a reference with a
    /// lifetime which indicates the lifetime the result should have.
    ///
    /// Thus, rather than generating a reference whose lifetime is arbitrary (which it in general is
    /// not), the caller needs to create an indicator like this:
    ///
    /// ```
    /// unsafe extern "C" fn f(argument: *const i8) {
    ///     let marker: ();
    ///     let argument = CStr::from_ptr_with_lifetime(argument, &marker);
    ///     ...
    /// }
    /// ```
    ///
    /// This indicates that the argument pointer is expected to be valid for no longer than a reference
    /// to the marker is valid, which is the duration of the f call.
    #[deprecated(since="0.3.5", note="Not present in core_cstr or stdlib's cstr. Instead, build
                 helpers where you have the actual lifetime to use around.")]
    pub unsafe fn from_ptr_with_lifetime<'a>(ptr: *const c_char, _marker: &'a ()) -> &'a CStr {
        CStr::from_ptr(ptr)
    }
}

// This is similar to the cstr-macro crate definition, but without the std dependency
#[macro_export]
macro_rules! cstr {
    ($s:expr) => {{
        let a = concat!($s, "\0");
        assert!($crate::libc::CStr::bytes_are_valid(a.as_bytes()).is_ok());
        unsafe { $crate::libc::CStr::from_bytes_with_nul_unchecked(a.as_bytes()) }
    }};
}

#[test]
fn test() {
    let a = cstr!("Hello");
    assert!(a.to_bytes_with_nul() == "Hello\0".as_bytes());
}
