# memsec
[![Build Status](https://travis-ci.org/quininer/memsec.svg?branch=master)](https://travis-ci.org/quininer/memsec)
[![Appveyor](https://ci.appveyor.com/api/projects/status/1w0qtl0grjfu0uac?svg=true)](https://ci.appveyor.com/project/quininer/memsec)

Rust implementation `libsodium/utils`.

* [x] `memcmp`
* [x] `memset`/`memzero`
* [x] `mlock`/`munlock`
* [x] `mprotect`
* [x] `alloc`/`free`

ref
---

* [Securing memory allocations](https://download.libsodium.org/doc/helpers/memory_management.html)
* [rlibc](https://github.com/alexcrichton/rlibc)
* [aligned_alloc.rs](https://github.com/jonas-schievink/aligned_alloc.rs)
