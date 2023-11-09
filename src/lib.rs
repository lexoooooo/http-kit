#![deny(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]
//! A plenty of utlity for HTTP operation.
//! # Example
//! ```rust
//! use http_kit::{Request,Response};
//!
//! async fn echo(request:Request) -> http_kit::Result<Response>{
//!     let body = request.take_body()?;
//!     Ok(Response::new(200,body))
//! }
//!
//! let mut request = Request::get("/echo");
//! request.replace_body("Hello,world");
//! echo(request).await?;
//!
//! ```
#[macro_use]
mod macros;

mod error;
pub use error::{Error, Result, ResultExt};

mod body;
pub use body::Body;
pub use body::Error as BodyError;

pub mod middleware;
#[doc(inline)]
pub use middleware::Middleware;

mod hook;
pub use hook::Hook;

mod request;
pub use request::Request;
mod response;
pub use response::Response;

pub use http::{header, method, uri, version, Extensions, Method, StatusCode, Uri, Version};
