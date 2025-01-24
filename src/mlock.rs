//! mlock / munlock

#![cfg(feature = "use_os")]

use crate::alloc::{alloc_init, ALLOC_INIT, PAGE_SIZE};

unsafe fn page_size() -> usize {
    ALLOC_INIT.call_once(|| alloc_init());

    PAGE_SIZE
}

/// Cross-platform `mlock`.
///
/// * Unix `mlock`.
/// * Windows `VirtualLock`.
pub unsafe fn mlock(addr: *mut u8, len: usize) -> bool {
    #[cfg(unix)]
    {
        let page_size = page_size();

        let (start_addr, end_addr) = get_page_aligned_addrs(addr, len, page_size);

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

        let (start_addr, end_addr) = get_page_aligned_addrs(addr, len, page_size);

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

unsafe fn get_page_aligned_addrs(addr: *mut u8, len: usize, ps: usize) -> (usize, usize) {
    // start address of the page obtained from masked the value of the page size
    // with the memory address
    //
    let start_addr = (addr as usize) & !(ps - 1);

    // nearest end address of the overlapping page
    let end_addr = ((addr as usize) + len + ps - 1) & !(ps - 1);
    (start_addr, end_addr)
}

#[cfg(test)]
mod tests {
    use super::*;


  #[test]
    fn test_get_page_aligned_addrs_exact_page_boundary() {
        let addr = 0x1000 as *mut u8;
        let len = 0x1000; // 4KB
        let page_size = 0x1000; // 4KB page size

        unsafe {
            let (start_addr, end_addr) = get_page_aligned_addrs(addr, len, page_size);
            assert_eq!(start_addr, 0x1000);
            assert_eq!(end_addr, 0x2000);
        }
    }

    #[test]
    fn test_get_page_aligned_addrs_with_offset() {
        let addr = 0x1234 as *mut u8;
        let len = 0x1000; // 4KB
        let test_page_size = 0x1000; // 4KB page size

        unsafe {
            let (start_addr, end_addr) = get_page_aligned_addrs(addr, len, test_page_size);
            assert_eq!(start_addr, 0x1000);
            assert_eq!(end_addr, 0x3000);
        }
    }

    #[test]
    fn test_get_page_aligned_addrs_small_length() {
        let addr = 0x2000 as *mut u8;
        let len = 0x100; // 256 bytes
        let page_size = 0x1000; // 4KB page size

        unsafe {
            let (start_addr, end_addr) = get_page_aligned_addrs(addr, len, page_size);
            assert_eq!(start_addr, 0x2000);
            assert_eq!(end_addr, 0x3000);
        }
    }
}
