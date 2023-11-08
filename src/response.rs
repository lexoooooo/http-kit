use std::fmt::Debug;

use bytes::Bytes;
use bytestr::ByteStr;
use http::{header::HeaderName, Extensions, HeaderMap, HeaderValue, StatusCode, Version};

use crate::{body::BodyFrozen, Body};

/// The HTTP response parts.
pub type ResponseParts = http::response::Parts;

/// A HTTP response.
#[derive(Debug)]
pub struct Response {
    parts: ResponseParts,
    body: Body,
}

impl From<http::Response<Body>> for Response {
    fn from(response: http::Response<Body>) -> Self {
        let (parts, body) = response.into_parts();
        Self { parts, body }
    }
}

impl From<Response> for http::Response<Body> {
    fn from(response: Response) -> Self {
        Self::from_parts(response.parts, response.body)
    }
}

macro_rules! impl_response_from {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Response {
                fn from(value: $ty) -> Self {
                    Self::new(StatusCode::OK, value)
                }
            }
        )*
    };
}

impl_response_from![ByteStr, String, Vec<u8>, Bytes];

impl Response {
    /// Create a new `Response` with a body.
    pub fn new<S>(status: S, body: impl Into<Body>) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        let mut response: Self = http::Response::new(body.into()).into();
        response.set_status(status.try_into().unwrap());
        response
    }

    /// Create a empty `Response`.
    pub fn empty() -> Self {
        Self::new(StatusCode::OK, Body::empty())
    }

    /// Return the status code.
    pub const fn status(&self) -> StatusCode {
        self.parts.status
    }
    /// Return the mutable reference of status code.

    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.parts.status
    }
    /// Set the status code.
    pub fn set_status(&mut self, status: StatusCode) {
        *self.status_mut() = status;
    }
    /// Return the HTTP version.

    pub const fn version(&self) -> Version {
        self.parts.version
    }
    /// Return the mutable reference of HTTP version.

    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.parts.version
    }
    /// Set the HTTP version by `version`

    pub fn set_version(&mut self, version: Version) {
        *self.version_mut() = version;
    }
    /// Return the reference of the HTTP header.

    pub const fn headers(&self) -> &HeaderMap {
        &self.parts.headers
    }
    /// Return the mutable reference of the HTTP header.

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.parts.headers
    }
    /// Acquire the first value of header by header name.

    pub fn get_header(&self, name: HeaderName) -> Option<&HeaderValue> {
        self.headers().get(name)
    }
    /// Append a header,the previous header (if exists) wouldn't be removed.

    pub fn append_header(&mut self, name: HeaderName, value: HeaderValue) {
        self.headers_mut().append(name, value);
    }
    /// Insert a header,if the header already exists,the previous header will be removed.

    pub fn insert_header(&mut self, name: HeaderName, value: HeaderValue) -> Option<HeaderValue> {
        self.headers_mut().insert(name, value)
    }

    /// Return the reference of the extension.

    pub const fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }
    /// Return the mutable reference of the extension.

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }
    /// Returns a refernece of associated extension.

    pub fn get_extension<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions().get()
    }

    /// Returns a mutable refernece of associated extension.
    pub fn get_mut_extension<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.extensions_mut().get_mut()
    }
    /// Remove a type from extensions.

    pub fn remove_extension<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.extensions_mut().remove()
    }
    /// Insert a type into extensions,if the type already exists,the old value will be returned.

    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, extension: T) -> Option<T> {
        self.extensions_mut().insert(extension)
    }

    /// Take the response body,leaving a frozen body.
    pub fn take_body(&mut self) -> Result<Body, BodyFrozen> {
        self.body.take()
    }
    /// Replace the value of the response body and return the old body.

    pub fn replace_body(&mut self, body: impl Into<Body>) -> Body {
        self.body.replace(body.into())
    }

    /// Swap the value of the response body with another body if the original body is not frozen.
    pub fn swap_body(&mut self, body: &mut Body) -> Result<(), BodyFrozen> {
        self.body.swap(body)
    }

    /// Map the body to a different value.
    pub fn map_body<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Body) -> Body,
    {
        self.body = f(self.body);
        self
    }
}
