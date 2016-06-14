use std::sync::{ Once, ONCE_INIT };
use std::intrinsics::abort;
use std::{ mem, ptr };
use rand::{ thread_rng, Rng, OsRng };


const GARBAGE_VALUE: u8 = 0xd0;
const CANARY_SIZE: usize = 16;
static ALLOC_INIT: Once = ONCE_INIT;
static mut PAGE_SIZE: usize = 0;
static mut CANARY: [u8; CANARY_SIZE] = [0; CANARY_SIZE];

// -- alloc init --

unsafe fn alloc_init() {
    #[cfg(unix)] {
        let page_size = ::libc::sysconf(::libc::_SC_PAGESIZE);
        if page_size > 0 {
            PAGE_SIZE = page_size as usize;
        }
    }

    #[cfg(windows)] {
        let si = mem::uninitialized();
        ::kernel32::GetSystemInfo(si);
        PAGE_SIZE = ptr::read(si).dwPageSize as usize;
    }

    if PAGE_SIZE < CANARY_SIZE || PAGE_SIZE < mem::size_of::<usize>() {
        abort();
    }

    match OsRng::new() {
        Ok(mut rng) => rng.fill_bytes(&mut CANARY),
        Err(_) => thread_rng().fill_bytes(&mut CANARY)
    }
}

// -- aligned alloc / aligned free --

#[cfg(unix)]
unsafe fn alloc_aligned<T>(size: usize) -> Option<*mut T> {
    let mut memptr = mem::uninitialized();
    match ::libc::posix_memalign(&mut memptr, PAGE_SIZE, size) {
        0 => Some(memptr as *mut T),
        ::libc::EINVAL => panic!("EINVAL: invalid alignmen. {}", PAGE_SIZE),
        ::libc::ENOMEM => None,
        _ => unreachable!()
    }
}

#[cfg(windows)]
unsafe fn alloc_aligned<T>(size: usize) -> Option<*mut T> {
    Some(::kernel32::VirtualAlloc(
        ptr::null(),
        size,
        ::winapi::MEM_COMMIT | ::winapi::MEM_RESERVE,
        ::winapi::PAGE_READWRITE
    ) as *mut T)
}

#[cfg(unix)]
unsafe fn free_aligned<T>(memptr: *mut T) {
    ::libc::free(memptr as *mut ::libc::c_void);
}

#[cfg(windows)]
unsafe fn free_aligned<T>(memptr: *mut T) {
    ::kernel32::VirtualFree(memptr as ::winapi::LPVOID, 0, ::winapi::MEM_RELEASE);
}

// -- malloc / free --

#[inline]
unsafe fn page_round(size: usize) -> usize {
    let page_mask = PAGE_SIZE - 1;
    (size + page_mask) & (!page_mask)
}

unsafe fn unprotected_ptr_from_user_ptr<T>(memptr: *const T) -> *mut T {
    let canary_ptr = memptr.offset(-(mem::size_of_val(&CANARY) as isize));
    let page_mask = PAGE_SIZE - 1;
    let unprotected_ptr_u = canary_ptr as usize & !page_mask;
    if unprotected_ptr_u <= PAGE_SIZE * 2 {
        abort();
    }
    unprotected_ptr_u as *mut T
}

unsafe fn _malloc<T>(size: usize) -> Option<*mut T> {
    ALLOC_INIT.call_once(|| alloc_init());

    if size >= ::std::usize::MAX - PAGE_SIZE * 4 {
        return None;
    }
    if PAGE_SIZE <= mem::size_of_val(&CANARY) || PAGE_SIZE < mem::size_of::<usize>() {
        abort();
    }

    // aligned alloc ptr
    let size_with_canary = mem::size_of_val(&CANARY) + size;
    let unprotected_size = page_round(size_with_canary);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    let base_ptr = match alloc_aligned(total_size) {
        Some(memptr) => memptr,
        None => return None
    };

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
    ptr::write(base_ptr as *mut usize, unprotected_size);
    ::mprotect(base_ptr, PAGE_SIZE, ::Prot::ReadOnly);

    debug_assert_eq!(unprotected_ptr_from_user_ptr(user_ptr), unprotected_ptr);

    Some(user_ptr)
}

