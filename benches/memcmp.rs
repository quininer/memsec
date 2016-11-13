//! memcmp bench
//!
//! Result:
//! ```
//! running 6 tests
//! test libc_memcmp_eq_bench      ... bench:          38 ns/iter (+/- 7)
//! test libc_memcmp_nq_bench      ... bench:           5 ns/iter (+/- 0)
//! test libsodium_memcmp_eq_bench ... bench:         611 ns/iter (+/- 71)
//! test libsodium_memcmp_nq_bench ... bench:         616 ns/iter (+/- 41)
//! test memsec_memcmp_eq_bench    ... bench:          40 ns/iter (+/- 3)
//! test memsec_memcmp_nq_bench    ... bench:          40 ns/iter (+/- 4)
//! ```

#![feature(test)]

extern crate test;
extern crate libc;
extern crate memsec;
#[cfg(unix)] extern crate libsodium_sys;

use test::Bencher;
use std::mem::size_of_val;
use libc::c_void;


#[bench]
fn memsec_memcmp_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        memsec::memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&y))
    });
}

#[bench]
fn memsec_memcmp_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        memsec::memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&z))
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_memcmp_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&y))
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_memcmp_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&z))
    });
}

#[bench]
fn libc_memcmp_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        libc::memcmp(x.as_ptr() as *const c_void, y.as_ptr() as *const c_void, size_of_val(&y))
    });
}

#[bench]
fn libc_memcmp_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        libc::memcmp(x.as_ptr() as *const c_void, z.as_ptr() as *const c_void, size_of_val(&z))
    });
}
