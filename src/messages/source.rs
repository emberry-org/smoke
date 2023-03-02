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
            agg: Vec<u8>,
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
            agg: Vec::new(),
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

            // if the aggregator is empty try to read without copying the data
            let data: &[u8];
            if me.agg.is_empty() {
                data = ready!(Pin::new(&mut *me.buf_reader).poll_fill_buf(cx))?;
                let (message, used) = try_deser::<M>(data);

                // if we have no message push the data to the aggregator
                if let Ok(None) = message {
                    me.agg.extend_from_slice(data);
                }

                // in any way consume used data
                // used will be data.len() if msg was none or err
                me.buf_reader.consume(used);

                try_finalize(message?, used, cx)
            } else {
                let data = ready!(Pin::new(&mut *me.buf_reader).poll_fill_buf(cx))?;
                let agg_size_before = me.agg.len();
                me.agg.extend_from_slice(data);

                let (message, used_total) = try_deser::<M>(me.agg);

                // subtract agg_size_before to get the "new" bytes that were used
                let used_data = used_total - agg_size_before;

                // consume the data
                // used will be data.len() if msg was none or err
                me.buf_reader.consume(used_data);

                let message = message?;

                try_finalize(message, used_data, cx)
            }
        }
    }

    fn try_finalize<M>(message: Option<M>, used: usize, cx: &mut Context<'_>) -> Poll<Result<M, io::Error>> {
        match message {
            Some(msg) => Poll::Ready(Ok(msg)),
            None => {
                if used == 0 {
                    Poll::Ready(Err(io::Error::new(
                        ErrorKind::ConnectionReset,
                        "EOF reached",
                    )))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
    }

    fn try_deser<M: DeserializeOwned>(data: &[u8]) -> (io::Result<Option<M>>, usize) {
        match postcard::take_from_bytes::<M>(data) {
            Err(postcard::Error::DeserializeUnexpectedEnd) => (Ok(None), data.len()),
            Err(err) => (Err(io::Error::new(ErrorKind::Other, err)), data.len()),
            Ok((msg, rest)) => (Ok(Some(msg)), data.len() - rest.len()),
        }
    }
}
