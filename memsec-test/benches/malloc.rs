#![cfg(feature = "alloc")]

//! malloc bench
//!
//! Result:
//! ```
//! running 2 tests
//! test libsodium_malloc ... bench:       7,331 ns/iter (+/- 23)
//! test memsec_malloc    ... bench:       6,677 ns/iter (+/- 20)
//! ```

#![feature(test)]

extern crate test;
extern crate libc;
extern crate memsec;
#[cfg(unix)] extern crate libsodium_sys;

use test::Bencher;


#[bench]
fn memsec_malloc(b: &mut Bencher) {
    b.iter(|| unsafe {
        let ptr: *mut u8 = memsec::malloc(512).unwrap();
        memsec::free(ptr);
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_malloc(b: &mut Bencher) {
    unsafe { libsodium_sys::sodium_init() };
    b.iter(|| unsafe {
        let ptr: *mut u8 = libsodium_sys::sodium_malloc(512) as *mut u8;
        libsodium_sys::sodium_free(ptr as *mut libc::c_void);
    });
}
