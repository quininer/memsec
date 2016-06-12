extern crate memsec;

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
fn malloc_mprotect_test() {
    let x: *mut u8 = unsafe { memsec::malloc(16 * mem::size_of::<u8>()) };

    unsafe { memsec::memset(x, 1, 16 * mem::size_of::<u8>()) };
    assert!(unsafe { memsec::unprotected_mprotect(x, memsec::Prot::ReadOnly) });
    assert!(unsafe { memsec::memcmp(x, [1; 16].as_ptr(), 16 * mem::size_of::<u8>()) });
    assert!(unsafe { memsec::unprotected_mprotect(x, memsec::Prot::NoAccess) });
    assert!(unsafe { memsec::unprotected_mprotect(x, memsec::Prot::ReadWrite) });
    unsafe { memsec::free(x) };
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

    let buf: *mut u8 = unsafe { memsec::allocarray(mem::size_of::<u8>(), 16) };
    unsafe { memsec::memzero(buf, 16 * mem::size_of::<u8>()) };
    assert!(unsafe { memsec::memcmp(buf, [0; 16].as_ptr(), 16 * mem::size_of::<u8>()) });
    unsafe { memsec::free(buf) };
}
