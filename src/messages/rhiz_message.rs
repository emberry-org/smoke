use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_rustls::client::TlsStream as CTlsStream;
use tokio_rustls::server::TlsStream as STlsStream;

use crate::User;

use std::io::{self, ErrorKind};
pub const MAX_MESSAGE_BUF_SIZE: usize = 1088;

use super::RoomId;

/// Container for all possible messages that are being sent from Rhizome (server) to Emberry (client)
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum RhizMessage {
    /// "User" IS connected to the Rhizome network
    HasRoute(User),
    /// "User" is NOT connected to the Rhizome network
    NoRoute(User),
    /// "User" wants to establish a peer to peer connection
    WantsRoom(User),
    /// "User" is waiting for UDP Holepunching at "RoomId". "Option" is NONE when "User" denied P2P connection.
    AcceptedRoom(Option<RoomId>, User),
    /// Rhizome internal server error. "String" is the error message. This is sent for debugability.
    ServerError(String),
    /// Rhizome wants to terminate the connection. OR The connection has been closed (read yielded Ok(0))
    Shutdown(),
}

impl RhizMessage {
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
    /// An [io::Error] when "self" was to large to be serialized within [MAX_MESSAGE_BUF_SIZE].</br>
    /// The first error returned by writing to the Tls Stream.
    pub async fn send_with<T>(self, tls: &mut BufReader<STlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // Serialize and packetize the message
        let bytes = postcard::to_vec_cobs::<Self, MAX_MESSAGE_BUF_SIZE>(&self).map_err(|_| {
            io::Error::new(
                ErrorKind::OutOfMemory,
                "Unable to serialize RhizMessage, more then 64 byte",
            )
        })?;

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&bytes).await
    }

    /// Reads a [RhizMessage] from the Tls Stream, depacketizing (COBS) and deserializing ([postcard]) the data.
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
    /// An [io::Error] when "self" was to large to be serialized within [MAX_MESSAGE_BUF_SIZE].</br>
    pub async fn recv_with<T>(
        tls: &mut BufReader<CTlsStream<T>>,
        buf: &mut Vec<u8>,
    ) -> io::Result<RhizMessage>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        buf.clear();
        // 0 means EOF so we shutdown the connection
        if 0 == tls.read_until(0, buf).await? {
            return Ok(RhizMessage::Shutdown());
        }

        // depacketize and deserialize the message
        postcard::from_bytes_cobs(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
