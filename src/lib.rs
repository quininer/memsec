#![no_std]

#[cfg(unix)] extern crate libc;
#[cfg(windows)] extern crate kernel32;


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

#[cfg(not(HAVE_MEMSET_S))]
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
        fn memset_s(s: *mut libc::c_void, smax: libc::size_t, c: libc::c_int, n: libc::size_t) -> libc::c_int;
    }
    memset_s(s as libc::c_void, n, c, n);
}


#[cfg(all(unix, not(HAVE_EXPLICIT_BZERO)))]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    memset(dest, 0, n);
}

#[cfg(all(unix, HAVE_EXPLICIT_BZERO))]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    extern {
        fn explicit_bzero(s: *mut libc::c_void, n: libc::size_t);
    }
    explicit_bzero(dest as *mut libc::c_void, n);
}

#[cfg(windows)]
pub unsafe fn memzero<T>(s: *mut T, n: usize) {
    extern "system" {
        fn SecureZeroMemory(ptr: winapi::PVOID, cnt: winapi::SIZE_T);
    }
    SecureZeroMemory(s as winapi::PVOID, n as winapi::SIZE_T);
}


// -- mlock / munlock --

#[cfg(unix)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    libc::mlock(addr as *mut libc::c_void, len) == 0
}

#[cfg(windows)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    kernel32::VirtualLock(addr as winapi::LPVOID, len as winapi::SIZE_T) != 0
}


#[cfg(unix)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    libc::munlock(addr as *mut libc::c_void, len) == 0
}

#[cfg(windows)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    kernel32::VirtualUnlock(addr as winapi::LPVOID, len as winapi::SIZE_T) != 0
}
