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

#[cfg(any(feature = "json", feature = "form"))]
macro_rules! assert_content_type {
    ($mime:expr,$headermap:expr) => {
        impl_error!(
            ContentTypeMismatched,
            concat!("Content-type is mismatched, expected `", $mime, "`")
        );
        let content_type = $headermap
            .get(crate::header::CONTENT_TYPE)
            .map(|s| s.as_bytes())
            .unwrap_or(b"");
        if content_type != $mime.as_bytes() {
            return Err(crate::Error::new(
                ContentTypeMismatched::new(),
                crate::StatusCode::BAD_REQUEST,
            ));
        }
    };
}
