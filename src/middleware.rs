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

use crate::{Request, Response, Result};
use async_trait::async_trait;
use std::{future::Future, pin::Pin};

// Recursive expansion of async_trait macro
// =========================================

impl<T: Middleware> Middleware for &T {
    fn call_middleware<'life0, 'life1, 'async_trait>(
        &'life0 self,
        request: &'life1 mut Request,
        next: impl 'async_trait + Middleware,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        (*self).call_middleware(request, next)
    }
}

impl<T1: Middleware, T2: Middleware> Middleware for (T1, T2) {
    fn call_middleware<'life0, 'life1, 'async_trait>(
        &'life0 self,
        request: &'life1 mut Request,
        _next: impl 'async_trait + Middleware,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.0.call_middleware(request, &self.1)
    }
}

/// Middleware allows reading and modifying requests or responses during the request handling process.
/// It is often used to implement functionalities such as timeouts, compression, etc.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Handle this request and return a response.Call `next` method of `Next` to handle remain middleware chain.
    async fn call_middleware(
        &self,
        request: &mut Request,
        next: impl Middleware,
    ) -> Result<Response>;
}
