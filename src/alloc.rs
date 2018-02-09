//! alloc

#![cfg(feature = "alloc")]

extern crate rand;

use std::sync::{ Once, ONCE_INIT };
use std::{ mem, ptr, process };
use self::rand::{ Rng, OsRng };


const GARBAGE_VALUE: u8 = 0xd0;
const CANARY_SIZE: usize = 16;
static ALLOC_INIT: Once = ONCE_INIT;
static mut PAGE_SIZE: usize = 0;
static mut PAGE_MASK: usize = 0;
static mut CANARY: [u8; CANARY_SIZE] = [0; CANARY_SIZE];


// -- alloc init --

#[inline]
unsafe fn alloc_init() {
    #[cfg(unix)] {
        PAGE_SIZE = ::libc::sysconf(::libc::_SC_PAGESIZE) as usize;
    }

    #[cfg(windows)] {
        let mut si = mem::uninitialized();
        ::winapi::um::sysinfoapi::GetSystemInfo(&mut si);
        PAGE_SIZE = si.dwPageSize as usize;
    }

    if PAGE_SIZE < CANARY_SIZE || PAGE_SIZE < mem::size_of::<usize>() {
        process::abort();
    }

    PAGE_MASK = PAGE_SIZE - 1;

    OsRng::new().unwrap().fill_bytes(&mut CANARY);
}


// -- aligned alloc / aligned free --

#[cfg(unix)]
#[inline]
unsafe fn alloc_aligned(size: usize) -> Option<*mut u8> {
    let mut memptr = mem::uninitialized();
    match ::libc::posix_memalign(&mut memptr, PAGE_SIZE, size) {
        0 => Some(memptr as *mut u8),
        ::libc::EINVAL => process::abort(),
        ::libc::ENOMEM => None,
        _ => unreachable!()
    }
}

#[cfg(windows)]
#[inline]
unsafe fn alloc_aligned(size: usize) -> Option<*mut u8> {
    let memptr = ::winapi::um::memoryapi::VirtualAlloc(
        ptr::null_mut(),
        size as ::winapi::shared::basetsd::SIZE_T,
        ::winapi::um::winnt::MEM_COMMIT | ::winapi::um::winnt::MEM_RESERVE,
        ::winapi::um::winnt::PAGE_READWRITE
    );
    if memptr.is_null() {
        None
    } else {
        Some(memptr as *mut u8)
    }
}

#[cfg(unix)]
#[inline]
unsafe fn free_aligned(memptr: *mut u8, _size: usize) {
    ::libc::free(memptr as *mut ::libc::c_void);
}

#[cfg(windows)]
#[inline]
unsafe fn free_aligned(memptr: *mut u8, _size: usize) {
    ::winapi::um::memoryapi::VirtualFree(
        memptr as ::winapi::shared::minwindef::LPVOID,
        0,
        ::winapi::um::winnt::MEM_RELEASE
    );
}

// -- malloc / free --

#[inline]
unsafe fn page_round(size: usize) -> usize {
    (size + PAGE_MASK) & !PAGE_MASK
}

#[inline]
unsafe fn unprotected_ptr_from_user_ptr(memptr: *const u8) -> *mut u8 {
    let canary_ptr = memptr.offset(-(CANARY_SIZE as isize));
    let unprotected_ptr_u = canary_ptr as usize & !PAGE_MASK;
    if unprotected_ptr_u <= PAGE_SIZE * 2 {
        process::abort();
    }
    unprotected_ptr_u as *mut u8
}

unsafe fn _malloc(size: usize) -> Option<*mut u8> {
    ALLOC_INIT.call_once(|| alloc_init());

    if size >= ::std::usize::MAX - PAGE_SIZE * 4 {
        return None;
    }

    // aligned alloc ptr
    let size_with_canary = CANARY_SIZE + size;
    let unprotected_size = page_round(size_with_canary);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    let base_ptr = match alloc_aligned(total_size) {
        Some(memptr) => memptr,
        None => return None
    };
    let unprotected_ptr = base_ptr.offset(PAGE_SIZE as isize * 2);

    // mprotect ptr
    _mprotect(base_ptr.offset(PAGE_SIZE as isize), PAGE_SIZE, Prot::NoAccess);
    _mprotect(unprotected_ptr.offset(unprotected_size as isize), PAGE_SIZE, Prot::NoAccess);
    ::mlock(unprotected_ptr, unprotected_size);
    let canary_ptr = unprotected_ptr.offset(unprotected_size as isize - size_with_canary as isize);
    let user_ptr = canary_ptr.offset(CANARY_SIZE as isize);
    ptr::copy_nonoverlapping(CANARY.as_ptr(), canary_ptr, CANARY_SIZE);
    ptr::write_unaligned(base_ptr as *mut usize, unprotected_size);
    _mprotect(base_ptr, PAGE_SIZE, Prot::ReadOnly);

    assert_eq!(unprotected_ptr_from_user_ptr(user_ptr), unprotected_ptr);

    Some(user_ptr)
}

/// Secure `malloc`.
#[inline]
pub unsafe fn malloc(size: usize) -> Option<*mut u8> {
    _malloc(size)
        .map(|memptr| {
            ptr::write_bytes(memptr, GARBAGE_VALUE, size);
            memptr
        })
}

