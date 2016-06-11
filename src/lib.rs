#![no_std]

extern crate libc;

#[allow(unused_imports)]
use libc::{
    c_void, c_int,
    size_t
};
use core::mem;


// -- memcmp --

pub unsafe fn memcmp<T>(b1: *const T, b2: *const T, len: usize) -> bool {
    let b1 = b1 as *const u8;
    let b2 = b2 as *const u8;
    let mut d = 0;
    for i in 0..len as isize {
        d |= *b1.offset(i) ^ *b2.offset(i);
    }
    d == 0
}

// -- memset / memzero --

#[cfg(all(unix, not(HAVE_MEMSET_S)))]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    let s = s as *mut u8;
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }
}

#[cfg(HAVE_MEMSET_S)]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    extern {
        fn memset_s(s: *mut c_void, smax: size_t, c: c_int, n: size_t) -> c_int;
    }
    memset_s(mem::transmute(s), n, c, n);
}


#[cfg(all(unix, not(HAVE_EXPLICIT_BZERO)))]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    memset(dest, 0, n);
}

#[cfg(all(unix, HAVE_EXPLICIT_BZERO))]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    extern {
        fn explicit_bzero(s: *mut c_void, n: size_t);
    }
    explicit_bzero(mem::transmute(dest), n);
}

#[cfg(windows)]
pub unsafe fn memzero<T>(s: *mut T, n: usize) {
    extern {
        fn SecureZeroMemory(s: *mut c_void, n: size_t);
    }
    SecureZeroMemory(mem::transmute(s), n);
}


// -- mlock / munlock --

#[cfg(unix)]
pub unsafe fn mlock<T>(addr: *mut T, len: size_t) -> bool {
    ::libc::mlock(mem::transmute(addr), len) == 0
}

#[cfg(windows)]
pub unsafe fn mlock<T>(addr: *mut T, len: size_t) -> bool {
    extern {
        fn VirtualLock(addr: *mut c_void, len: size_t) -> c_int;
    }
    VirtualLock(mem::transmute(addr), len) != 0
}


#[cfg(unix)]
pub unsafe fn munlock<T>(addr: *mut T, len: size_t) -> bool {
    ::libc::munlock(mem::transmute(addr), len) == 0
}

#[cfg(windows)]
pub unsafe fn munlock<T>(addr: *mut T, len: size_t) -> bool {
    extern {
        fn VirtualUnlock(addr: *mut c_void, len: size_t) -> c_int;
    }
    VirtualUnlock(mem::transmute(addr), len) != 0
}
