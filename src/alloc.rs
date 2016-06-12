use std::sync::{ Once, ONCE_INIT };
use std::intrinsics::abort;
use std::{ mem, ptr };
use rand::{ thread_rng, Rng };
use rand::os::OsRng;
use errno::{ Errno, set_errno };


const GARBAGE_VALUE: u8 = 0xd0;
const CANARY_SIZE: usize = 16;
static ALLOC_INIT: Once = ONCE_INIT;
static mut PAGE_SIZE: usize = 0;
static mut CANARY: [u8; CANARY_SIZE] = [0; CANARY_SIZE];


// -- alloc init --

#[cfg(unix)]
unsafe fn alloc_init() {
    let page_size = ::libc::sysconf(::libc::_SC_PAGESIZE);
    if page_size > 0 {
        PAGE_SIZE = page_size as usize;
    }

    if (PAGE_SIZE < CANARY_SIZE)
        || (PAGE_SIZE < mem::size_of::<usize>())
    {
        abort();
    }

    match OsRng::new() {
        Ok(mut rng) => rng.fill_bytes(&mut CANARY),
        Err(_) => thread_rng().fill_bytes(&mut CANARY)
    }
}

#[cfg(windows)]
unsafe fn alloc_init() {
    let si = mem::uninitialized();
    ::kernel32::GetSystemInfo(si);
    PAGE_SIZE = ptr::read(si).dwPageSize as usize;

    if PAGE_SIZE < CANARY_SIZE || PAGE_SIZE < mem::size_of::<usize>() {
        abort();
    }

    match OsRng::new() {
        Ok(mut rng) => rng.fill_bytes(&mut CANARY),
        Err(_) => thread_rng().fill_bytes(&mut CANARY)
    }
}


// -- aligned alloc/free --

#[cfg(unix)]
unsafe fn alloc_aligned<T>(size: usize) -> *mut T {
    let mut memptr = mem::uninitialized();
    match ::libc::posix_memalign(&mut memptr, PAGE_SIZE, size) {
        0 => memptr as *mut T,
        ::libc::EINVAL => panic!("EINVAL: invalid alignmen. {}", PAGE_SIZE),
        ::libc::ENOMEM => ptr::null_mut(),
        _ => unreachable!()
    }
}

#[cfg(windows)]
unsafe fn alloc_aligned<T>(size: usize) -> *mut T {
    ::kernel32::VirtualAlloc(
        ptr::null(),
        size,
        ::winapi::MEM_COMMIT | ::winapi::MEM_RESERVE,
        ::winapi::PAGE_READWRITE
    ) as *mut T
}

#[cfg(unix)]
unsafe fn free_aligned<T>(memptr: *mut T) {
    ::libc::free(memptr as *mut ::libc::c_void);
}

#[cfg(windows)]
unsafe fn free_aligned<T>(memptr: *mut T) {
    ::kernel32::VirtualFree(memptr as ::winapi::LPVOID, 0, ::winapi::MEM_RELEASE);
}

// -- malloc --

#[inline]
unsafe fn page_round(size: usize) -> usize {
    let page_mask = PAGE_SIZE - 1;
    (size + page_mask) & (!page_mask)
}

pub unsafe fn unprotected_ptr_from_user_ptr<T>(memptr: *const T) -> *mut T {
    let canary_ptr = memptr.offset(-(mem::size_of_val(&CANARY) as isize));
    let page_mask = PAGE_SIZE - 1;
    let unprotected_ptr_u = canary_ptr as usize & !page_mask;
    if unprotected_ptr_u <= PAGE_SIZE * 2 {
        abort();
    }
    unprotected_ptr_u as *mut T
}

