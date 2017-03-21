//! mlock / munlock

#![cfg(feature = "use_os")]

#[cfg(not(windows))]
#[inline]
unsafe fn dontdump<T>(addr: *mut T, len: usize) {
    #[cfg(target_os = "linux")]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_DONTDUMP);

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_NOCORE);
}

#[cfg(not(windows))]
#[inline]
unsafe fn dodump<T>(addr: *mut T, len: usize) {
    #[cfg(target_os = "linux")]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_DODUMP);

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_CORE);
}

/// Unix mlock.
#[cfg(unix)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    dontdump(addr, len);
    ::libc::mlock(addr as *mut ::libc::c_void, len) == 0
}

/// Windows VirtualLock.
#[cfg(windows)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    ::kernel32::VirtualLock(addr as ::winapi::LPVOID, len as ::winapi::SIZE_T) != 0
}

/// Unix munlock.
#[cfg(unix)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    ::memzero(addr, len);
    dodump(addr, len);
    ::libc::munlock(addr as *mut ::libc::c_void, len) == 0
}

/// Windows VirtualUnlock.
#[cfg(windows)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    ::memzero(addr, len);
    ::kernel32::VirtualUnlock(addr as ::winapi::LPVOID, len as ::winapi::SIZE_T) != 0
}
