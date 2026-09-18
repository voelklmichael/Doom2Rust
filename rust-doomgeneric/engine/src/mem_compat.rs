//! Drop-in replacements for the handful of libc memory primitives the
//! transpiled code still calls, implemented on top of `core`/`std` instead
//! of linking against libc.
#![allow(unsafe_code)]
use crate::stdint_types::size_t;
use std::alloc::Layout;

pub unsafe fn memcpy(
    dest: *mut ::core::ffi::c_void,
    src: *const ::core::ffi::c_void,
    n: size_t,
) -> *mut ::core::ffi::c_void {
    ::core::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
    dest
}

pub unsafe fn memset(
    s: *mut ::core::ffi::c_void,
    c: ::core::ffi::c_int,
    n: size_t,
) -> *mut ::core::ffi::c_void {
    ::core::ptr::write_bytes(s as *mut u8, c as u8, n);
    s
}

pub unsafe fn malloc(size: size_t) -> *mut ::core::ffi::c_void {
    if size == 0 {
        return ::core::ptr::null_mut();
    }
    let layout = Layout::from_size_align(size, ::core::mem::align_of::<usize>()).unwrap();
    std::alloc::alloc(layout) as *mut ::core::ffi::c_void
}
