//! mlock / munlock

#![cfg(feature = "use_os")]

use libc::_SC_PAGESIZE;

/// Retrieve page size of the system
///
/// Used for alignment
unsafe fn page_size() -> usize {
    use libc::sysconf;
    sysconf(_SC_PAGESIZE) as usize
}

/// Cross-platform `mlock`.
///
/// * Unix `mlock`.
/// * Windows `VirtualLock`.
pub unsafe fn mlock(addr: *mut u8, len: usize) -> bool {
    #[cfg(unix)]
    {
        let page_size = page_size();

        // start address of the page obtained from masked the value of the page size
        // with the memory address
        //
        let start_addr = (addr as usize) & !(page_size - 1);

        // End address of the page
        let end_addr = ((addr as usize) + len + page_size - 1) & !(page_size - 1);

        let aligned_len = end_addr - start_addr;

        #[cfg(target_os = "linux")]
        libc::madvise(
            start_addr as *mut libc::c_void,
            aligned_len,
            libc::MADV_DONTDUMP,
        );

        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        libc::madvise(
            start_addr as *mut libc::c_void,
            aligned_len,
            libc::MADV_NOCORE,
        );

        libc::mlock(start_addr as *mut libc::c_void, aligned_len) == 0
    }

    #[cfg(windows)]
    {
        windows_sys::Win32::System::Memory::VirtualLock(addr.cast(), len) != 0
    }
}

/// Cross-platform `munlock`.
///
/// * Unix `munlock`.
/// * Windows `VirtualUnlock`.
pub unsafe fn munlock(addr: *mut u8, len: usize) -> bool {
    crate::memzero(addr, len);

    #[cfg(unix)]
    {
        let page_size = page_size();

        // start address of the page obtained from masked the value of the page size
        // with the memory address
        //
        let start_addr = (addr as usize) & !(page_size - 1);

        // End address of the page
        let end_addr = ((addr as usize) + len + page_size - 1) & !(page_size - 1);

        let aligned_len = end_addr - start_addr;
        #[cfg(target_os = "linux")]
        libc::madvise(
            start_addr as *mut libc::c_void,
            aligned_len,
            libc::MADV_DODUMP,
        );

        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        libc::madvise(
            start_addr as *mut libc::c_void,
            aligned_len,
            libc::MADV_CORE,
        );

        libc::munlock(start_addr as *mut libc::c_void, aligned_len) == 0
    }

    #[cfg(windows)]
    {
        windows_sys::Win32::System::Memory::VirtualUnlock(addr.cast(), len) != 0
    }
}
