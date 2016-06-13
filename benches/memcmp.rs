//! memcmp bench
//!
//! Result:
//! ```
//! running 6 tests
//! test libc_memcmp_eq_bench      ... bench:         119 ns/iter (+/- 4)
//! test libc_memcmp_nq_bench      ... bench:           5 ns/iter (+/- 0)
//! test libsodium_memcmp_eq_bench ... bench:         602 ns/iter (+/- 16)
//! test libsodium_memcmp_nq_bench ... bench:         605 ns/iter (+/- 15)
//! test memsec_memcmp_eq_bench    ... bench:         113 ns/iter (+/- 1)
//! test memsec_memcmp_nq_bench    ... bench:         113 ns/iter (+/- 1)
//! ```

#![feature(test)]

extern crate test;
extern crate libc;
extern crate libsodium_sys;
extern crate memsec;

use test::Bencher;
use std::mem::size_of_val;
use libc::c_void;


#[bench]
fn memsec_memcmp_eq_bench(b: &mut Bencher) {
    let x = [9; 1024];
    let y = [9; 1024];

    b.iter(|| unsafe {
        memsec::memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&y))
    });
}

#[bench]
fn memsec_memcmp_nq_bench(b: &mut Bencher) {
    let x = [8; 1024];
    let z = [3; 1024];

    b.iter(|| unsafe {
        memsec::memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&z))
    });
}

#[bench]
fn libsodium_memcmp_eq_bench(b: &mut Bencher) {
    let x = [9; 1024];
    let y = [9; 1024];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&y))
    });
}

#[bench]
fn libsodium_memcmp_nq_bench(b: &mut Bencher) {
    let x = [8; 1024];
    let z = [3; 1024];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&z))
    });
}

#[bench]
fn libc_memcmp_eq_bench(b: &mut Bencher) {
    let x = [9; 1024];
    let y = [9; 1024];

    b.iter(|| unsafe {
        libc::memcmp(x.as_ptr() as *const c_void, y.as_ptr() as *const c_void, size_of_val(&y))
    });
}

#[bench]
fn libc_memcmp_nq_bench(b: &mut Bencher) {
    let x = [8; 1024];
    let z = [3; 1024];

    b.iter(|| unsafe {
        libc::memcmp(x.as_ptr() as *const c_void, z.as_ptr() as *const c_void, size_of_val(&z))
    });
}
