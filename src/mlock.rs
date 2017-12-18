//! mlock / munlock

#![cfg(feature = "use_os")]


/// Unix `mlock`.
#[cfg(unix)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    #[cfg(target_os = "linux")]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_DONTDUMP);

    #[cfg(freebsdlike)]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_NOCORE);

    ::libc::mlock(addr as *mut ::libc::c_void, len) == 0
}

/// Windows `VirtualLock`.
#[cfg(windows)]
pub unsafe fn mlock<T>(addr: *mut T, len: usize) -> bool {
    ::winapi::um::memoryapi::VirtualLock(
        addr as ::winapi::shared::minwindef::LPVOID,
        len as ::winapi::shared::basetsd::SIZE_T
    ) != 0
}

/// Unix `munlock`.
#[cfg(unix)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    ::memzero(addr, len);

    #[cfg(target_os = "linux")]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_DODUMP);

    #[cfg(freebsdlike)]
    ::libc::madvise(addr as *mut ::libc::c_void, len, ::libc::MADV_CORE);

    ::libc::munlock(addr as *mut ::libc::c_void, len) == 0
}

/// Windows `VirtualUnlock`.
#[cfg(windows)]
pub unsafe fn munlock<T>(addr: *mut T, len: usize) -> bool {
    ::memzero(addr, len);
    ::winapi::um::memoryapi::VirtualUnlock(
        addr as ::winapi::shared::minwindef::LPVOID,
        len as ::winapi::shared::basetsd::SIZE_T
    ) != 0
}
