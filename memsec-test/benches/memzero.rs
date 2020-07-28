#![feature(test)]

extern crate test;

use test::Bencher;
use std::mem::size_of_val;

#[bench]
fn ptr_write_zeroed_bench(b: &mut Bencher) {
    type U8ARRAY = [u8; 1025];
    let mut x: U8ARRAY = [0; 1025];

    b.iter(|| unsafe {
        ::std::ptr::write_volatile(&mut x, std::mem::zeroed());
    });
}

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
        libsodium_sys::sodium_memzero(x.as_mut_ptr() as *mut _, size_of_val(&x))
    });
}
