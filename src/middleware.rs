//! Middleware allows you modify and read request or response.
//! 
//! # Example
//! ```rust
//! // An implement of timeout middleware
//! use async_std::future::timeout;
//! use std::time::Duration;
//! use async_trait::async_trait;
//! use http_util::{Request,middleware::{Middleware,Next}};

//! struct TimeOut(Duration);
//! 
//! #[async_trait]
//! impl Middleware for TimeOut{
//!     async fn call_middleware(&self, request: &mut Request, next: Next<'_>) -> Result<Response>{
//!         timeout(self.duration,next.run(request)).await?
//!     }
//! }
//! ```



use std::{fmt::Debug, ops::Deref, pin::Pin, sync::Arc};

use crate::{Endpoint, Request, Response, Result};
use async_trait::async_trait;
use std::future::Future;

/// The remain part of the handling chain,inclduing endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a> {
    remain_middlewares: &'a [SharedMiddleware],
    endpoint: &'a (dyn Endpoint + Send + Sync),
}

impl<'a> Next<'a> {
    /// Create a `Next` instance with a complete handling chain.
    pub fn new(
        middlewares: &'a [SharedMiddleware],
        endpoint: &'a (dyn Endpoint + Send + Sync),
    ) -> Self {
        Self {
            remain_middlewares: middlewares,
            endpoint,
        }
    }

    /// Execute the remain part of the handling chain.
    pub async fn run(mut self, request: &mut Request) -> Result<Response> {
        if let Some((middleware, remain)) = self.remain_middlewares.split_first() {
            self.remain_middlewares = remain;
            middleware.call_middleware(request, self).await
        } else {
            self.endpoint.call_endpoint(request).await
        }
    }
}

/// Middleware can read and modify the request or response.
/// It is always used to implement timeout,compression,etc.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Handle this request and return a response.Call `next` method of `Next` to handle remain middleware chain.
    async fn call_middleware(&self, request: &mut Request, next: Next<'_>) -> Result<Response>;

    /// Accquire the name of the middleware.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl Debug for (dyn Middleware) {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Debug for (dyn Middleware + Send + Sync) {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

type BoxMiddleware = Pin<Box<dyn Middleware + Send + Sync + 'static>>;
type SharedMiddleware = Pin<Arc<dyn Middleware + Send + Sync + 'static>>;

impl_middleware![BoxMiddleware, SharedMiddleware];
