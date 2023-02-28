use serde::Serialize;
use tokio::io::AsyncWrite;

use self::tokio_copy::*;

pub trait Drain {
    fn serialize_to<'a, T: AsyncWrite + Unpin>(
        &self,
        writer: &'a mut T,
        ser_buf: &'a mut [u8],
    ) -> Result<WriteAll<'a, T>, postcard::Error>;
}

impl<M> Drain for M
where
    M: Serialize + ?Sized,
{
    /// Serializes ([postcard]) "self" and asyncronously sends the resulting binary data using the supplied writer
    /// 
    /// After handleing the [postcard::Error] this is equivalent to
    /// ```ignore
    /// async fn serialize_to(&self, writer: &mut T, ser_buf: &mut [u8]) -> io::Result<()>
    /// ```
    ///
    /// # Cancel safety
    /// This Future is not cancellation safe. If it is used as the event
    /// in a tokio::select statement and some other branch completes first,
    /// then the serialized message may have been partially written, but
    /// future calls will start from the beginning.
    ///
    /// # Errors
    /// This function will return:</br>
    /// An [postcard::Error] when `self` was unable to be serialized into `ser_buf`<br>
    /// The first error returned by the writer
    fn serialize_to<'a, T>(
        &self,
        writer: &'a mut T,
        ser_buf: &'a mut [u8],
    ) -> Result<WriteAll<'a, T>, postcard::Error>
    where
        T: AsyncWrite + Unpin,
    {
        // Serialize and packetize the message
        let bytes = postcard::to_slice(self, ser_buf)?;

        Ok(write_all(writer, bytes))
    }
}

mod tokio_copy {
    //! This is a copy from tokio-1.21.1/src/io/util/write_all.rs

    use pin_project_lite::pin_project;
    use std::future::Future;
    use std::io;
    use std::marker::PhantomPinned;
    use std::mem;
    use std::pin::Pin;
    use std::task::{ready, Context, Poll};
    use tokio::io::AsyncWrite;

    pin_project! {
        #[derive(Debug)]
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        pub struct WriteAll<'a, W: ?Sized> {
            writer: &'a mut W,
            buf: &'a [u8],
            // Make this future `!Unpin` for compatibility with async trait methods.
            #[pin]
            _pin: PhantomPinned,
        }
    }

    pub(crate) fn write_all<'a, W>(writer: &'a mut W, buf: &'a [u8]) -> WriteAll<'a, W>
    where
        W: AsyncWrite + Unpin + ?Sized,
    {
        WriteAll {
            writer,
            buf,
            _pin: PhantomPinned,
        }
    }

    impl<W> Future for WriteAll<'_, W>
    where
        W: AsyncWrite + Unpin + ?Sized,
    {
        type Output = io::Result<()>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            let me = self.project();
            while !me.buf.is_empty() {
                let n = ready!(Pin::new(&mut *me.writer).poll_write(cx, me.buf))?;
                {
                    let (_, rest) = mem::take(&mut *me.buf).split_at(n);
                    *me.buf = rest;
                }
                if n == 0 {
                    return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                }
            }

            Poll::Ready(Ok(()))
        }
    }
}
