#![cfg(feature = "alloc")]

#[cfg(target_os = "linux")] extern crate nix;
extern crate memsec;

use std::ptr::NonNull;


#[test]
fn malloc_u64_test() {
    unsafe {
        let mut p: NonNull<u64> = memsec::malloc().unwrap();
        *p.as_mut() = std::u64::MAX;
        assert_eq!(*p.as_ref(), std::u64::MAX);
        memsec::free(p);
    }
}

#[test]
fn malloc_free_test() {
    unsafe {
        let memptr: Option<NonNull<u8>> = memsec::malloc();
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free(memptr);
        }

        let memptr: Option<NonNull<()>> = memsec::malloc();
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free(memptr);
        }

        // let memptr: Option<NonNull<[u8; std::usize::MAX - 1]>> = memsec::malloc();
        // assert!(memptr.is_none());
    }
}

#[test]
fn malloc_mprotect_1_test() {
    unsafe {
        let mut x: NonNull<[u8; 16]> = memsec::malloc().unwrap();

        memsec::memset(x.as_mut().as_mut_ptr(), 0x01, 16);
        assert!(memsec::mprotect(x, memsec::Prot::ReadOnly));
        assert!(memsec::memeq(x.as_ref().as_ptr(), [1; 16].as_ptr(), 16));
        assert!(memsec::mprotect(x, memsec::Prot::NoAccess));
        assert!(memsec::mprotect(x, memsec::Prot::ReadWrite));
        memsec::memzero(x.as_mut().as_mut_ptr(), 16);
        memsec::free(x);
    }

    unsafe {
        let mut x: NonNull<[u8; 4096]> = memsec::malloc().unwrap();
        memsec::memset(x.as_mut().as_mut_ptr(), 0x02, 96);
        memsec::free(x);
    }

    unsafe {
        let mut x: NonNull<[u8; 4100]> = memsec::malloc().unwrap();
        memsec::memset(x.as_mut().as_mut_ptr().offset(100), 0x03, 3000);
        memsec::free(x);
    }
}
