#![cfg_attr(not(feature = "use_os"), no_std)]

#[cfg(feature = "alloc")] extern crate rand;
#[cfg(all(unix, feature = "use_os"))] extern crate libc;

#[cfg(all(windows, feature = "use_os"))] extern crate winapi;
#[cfg(all(windows, feature = "use_os"))] extern crate kernel32;

#[cfg(all(apple, feature = "use_os"))]
extern crate mach_o_sys;

mod mlock;
mod alloc;

#[cfg(feature = "use_os")] use std::ptr;
#[cfg(not(feature = "use_os"))] use core::ptr;

#[cfg(feature = "use_os")] pub use mlock::{ mlock, munlock };
#[cfg(feature = "alloc")] pub use alloc::{ Prot, mprotect, malloc, allocarray, free };


// -- memcmp --

/// Constant time memeq.
#[inline(never)]
pub unsafe fn memeq<T>(b1: *const T, b2: *const T, len: usize) -> bool {
    let b1 = b1 as *const u8;
    let b2 = b2 as *const u8;

    (0..len as isize)
        .map(|i| ptr::read_volatile(b1.offset(i)) ^ ptr::read_volatile(b2.offset(i)))
        .fold(0, |sum, next| sum | next)
        .eq(&0)
}


/// Constant time memcmp.
#[inline(never)]
pub unsafe fn memcmp<T>(b1: *const T, b2: *const T, len: usize) -> i32 {
    let b1 = b1 as *const u8;
    let b2 = b2 as *const u8;
    let mut res = 0;
    for i in (0..len as isize).rev() {
        let diff = ptr::read_volatile(b1.offset(i)) as i32
            - ptr::read_volatile(b2.offset(i)) as i32;
        res = (res & (((diff - 1) & !diff) >> 8)) | diff;
    }
    ((res - 1) >> 8) + (res >> 8) + 1
}


// -- memset / memzero --

/// General memset.
#[cfg(any(not(apple), not(feature = "use_os")))]
#[inline(never)]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    let s = s as *mut u8;
    let c = c as u8;

    for i in 0..n as isize {
        ptr::write_volatile(s.offset(i), c);
    }
}

/// Call memset_s.
#[cfg(all(apple, feature = "use_os"))]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    use libc::{ c_void, c_int };
    use mach_o_sys::ranlib::{ rsize_t, errno_t };

    extern {
        fn memset_s(s: *mut c_void, smax: rsize_t, c: c_int, n: rsize_t) -> errno_t;
    }

    if n > 0 && memset_s(s as *mut c_void, n as rsize_t, c, n as rsize_t) != 0 {
        std::process::abort()
    }
}


/// General memzero.
#[cfg(any(
    not(any(
        all(windows, not(target_env = "msvc")),
        freebsdlike, netbsdlike
    )),
    not(feature = "use_os")
))]
#[inline]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    memset(dest, 0, n);
}

/// Call explicit_bzero.
#[cfg(all(any(freebsdlike, netbsdlike), feature = "use_os"))]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    extern {
        fn explicit_bzero(s: *mut libc::c_void, n: libc::size_t);
    }
    explicit_bzero(dest as *mut libc::c_void, n);
}

/// Call SecureZeroMemory.
#[cfg(all(windows, not(target_env = "msvc"), feature = "use_os"))]
pub unsafe fn memzero<T>(s: *mut T, n: usize) {
    extern "system" {
        fn RtlSecureZeroMemory(ptr: winapi::PVOID, cnt: winapi::SIZE_T);
    }
    RtlSecureZeroMemory(s as winapi::PVOID, n as winapi::SIZE_T);
}
