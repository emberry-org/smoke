use serde::de::DeserializeOwned;
use tokio::io::AsyncBufRead;

use self::partial_tokio_copy::*;

pub trait Source {
    fn read_message<M: DeserializeOwned>(&mut self) -> ReadMsg<Self, M>;
}

impl<R> Source for R
where
    R: AsyncBufRead + Unpin,
{
    /// Reads a [M] from the buf_reader, deserializing ([postcard]) the data.
    /// 
    /// Equivalent to
    /// ```ignore
    /// async fn read_message<M>(&mut self) -> io::Result<M>
    /// ```
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If the method is used as
    /// the event in a tokio::select statement and some other branch
    /// completes first, then some data may have been partially read.
    ///
    /// Any partially read bytes are appended to buf. Calling this method
    /// again however will use a new buf and will most likely result in deserialization
    /// failure.
    ///
    /// # Errors
    /// This function will return:</br>
    /// The first error returned by [self]<br>
    /// An [ErrorKind::Other] when buf_reader's buffer does not contain a valid [M]
    /// In this case calling the function again might repeatedly yield errors until a message
    /// is magically perfectly aligned.
    fn read_message<M: DeserializeOwned>(&mut self) -> ReadMsg<Self, M>
    where
        R: AsyncBufRead + Unpin,
    {
        read_message(self)
    }
}

mod partial_tokio_copy {
    //! This is a modified copy from tokio-1.21.1/src/io/util/write_all.rs

    use pin_project_lite::pin_project;
    use serde::de::DeserializeOwned;
    use std::future::Future;
    use std::io::{self, ErrorKind};
    use std::marker::{PhantomData, PhantomPinned};
    use std::pin::Pin;
    use std::task::{ready, Context, Poll};
    use tokio::io::{AsyncBufRead, AsyncBufReadExt};

    pin_project! {
        #[derive(Debug)]
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        pub struct ReadMsg<'a, R: ?Sized, M: DeserializeOwned> {
            buf_reader: &'a mut R,
            agg: Option<Vec<u8>>,
            // Make this future `!Unpin` for compatibility with async trait methods.
            #[pin]
            _pin: PhantomPinned,
            _message: PhantomData<M>,
        }
    }

    pub(crate) fn read_message<R, M: DeserializeOwned>(
        buf_reader: &mut R,
        // could give in vec
    ) -> ReadMsg<R, M>
    where
        R: AsyncBufRead + Unpin + ?Sized,
    {
        ReadMsg {
            buf_reader,
            agg: None,
            _pin: PhantomPinned,
            _message: PhantomData,
        }
    }

    impl<'a, R, M: DeserializeOwned> Future for ReadMsg<'a, R, M>
    where
        R: AsyncBufRead + Unpin + ?Sized,
    {
        type Output = io::Result<M>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<M>> {
            let me = self.project();

            loop {
                // if the aggregator is empty try to read without copying the data
                let Some(agg) = me.agg else {
                    let data = ready!(Pin::new(&mut *me.buf_reader).poll_fill_buf(cx))?;

                    let (message, used) = try_deser::<M>(data);

                    // if we have no message push the data to the aggregator
                    if let Ok(None) = message {
                        let mut vec = Vec::with_capacity(data.len() * 2);
                        vec.extend_from_slice(data);
                        *me.agg = Some(vec);
                    }

                    // in any way consume used data
                    // used will be data.len() if msg was none or err
                    me.buf_reader.consume(used);

                    if let Some(message) = message? {
                        return Poll::Ready(Ok(message));
                    }

                    if used == 0 {
                        return Poll::Ready(Err(io::Error::new(
                            ErrorKind::ConnectionReset,
                            "EOF reached",
                        )));
                    }

                    continue;
                };

                let data = ready!(Pin::new(&mut *me.buf_reader).poll_fill_buf(cx))?;

                let agg_size_before = agg.len();
                agg.extend_from_slice(data); //0001

                let (message, used_total) = try_deser::<M>(agg);

                // subtract agg_size_before to get the "new" bytes that were used
                let used_data = used_total - agg_size_before;

                // consume the data
                // used will be data.len() if msg was none or err
                me.buf_reader.consume(used_data);

                if let Some(message) = message? {
                    return Poll::Ready(Ok(message));
                }

                if used_data == 0 {
                    return Poll::Ready(Err(io::Error::new(
                        ErrorKind::ConnectionReset,
                        "EOF reached",
                    )));
                }
            }
        }
    }

    #[inline]
    fn try_deser<M: DeserializeOwned>(data: &[u8]) -> (io::Result<Option<M>>, usize) {
        match postcard::take_from_bytes::<M>(data) {
            Err(postcard::Error::DeserializeUnexpectedEnd) => (Ok(None), data.len()),
            Err(err) => (Err(io::Error::new(ErrorKind::Other, err)), data.len()),
            Ok((msg, rest)) => (Ok(Some(msg)), data.len() - rest.len()),
        }
    }
}
