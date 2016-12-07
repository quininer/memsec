extern crate rand;
#[cfg(unix)] extern crate libc;
#[cfg(windows)] extern crate winapi;
#[cfg(windows)] extern crate kernel32;
#[cfg(any(target_os = "macos", target_os = "ios"))] extern crate mach_o_sys;
#[cfg(all(unix, test))] extern crate nix;

mod alloc;

pub use alloc::{ Prot, mprotect, malloc, allocarray, free };


// -- memcmp --

/// Constant time memcmp.
#[inline(never)]
pub unsafe fn memcmp<T>(b1: *const T, b2: *const T, len: usize) -> i32 {
    let b1 = b1 as *const u8;
    let b2 = b2 as *const u8;
    let mut res = 0;
    for i in (0..len as isize).rev() {
        let diff = *b1.offset(i) as i32 - *b2.offset(i) as i32;
        res = (res & (((diff - 1) & !diff) >> 8)) | diff;
    }
    ((res - 1) >> 8) + (res >> 8) + 1
}


// -- memset / memzero --

/// General memset.
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
#[inline(never)]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    let s = s as *mut u8;
    let c = c as u8;

    for i in 0..n as isize {
        ::std::ptr::write_volatile(s.offset(i), c);
    }
}

/// Call memset_s.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    use mach_o_sys::ranlib::{ rsize_t, errno_t };

    extern {
        fn memset_s(s: *mut libc::c_void, smax: rsize_t, c: libc::c_int, n: rsize_t) -> errno_t;
    }

    if n > 0 {
        match memset_s(s as *mut libc::c_void, n as rsize_t, c, n as rsize_t) {
            0 => (),
            ret => panic!("memset_s return with error value {}", ret)
        }
    }
}


/// General memzero.
#[cfg(not(any(
    all(windows, not(target_env = "msvc")),
    target_os = "freebsd",
    target_os = "openbsd"
)))]
#[inline]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    memset(dest, 0, n);
}

/// Call explicit_bzero.
#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
pub unsafe fn memzero<T>(dest: *mut T, n: usize) {
    extern {
        fn explicit_bzero(s: *mut libc::c_void, n: libc::size_t);
    }
    explicit_bzero(dest as *mut libc::c_void, n);
}

/// Call SecureZeroMemory.
#[cfg(all(windows, not(target_env = "msvc")))]
pub unsafe fn memzero<T>(s: *mut T, n: usize) {
    extern "system" {
        fn RtlSecureZeroMemory(ptr: winapi::PVOID, cnt: winapi::SIZE_T);
    }
    RtlSecureZeroMemory(s as winapi::PVOID, n as winapi::SIZE_T);
}


// -- mlock / munlock --


#[cfg(not(windows))]
#[inline]
unsafe fn dontdump<T>(addr: *mut T, len: usize) {
    #[cfg(target_os = "linux")]
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_DONTDUMP);

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_NOCORE);
}

#[cfg(not(windows))]
#[inline]
unsafe fn dodump<T>(addr: *mut T, len: usize) {
    #[cfg(target_os = "linux")]
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_DODUMP);

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_CORE);
}

/// Unix mlock.
#[cfg(unix)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    dontdump(addr, len);
    libc::mlock(addr as *mut libc::c_void, len) == 0
}

/// Windows VirtualLock.
#[cfg(windows)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    kernel32::VirtualLock(addr as winapi::LPVOID, len as winapi::SIZE_T) != 0
}

/// Unix munlock.
#[cfg(unix)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    memzero(addr, len);
    dodump(addr, len);
    libc::munlock(addr as *mut libc::c_void, len) == 0
}

/// Windows VirtualUnlock.
#[cfg(windows)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    memzero(addr, len);
    kernel32::VirtualUnlock(addr as winapi::LPVOID, len as winapi::SIZE_T) != 0
}
