extern crate memsec;

use std::mem::size_of_val;

#[test]
fn memzero_test() {
    let mut x: [usize; 16] = [1; 16];
    unsafe { memsec::memzero(x.as_mut_ptr(), size_of_val(&x)) };
    assert_eq!(x, [0; 16]);
    x.clone_from_slice(&[1; 16]);
    assert_eq!(x, [1; 16]);
    unsafe { memsec::memzero(x[1..11].as_mut_ptr(), 10 * size_of_val(&x[0])) };
    assert_eq!(x, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1]);
}

#[test]
fn memcmp_test() {
    let x = [9; 16];
    let y = [9; 17];
    let z = [4; 16];

    assert!(unsafe { memsec::memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&x)) });
    assert!(! unsafe { memsec::memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&x)) });
}

// TODO how test mlock/munlock?

#[test]
fn mlock_munlock_test() {
    let mut x = [1; 16];

    assert!(unsafe { memsec::mlock(x.as_mut_ptr(), size_of_val(&x)) });
    assert!(unsafe { memsec::munlock(x.as_mut_ptr(), size_of_val(&x)) });
    assert_eq!(x, [0; 16]);
}