/// Secure `free`.
pub unsafe fn free(memptr: *mut u8) {
    if memptr.is_null() { return () };

    // get unprotected ptr
    let canary_ptr = memptr.offset(-(CANARY_SIZE as isize));
    let unprotected_ptr = unprotected_ptr_from_user_ptr(memptr);
    let base_ptr = unprotected_ptr.offset(-(PAGE_SIZE as isize * 2));
    let unprotected_size = ptr::read(base_ptr as *const usize);
    let total_size = PAGE_SIZE + PAGE_SIZE + unprotected_size + PAGE_SIZE;
    _mprotect(base_ptr, total_size, Prot::ReadWrite);

    // check
    assert!(::memeq(canary_ptr as *const u8, CANARY.as_ptr(), CANARY_SIZE));

    // free
    ::munlock(unprotected_ptr, unprotected_size);
    free_aligned(base_ptr, total_size);
}


// -- mprotect --

/// Prot enum.
#[cfg(unix)]
#[allow(non_snake_case, non_upper_case_globals)]
pub mod Prot {
    pub use ::libc::c_int as Ty;

    pub const NoAccess: Ty = ::libc::PROT_NONE;
    pub const ReadOnly: Ty = ::libc::PROT_READ;
    pub const WriteOnly: Ty = ::libc::PROT_WRITE;
    pub const ReadWrite: Ty = (::libc::PROT_READ | ::libc::PROT_WRITE);
    pub const Execute: Ty = ::libc::PROT_EXEC;
    pub const ReadExec: Ty = (::libc::PROT_READ | ::libc::PROT_EXEC);
    pub const WriteExec: Ty = (::libc::PROT_WRITE | ::libc::PROT_EXEC);
    pub const ReadWriteExec: Ty = (::libc::PROT_READ | ::libc::PROT_WRITE | ::libc::PROT_EXEC);
}

/// Prot enum.
#[cfg(windows)]
#[allow(non_snake_case, non_upper_case_globals)]
pub mod Prot {
    pub use ::winapi::shared::minwindef::DWORD as Ty;

    pub const NoAccess: Ty = ::winapi::um::winnt::PAGE_NOACCESS;
    pub const ReadOnly: Ty = ::winapi::um::winnt::PAGE_READONLY;
    pub const ReadWrite: Ty = ::winapi::um::winnt::PAGE_READWRITE;
    pub const WriteCopy: Ty = ::winapi::um::winnt::PAGE_WRITECOPY;
    pub const Execute: Ty = ::winapi::um::winnt::PAGE_EXECUTE;
    pub const ReadExec: Ty = ::winapi::um::winnt::PAGE_EXECUTE_READ;
    pub const ReadWriteExec: Ty = ::winapi::um::winnt::PAGE_EXECUTE_READWRITE;
    pub const WriteCopyExec: Ty = ::winapi::um::winnt::PAGE_EXECUTE_WRITECOPY;
    pub const Guard: Ty = ::winapi::um::winnt::PAGE_GUARD;
    pub const NoCache: Ty = ::winapi::um::winnt::PAGE_NOCACHE;
    pub const WriteCombine: Ty = ::winapi::um::winnt::PAGE_WRITECOMBINE;
    pub const RevertToFileMap: Ty = ::winapi::um::winnt::PAGE_REVERT_TO_FILE_MAP;
    pub const TargetsInvalid: Ty = ::winapi::um::winnt::PAGE_TARGETS_INVALID;

    // ::winapi::um::winnt::PAGE_TARGETS_INVALID == ::winapi::um::winnt::PAGE_TARGETS_NO_UPDATE
    // pub const TargetsNoUpdate: Ty = ::winapi::um::winnt::PAGE_TARGETS_NO_UPDATE;
}

/// Unix `mprotect`.
#[cfg(unix)]
#[inline]
unsafe fn _mprotect(ptr: *mut u8, len: usize, prot: Prot::Ty) -> bool {
    ::libc::mprotect(ptr as *mut ::libc::c_void, len, prot as ::libc::c_int) == 0
}

/// Windows `VirtualProtect`.
#[cfg(windows)]
#[inline]
unsafe fn _mprotect(ptr: *mut u8, len: usize, prot: Prot::Ty) -> bool {
    let mut old = ::std::mem::uninitialized();
    ::winapi::um::memoryapi::VirtualProtect(
        ptr as ::winapi::shared::minwindef::LPVOID,
        len as ::winapi::shared::basetsd::SIZE_T,
        prot as ::winapi::shared::minwindef::DWORD,
        &mut old as ::winapi::shared::minwindef::PDWORD
    ) != 0
}

/// Secure `mprotect`.
pub unsafe fn mprotect(memptr: *mut u8, prot: Prot::Ty) -> bool {
    let unprotected_ptr = unprotected_ptr_from_user_ptr(memptr);
    let base_ptr = unprotected_ptr.offset(-(PAGE_SIZE as isize * 2));
    let unprotected_size = ptr::read(base_ptr as *const usize);
    _mprotect(unprotected_ptr, unprotected_size, prot)
}
