use std::{borrow::Cow, pin::Pin};

use bytes::Bytes;
use bytestr::ByteStr;
use futures_lite::AsyncBufRead;

use super::{Body, BodyInner};

impl From<Bytes> for Body {
    fn from(data: Bytes) -> Self {
        Body::from_bytes(data)
    }
}

impl From<Vec<u8>> for Body {
    fn from(data: Vec<u8>) -> Self {
        Body::from_bytes(data)
    }
}

impl<'a> From<Cow<'a, [u8]>> for Body {
    fn from(data: Cow<[u8]>) -> Self {
        Body::from_bytes(data.into_owned())
    }
}

impl From<&[u8]> for Body {
    fn from(data: &[u8]) -> Self {
        Body::from_bytes(data.to_vec())
    }
}

impl From<ByteStr> for Body {
    fn from(data: ByteStr) -> Self {
        Body::from_bytes(data)
    }
}

impl From<String> for Body {
    fn from(data: String) -> Self {
        data.into_bytes().into()
    }
}

impl<'a> From<Cow<'a, str>> for Body {
    fn from(data: Cow<str>) -> Self {
        data.as_bytes().into()
    }
}

impl From<&str> for Body {
    fn from(data: &str) -> Self {
        data.as_bytes().into()
    }
}

impl From<Box<dyn AsyncBufRead + Send + Sync + 'static>> for Body {
    fn from(reader: Box<dyn AsyncBufRead + Send + Sync + 'static>) -> Self {
        Pin::from(reader).into()
    }
}

impl From<Pin<Box<dyn AsyncBufRead + Send + Sync + 'static>>> for Body {
    fn from(reader: Pin<Box<dyn AsyncBufRead + Send + Sync + 'static>>) -> Self {
        Self {
            inner: BodyInner::Reader {
                reader,
                length: None,
            },
        }
    }
}
