//! memcmp bench
//!
//! Result:
//! ```
//! running 6 tests
//! test libc_memcmp_eq_bench       ... bench:          33 ns/iter (+/- 3)
//! test libc_memcmp_nq_bench       ... bench:           6 ns/iter (+/- 2)
//! test libsodium_compare_eq_bench ... bench:       1,531 ns/iter (+/- 458)
//! test libsodium_compare_nq_bench ... bench:       1,443 ns/iter (+/- 218)
//! test libsodium_memcmp_eq_bench  ... bench:         630 ns/iter (+/- 127)
//! test libsodium_memcmp_nq_bench  ... bench:         628 ns/iter (+/- 134)
//! test memsec_memcmp_eq_bench     ... bench:       1,093 ns/iter (+/- 230)
//! test memsec_memcmp_nq_bench     ... bench:       1,040 ns/iter (+/- 85)
//! test memsec_memeq_eq_bench      ... bench:         466 ns/iter (+/- 53)
//! test memsec_memeq_nq_bench      ... bench:         500 ns/iter (+/- 182)
//! ```

#![feature(test)]

extern crate test;
extern crate libc;
extern crate memsec;
#[cfg(unix)] extern crate libsodium_sys;

use test::Bencher;
use std::mem::size_of_val;
use libc::{ c_void, c_int, size_t };


#[bench]
fn memsec_memeq_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        memsec::memeq(x.as_ptr(), y.as_ptr(), size_of_val(&y))
    });
}

#[bench]
fn memsec_memeq_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        memsec::memeq(x.as_ptr(), z.as_ptr(), size_of_val(&z))
    });
}

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

#[cfg(unix)]
#[bench]
fn libsodium_compare_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        sodium_compare(x.as_ptr(), z.as_ptr(), size_of_val(&z))
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_compare_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        sodium_compare(x.as_ptr(), y.as_ptr(), size_of_val(&y))
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


#[cfg(unix)]
#[link = "libsodium"]
extern "C" {
    fn sodium_compare(b1_: *const u8, b2_: *const u8, len: size_t) -> c_int;
}
