use std::{future::Future, ops::Deref, pin::Pin, sync::Arc};

use async_trait::async_trait;

use crate::{Request, Response, Result};

/// A HTTP request processor.
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// The endpoint handles request and return a response.
    async fn call_endpoint(&self, request: &mut Request) -> Result<Response>;
}

impl_endpoint![
    Pin<Box<dyn Endpoint + Send + Sync + 'static>>,
    Pin<Arc<dyn Endpoint + Send + Sync + 'static>>
];
