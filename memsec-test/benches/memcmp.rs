#![feature(test)]

extern crate test;

use libc::c_void;
use std::mem::size_of_val;
use test::Bencher;

#[bench]
fn memsec_memeq_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe { memsec::memeq(x.as_ptr(), y.as_ptr(), size_of_val(&y)) });
}

#[bench]
fn memsec_memeq_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe { memsec::memeq(x.as_ptr(), z.as_ptr(), size_of_val(&z)) });
}

#[bench]
fn memsec_memcmp_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe { memsec::memcmp(x.as_ptr(), y.as_ptr(), size_of_val(&y)) });
}

#[bench]
fn memsec_memcmp_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe { memsec::memcmp(x.as_ptr(), z.as_ptr(), size_of_val(&z)) });
}

#[cfg(unix)]
#[bench]
fn libsodium_memcmp_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memcmp(
            x.as_ptr() as *const _,
            y.as_ptr() as *const _,
            size_of_val(&y),
        )
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_memcmp_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_memcmp(
            x.as_ptr() as *const _,
            z.as_ptr() as *const _,
            size_of_val(&z),
        )
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_compare_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_compare(
            x.as_ptr() as *const _,
            z.as_ptr() as *const _,
            size_of_val(&z),
        )
    });
}

#[cfg(unix)]
#[bench]
fn libsodium_compare_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        libsodium_sys::sodium_compare(
            x.as_ptr() as *const _,
            y.as_ptr() as *const _,
            size_of_val(&y),
        )
    });
}

#[bench]
fn libc_memcmp_eq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [9; 1025];
    let y: [u8; 1025] = [9; 1025];

    b.iter(|| unsafe {
        libc::memcmp(
            x.as_ptr() as *const c_void,
            y.as_ptr() as *const c_void,
            size_of_val(&y),
        )
    });
}

#[bench]
fn libc_memcmp_nq_bench(b: &mut Bencher) {
    let x: [u8; 1025] = [8; 1025];
    let z: [u8; 1025] = [3; 1025];

    b.iter(|| unsafe {
        libc::memcmp(
            x.as_ptr() as *const c_void,
            z.as_ptr() as *const c_void,
            size_of_val(&z),
        )
    });
}
