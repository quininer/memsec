//! allocext
//! OS Specific allocation
//!
//!
#![cfg(feature = "alloc_ext")]

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use self::linux::*;