unsafe fn _malloc<T>(size: usize) -> *mut T {
    ALLOC_INIT.call_once(|| alloc_init());

    if size >= ::std::usize::MAX - PAGE_SIZE * 4 {
        set_errno(Errno(::libc::ENOMEM));
        return ptr::null_mut();
    }
    if PAGE_SIZE <= mem::size_of_val(&CANARY) || PAGE_SIZE < mem::size_of::<usize>() {
        abort();
    }

    // aligned alloc ptr
    let size_with_canary = mem::size_of_val(&CANARY) + size;
    let unprotected_size = page_round(size_with_canary);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    let base_ptr = alloc_aligned(total_size);
    if base_ptr.is_null() {
        return base_ptr;
    }

    // canary offset
    let unprotected_ptr = base_ptr.offset(PAGE_SIZE as isize * 2);
    ::mprotect(base_ptr.offset(PAGE_SIZE as isize), PAGE_SIZE, ::Prot::NoAccess);
    ptr::copy(
        CANARY.as_ptr(),
        unprotected_ptr.offset(unprotected_size as isize) as *mut u8,
        mem::size_of_val(&CANARY)
    );

    // mprotect ptr
    ::mprotect(unprotected_ptr.offset(unprotected_size as isize), PAGE_SIZE, ::Prot::NoAccess);
    ::mlock(unprotected_ptr, unprotected_size);
    let canary_ptr = unprotected_ptr
        .offset(page_round(size_with_canary) as isize)
        .offset(-(size_with_canary as isize));
    let user_ptr = canary_ptr.offset(mem::size_of_val(&CANARY) as isize);
    ptr::copy(CANARY.as_ptr(), canary_ptr as *mut u8, mem::size_of_val(&CANARY));
    ptr::copy(&unprotected_size, base_ptr as *mut usize, mem::size_of_val(&unprotected_size));
    ::mprotect(base_ptr, PAGE_SIZE, ::Prot::ReadOnly);

    debug_assert_eq!(unprotected_ptr_from_user_ptr(user_ptr), unprotected_ptr);

    user_ptr
}

pub unsafe fn malloc<T>(size: usize) -> *mut T {
    let memptr = _malloc(size);
    if !memptr.is_null() {
        ::memset(memptr, GARBAGE_VALUE as i32, size);
    }
    memptr
}

pub unsafe fn allocarray<T>(count: usize, size: usize) -> *mut T {
    if count > mem::size_of::<usize>() && size >= ::std::usize::MAX / count {
        set_errno(Errno(::libc::ENOMEM));
        return ptr::null_mut();
    }
    malloc(count * size)
}

pub unsafe fn free<T>(memptr: *mut T) {
    if memptr.is_null() { return () };

    let canary_ptr = memptr.offset(-(mem::size_of_val(&CANARY) as isize));
    let unprotected_ptr = unprotected_ptr_from_user_ptr(memptr);
    let base_ptr = unprotected_ptr.offset(-(PAGE_SIZE as isize * 2));
    let unprotected_size = ptr::read(base_ptr as *const usize);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    ::mprotect(base_ptr, total_size, ::Prot::ReadWrite);

    assert!(::memcmp(canary_ptr as *const u8, CANARY.as_ptr(), mem::size_of_val(&CANARY)));
    assert!(::memcmp(
        unprotected_ptr.offset(unprotected_size as isize) as *const u8,
        CANARY.as_ptr(),
        mem::size_of_val(&CANARY)
    ));

    ::munlock(unprotected_ptr, unprotected_size);
    free_aligned(base_ptr);
}

// -- unprotected mprotect --

pub unsafe fn unprotected_mprotect<T>(ptr: *mut T, prot: ::Prot) -> bool {
    let unprotected_ptr = unprotected_ptr_from_user_ptr(ptr);
    let base_ptr = unprotected_ptr.offset(PAGE_SIZE as isize * 2);
    ::mprotect(unprotected_ptr, ptr::read(base_ptr as *const usize), prot)
}

#[cfg(all(unix, test))]
mod test {
    use std::mem;

    #[should_panic]
    #[test]
    fn mprotect_2_test() {
        use nix::sys::signal;

        super::ALLOC_INIT.call_once(|| unsafe { super::alloc_init() });

        extern fn sigsegv(_: i32) { panic!() }
        let sigaction = signal::SigAction::new(
            signal::SigHandler::Handler(sigsegv),
            signal::SA_SIGINFO,
            signal::SigSet::empty(),
        );
        unsafe { signal::sigaction(signal::SIGSEGV, &sigaction).ok() };

        let x: *mut u8 = unsafe { super::alloc_aligned(16 * mem::size_of::<u8>()) };
        unsafe { ::mprotect(x, 16 * mem::size_of::<u8>(), ::Prot::NoAccess) };

        unsafe { ::memzero(x, 16 * mem::size_of::<u8>()) }; // SIGSEGV!
    }
}
