use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_rustls::client::TlsStream as CTlsStream;
use tokio_rustls::server::TlsStream as STlsStream;

use crate::User;

pub const EMB_MESSAGE_BUF_SIZE: usize = 64;

/// Container for all possible messages that are being sent from Emberry (client) to Rhizome (server)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum EmbMessage {
    /// Request Rhizome to inform "User" that <we> would like to initiate a P2P connection
    Room(User),
    /// Accept/Deny a pending P2P connection request (true = Accept, false = Deny)
    Accept(bool),
    /// Message used for keepalive message activity if nessecary
    Heartbeat,
    /// Inform Rhizome about the termination of this connection. OR The connection has been closed (read yielded Ok(0))
    Shutdown,
}

impl EmbMessage {
    /// Serializes ([postcard]) and packetizes (COBS) "self" and sends the resulting binary data using the supplied Tls Stream
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If it is used as the event
    /// in a tokio::select statement and some other branch completes first,
    /// then the serialized message may have been partially written, but
    /// future calls will start from the beginning.
    ///
    /// # Errors
    /// This function will return:</br>
    /// The first error returned by writing to the Tls Stream.
    ///
    /// # Panics
    /// If "self" was to large to be serialized within [EMB_MESSAGE_BUF_SIZE].
    /// As of writing, this is technically impossible
    pub async fn send_with<T>(self, tls: &mut BufReader<CTlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // Serialize and packetize the message
        // use expect here as it is technically not possible for this to fail (Compiler cannot know that)
        let bytes = postcard::to_vec_cobs::<Self, EMB_MESSAGE_BUF_SIZE>(&self).expect(
            "there was a big oopsie during developement!
            serialization size for EmbMessage was bigger then the specified buffer size",
        );

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&bytes).await
    }

    /// Reads a [EmbMessage] from the Tls Stream, depacketizing (COBS) and deserializing ([postcard]) the data.
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
    /// The first error returned by reading from the Tls Stream.
    /// In this case the read data is written to the supplied buffer "buf" but not handled in any way.
    /// An [io::Error] when "self" was to large to be serialized within [EMB_MESSAGE_BUF_SIZE].</br>
    pub async fn recv_req<T>(
        tls: &mut BufReader<STlsStream<T>>,
        buf: &mut Vec<u8>,
    ) -> io::Result<EmbMessage>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        buf.clear();
        // 0 means EOF so we shutdown the connection
        if 0 == tls.read_until(0, buf).await? {
            return Ok(EmbMessage::Shutdown);
        }

        // depacketize and deserialize the message
        postcard::from_bytes_cobs(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
