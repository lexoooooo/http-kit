use std::fmt::Debug;

use crate::{
    middleware::{Next, SharedMiddleware},
    Endpoint, Request, Response,
};

/// An App containing endpoint and middlewares.
#[derive(Default)]
pub struct App<E: Endpoint> {
    endpoint: E,
    middlewares: Box<[SharedMiddleware]>,
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
    /// Create an app from endpoint and middlewares.
    pub const fn new(endpoint: E, middlewares: Box<[SharedMiddleware]>) -> Self {
        Self {
            endpoint,
            middlewares,
        }
    }

    /// Run the app with a provided request.
    pub async fn run(&self, mut request: Request) -> crate::Result<Response> {
        Next::new(&self.middlewares, &self.endpoint)
            .run(&mut request)
            .await
    }
}
