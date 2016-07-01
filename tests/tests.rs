#![feature(plugin)]
#![plugin(quickcheck_macros)]

extern crate libsodium_sys;
extern crate memsec;
extern crate quickcheck;
#[cfg(unix)] extern crate nix;

use std::{ mem, cmp };


#[test]
fn memzero_test() {
    let mut x: [usize; 16] = [1; 16];
    unsafe { memsec::memzero(x.as_mut_ptr(), mem::size_of_val(&x)) };
    assert_eq!(x, [0; 16]);
    x.clone_from_slice(&[1; 16]);
    assert_eq!(x, [1; 16]);
    unsafe { memsec::memzero(x[1..11].as_mut_ptr(), 10 * mem::size_of_val(&x[0])) };
    assert_eq!(x, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1]);
}

#[quickcheck]
fn memcmp(x: Vec<u8>, y: Vec<u8>) -> bool {
    unsafe {
        memsec::memcmp(
            x.as_ptr(),
            y.as_ptr(),
            cmp::min(x.len(), y.len()) * mem::size_of::<u8>()
        ) == libsodium_sys::sodium_memcmp(
            x.as_ptr() as *const u8,
            y.as_ptr() as *const u8,
            cmp::min(x.len(), y.len()) * mem::size_of::<u8>()
        )
    }
}

#[test]
fn mlock_munlock_test() {
    let mut x = [1; 16];

    assert!(unsafe { memsec::mlock(x.as_mut_ptr(), mem::size_of_val(&x)) });
    assert!(unsafe { memsec::munlock(x.as_mut_ptr(), mem::size_of_val(&x)) });
    assert_eq!(x, [0; 16]);
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
    unsafe { memsec::memzero(buf, 16 * mem::size_of::<u8>()) };
    assert_eq!(unsafe { memsec::memcmp(buf, [0; 16].as_ptr(), 16 * mem::size_of::<u8>()) }, 0);
    unsafe { memsec::free(buf) };
}

#[test]
fn malloc_mprotect_1_test() {
    let x: *mut u8 = unsafe { memsec::malloc(16 * mem::size_of::<u8>()).unwrap() };

    unsafe { memsec::memset(x, 1, 16 * mem::size_of::<u8>()) };
    assert!(unsafe { memsec::unprotected_mprotect(x, memsec::Prot::ReadOnly) });
    assert_eq!(unsafe { memsec::memcmp(x, [1; 16].as_ptr(), 16 * mem::size_of::<u8>()) }, 0);
    assert!(unsafe { memsec::unprotected_mprotect(x, memsec::Prot::NoAccess) });
    assert!(unsafe { memsec::unprotected_mprotect(x, memsec::Prot::ReadWrite) });
    unsafe { memsec::memzero(x, 16 * mem::size_of::<u8>()) };
    unsafe { memsec::free(x) };
}

#[cfg(unix)]
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

    unsafe { memsec::memset(x, 1, 16 * mem::size_of::<u8>()) };
    unsafe { memsec::unprotected_mprotect(x, memsec::Prot::ReadOnly) };
    unsafe { memsec::memzero(x, 16 * mem::size_of::<u8>()) }; // SIGSEGV!
}
