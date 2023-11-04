use http::StatusCode;
use std::error::Error as StdError;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// The error type for HTTP operations.
pub struct Error {
    error: anyhow::Error,
    status: StatusCode,
}

/// A specialized Result type for http operations.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a `Error` object from any error type with the status code.
    pub fn new<E, S>(error: E, status: S) -> Self
    where
        E: Into<anyhow::Error>,
        S: TryInto<StatusCode>,
        S::Error: fmt::Debug,
    {
        Self {
            error: error.into(),
            status: status.try_into().unwrap(), //may panic if user delivers an illegal code.
        }
    }

    /// Create a `Error` object from a string or a object can be transformed into string.
    pub fn msg<S>(msg: S) -> Self
    where
        S: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        anyhow::Error::msg(msg).into()
    }

    /// Set the status code of the error.Only error status code can be set.
    pub fn set_status<S>(mut self, status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: fmt::Debug,
    {
        let status = status.try_into().expect("Invalid status code");
        if cfg!(debug_assertions) {
            assert!(
                (400..=599).contains(&status.as_u16()),
                "Expected a status code within 400~599"
            )
        }

        self.status = status;

        self
    }

    /// Return the status code of the error.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Try to downcast the inner error type and return `Box<E>`.
    pub fn downcast<E>(self) -> std::result::Result<Box<E>, Self>
    where
        E: StdError + Send + Sync + 'static,
    {
        let Self { status, error } = self;
        error.downcast().map_err(|error| Self { status, error })
    }

    /// Try to downcast the inner error type and return the reference of the mutable reference of `E`.
    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: StdError + Send + Sync + 'static,
    {
        self.error.downcast_ref()
    }

    /// Try to downcast the inner error type and return `E`.
    pub fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: StdError + Send + Sync + 'static,
    {
        self.error.downcast_mut()
    }

    /// Throw the status code and return inner error type.
    pub fn into_inner(self) -> Box<dyn StdError + Send + Sync + 'static> {
        self.error.into()
    }
}

impl<E: Into<anyhow::Error>> From<E> for Error {
    fn from(error: E) -> Self {
        Self::new(error, StatusCode::SERVICE_UNAVAILABLE)
    }
}

impl From<Error> for Box<dyn StdError> {
    fn from(error: Error) -> Self {
        error.error.into()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.error, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl AsRef<dyn StdError + Send + 'static> for Error {
    fn as_ref(&self) -> &(dyn StdError + Send + 'static) {
        self.deref()
    }
}

impl AsMut<dyn StdError + Send + 'static> for Error {
    fn as_mut(&mut self) -> &mut (dyn StdError + Send + 'static) {
        self.deref_mut()
    }
}

impl Deref for Error {
    type Target = dyn StdError + Send + 'static;

    fn deref(&self) -> &Self::Target {
        self.error.deref()
    }
}

impl DerefMut for Error {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.error.deref_mut()
    }
}

/// Provide `status` method for `Result`.
pub trait ResultExt<T>
where
    Self: Sized,
{
    /// Wrap the error type with status code.
    /// # Example
    /// ```
    /// use std::fs::File;
    /// use std::io::prelude::*;
    /// use http_util::{Body,Result,ResultExt};
    /// use async_fs::File;
    /// fn handler() -> Result<Body>{
    ///     Ok(Body::from_reader(File::open("index.html").await.status(404)?))
    /// }

    /// ```
    fn status<S>(self, status: S) -> Result<T>
    where
        S: TryInto<StatusCode>,
        S::Error: fmt::Debug;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn status<S>(self, status: S) -> Result<T>
    where
        S: TryInto<StatusCode>,
        S::Error: fmt::Debug,
    {
        self.map_err(|error| Error::new(error, status))
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn status<S>(self, status: S) -> Result<T>
    where
        S: TryInto<StatusCode>,
        S::Error: fmt::Debug,
    {
        self.ok_or(Error::msg("None Error").set_status(status))
    }
}
