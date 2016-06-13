//! memcmp bench
//!
//! Result:
//! ```
//! running 4 tests
//! test libc_memcmp_eq_bench ... bench:         118 ns/iter (+/- 5)
//! test libc_memcmp_nq_bench ... bench:           5 ns/iter (+/- 2)
//! test memcmp_eq_bench      ... bench:         113 ns/iter (+/- 25)
//! test memcmp_nq_bench      ... bench:         113 ns/iter (+/- 17)
//! ```

#![feature(test)]

extern crate test;
extern crate libc;
extern crate memsec;

use test::Bencher;
use std::mem::size_of_val;
use libc::c_void;


#[bench]
fn memcmp_eq_bench(b: &mut Bencher) {
    let x = [9; 1024];
    let y = [9; 1024];

    b.iter(|| unsafe {
        memsec::memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&y))
    });
}

#[bench]
fn memcmp_nq_bench(b: &mut Bencher) {
    let x = [8; 1024];
    let z = [3; 1024];

    b.iter(|| unsafe {
        memsec::memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&z))
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
