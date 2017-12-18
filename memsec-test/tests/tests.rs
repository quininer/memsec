extern crate memsec;
extern crate quickcheck;
extern crate libc;
#[cfg(unix)] extern crate libsodium_sys;

use std::{ mem, cmp };
use quickcheck::quickcheck;


#[test]
fn memzero_test() {
    unsafe {
        let mut x: [usize; 16] = [1; 16];
        memsec::memzero(x.as_mut_ptr(), mem::size_of_val(&x));
        assert_eq!(x, [0; 16]);
        x.clone_from_slice(&[1; 16]);
        assert_eq!(x, [1; 16]);
        memsec::memzero(x[1..11].as_mut_ptr(), 10 * mem::size_of_val(&x[0]));
        assert_eq!(x, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1]);
    }
}

#[test]
fn memeq_test() {
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    fn memeq(x: Vec<u8>, y: Vec<u8>) -> bool {
        unsafe {
            let memsec_output = memsec::memeq(
                x.as_ptr(),
                y.as_ptr(),
                cmp::min(x.len(), y.len())
            );
            let libc_output = libc::memcmp(
                x.as_ptr() as *const libc::c_void,
                y.as_ptr() as *const libc::c_void,
                cmp::min(x.len(), y.len())
            ) == 0;
            memsec_output == libc_output
        }
    }
    quickcheck(memeq as fn(Vec<u8>, Vec<u8>) -> bool);
}

#[test]
fn memcmp_test() {
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    fn memcmp(x: Vec<u8>, y: Vec<u8>) -> bool {
        unsafe {
            let memsec_output = memsec::memcmp(
                x.as_ptr(),
                y.as_ptr(),
                cmp::min(x.len(), y.len())
            );
            let libc_output = libc::memcmp(
                x.as_ptr() as *const libc::c_void,
                y.as_ptr() as *const libc::c_void,
                cmp::min(x.len(), y.len())
            );
            (memsec_output > 0) == (libc_output > 0)
                && (memsec_output < 0) == (libc_output < 0)
                && (memsec_output == 0) == (libc_output == 0)
        }
    }
    quickcheck(memcmp as fn(Vec<u8>, Vec<u8>) -> bool);
}

#[cfg(feature = "use_os")]
#[test]
fn mlock_munlock_test() {
    unsafe {
        let mut x = [1; 16];

        assert!(memsec::mlock(x.as_mut_ptr(), mem::size_of_val(&x)));
        assert!(memsec::munlock(x.as_mut_ptr(), mem::size_of_val(&x)));
        assert_eq!(x, [0; 16]);
    }
}
