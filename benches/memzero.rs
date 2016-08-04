//! memzero bench
//!
//! Result:
//! ```
//! running 2 tests
//! test libsodium_memzero_bench ... bench:         403 ns/iter (+/- 125)
//! test memsec_memzero_bench    ... bench:         364 ns/iter (+/- 55)
//! ```


#![feature(test)]

extern crate test;
extern crate libsodium_sys;
extern crate memsec;

use test::Bencher;
use std::mem::size_of_val;


#[bench]
fn memsec_memzero_bench(b: &mut Bencher) {
    let mut x: [u8; 1024] = [0; 1024];

    b.iter(|| unsafe {
        memsec::memzero(x.as_mut_ptr(), size_of_val(&x))
    });
}

#[bench]
fn libsodium_memzero_bench(b: &mut Bencher) {
    let mut x: [u8; 1024] = [0; 1024];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memzero(x.as_mut_ptr(), size_of_val(&x))
    });
}
