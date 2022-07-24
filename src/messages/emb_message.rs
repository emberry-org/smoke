use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncRead, AsyncWrite, BufReader};
use tokio_rustls::server::TlsStream as STlsStream;
use tokio_rustls::client::TlsStream as CTlsStream;

use crate::User;


pub const EMB_MESSAGE_BUF_SIZE: usize = 64;
pub type EmbMessageBuf = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum EmbMessage {
    Room(User),
    Accept(bool),
    Heartbeat,
    Shutdown,
}

impl EmbMessage {
    pub async fn recv_req<T>(
        tls: &mut BufReader<STlsStream<T>>,
        buf: &mut EmbMessageBuf,
    ) -> io::Result<EmbMessage>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // 0 means EOF so we shutdown the connection
        if 0 == tls.read_until(0, buf).await? {
            return Ok(EmbMessage::Shutdown);
        }

        postcard::from_bytes_cobs(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    pub async fn send_with<T>(self, tls: &mut BufReader<CTlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // use 192 byte buffer since we restrict wants room string to max 128 characters
        let bytes = postcard::to_vec_cobs::<Self, EMB_MESSAGE_BUF_SIZE>(&self)
            .expect("there was a big oopsie during developement!
            serialization size for EmbMessage was bigger then the specified buffer size");

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&bytes).await
    }
}
