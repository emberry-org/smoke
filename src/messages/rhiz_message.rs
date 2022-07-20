use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_rustls::server::TlsStream as STlsStream;
use tokio_rustls::client::TlsStream as CTlsStream;

use crate::User;

use std::io::{self, ErrorKind};

#[derive(Clone, Serialize, Deserialize)]
pub enum RhizMessage {
    HasRoute(User),
    NoRoute(User),
    WantsRoom(User),
    AcceptedRoom(Option<[u8; 32]>),
    ServerError(String),
    Shutdown(),
}

impl RhizMessage {
    pub async fn send_with<T>(self, tls: &mut BufReader<STlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // use 192 byte buffer since we restrict wants room string to max 128 characters
        let bytes = postcard::to_vec_cobs::<Self, 64>(&self).map_err(|_| {
            io::Error::new(ErrorKind::OutOfMemory, "Unable to serialize RhizMessage, more then 64 byte")
        })?;

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&bytes).await
    }

    pub async fn recv_with<T>(
        tls: &mut BufReader<CTlsStream<T>>,
        buf: &mut Vec<u8>,
    ) -> io::Result<RhizMessage>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // 0 means EOF so we shutdown the connection
        if 0 == tls.read_until(0, buf).await? {
            return Ok(RhizMessage::Shutdown());
        }

        postcard::from_bytes_cobs(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
