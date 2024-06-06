extern crate std;
use self::std::process::abort;
use crate::{alloc::*, Prot};
use core::mem::{self, size_of};
use core::ptr::{self, NonNull};
use core::slice;

use self::memfd_secret_alloc::*;

mod memfd_secret_alloc {
    use super::*;
    use core::convert::TryInto;

    #[inline]
    pub unsafe fn alloc_memfd_secret(size: usize) -> Option<(NonNull<u8>, libc::c_int)> {
        let fd: Result<libc::c_int, _> = libc::syscall(libc::SYS_memfd_secret, 0).try_into();

        let fd = fd.ok().filter(|&fd| fd >= 0)?;

        // File size is set using ftruncate
        let _ = libc::ftruncate(fd, size as libc::off_t);

        let ptr = libc::mmap(
            ptr::null_mut(),
            size,
            Prot::ReadWrite,
            libc::MAP_SHARED,
            fd,
            0,
        );

        if ptr == libc::MAP_FAILED {
            return None;
        }

        NonNull::new(ptr as *mut u8).map(|ptr| (ptr, fd))
    }
}

unsafe fn _memfd_secret(size: usize) -> Option<*mut u8> {
    ALLOC_INIT.call_once(|| alloc_init());

    //Assert size of unprotected_size (usize) and fd (i32) is less than PAGE_SIZE before allocating memory
    assert!(size_of::<usize>() + size_of::<i32>() <= PAGE_SIZE);

    if size >= ::core::usize::MAX - PAGE_SIZE * 4 {
        return None;
    }

    // aligned alloc ptr
    let size_with_canary = CANARY_SIZE + size;
    let unprotected_size = page_round(size_with_canary);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    let (base_ptr, fd) = alloc_memfd_secret(total_size)?;
    let base_ptr = base_ptr.as_ptr();
    let fd_ptr = base_ptr.add(size_of::<usize>());
    let unprotected_ptr = base_ptr.add(PAGE_SIZE * 2);

    // mprotect can be used to change protection flag after mmap setup
    // https://www.gnu.org/software/libc/manual/html_node/Memory-Protection.html#index-mprotect
    _mprotect(base_ptr.add(PAGE_SIZE), PAGE_SIZE, Prot::NoAccess);
    _mprotect(
        unprotected_ptr.add(unprotected_size),
        PAGE_SIZE,
        Prot::NoAccess,
    );

    let canary_ptr = unprotected_ptr.add(unprotected_size - size_with_canary);
    let user_ptr = canary_ptr.add(CANARY_SIZE);
    ptr::copy_nonoverlapping(CANARY.as_ptr(), canary_ptr, CANARY_SIZE);
    ptr::write_unaligned(base_ptr as *mut usize, unprotected_size);
    ptr::write_unaligned(fd_ptr as *mut libc::c_int, fd);
    _mprotect(base_ptr, PAGE_SIZE, Prot::ReadOnly);

    assert_eq!(unprotected_ptr_from_user_ptr(user_ptr), unprotected_ptr);

    Some(user_ptr)
}

/// Linux specific `memfd_secret` backed allocation
#[inline]
pub unsafe fn memfd_secret<T>() -> Option<NonNull<T>> {
    _memfd_secret(mem::size_of::<T>()).map(|memptr| {
        ptr::write_bytes(memptr, GARBAGE_VALUE, mem::size_of::<T>());
        NonNull::new_unchecked(memptr as *mut T)
    })
}

/// Linux specific `memfd_secret` backed `sized` allocation
#[inline]
pub unsafe fn memfd_secret_sized(size: usize) -> Option<NonNull<[u8]>> {
    _memfd_secret(size).map(|memptr| {
        ptr::write_bytes(memptr, GARBAGE_VALUE, size);
        NonNull::new_unchecked(slice::from_raw_parts_mut(memptr, size))
    })
}

/// Secure `free` for memfd_secret allocations,
/// i.e. provides read write access back to mprotect guard pages
/// and unmaps mmaped secrets
pub unsafe fn free_memfd_secret<T: ?Sized>(memptr: NonNull<T>) {
    use libc::c_void;

    let memptr = memptr.as_ptr() as *mut u8;

    // get unprotected ptr
    let canary_ptr = memptr.sub(CANARY_SIZE);
    let unprotected_ptr = unprotected_ptr_from_user_ptr(memptr);
    let base_ptr = unprotected_ptr.sub(PAGE_SIZE * 2);
    let fd_ptr = base_ptr.add(size_of::<usize>()) as *mut libc::c_int;
    let unprotected_size = ptr::read(base_ptr as *const usize);
    let fd = ptr::read(fd_ptr);

    // check
    if !crate::memeq(canary_ptr as *const u8, CANARY.as_ptr(), CANARY_SIZE) {
        abort();
    }

    // free
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    _mprotect(base_ptr, total_size, Prot::ReadWrite);

    crate::memzero(unprotected_ptr, unprotected_size);

    let res = libc::munmap(base_ptr as *mut c_void, total_size);
    if res < 0 {
        abort();
    }

    let res = libc::close(fd);
    if res < 0 {
        abort();
    }
}
