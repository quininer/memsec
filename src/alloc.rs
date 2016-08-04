use std::sync::{ Once, ONCE_INIT };
use std::{ mem, ptr };
use rand::{ Rng, OsRng };


const GARBAGE_VALUE: u8 = 0xd0;
const CANARY_SIZE: usize = 16;
static ALLOC_INIT: Once = ONCE_INIT;
static mut PAGE_SIZE: usize = 0;
static mut PAGE_MASK: usize = 0;
static mut CANARY: [u8; CANARY_SIZE] = [0; CANARY_SIZE];

// -- get page size --
#[cfg(unix)]
#[inline]
unsafe fn get_page_size() -> usize {
    return ::libc::sysconf(::libc::_SC_PAGESIZE) as usize;
}

#[cfg(windows)]
#[inline]
unsafe fn get_page_size() -> usize {
    let mut si = mem::uninitialized();
    ::kernel32::GetSystemInfo(&mut si);
    return si.dwPageSize as usize;
}

// -- alloc init --

unsafe fn alloc_init() {
    PAGE_SIZE = get_page_size();

    if PAGE_SIZE < CANARY_SIZE || PAGE_SIZE < mem::size_of::<usize>() {
        panic!("page size too small");
    }

    PAGE_MASK = PAGE_SIZE - 1;

    OsRng::new().unwrap().fill_bytes(&mut CANARY);
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
        ptr::null_mut(),
        size as ::winapi::SIZE_T,
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
    (size + PAGE_MASK) & (!PAGE_MASK)
}

unsafe fn unprotected_ptr_from_user_ptr<T>(memptr: *const T) -> *mut T {
    let canary_ptr = memptr.offset(-(mem::size_of_val(&CANARY) as isize));
    let unprotected_ptr_u = canary_ptr as usize & !PAGE_MASK;
    if unprotected_ptr_u <= PAGE_SIZE * 2 {
        panic!("user address {} too small", memptr as usize);
    }
    unprotected_ptr_u as *mut T
}

unsafe fn _malloc<T>(size: usize) -> Option<*mut T> {
    ALLOC_INIT.call_once(|| alloc_init());

    if size >= ::std::usize::MAX - PAGE_SIZE * 4 {
        return None;
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


// -- test --

#[cfg(all(unix, test, not(any(target_os = "macos", target_os = "ios"))))]
#[should_panic]
#[test]
fn mprotect_test() {
    use nix::sys::signal;

    ALLOC_INIT.call_once(|| unsafe { alloc_init() });

    extern fn sigsegv(_: i32) { panic!() }
    let sigaction = signal::SigAction::new(
        signal::SigHandler::Handler(sigsegv),
        signal::SA_SIGINFO,
        signal::SigSet::empty(),
    );
    unsafe { signal::sigaction(signal::SIGSEGV, &sigaction).ok() };

    let x: *mut u8 = unsafe { alloc_aligned(16 * mem::size_of::<u8>()).unwrap() };
    unsafe { ::mprotect(x, 16 * mem::size_of::<u8>(), ::Prot::NoAccess) };

    unsafe { ::memzero(x, 16 * mem::size_of::<u8>()) }; // SIGSEGV!
}

#[test]
fn alloc_free_aligned() {
    ALLOC_INIT.call_once(|| unsafe { alloc_init() });

    let x: *mut u8 = unsafe { alloc_aligned(16 * mem::size_of::<u8>()).unwrap() };
    unsafe { ::memzero(x, 16 * mem::size_of::<u8>()) };
    assert!(unsafe { ::mprotect(x, 16 * mem::size_of::<u8>(), ::Prot::ReadOnly) });
    assert_eq!(unsafe { ::memcmp(x, [0; 16].as_ptr(), 16 * mem::size_of::<u8>()) }, 0);
    assert!(unsafe { ::mprotect(x, 16 * mem::size_of::<u8>(), ::Prot::NoAccess) });
    assert!(unsafe { ::mprotect(x, 16 * mem::size_of::<u8>(), ::Prot::ReadWrite) });
    unsafe { ::memzero(x, 16 * mem::size_of::<u8>()) };
    unsafe { free_aligned(x) };
}
