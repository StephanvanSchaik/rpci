#![cfg_attr(not(feature = "std"), no_std)]

pub mod arch;
pub mod error;

#[cfg(all(feature = "std", target_os = "linux"))]
pub mod linux;

pub use error::Error;
