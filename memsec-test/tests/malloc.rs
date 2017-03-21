#![cfg(feature = "alloc")]

extern crate nix;
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
    let memptr: *mut u8 = unsafe { memsec::malloc(1).unwrap() };
    assert!(!memptr.is_null());
    unsafe { memsec::free(memptr) };

    let memptr: *mut u8 = unsafe { memsec::malloc(0).unwrap() };
    assert!(!memptr.is_null());
    unsafe { memsec::free(memptr) };

    let memptr = unsafe { memsec::malloc::<u8>(std::usize::MAX - 1) };
    assert!(memptr.is_none());

    let buf: *mut u8 = unsafe { memsec::allocarray(16).unwrap() };
    unsafe { memsec::memzero(buf, 16) };
    assert!(unsafe { memsec::memeq(buf, [0; 16].as_ptr(), 16) });
    unsafe { memsec::free(buf) };
}

#[test]
fn allocarray_test() {
    let memptr: *mut u8 = unsafe { memsec::allocarray(8).unwrap() };
    let array = unsafe { slice::from_raw_parts_mut(memptr, 8) };
    assert_eq!(array, [0xd0; 8]);
    unsafe { memsec::memzero(memptr, 8) };
    assert_eq!(array, [0; 8]);
    array[0] = 1;
    assert!(unsafe { memsec::memeq(memptr, [1, 0, 0, 0, 0, 0, 0, 0].as_ptr(), 8) });
    unsafe { memsec::free(memptr) };
}

#[test]
fn malloc_mprotect_1_test() {
    let x: *mut u8 = unsafe { memsec::malloc(16).unwrap() };

    unsafe { memsec::memset(x, 1, 16) };
    assert!(unsafe { memsec::mprotect(x, memsec::Prot::ReadOnly) });
    assert!(unsafe { memsec::memeq(x, [1; 16].as_ptr(), 16) });
    assert!(unsafe { memsec::mprotect(x, memsec::Prot::NoAccess) });
    assert!(unsafe { memsec::mprotect(x, memsec::Prot::ReadWrite) });
    unsafe { memsec::memzero(x, 16) };
    unsafe { memsec::free(x) };
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

    let x: *mut u8 = unsafe { memsec::allocarray(16).unwrap() };

    unsafe { memsec::memset(x, 1, 16) };
    unsafe { memsec::mprotect(x, memsec::Prot::ReadOnly) };
    unsafe { memsec::memzero(x, 16) }; // SIGSEGV!
}
