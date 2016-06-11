memsec
------

I have not tested this library in Windows, it may contain insecurity implementation.

If the code have any issue, please remind me.

* [x] `memcmp`
* [x] `memset`/`memzero`
* [x] `mlock`/`munlock`
* [x] `mprotect`
* [ ] `alloc`/`free`

ref
---

* [Securing memory allocations](https://download.libsodium.org/doc/helpers/memory_management.html)
* [rlibc](https://github.com/alexcrichton/rlibc)
* [aligned_alloc.rs](https://github.com/jonas-schievink/aligned_alloc.rs)
