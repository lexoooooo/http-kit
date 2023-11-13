use std::{any::type_name, fmt::Debug, future::Future, ops::Deref, pin::Pin, sync::Arc};

use async_trait::async_trait;

use crate::{Request, Response, Result};

/// A HTTP request processor.
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// The endpoint handles request and return a response.
    async fn call_endpoint(&self, request: &mut Request) -> Result<Response>;
    /// Get the name of the middleware, which will default to the type name of this middleware.
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
}

type SharedEndpoint = Box<dyn Endpoint>;
type BoxEndpoint = Arc<dyn Endpoint>;

macro_rules! impl_endpoint {
    ($($ty:ty),*) => {
        $(
            impl Endpoint for $ty {
                fn call_endpoint<'life0, 'life1, 'async_trait>(
                    &'life0 self,
                    request: &'life1 mut Request,
                ) -> Pin<Box<dyn Future<Output = crate::Result<Response>>+ Send+ 'async_trait>>
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait
                {
                    self.deref().call_endpoint(request)
                }

                fn name(&self) -> &'static str{
                    self.deref().name()
                }
            }
        )*
    };
}

impl_endpoint![SharedEndpoint, BoxEndpoint];

#[async_trait]
impl Endpoint for () {
    async fn call_endpoint(&self, _request: &mut Request) -> Result<Response> {
        Ok(Response::empty())
    }
}

impl Debug for dyn Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
