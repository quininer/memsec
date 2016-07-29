#![feature(core_intrinsics)]
#![feature(stmt_expr_attributes)]

extern crate rand;
#[cfg(unix)] extern crate libc;
#[cfg(windows)] extern crate winapi;
#[cfg(windows)] extern crate kernel32;
#[cfg(any(target_os = "macos", target_os = "ios"))] extern crate mach_o_sys;
#[cfg(all(unix, test))] extern crate nix;

mod alloc;

pub use alloc::{ unprotected_mprotect, malloc, allocarray, free };


// -- memcmp --

/// Constant time memcmp.
pub unsafe fn memcmp<T>(b1: *const T, b2: *const T, len: usize) -> i32 {
    let b1 = b1 as *const u8;
    let b2 = b2 as *const u8;
    let mut d = 0;
    for i in 0..len as isize {
        d |= *b1.offset(i) ^ *b2.offset(i);
    }
    (1 & (d as i32 - 1) >> 8) - 1
}

// -- memset / memzero --

/// General memset.
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    let s = s as *mut u8;
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }
}

/// Call memset_s.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub unsafe fn memset<T>(s: *mut T, c: i32, n: usize) {
    extern {
        fn memset_s(s: *mut libc::c_void, smax: mach_o_sys::ranlib::rsize_t, c: libc::c_int, n: mach_o_sys::ranlib::rsize_t)
            -> mach_o_sys::ranlib::errno_t;
    }

    if n > 0 && memset_s(s as *mut libc::c_void, n as mach_o_sys::ranlib::rsize_t, c, n as mach_o_sys::ranlib::rsize_t) != 0 {
        std::intrinsics::abort();
    }
}


/// General memzero.
#[cfg(not(any(windows, target_os = "freebsd", target_os = "openbsd")))]
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

/// call SecureZeroMemory.
#[cfg(windows)]
pub unsafe fn memzero<T>(s: *mut T, n: usize) {
    extern "system" {
        fn RtlSecureZeroMemory(ptr: winapi::PVOID, cnt: winapi::SIZE_T);
    }
    RtlSecureZeroMemory(s as winapi::PVOID, n as winapi::SIZE_T);
}


// -- mlock / munlock --

#[cfg(target_os = "linux")]
unsafe fn dontdump<T>(addr: *mut T, len: usize) {
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_DONTDUMP);
}

#[cfg(target_os = "linux")]
unsafe fn dodump<T>(addr: *mut T, len: usize) {
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_DODUMP);
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
unsafe fn dontdump<T>(addr: *mut T, len: usize) {
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_NOCORE);
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
unsafe fn dodump<T>(addr: *mut T, len: usize) {
    libc::madvise(addr as *mut libc::c_void, len, libc::MADV_CORE);
}

#[cfg(not(any(windows, target_os = "linux", target_os = "freebsd", target_os = "dragonfly")))]
fn dontdump<T>(_addr: *mut T, _len: usize) { }

#[cfg(not(any(windows, target_os = "linux", target_os = "freebsd", target_os = "dragonfly")))]
fn dodump<T>(_addr: *mut T, _len: usize) { }

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

// -- mprotect --

/// Prot enum.
#[derive(Debug, Clone, PartialEq)]
pub enum Prot {
    #[cfg(unix)] NoAccess = libc::PROT_NONE as isize,
    #[cfg(unix)] ReadOnly = libc::PROT_READ as isize,
    #[cfg(unix)] WriteOnly = libc::PROT_WRITE as isize,
    #[cfg(unix)] ReadWrite = (libc::PROT_READ | libc::PROT_WRITE) as isize,
    #[cfg(unix)] Execute = libc::PROT_EXEC as isize,
    #[cfg(unix)] ReadExec = (libc::PROT_READ | libc::PROT_EXEC) as isize,
    #[cfg(unix)] WriteExec = (libc::PROT_WRITE | libc::PROT_EXEC) as isize,
    #[cfg(unix)] ReadWriteExec = (libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC) as isize,

    #[cfg(windows)] NoAccess = winapi::PAGE_NOACCESS as isize,
    #[cfg(windows)] ReadOnly = winapi::PAGE_READONLY as isize,
    #[cfg(windows)] ReadWrite = winapi::PAGE_READWRITE as isize,
    #[cfg(windows)] WriteCopy = winapi::PAGE_WRITECOPY as isize,
    #[cfg(windows)] Execute = winapi::PAGE_EXECUTE as isize,
    #[cfg(windows)] ReadExec = winapi::PAGE_EXECUTE_READ as isize,
    #[cfg(windows)] ReadWriteExec = winapi::PAGE_EXECUTE_READWRITE as isize,
    #[cfg(windows)] WriteCopyExec = winapi::PAGE_EXECUTE_WRITECOPY as isize,
    #[cfg(windows)] Guard = winapi::PAGE_GUARD as isize,
    #[cfg(windows)] NoCache = winapi::PAGE_NOCACHE as isize,
    #[cfg(windows)] WriteCombine = winapi::PAGE_WRITECOMBINE as isize,
    #[cfg(windows)] RevertToFileMap = winapi::PAGE_REVERT_TO_FILE_MAP as isize,
    #[cfg(windows)] TargetsNoUpdate = winapi::PAGE_TARGETS_NO_UPDATE as isize,

    // winapi::PAGE_REVERT_TO_FILE_MAP == winapi::PAGE_TARGETS_NO_UPDATE
    // #[cfg(windows)] TargetsInvalid = winapi::PAGE_TARGETS_INVALID as isize,
}

/// Unix mprotect.
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub unsafe fn mprotect<T>(ptr: *mut T, len: usize, prot: Prot) -> bool {
    libc::mprotect(ptr as *mut libc::c_void, len, prot as libc::c_int) == 0
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub unsafe fn mprotect<T>(_: *mut T, _: usize, _: Prot) -> bool { false }

/// Windows VirtualProtect.
#[cfg(windows)]
pub unsafe fn mprotect<T>(ptr: *mut T, len: usize, prot: Prot) -> bool {
    let mut old = std::mem::uninitialized();
    kernel32::VirtualProtect(
        ptr as winapi::LPVOID,
        len as winapi::SIZE_T,
        prot as winapi::DWORD,
        &mut old as winapi::PDWORD
    ) != 0
}
