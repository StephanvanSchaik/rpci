#[cfg(feature = "std")]
use thiserror::Error;

#[cfg(feature = "std")]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Nix(#[from] nix::Error),
}

#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub struct Error;
