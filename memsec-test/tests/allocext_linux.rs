#![cfg(feature = "alloc_ext")]
#![cfg(target_os = "linux")]

use std::ptr::NonNull;


#[test]
fn memfd_secret_u64_test() {
    unsafe {
        let mut p: NonNull<u64> = memsec::memfd_secret().unwrap();
        *p.as_mut() = std::u64::MAX;
        assert_eq!(*p.as_ref(), std::u64::MAX);
        memsec::free_memfd_secret(p);
    }
}

#[test]
fn memfd_secret_free_test() {
    unsafe {
        let memptr: Option<NonNull<u8>> = memsec::memfd_secret();
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free_memfd_secret(memptr);
        }

        let memptr: Option<NonNull<()>> = memsec::memfd_secret();
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free_memfd_secret(memptr);
        }

        let memptr: Option<NonNull<[u8]>> = memsec::memfd_secret_sized(1024);
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free_memfd_secret(memptr);
        }

        // let memptr: Option<NonNull<[u8; std::usize::MAX - 1]>> = memsec::memfd_secret();
        // assert!(memptr.is_none());
    }
}

#[test]
fn memfd_secret_mprotect_1_test() {
    unsafe {
        let mut x: NonNull<[u8; 16]> = memsec::memfd_secret().unwrap();

        memsec::memset(x.as_mut().as_mut_ptr(), 0x01, 16);
        assert!(memsec::mprotect(x, memsec::Prot::ReadOnly));
        assert!(memsec::memeq(x.as_ref().as_ptr(), [1; 16].as_ptr(), 16));
        assert!(memsec::mprotect(x, memsec::Prot::NoAccess));
        assert!(memsec::mprotect(x, memsec::Prot::ReadWrite));
        memsec::memzero(x.as_mut().as_mut_ptr(), 16);
        memsec::free_memfd_secret(x);
    }

    unsafe {
        let mut x: NonNull<[u8; 4096]> = memsec::memfd_secret().unwrap();
        memsec::memset(x.as_mut().as_mut_ptr(), 0x02, 96);
        memsec::free_memfd_secret(x);
    }

    unsafe {
        let mut x: NonNull<[u8; 4100]> = memsec::memfd_secret().unwrap();
        memsec::memset(x.as_mut().as_mut_ptr().offset(100), 0x03, 3000);
        memsec::free_memfd_secret(x);
    }

    unsafe {
        let mut x = memsec::memfd_secret_sized(16).unwrap();

        memsec::memset(x.as_mut().as_mut_ptr(), 0x01, 16);
        assert!(memsec::mprotect(x, memsec::Prot::ReadOnly));
        assert!(memsec::memeq(x.as_ref().as_ptr(), [1; 16].as_ptr(), 16));
        assert!(memsec::mprotect(x, memsec::Prot::NoAccess));
        assert!(memsec::mprotect(x, memsec::Prot::ReadWrite));
        memsec::memzero(x.as_mut().as_mut_ptr(), 16);
        memsec::free_memfd_secret(x);
    }

    unsafe {
        let mut x = memsec::memfd_secret_sized(4100).unwrap();
        memsec::memset(x.as_mut().as_mut_ptr().offset(100), 0x03, 3000);
        memsec::free_memfd_secret(x);
    }
}
