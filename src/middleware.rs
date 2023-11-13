//! Middleware allows you modify and read request or response during the request handling process.
//!
//! # Example
//! ```rust
//! // An implement of timeout middleware
//! use async_std::future::timeout;
//! use std::time::Duration;
//! use async_trait::async_trait;
//! use http_kit::{Request,Response,middleware::{Middleware,Next}};
//! struct TimeOut(Duration);
//!
//! #[async_trait]
//! impl Middleware for TimeOut{
//!     async fn call_middleware(&self, request: &mut Request, next: Next<'_>) -> http_kit::Result<Response>{
//!         timeout(self.duration,next.run(request)).await?
//!     }
//! }
//! ```

use crate::{Endpoint, Request, Response, Result};
use async_trait::async_trait;
use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc};

type SharedMiddleware = Arc<dyn Middleware>;
type BoxMiddleware = Box<dyn Middleware>;

/// Middleware allows reading and modifying requests or responses during the request handling process.
/// It is often used to implement functionalities such as timeouts, compression, etc.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Handle this request and return a response.Call `next` method of `Next` to handle remain middleware chain.
    async fn call_middleware(&self, request: &mut Request, next: Next<'_>) -> Result<Response>;
}

/// Represents the remaining part of the request handling chain.
pub struct Next<'a> {
    remain: &'a [SharedMiddleware],
    endpoint: &'a dyn Endpoint,
}

impl Debug for Next<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Next").finish()
    }
}

impl<'a> Next<'a> {
    /// Create a new `Next` instance ( normally having a complete handling chain).
    pub fn new(remain: &'a [SharedMiddleware], endpoint: &'a dyn Endpoint) -> Self {
        Self { remain, endpoint }
    }

    /// Execute the remain part of the handling chain.
    pub async fn run(self, request: &mut Request) -> Result<Response> {
        if let Some((first, remain)) = self.remain.split_first() {
            first
                .call_middleware(request, Next::new(remain, self.endpoint))
                .await
        } else {
            self.endpoint.call_endpoint(request).await
        }
    }
}

macro_rules! impl_middleware {
    ($($ty:ty),*) => {
        $(
            impl Middleware for $ty {
                fn call_middleware<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    request: &'life1 mut Request,
                    next: Next<'life2>,
                ) -> Pin<Box<dyn Future<Output = crate::Result<Response>>+Send+ 'async_trait>,>
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,

                    Self: 'async_trait
                {
                    use std::ops::Deref;
                    self.deref().call_middleware(request,next)
                }
            }
        )*
    };
}

impl_middleware![SharedMiddleware, BoxMiddleware];

#[async_trait]
impl Middleware for () {
    async fn call_middleware(&self, _request: &mut Request, _next: Next<'_>) -> Result<Response> {
        Ok(Response::empty())
    }
}
