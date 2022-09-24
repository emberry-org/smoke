use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

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
    /// Serializes ([postcard]) and packetizes (COBS) "self" and sends the resulting binary data using the supplied io
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If it is used as the event
    /// in a tokio::select statement and some other branch completes first, 
    /// then the serialized message may have been partially written, but 
    /// future calls will start from the beginning.
    ///
    /// # Errors
    /// This function will return:</br>
    /// An [io::Error] when "self" was to large to be serialized within [MAX_SIGNAL_BUF_SIZE].</br>
    /// The first error returned by writing to the io
    pub async fn send_with<T>(self, tls: &mut BufReader<T>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // Serialize and packetize the message
        let bytes = postcard::to_vec_cobs::<Self, MAX_SIGNAL_BUF_SIZE>(&self).map_err(|_| {
            io::Error::new(
                ErrorKind::OutOfMemory,
                "Unable to serialize RhizMessage, more then 64 byte",
            )
        })?;

        trace!("sent {}", bytes.len());
        tls.write_all(&bytes).await
    }

    /// Reads a [Signal] from the IO, depacketizing (COBS) and deserializing ([postcard]) the data.
    /// This method clears the provided buffer before reading to it from the TlsStream
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
    /// The first error returned by reading from the IO
    /// In this case the read data is written to the supplied buffer "buf" but not handled in any way.
    /// An [io::Error] when "self" was to large to be serialized within [MAX_SIGNAL_BUF_SIZE].</br>
    pub async fn recv_with<T>(
        tls: &mut BufReader<T>,
        buf: &mut Vec<u8>,
    ) -> io::Result<Self>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        buf.clear();
        // 0 means EOF
        if 0 == tls.read_until(0, buf).await? {
            return Ok(Self::EOC);
        }

        trace!("recv {}", buf.len());
        // depacketize and deserialize the message
        postcard::from_bytes_cobs(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
