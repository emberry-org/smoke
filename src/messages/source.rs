use serde::de::DeserializeOwned;
use tokio::io::AsyncBufRead;

use self::partial_tokio_copy::*;

pub trait Source {
    fn read_message<M: DeserializeOwned>(&mut self) -> ReadMsg<Self, M>;
    fn read_message_cancelable<'a, M: DeserializeOwned>(
        &'a mut self,
        agg: &'a mut Vec<u8>,
    ) -> ReadMsgCancel<Self, M>;
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

    /// Reads a [M] from the buf_reader, deserializing ([postcard]) the data.
    ///
    /// Equivalent to
    /// ```ignore
    /// async fn read_message_cancelable<M>(&mut self, agg: &mut Vec<u8>) -> io::Result<M>
    /// ```
    ///
    /// # Cancel safety
    /// This method is potentially cancellation safe. If the method is used as
    /// the event in a tokio::select statement and some other branch
    /// completes first, then some data may have been read to the `agg`.
    ///
    /// Calling this method again with the same buffer will take into account
    /// the data inside `agg` and complete successfully.
    ///
    /// # Note
    /// Modifying the `agg` directly is not considered a feature of this
    /// implementaiton and can lead to data loss.
    ///
    /// # Errors
    /// This function will return:</br>
    /// The first error returned by [self]<br>
    /// An [ErrorKind::Other] when buf_reader's buffer does not contain a valid [M]
    /// In this case calling the function again might repeatedly yield errors until a message
    /// is magically perfectly aligned.
    fn read_message_cancelable<'a, M: DeserializeOwned>(
        &'a mut self,
        agg: &'a mut Vec<u8>,
    ) -> ReadMsgCancel<Self, M>
    where
        R: AsyncBufRead + Unpin,
    {
        read_message_cancelable(self, agg)
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

    pin_project! {
        #[derive(Debug)]
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        pub struct ReadMsgCancel<'a, R: ?Sized, M: DeserializeOwned> {
            buf_reader: &'a mut R,
            agg: &'a mut Vec<u8>,
            // Make this future `!Unpin` for compatibility with async trait methods.
            #[pin]
            _pin: PhantomPinned,
            _message: PhantomData<M>,
        }
    }

    pub(crate) fn read_message_cancelable<'a, R, M: DeserializeOwned>(
        buf_reader: &'a mut R,
        agg: &'a mut Vec<u8>,
    ) -> ReadMsgCancel<'a, R, M>
    where
        R: AsyncBufRead + Unpin + ?Sized,
    {
        ReadMsgCancel {
            buf_reader,
            agg,
            _pin: PhantomPinned,
            _message: PhantomData,
        }
    }

    impl<'a, R, M: DeserializeOwned> Future for ReadMsgCancel<'a, R, M>
    where
        R: AsyncBufRead + Unpin + ?Sized,
    {
        type Output = io::Result<M>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<M>> {
            let me = self.project();

            // loop breaks on all ready values that need cleaning of the aggregator
            let ready = loop {
                // if the aggregator is empty try to read without copying the data
                if me.agg.is_empty() {
                    let data = ready!(Pin::new(&mut *me.buf_reader).poll_fill_buf(cx))?;

                    let (message, used) = try_deser::<M>(data);

                    // if we have no message push the data to the aggregator
                    if let Ok(None) = message {
                        me.agg.extend_from_slice(data);
                    }

                    // in any way consume used data
                    // used will be data.len() if msg was none or err
                    me.buf_reader.consume(used);

                    // ? returns Ready err if message has error variant
                    // return Ready msg if message has OK(Some()) variant
                    match message {
                        Ok(None) => {
                            if used == 0 {
                                // at this point `me.agg` was extended by &[] which means its still empty
                                // and we can return immediatly.
                                return Poll::Ready(Err(io::Error::new(
                                    ErrorKind::ConnectionReset,
                                    "EOF reached",
                                )));
                            }
                        }
                        Ok(Some(msg)) => {
                            // at this point the `me.agg` is still empty as its only modified when Ok(None)
                            // this means we can return instead of break
                            return Poll::Ready(Ok(msg));
                        }
                        Err(err) => {
                            // at this point the `me.agg` is still empty as its only modified when Ok(None)
                            // this means we can return instead of break
                            return Poll::Ready(Err(err));
                        }
                    }
                } else {
                    // `me.agg` is not empty so we cannot return error here but rather just break the loop
                    let data = match ready!(Pin::new(&mut *me.buf_reader).poll_fill_buf(cx)) {
                        Ok(v) => v,
                        Err(err) => break Err(err),
                    };

                    let agg_size_before = me.agg.len();
                    me.agg.extend_from_slice(data); //0001

                    let (message, used_total) = try_deser::<M>(me.agg);

                    // subtract agg_size_before to get the "new" bytes that were used
                    let used_data = used_total - agg_size_before;

                    // consume the data
                    // used will be data.len() if msg was none or err
                    me.buf_reader.consume(used_data);

                    match message {
                        Ok(None) => {
                            if used_data == 0 {
                                break Err(io::Error::new(
                                    ErrorKind::ConnectionReset,
                                    "EOF reached",
                                ));
                            }
                        }
                        Ok(Some(msg)) => {
                            break Ok(msg);
                        }
                        Err(err) => {
                            break Err(err);
                        }
                    }
                }
            };

            // clear the aggregator if something completed
            me.agg.clear();
            Poll::Ready(ready)
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
