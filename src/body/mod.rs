mod convert;
mod error_type;
mod utils;
pub use error_type::Error;
use futures_lite::{ready, Stream, StreamExt};

use self::utils::IntoAsyncRead;
use bytestr::ByteStr;

use bytes::Bytes;
use futures_lite::{AsyncBufRead, AsyncBufReadExt};

use std::fmt::Debug;
use std::mem::{replace, swap, take};
use std::pin::Pin;
use std::task::{Context, Poll};
type BoxStdError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A boxed steam object.
pub type BoxStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, BoxStdError>> + Send + Sync + 'static>>;

/// A boxed bufreader object.
pub type BoxBufReader = Pin<Box<dyn AsyncBufRead + Send + Sync + 'static>>;

#[cfg(feature = "http_body")]
pub use http_body::Body as HttpBody;

/// Flexible HTTP body.
pub struct Body {
    inner: BodyInner,
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Body")
    }
}

impl_error!(
    BodyFrozen,
    "Body was frozen,it may have been consumed by `take()`"
);

enum BodyInner {
    Once(Bytes),
    Reader {
        reader: BoxBufReader,
        length: Option<usize>,
    },
    Stream(BoxStream),
    Freeze,
}

impl Default for BodyInner {
    fn default() -> Self {
        Self::Once(Bytes::new())
    }
}

impl Body {
    /// Create a new empty body.
    pub const fn empty() -> Self {
        Self {
            inner: BodyInner::Once(Bytes::new()),
        }
    }

    /// Create a new frozen body.
    pub const fn frozen() -> Self {
        Self {
            inner: BodyInner::Freeze,
        }
    }

    /// Create a body from a object implement `AsyncBufRead`.
    /// This method allows you to create a object implement `AsyncBufRead`, which is useful for reading data
    /// from a file or any other source that implements the `AsyncBufRead` trait.
    /// #Example
    /// ```rust
    /// use async_std::fs::File;
    /// use async_std::io::BufReader;
    /// use http_util::Body;
    /// let file = BufReader::new(File::open("index.html").await?);
    /// Body::from_reader(file,file.metadata().await?.len())
    /// ```
    pub fn from_reader(
        reader: impl AsyncBufRead + Send + Sync + 'static,
        length: impl Into<Option<usize>>,
    ) -> Self {
        Self {
            inner: BodyInner::Reader {
                reader: Box::pin(reader),
                length: length.into(),
            },
        }
    }

    /// Create a body from a steam.
    pub fn from_stream<T, E, S>(stream: S) -> Self
    where
        T: Into<Bytes> + Send + 'static,
        E: Into<BoxStdError>,
        S: Stream<Item = Result<T, E>> + Send + Sync + 'static,
    {
        Self {
            inner: BodyInner::Stream(Box::pin(
                stream.map(|result| result.map(|data| data.into()).map_err(|error| error.into())),
            )),
        }
    }

    /// Create a body by serializing a object into JSON.
    #[cfg(feature = "json")]
    pub fn from_json<T: serde::Serialize>(value: &T) -> Result<Self, Error> {
        Ok(Self::from_bytes(serde_json::to_string(value)?))
    }

    /// Create a body by serializing a object into form.
    #[cfg(feature = "form")]
    pub fn from_form<T: serde::Serialize>(value: &T) -> Result<Self, Error> {
        Ok(Self::from_bytes(serde_urlencoded::to_string(value)?))
    }

    /// Create a body from a chunk of bytes.
    pub fn from_bytes(data: impl Into<Bytes>) -> Self {
        Self {
            inner: BodyInner::Once(data.into()),
        }
    }

    /// Try to get the length of the body.This method is primarily used in optimizations, but it is only an estimation,having no warranty of any kind.
    pub const fn len(&self) -> Option<usize> {
        if let BodyInner::Once(bytes) = &self.inner {
            Some(bytes.len())
        } else {
            None
        }
    }

