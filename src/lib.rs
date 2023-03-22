#![cfg_attr(not(feature = "std"), no_std)]

pub mod arch;
pub mod error;

pub use error::Error;
