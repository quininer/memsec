#![no_std]

#![cfg_attr(feature = "nightly", feature(core_intrinsics))]

#[cfg(feature = "use_os")] extern crate std;
#[cfg(all(unix, feature = "use_os"))] extern crate libc;
#[cfg(all(windows, feature = "use_os"))] extern crate winapi;
#[cfg(all(apple, feature = "use_os"))] extern crate mach_o_sys;

mod mlock;
mod alloc;

use core::ptr;
#[cfg(feature = "use_os")] pub use mlock::{ mlock, munlock };
#[cfg(feature = "alloc")] pub use alloc::{ Prot, mprotect, malloc, malloc_sized, free };


// -- memcmp --

/// Secure `memeq`.
#[inline(never)]
pub unsafe fn memeq(b1: *const u8, b2: *const u8, len: usize) -> bool {
    (0..len as isize)
        .map(|i| ptr::read_volatile(b1.offset(i)) ^ ptr::read_volatile(b2.offset(i)))
        .fold(0, |sum, next| sum | next)
        .eq(&0)
}


/// Secure `memcmp`.
#[inline(never)]
pub unsafe fn memcmp(b1: *const u8, b2: *const u8, len: usize) -> i32 {
    let mut res = 0;
    for i in (0..len as isize).rev() {
        let diff = i32::from(ptr::read_volatile(b1.offset(i)))
            - i32::from(ptr::read_volatile(b2.offset(i)));
        res = (res & (((diff - 1) & !diff) >> 8)) | diff;
    }
    ((res - 1) >> 8) + (res >> 8) + 1
}


// -- memset / memzero --

/// General `memset`.
#[cfg(feature = "nightly")]
#[cfg(any(not(apple), not(feature = "use_os")))]
#[inline(never)]
pub unsafe fn memset(s: *mut u8, c: u8, n: usize) {
    core::intrinsics::volatile_set_memory(s, c, n);
}

/// General `memset`.
#[cfg(not(feature = "nightly"))]
#[cfg(any(not(apple), not(feature = "use_os")))]
#[inline(never)]
pub unsafe fn memset(s: *mut u8, c: u8, n: usize) {
    for i in 0..n {
        ptr::write_volatile(s.add(i), c);
    }
}

/// Call `memset_s`.
#[cfg(all(apple, feature = "use_os"))]
pub unsafe fn memset(s: *mut u8, c: u8, n: usize) {
    use libc::{ c_void, c_int };
    use mach_o_sys::ranlib::{ rsize_t, errno_t };

    extern {
        fn memset_s(s: *mut c_void, smax: rsize_t, c: c_int, n: rsize_t) -> errno_t;
    }

    if n > 0 && memset_s(s as *mut c_void, n as _, c as _, n as _) != 0 {
        std::process::abort()
    }
}


/// General `memzero`.
#[cfg(any(
    not(any(
        all(windows, not(target_env = "msvc")),
        freebsdlike, netbsdlike
    )),
    not(feature = "use_os")
))]
#[inline]
pub unsafe fn memzero(dest: *mut u8, n: usize) {
    memset(dest, 0, n);
}

/// Call `explicit_bzero`.
#[cfg(all(any(freebsdlike, netbsdlike), feature = "use_os"))]
pub unsafe fn memzero(dest: *mut u8, n: usize) {
    extern {
        fn explicit_bzero(s: *mut libc::c_void, n: libc::size_t);
    }
    explicit_bzero(dest as *mut libc::c_void, n);
}

/// Call `SecureZeroMemory`.
#[cfg(all(windows, not(target_env = "msvc"), feature = "use_os"))]
pub unsafe fn memzero(s: *mut u8, n: usize) {
    extern "system" {
        fn RtlSecureZeroMemory(ptr: winapi::shared::ntdef::PVOID, cnt: winapi::shared::basetsd::SIZE_T);
    }
    RtlSecureZeroMemory(s as winapi::shared::ntdef::PVOID, n as winapi::shared::basetsd::SIZE_T);
}