    /// Try to read the body and return a `Bytes` object.
    pub async fn into_bytes(self) -> Result<Bytes, Error> {
        match self.inner {
            BodyInner::Once(bytes) => Ok(bytes),
            BodyInner::Reader { mut reader, length } => {
                let mut vec = Vec::with_capacity(length.unwrap_or_default());
                loop {
                    let data = reader.fill_buf().await?;
                    if data.is_empty() {
                        break;
                    } else {
                        vec.extend_from_slice(data);
                    }
                }
                Ok(vec.into())
            }

            BodyInner::Stream(mut body) => {
                let first = body.try_next().await?.unwrap_or_default();
                let second = body.try_next().await?;
                if let Some(second) = second {
                    let remain_size_hint = body.size_hint();
                    let mut vec = Vec::with_capacity(
                        first.len()
                            + second.len()
                            + remain_size_hint.1.unwrap_or(remain_size_hint.0),
                    );
                    vec.extend_from_slice(&first);
                    vec.extend_from_slice(&second);
                    while let Some(data) = body.try_next().await? {
                        vec.extend_from_slice(&data);
                    }
                    Ok(vec.into())
                } else {
                    Ok(first)
                }
            }
            BodyInner::Freeze => Err(Error::BodyFrozen),
        }
    }

    /// Try to read the body as a UTF-8 string and return a `ByteStr`.
    pub async fn into_string(self) -> Result<ByteStr, Error> {
        Ok(ByteStr::from_utf8(self.into_bytes().await?)?)
    }

    /// Prepare data in the inner representation,then try to read the body as JSON.
    /// This method allows you to deserialize data with zero copy.
    #[cfg(feature = "json")]
    pub async fn into_json<'a, T: serde::Deserialize<'a>>(&'a mut self) -> Result<T, Error> {
        let data = self.as_bytes().await?;
        Ok(serde_json::from_slice(data)?)
    }

    /// Prepare data in the inner representation,then try to read the body as a form.
    /// This method allows you to deserialize data with zero copy.
    #[cfg(feature = "form")]
    pub async fn into_form<'a, T: serde::Deserialize<'a>>(&'a mut self) -> Result<T, Error> {
        let data = self.as_bytes().await?;
        Ok(serde_urlencoded::from_bytes(data)?)
    }

    /// Return a wrapper which implement `AsyncBufRead`.
    pub fn into_reader(self) -> impl AsyncBufRead + Send + Sync {
        IntoAsyncRead::new(self)
    }

    /// Prepare a chunk of bytes in the inner representation, then return a reference to the bytes.
    pub async fn as_bytes(&mut self) -> Result<&[u8], Error> {
        self.inner = BodyInner::Once(self.take()?.into_bytes().await?);
        match self.inner {
            BodyInner::Once(ref bytes) => Ok(bytes),
            _ => unreachable!(),
        }
    }

    /// Replace the value of the body and return the old body.
    pub fn replace(&mut self, body: Body) -> Body {
        replace(self, body)
    }

    /// Swap the value of the body with another body if the orginal body is not frozen.
    pub fn swap(&mut self, body: &mut Body) -> Result<(), BodyFrozen> {
        if self.is_frozen() {
            Err(BodyFrozen::new())
        } else {
            swap(self, body);
            Ok(())
        }
    }

    /// Freeze the orginal body and return its value.
    pub fn take(&mut self) -> Result<Self, BodyFrozen> {
        if self.is_frozen() {
            Err(BodyFrozen::new())
        } else {
            Ok(self.replace(Self::frozen()))
        }
    }

    /// Return `true` if the body if frozen,and `false` otherwise.
    pub const fn is_frozen(&self) -> bool {
        matches!(self.inner, BodyInner::Freeze)
    }

    /// Freeze the body,so that it can not be consumed anymore.The original value of the body will be dropped.
    pub fn freeze(&mut self) {
        self.replace(Self::frozen());
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl Stream for Body {
    type Item = Result<Bytes, BoxStdError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.inner {
            BodyInner::Once(bytes) => {
                if bytes.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(take(bytes))))
                }
            }
            BodyInner::Reader { reader, length } => {
                let data = ready!(reader.as_mut().poll_fill_buf(cx))?;
                if data.is_empty() {
                    return Poll::Ready(None);
                }
                let data = Bytes::copy_from_slice(data);
                reader.as_mut().consume(data.len());
                if let Some(length) = length {
                    *length -= data.len();
                }
                Poll::Ready(Some(Ok(data)))
            }
            BodyInner::Stream(stream) => stream.as_mut().poll_next(cx),
            BodyInner::Freeze => Poll::Ready(Some(Err(Error::BodyFrozen.into()))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            BodyInner::Once(bytes) => (bytes.len(), Some(bytes.len())),
            BodyInner::Reader { length, .. } => (0, *length),
            BodyInner::Stream(body) => body.size_hint(),
            BodyInner::Freeze => (0, None),
        }
    }
}

#[cfg(feature = "http_body")]
impl HttpBody for Body {
    type Data = Bytes;

    type Error = BoxStdError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