/// Secure malloc.
pub unsafe fn malloc<T>(size: usize) -> Option<*mut T> {
    _malloc(size)
        .map(|memptr| {
            ::memset(memptr, GARBAGE_VALUE as i32, size);
            memptr
        })
}

/// Alloc array.
///
/// ```
/// use std::{ slice, mem };
/// use memsec::{ allocarray, free, memzero, memset, memcmp };
///
/// let memptr: *mut u8 = unsafe { allocarray(8).unwrap() };
/// let array = unsafe { slice::from_raw_parts_mut(memptr, 8) };
/// assert_eq!(array, [0xd0; 8]);
/// unsafe { memzero(memptr, 8) };
/// assert_eq!(array, [0; 8]);
/// array[0] = 1;
/// assert_eq!(unsafe { memcmp(memptr, [1, 0, 0, 0, 0, 0, 0, 0].as_ptr(), 8) }, 0);
/// unsafe { free(memptr) };
/// ```
pub unsafe fn allocarray<T>(count: usize) -> Option<*mut T> {
    let size = mem::size_of::<T>();
    if count > mem::size_of::<usize>() && size >= ::std::usize::MAX / count {
        None
    } else {
        malloc(count * size)
    }
}

/// Secure free.
pub unsafe fn free<T>(memptr: *mut T) {
    if memptr.is_null() { return () };

    // get unprotected ptr
    let canary_ptr = memptr.offset(-(mem::size_of_val(&CANARY) as isize));
    let unprotected_ptr = unprotected_ptr_from_user_ptr(memptr);
    let base_ptr = unprotected_ptr.offset(-(PAGE_SIZE as isize * 2));
    let unprotected_size = ptr::read(base_ptr as *const usize);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    ::mprotect(base_ptr, total_size, ::Prot::ReadWrite);

    // check
    debug_assert_eq!(::memcmp(canary_ptr as *const u8, CANARY.as_ptr(), mem::size_of_val(&CANARY)), 0);
    debug_assert_eq!(::memcmp(
        unprotected_ptr.offset(unprotected_size as isize) as *const u8,
        CANARY.as_ptr(),
        mem::size_of_val(&CANARY)
    ), 0);

    // free
    ::munlock(unprotected_ptr, unprotected_size);
    free_aligned(base_ptr);
}

// -- unprotected mprotect --

/// Secure mprotect.
pub unsafe fn unprotected_mprotect<T>(ptr: *mut T, prot: ::Prot) -> bool {
    let unprotected_ptr = unprotected_ptr_from_user_ptr(ptr);
    let base_ptr = unprotected_ptr.offset(-(PAGE_SIZE as isize * 2));
    let unprotected_size = ptr::read(base_ptr as *const usize);
    ::mprotect(unprotected_ptr, unprotected_size, prot)
}

#[cfg(all(unix, test))]
mod test {
    use std::mem;

    #[should_panic]
    #[test]
    fn mprotect_test() {
        use nix::sys::signal;

        super::ALLOC_INIT.call_once(|| unsafe { super::alloc_init() });

        extern fn sigsegv(_: i32) { panic!() }
        let sigaction = signal::SigAction::new(
            signal::SigHandler::Handler(sigsegv),
            signal::SA_SIGINFO,
            signal::SigSet::empty(),
        );
        unsafe { signal::sigaction(signal::SIGSEGV, &sigaction).ok() };

        let x: *mut u8 = unsafe { super::alloc_aligned(16 * mem::size_of::<u8>()).unwrap() };
        unsafe { ::mprotect(x, 16 * mem::size_of::<u8>(), ::Prot::NoAccess) };

        unsafe { ::memzero(x, 16 * mem::size_of::<u8>()) }; // SIGSEGV!
    }
}
