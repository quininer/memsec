extern crate memsec;
#[cfg(unix)] extern crate nix;
extern crate libsodium_sys;

use std::mem;


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

#[test]
fn memcmp_test() {
    let x = [9; 16];
    let y = [9; 17];
    let z = [4; 16];

    assert_eq!(unsafe { memsec::memcmp(x.as_ptr(), y.as_ptr(), mem::size_of_val(&x)) }, 0);
    assert_eq!(
        unsafe { memsec::memcmp(x.as_ptr(), z.as_ptr(), mem::size_of_val(&x)) },
        unsafe { libsodium_sys::sodium_memcmp(
            x.as_ptr() as *const u8,
            z.as_ptr() as *const u8,
            mem::size_of_val(&x)
        ) }
    );
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
    let memptr: *mut u8 = unsafe { memsec::malloc(1) };
    assert!(!memptr.is_null());
    unsafe { memsec::free(memptr) };

    let memptr: *mut u8 = unsafe { memsec::malloc(0) };
    assert!(!memptr.is_null());
    unsafe { memsec::free(memptr) };

    let memptr: *mut u8 = unsafe { memsec::malloc(std::usize::MAX - 1) };
    assert!(memptr.is_null());
    unsafe { memsec::free(memptr) };

    let buf: *mut u8 = unsafe { memsec::allocarray(16) };
    unsafe { memsec::memzero(buf, 16 * mem::size_of::<u8>()) };
    assert_eq!(unsafe { memsec::memcmp(buf, [0; 16].as_ptr(), 16 * mem::size_of::<u8>()) }, 0);
    unsafe { memsec::free(buf) };
}

#[test]
fn malloc_mprotect_1_test() {
    let x: *mut u8 = unsafe { memsec::malloc(16 * mem::size_of::<u8>()) };

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

    let x: *mut u8 = unsafe { memsec::allocarray(16) };

    unsafe { memsec::memset(x, 1, 16 * mem::size_of::<u8>()) };
    unsafe { memsec::unprotected_mprotect(x, memsec::Prot::ReadOnly) };
    unsafe { memsec::memzero(x, 16 * mem::size_of::<u8>()) }; // SIGSEGV!
}
