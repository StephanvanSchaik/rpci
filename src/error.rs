#[derive(Debug)]
pub enum Error {
    TryFromInt(core::num::TryFromIntError),
    ParseInt(core::num::ParseIntError),
    Errno(i32),
    #[cfg(feature = "std")]
    Io(std::io::Error),
    #[cfg(feature = "std")]
    Nix(nix::Error),
}

impl From<core::num::TryFromIntError> for Error {
    fn from(e: core::num::TryFromIntError) -> Error {
        Error::TryFromInt(e)
    }
}

impl From<core::num::ParseIntError> for Error {
    fn from(e: core::num::ParseIntError) -> Error {
        Error::ParseInt(e)
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

#[cfg(feature = "std")]
impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Error {
        Error::Nix(e)
    }
}
