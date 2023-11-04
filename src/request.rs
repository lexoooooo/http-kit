use crate::{body::BodyFrozen, Body};
use http::{header::HeaderName, Extensions, HeaderMap, HeaderValue, Method, Uri, Version};
use std::fmt::Debug;

type RequestParts = http::request::Parts;

/// An HTTP request.
#[derive(Debug)]
pub struct Request {
    parts: RequestParts,
    body: Body,
}

impl From<http::Request<Body>> for Request {
    fn from(request: http::Request<Body>) -> Self {
        let (parts, body) = request.into_parts();
        Self { parts, body }
    }
}

impl From<Request> for http::Request<Body> {
    fn from(request: Request) -> Self {
        Self::from_parts(request.parts, request.body)
    }
}

impl Request {
    /// Create a new `Request`.
    pub fn new<U>(method: Method, uri: U) -> Self
    where
        U: TryInto<Uri>,
        U::Error: Debug,
    {
        http::Request::builder()
            .method(method)
            .uri(uri.try_into().unwrap())
            .body(Body::empty())
            .unwrap()
            .into()
    }

    /// Create a GET `Request`.
    pub fn get<U>(uri: U) -> Self
    where
        U: TryInto<Uri>,
        U::Error: Debug,
    {
        Self::new(Method::GET, uri)
    }
    /// Create a POST `Request`.

    pub fn post<U>(uri: U) -> Self
    where
        U: TryInto<Uri>,
        U::Error: Debug,
    {
        Self::new(Method::POST, uri)
    }
    /// Create a PUT `Request`.

    pub fn put<U>(uri: U) -> Self
    where
        U: TryInto<Uri>,
        U::Error: Debug,
    {
        Self::new(Method::PUT, uri)
    }
    /// Create a DELETE `Request`.

    pub fn delete<U>(uri: U) -> Self
    where
        U: TryInto<Uri>,
        U::Error: Debug,
    {
        Self::new(Method::DELETE, uri)
    }
    /// Return the reference of reqeust parts.
    pub const fn parts(&self) -> &RequestParts {
        &self.parts
    }
    /// Return the mutable reference of request parts.

    pub fn parts_mut(&mut self) -> &mut RequestParts {
        &mut self.parts
    }

    /// Return the reference of request method.

    pub const fn method(&self) -> &Method {
        &self.parts.method
    }

    /// Return the mutable reference of request method.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.parts.method
    }

    /// Set the request method by `method`
    pub fn set_method(&mut self, method: Method) {
        *self.method_mut() = method;
    }

    /// Return the reference of request URI.
    pub const fn uri(&self) -> &Uri {
        &self.parts.uri
    }
    /// Return the mutable reference of URI method.

    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.parts.uri
    }

    /// Rewrite the HTTP uri.
    pub fn set_uri(&mut self, uri: Uri) {
        *self.uri_mut() = uri;
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

    /// Append a header,the previous header (if exists) would'nt be removed.
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

    /// Take the request body,leaving a frozen body.
    pub fn take_body(&mut self) -> Result<Body, BodyFrozen> {
        self.body.take()
    }

    /// Replace the value of the request body and return the old body.
    pub fn replace_body(&mut self, body: impl Into<Body>) -> Body {
        self.body.replace(body.into())
    }

    /// Swap the value of the request body with another body if the orginal body is not frozen.
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

    /// Set the MIME.
    #[cfg(feature = "mime")]
    pub fn set_mime(&mut self, mime: mime::Mime) {
        self.insert_header(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_str(mime.as_ref()).unwrap(),
        );
    }

    /// Try to parse the header and return a `Mime` instance.
    #[cfg(feature = "mime")]
    pub fn mime(&self) -> Option<mime::Mime> {
        Some(
            std::str::from_utf8(self.get_header(http::header::CONTENT_TYPE)?.as_bytes())
                .ok()?
                .parse()
                .ok()?,
        )
    }
}
