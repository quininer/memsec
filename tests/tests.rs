#[cfg(unix)] extern crate nix;
extern crate aligned_alloc;
extern crate memsec;

use std::mem;
use aligned_alloc::{ aligned_alloc, aligned_free };


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

    assert!(unsafe { memsec::memcmp(x.as_ptr(), y.as_ptr(), mem::size_of_val(&x)) });
    assert!(! unsafe { memsec::memcmp(x.as_ptr(), z.as_ptr(), mem::size_of_val(&x)) });
}

#[test]
fn mlock_munlock_test() {
    let mut x = [1; 16];

    assert!(unsafe { memsec::mlock(x.as_mut_ptr(), mem::size_of_val(&x)) });
    assert!(unsafe { memsec::munlock(x.as_mut_ptr(), mem::size_of_val(&x)) });
    assert_eq!(x, [0; 16]);
}

#[test]
fn mprotect_1_test() {
    let x = aligned_alloc(16 * mem::size_of::<u8>(), 128) as *mut u8;

    unsafe { memsec::memset(x, 1, 16 * mem::size_of::<u8>()) };
    assert!(unsafe { memsec::mprotect(x, 16 * mem::size_of::<u8>(), memsec::Prot::ReadOnly) });
    assert!(unsafe { memsec::memcmp(x, [1; 16].as_ptr(), 16 * mem::size_of::<u8>()) });
    assert!(unsafe { memsec::mprotect(x, 16 * mem::size_of::<u8>(), memsec::Prot::NoAccess) });
    unsafe { aligned_free(x as *mut ()) };
}

#[cfg(unix)]
#[should_panic]
#[test]
fn mprotect_2_test() {
    use nix::sys::signal;

    extern fn sigsegv(_: i32) { panic!() }
    let sigaction = signal::SigAction::new(
        signal::SigHandler::Handler(sigsegv),
        signal::SA_SIGINFO,
        signal::SigSet::empty(),
    );
    unsafe { signal::sigaction(signal::SIGSEGV, &sigaction).ok() };

    let x = aligned_alloc(16 * mem::size_of::<u8>(), 128) as *mut u8;
    unsafe { memsec::mprotect(x, 16 * mem::size_of::<u8>(), memsec::Prot::NoAccess) };

    unsafe { memsec::memzero(x, 16 * mem::size_of::<u8>()) }; // SIGSEGV!
}
