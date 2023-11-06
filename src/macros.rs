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
                    self.deref().call_middleware(request,next)
                }

                fn name(&self) -> &'static str
                {
                    self.deref().name()
                }
            }
        )*
    };
}

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
            }
        )*
    };
}

macro_rules! impl_error {
    ($ty:ident,$message:expr) => {
        #[doc = concat!("The error type of `", stringify!($ty), "`.")]
        #[derive(Debug)]
        pub struct $ty {
            _priv: (),
        }

        impl $ty {
            pub(crate) fn new() -> Self {
                Self { _priv: () }
            }
        }

        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($message)
            }
        }

        impl std::error::Error for $ty {}
    };
}
