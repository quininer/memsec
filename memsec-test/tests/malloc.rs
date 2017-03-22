#![cfg(feature = "alloc")]

#[cfg(target_os = "linux")] extern crate nix;
extern crate memsec;

use std::{ slice, mem };


#[test]
fn malloc_u64_test() {
    unsafe {
        let p: *mut u64 = memsec::malloc(mem::size_of::<u64>()).unwrap();
        *p = std::u64::MAX;
        assert_eq!(*p, std::u64::MAX);
        memsec::free(p);
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

        let memptr = memsec::malloc::<u8>(std::usize::MAX - 1);
        assert!(memptr.is_none());

        let buf: *mut u8 = memsec::allocarray(16).unwrap();
        memsec::memzero(buf, 16);
        assert!(memsec::memeq(buf, [0; 16].as_ptr(), 16));
        memsec::free(buf);
    }
}

#[test]
fn allocarray_test() {
    unsafe {
        let memptr: *mut u8 = memsec::allocarray(8).unwrap();
        let array = slice::from_raw_parts_mut(memptr, 8);
        assert_eq!(array, [0xd0; 8]);
        memsec::memzero(memptr, 8);
        assert_eq!(array, [0; 8]);
        array[0] = 1;
        assert!(memsec::memeq(memptr, [1, 0, 0, 0, 0, 0, 0, 0].as_ptr(), 8));
        memsec::free(memptr);
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

#[cfg(target_os = "linux")]
#[should_panic]
#[test]
fn malloc_mprotect_2_test() {
    use nix::sys::signal;
    extern fn sigsegv(_: i32) { panic!() }
    let sigaction = signal::SigAction::new(
        signal::SigHandler::Handler(sigsegv),
        signal::SA_SIGINFO,
        signal::SigSet::empty(),
    );
    unsafe { signal::sigaction(signal::SIGSEGV, &sigaction).ok() };

    unsafe {
        let x: *mut u8 = memsec::allocarray(16).unwrap();

        memsec::memset(x, 1, 16);
        memsec::mprotect(x, memsec::Prot::ReadOnly);
        memsec::memzero(x, 16); // SIGSEGV!
    }
}
