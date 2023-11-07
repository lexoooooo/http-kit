use super::{BodyFrozen, BoxStdError};
use std::error::Error as StdError;
use std::fmt::Display;
use std::io;
use std::str::Utf8Error;

/// Error type around `Body`.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An Error caused by a inner I/O error.
    Io(io::Error),
    /// The inner object provides a illgal UTF-8 chunk.
    Utf8(Utf8Error),
    /// The body has been consumed and can not provide data anymore.It is distinguished from a normal empty body.
    BodyFrozen,
    #[cfg(feature = "json")]
    /// Fail to serialize/deserialize object to JSON.
    JsonError(serde_json::Error),
    #[cfg(feature = "form")]
    /// Fail to serialize object to a form.
    SerializeForm(serde_urlencoded::ser::Error),
    #[cfg(feature = "form")]
    /// Fail to deserialize a form to object.
    DeserializeForm(serde_urlencoded::de::Error),
    /// Other inner error.
    Other(BoxStdError),
}

macro_rules! impl_body_error {
    ($(($field:tt,$ty:ty $(,$feature:tt)?)),*) => {
        $(
            $(#[cfg(feature = $feature)])*
            impl From<$ty> for Error {
                fn from(error: $ty) -> Self {
                    Self::$field(error)
                }
            }
        )*

        impl Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $(#[cfg(feature = $feature)])*
                        Self::$field(error) => error.fmt(f),
                    )*
                    Self::BodyFrozen => BodyFrozen::new().fmt(f),
                }
            }
        }

        impl StdError for Error {
            fn source(&self) -> Option<&(dyn StdError + 'static)> {
                match self {
                    $(
                        $(#[cfg(feature = $feature)])*
                        Self::$field(error) => error.source(),
                    )*
                    Error::BodyFrozen => None,
                }
            }
        }

    };
}

impl_body_error![
    (Io, io::Error),
    (Utf8, Utf8Error),
    (Other, BoxStdError),
    (JsonError, serde_json::Error, "json"),
    (SerializeForm, serde_urlencoded::ser::Error, "form"),
    (DeserializeForm, serde_urlencoded::de::Error, "form")
];

impl From<BodyFrozen> for Error {
    fn from(_error: BodyFrozen) -> Self {
        Self::BodyFrozen
    }
}
