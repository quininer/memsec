#![cfg(feature = "alloc")]
#![feature(test)]

extern crate test;

use std::ptr::NonNull;
use test::Bencher;

#[bench]
fn memsec_malloc(b: &mut Bencher) {
    b.iter(|| unsafe {
        let ptr: NonNull<[u8; 512]> = memsec::malloc().unwrap();
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
