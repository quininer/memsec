//! malloc bench
//!
//! Result:
//! ```
//! running 2 tests
//! test libsodium_malloc ... bench:       7,369 ns/iter (+/- 1,865)
//! test memsec_malloc    ... bench:       8,315 ns/iter (+/- 444)
//! ```

#![feature(test)]

extern crate test;
extern crate libc;
extern crate libsodium_sys;
extern crate memsec;

use test::Bencher;
use libc::c_void;


#[bench]
fn memsec_malloc(b: &mut Bencher) {
    b.iter(|| unsafe {
        let ptr: *mut u8 = memsec::malloc(1024).unwrap();
        memsec::free(ptr);
    });
}

#[bench]
fn libsodium_malloc(b: &mut Bencher) {
    unsafe { libsodium_sys::sodium_init() };
    b.iter(|| unsafe {
        let ptr: *mut u8 = libsodium_sys::sodium_malloc(1024) as *mut u8;
        libsodium_sys::sodium_free(ptr as *mut c_void);
    });
}
