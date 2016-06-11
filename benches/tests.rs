#![feature(test)]

extern crate test;
extern crate memsec;

use test::Bencher;
use std::mem::size_of_val;


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
