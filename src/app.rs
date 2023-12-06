use std::{fmt::Debug, sync::Arc};

use crate::{
    middleware::{Next, SharedMiddleware},
    Endpoint, Middleware, Request, Response,
};

/// An App containing endpoint and middlewares.
#[derive(Default)]
pub struct App<E: Endpoint> {
    endpoint: E,
    middlewares: Vec<SharedMiddleware>,
}

impl<E: Endpoint> Debug for App<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("endpoint", &self.endpoint.name())
            .field("middlewares", &self.middlewares)
            .finish()
    }
}

impl<E: Endpoint> App<E> {
    /// Create an app from endpoint.
    pub const fn new(endpoint: E) -> Self {
        Self {
            endpoint,
            middlewares: Vec::new(),
        }
    }

    /// Add a shared middleware to this app.
    pub fn add_middleware(&mut self, middleware: SharedMiddleware) {
        self.middlewares.push(middleware);
    }

    /// Add a middleware to this app.
    pub fn middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.add_middleware(Arc::new(middleware));
        self
    }

    /// Run the app with a provided request.
    pub async fn run(&self, mut request: Request) -> crate::Result<Response> {
        Next::new(&self.middlewares, &self.endpoint)
            .run(&mut request)
            .await
    }
}
