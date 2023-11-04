use bytes::buf::Reader;
use bytes::{Buf, Bytes};
use futures_lite::ready;
use std::io::{BufRead, Read};
use std::ops::DerefMut;
use std::task::{Context, Poll};
use std::{io, pin::Pin};

use futures_lite::{AsyncBufRead, AsyncRead};

use super::{Body, BodyInner, BoxBufReader, BoxStream};

pub(crate) enum IntoAsyncRead {
    Once(Reader<Bytes>),
    Reader(BoxBufReader),
    Stream {
        stream: Option<BoxStream>,
        buf: Reader<Bytes>,
    },
    Freeze,
}

impl IntoAsyncRead {
    pub fn new(body: Body) -> Self {
        match body.inner {
            BodyInner::Once(data) => Self::Once(data.reader()),
            BodyInner::Reader { reader, .. } => Self::Reader(reader),
            BodyInner::Stream(stream) => Self::Stream {
                stream: Some(stream),
                buf: Bytes::new().reader(),
            },
            BodyInner::Freeze => Self::Freeze,
        }
    }
}

fn poll_data(
    optional_stream: &mut Option<BoxStream>,
    buf: &mut Reader<Bytes>,
    cx: &mut Context<'_>,
) -> Poll<io::Result<()>> {
    let stream;
    if let Some(s) = optional_stream {
        stream = s;
    } else {
        return Poll::Ready(Ok(()));
    }

    if !buf.get_ref().is_empty() {
        return Poll::Ready(Ok(()));
    }

    if let Some(data) = ready!(stream.as_mut().poll_next(cx))
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?
    {
        if data.is_empty() {
            return poll_data(optional_stream, buf, cx);
        }
        *buf = data.reader();
    } else {
        // Calling `poll_next` after the stream finished may cause problem,
        // so that we drop the stream after it finished.
        *optional_stream = None;
    }

    Poll::Ready(Ok(()))
}
impl AsyncRead for IntoAsyncRead {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        read_buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.deref_mut() {
            Self::Once(bytes) => Poll::Ready(bytes.read(read_buf)),
            Self::Reader(reader) => reader.as_mut().poll_read(cx, read_buf),
            Self::Stream { stream, buf } => {
                ready!(poll_data(stream, buf, cx))?;
                Poll::Ready(buf.read(read_buf))
            }
            Self::Freeze => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                super::Error::BodyFrozen,
            ))),
        }
    }
}

impl AsyncBufRead for IntoAsyncRead {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        match self.get_mut() {
            Self::Once(data) => Poll::Ready(data.fill_buf()),
            Self::Reader(reader) => reader.as_mut().poll_fill_buf(cx),
            Self::Stream { stream, buf } => {
                ready!(poll_data(stream, buf, cx))?;
                Poll::Ready(buf.fill_buf())
            }
            Self::Freeze => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                super::Error::BodyFrozen,
            ))),
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        match self.get_mut() {
            Self::Once(data) => data.consume(amt),
            Self::Reader(reader) => reader.as_mut().consume(amt),
            Self::Stream { buf, .. } => buf.consume(amt),
            Self::Freeze => {}
        }
    }
}
// TODO: test them.
