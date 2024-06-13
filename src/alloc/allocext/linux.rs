extern crate std;
use libc::{MAP_ANONYMOUS, MAP_FIXED, MAP_NORESERVE, MAP_SHARED, PROT_NONE};

use self::std::process::abort;
use crate::{alloc::*, Prot};
use core::mem::{self, size_of};
use core::ptr::{self, NonNull};
use core::slice;

use self::memfd_secret_alloc::*;

mod memfd_secret_alloc {
    use libc::{MAP_LOCKED, MAP_POPULATE};

    use super::*;
    use core::convert::TryInto;

    /// Allocate memfd_secret with given size at given address ptr
    /// Returns tuple of ptr to memory and file descriptor of memfd_secret
    #[inline]
    pub unsafe fn alloc_memfd_secret_at_ptr(
        size: usize,
        ptr: *mut libc::c_void,
    ) -> Option<(NonNull<u8>, libc::c_int)> {
        let fd: Result<libc::c_int, _> = libc::syscall(libc::SYS_memfd_secret, 0).try_into();

        let fd = fd.ok().filter(|&fd| fd >= 0)?;

        // File size is set using ftruncate
        let _ = libc::ftruncate(fd, size as libc::off_t);

        let ptr_out = libc::mmap(
            ptr,
            size,
            Prot::ReadWrite,
            libc::MAP_SHARED | libc::MAP_FIXED | MAP_LOCKED | MAP_POPULATE,
            fd,
            0,
        );

        if ptr_out == libc::MAP_FAILED {
            return None;
        }

        if ptr_out != ptr {
            libc::munmap(ptr_out, size);
            return None;
        }

        NonNull::new(ptr_out as *mut u8).map(|ptr| (ptr, fd))
    }
}

unsafe fn _memfd_secret(size: usize) -> Option<*mut u8> {
    ALLOC_INIT.call_once(|| alloc_init());

    //Assert size of unprotected_size (usize) and fd (i32) is less than PAGE_SIZE before allocating memory
    assert!(size_of::<usize>() + size_of::<i32>() <= PAGE_SIZE);

    if size >= ::core::usize::MAX - PAGE_SIZE * 4 {
        return None;
    }

    let size_with_canary = CANARY_SIZE + size;
    let front_guard_size = PAGE_SIZE + PAGE_SIZE;
    let unprotected_size = page_round(size_with_canary);
    let back_guard_size = PAGE_SIZE;
    let total_size = front_guard_size + unprotected_size + back_guard_size;

    let base_ptr = libc::mmap(
        ptr::null_mut(),
        total_size,
        PROT_NONE,
        MAP_SHARED | MAP_ANONYMOUS | MAP_NORESERVE,
        -1,
        0,
    );

    if base_ptr == libc::MAP_FAILED {
        return None;
    }
    let base_ptr = base_ptr as *mut u8;

    let base_ptr_stored = libc::mmap(
        base_ptr as *mut libc::c_void,
        PAGE_SIZE,
        Prot::ReadWrite,
        MAP_SHARED | MAP_ANONYMOUS | MAP_FIXED,
        -1,
        0,
    ) as *mut u8;

    if base_ptr_stored == libc::MAP_FAILED as *mut u8 || base_ptr_stored != base_ptr {
        libc::munmap(base_ptr as *mut libc::c_void, total_size);
        return None;
    }

    let unprotected_ptr = base_ptr.add(front_guard_size);
    let Some((unprotected_ptr_val, fd)) =
        alloc_memfd_secret_at_ptr(unprotected_size, unprotected_ptr as *mut libc::c_void)
    else {
        libc::munmap(base_ptr_stored as *mut libc::c_void, PAGE_SIZE);
        libc::munmap(base_ptr as *mut libc::c_void, total_size);
        return None;
    };
    assert_eq!(unprotected_ptr_val.as_ptr(), unprotected_ptr as *mut u8);

    let fd_ptr = base_ptr.add(size_of::<usize>());

    let canary_ptr = unprotected_ptr.add(unprotected_size - size_with_canary);
    let user_ptr = canary_ptr.add(CANARY_SIZE);
    ptr::copy_nonoverlapping(CANARY.as_ptr(), canary_ptr, CANARY_SIZE);
    ptr::write_unaligned(base_ptr as *mut usize, unprotected_size);
    ptr::write_unaligned(fd_ptr as *mut libc::c_int, fd);

    // mprotect can be used to change protection flag after mmap setup
    // https://www.gnu.org/software/libc/manual/html_node/Memory-Protection.html#index-mprotect
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
    let back_guard_ptr = unprotected_ptr.add(unprotected_size);
    let fd = ptr::read(fd_ptr);

    // check canary value is same
    if !crate::memeq(canary_ptr as *const u8, CANARY.as_ptr(), CANARY_SIZE) {
        abort();
    }

    // free
    let front_guard_size = PAGE_SIZE + PAGE_SIZE;
    let back_guard_size = PAGE_SIZE;

    _mprotect(base_ptr.add(PAGE_SIZE), front_guard_size, Prot::ReadWrite);
    _mprotect(back_guard_ptr, back_guard_size, Prot::ReadWrite);

    crate::memzero(unprotected_ptr, unprotected_size);

    // Unmap memfd_secret mapping
    let res = libc::munmap(unprotected_ptr as *mut c_void, unprotected_size);
    if res < 0 {
        abort();
    }

    // Unmap header mapping
    let res = libc::munmap(base_ptr as *mut c_void, PAGE_SIZE);
    if res < 0 {
        abort();
    }

    // Unmap reserved space mapping
    let res = libc::munmap(base_ptr as *mut c_void, PAGE_SIZE);
    if res < 0 {
        abort();
    }

    let res = libc::close(fd);
    if res < 0 {
        abort();
    }
}
