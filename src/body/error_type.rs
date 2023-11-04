use super::{BodyFrozen, BoxStdError};
use std::error::Error as StdError;
use std::fmt::Display;
use std::io;
use std::str::Utf8Error;

/// Error which can ocuur when attempting to
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An Error caused by a inner I/O error.
    Io(io::Error),
    /// The inner object provides a illgal UTF-8 chunk.
    Utf8(Utf8Error),
    /// The body has been consumed and can not provide data anymore.It is distinguished from a normal empty body.
    BodyFrozen,

    /// Other inner error.
    Other(BoxStdError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl From<BoxStdError> for Error {
    fn from(error: BoxStdError) -> Self {
        Self::Other(error)
    }
}

impl From<BodyFrozen> for Error {
    fn from(_error: BodyFrozen) -> Self {
        Self::BodyFrozen
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Utf8(error) => error.fmt(f),
            Self::Other(error) => error.fmt(f),
            Self::BodyFrozen => BodyFrozen::new().fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(error) => error.source(),
            Error::Utf8(error) => error.source(),
            Error::BodyFrozen => None,
            Error::Other(error) => error.source(),
        }
    }
}
