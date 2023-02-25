use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use std::io::{self, ErrorKind};
pub const MAX_SIGNAL_BUF_SIZE: usize = 4096;
use log::trace;

/// Container for all possible messages that are being sent from Rhizome (server) to Emberry (client)
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Signal {
    /// Keep alive message
    Kap,
    /// End of conversation. Prompts to close the connection
    EOC,
    /// "String" is the username of the sender
    Username(String),
    /// "String" is the unsanitized UTF-8 message content of a chat message
    Chat(String),
}

impl Signal {
    /// Serializes ([postcard]) "self" and asyncronously sends the resulting binary data using the supplied writer
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If it is used as the event
    /// in a tokio::select statement and some other branch completes first,
    /// then the serialized message may have been partially written, but
    /// future calls will start from the beginning.
    ///
    /// # Errors
    /// This function will return:</br>
    /// An [io::ErrorKind::Other] when "self" was to large to be serialized within [MAX_SIGNAL_BUF_SIZE].</br>
    /// The first error returned by the writer
    pub async fn send_with<T>(self, writer: &mut T) -> io::Result<()>
    where
        T: AsyncWrite + Unpin,
    {
        // Serialize and packetize the message
        let bytes = postcard::to_vec::<Self, MAX_SIGNAL_BUF_SIZE>(&self).map_err(|_| {
            io::Error::new(
                ErrorKind::OutOfMemory,
                "Unable to serialize Signal, more then 4096 byte",
            )
        })?;

        trace!("sent {}", bytes.len());
        writer.write_all(&bytes).await
    }

    /// Reads a [Signal] from the buf_reader, deserializing ([postcard]) the data.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If the method is used as
    /// the event in a tokio::select statement and some other branch
    /// completes first, then some data may have been partially read.
    ///
    /// Any partially read bytes are appended to buf. Calling this method
    /// again however will clear buf and will most likely result in deserialization
    /// failure.
    ///
    /// # Errors
    /// This function will return:</br>
    /// The first error returned by buf_reader<br>
    /// An [ErrorKind::Other] when buf_reader's buffer does not contain a valid [Signal]
    /// In this case this behaves poisoned as it will repeatedly try to deserialize the
    /// same buffer. To fix this situation one has to manually consume the buf_readers buffer.
    /// If the buf_reader buffer was partially consumed the consumed data is returned as a vec.
    /// (TODO01: CUSTOM ERROR TYPE THAT DOES THIS)
    pub async fn recv_with<T>(buf_reader: &mut T) -> io::Result<Self>
    where
        T: AsyncBufRead + Unpin,
    {
        // 0 means EOF
        let data = buf_reader.fill_buf().await?;
        if data.is_empty() {
            return Ok(Self::EOC);
        }

        // depacketize and deserialize the message
        let (signal, rest) = match postcard::take_from_bytes::<Signal>(data) {
            Err(err) => match err {
                postcard::Error::DeserializeUnexpectedEnd => {
                    let len = data.len();
                    let mut aggregator = Vec::with_capacity(len * 2);
                    aggregator.extend_from_slice(data);
                    buf_reader.consume(len);

                    loop {
                        let data = buf_reader.fill_buf().await?;
                        if data.is_empty() {
                            return Ok(Self::EOC);
                        }
                        aggregator.extend_from_slice(data);

                        match postcard::take_from_bytes::<Signal>(&aggregator) {
                            Ok((signal, rest)) => {
                                let consumed = data.len() - rest.len();
                                buf_reader.consume(consumed);
                                return Ok(signal);
                            }
                            Err(postcard::Error::DeserializeUnexpectedEnd) => {
                                let len = data.len();
                                buf_reader.consume(len);
                                // 0001
                            }
                            Err(err) => {
                                let len = data.len();
                                buf_reader.consume(len);
                                // (TODO01: return aggregator)
                                return Err(io::Error::new(ErrorKind::Other, err));
                            }
                        }
                    }
                }
                _ => return Err(io::Error::new(ErrorKind::Other, err)),
            },
            Ok(f) => f,
        };

        let consumed = data.len() - rest.len();
        buf_reader.consume(consumed);

        Ok(signal)
    }
}
