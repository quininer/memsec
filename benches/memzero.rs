//! memzero bench
//!
//! Result:
//! ```
//! running 2 tests
//! test libsodium_memzero_bench ... bench:         384 ns/iter (+/- 27)
//! test memsec_memzero_bench    ... bench:         377 ns/iter (+/- 36)
//! ```


#![feature(test)]

extern crate test;
extern crate libsodium_sys;
extern crate memsec;

use test::Bencher;
use std::mem::size_of_val;


#[bench]
fn memsec_memzero_bench(b: &mut Bencher) {
    let mut x: [u8; 1025] = [0; 1025];

    b.iter(|| unsafe {
        memsec::memzero(x.as_mut_ptr(), size_of_val(&x))
    });
}

#[bench]
fn libsodium_memzero_bench(b: &mut Bencher) {
    let mut x: [u8; 1025] = [0; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memzero(x.as_mut_ptr(), size_of_val(&x))
    });
}
