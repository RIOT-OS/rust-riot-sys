//! Replacements for [core::intrinsics] to work around <https://github.com/immunant/c2rust/issues/372>
// These were observed to be used on riscv with the hello-world example; it's hard to predict when
// they are here, and allowing them to stay does no harm. (Given they are a workaround, they have
// an expiry date anyway).
#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, Ordering};

fn atomic_and_relaxed(dst: *mut u32, src: u32) -> u32 {
    let actual_atomic = unsafe { &*(dst as *mut AtomicU32) };
    actual_atomic.fetch_and(src, Ordering::Relaxed)
}

fn atomic_or_relaxed(dst: *mut u32, src: u32) -> u32 {
    let actual_atomic = unsafe { &*(dst as *mut AtomicU32) };
    actual_atomic.fetch_or(src, Ordering::Relaxed)
}
