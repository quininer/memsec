//! memzero bench
//!
//! Result:
//! ```
//! running 2 tests
//! test libsodium_memzero_bench ... bench:         371 ns/iter (+/- 3)
//! test memsec_memzero_bench    ... bench:         356 ns/iter (+/- 2)
//! ```


#![feature(test)]

extern crate test;
extern crate memsec;
#[cfg(unix)] extern crate libsodium_sys;

use test::Bencher;
use std::mem::size_of_val;


#[bench]
fn memsec_memzero_bench(b: &mut Bencher) {
    let mut x: [u8; 1025] = [0; 1025];

    b.iter(|| unsafe {
        memsec::memzero(x.as_mut_ptr(), size_of_val(&x))
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_memzero_bench(b: &mut Bencher) {
    let mut x: [u8; 1025] = [0; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memzero(x.as_mut_ptr(), size_of_val(&x))
    });
}
