#![cfg(feature = "alloc")]

#[cfg(target_os = "linux")] extern crate nix;
extern crate memsec;

use std::mem;


#[test]
fn malloc_u64_test() {
    unsafe {
        let p: *mut u64 = memsec::malloc(mem::size_of::<u64>()).unwrap() as *mut u64;
        *p = std::u64::MAX;
        assert_eq!(*p, std::u64::MAX);
        memsec::free(p as *mut u8);
    }
}

#[test]
fn malloc_free_test() {
    unsafe {
        let memptr: *mut u8 = memsec::malloc(1).unwrap();
        assert!(!memptr.is_null());
        memsec::free(memptr);

        let memptr: *mut u8 = memsec::malloc(0).unwrap();
        assert!(!memptr.is_null());
        memsec::free(memptr);

        let memptr = memsec::malloc(std::usize::MAX - 1);
        assert!(memptr.is_none());
    }
}

#[test]
fn malloc_mprotect_1_test() {
    unsafe {
        let x: *mut u8 = memsec::malloc(16).unwrap();

        memsec::memset(x, 1, 16);
        assert!(memsec::mprotect(x, memsec::Prot::ReadOnly));
        assert!(memsec::memeq(x, [1; 16].as_ptr(), 16));
        assert!(memsec::mprotect(x, memsec::Prot::NoAccess));
        assert!(memsec::mprotect(x, memsec::Prot::ReadWrite));
        memsec::memzero(x, 16);
        memsec::free(x);
    }
}
