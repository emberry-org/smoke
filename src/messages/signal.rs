use serde::{Deserialize, Serialize};

use super::hypha;

pub const MAX_SIGNAL_BUF_SIZE: usize = 4096;

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
    /// Transfers a Vlink data package
    Vlink(hypha::Signal),
    /// Requests opening a tcp tunnel to the peer
    /// "u16" is the port on the remotes local host to which you want to establish a connection
    RequestVlink(u16),
    /// "String" is the stringified io::Error in case opening the socket failed
    /// it will also contain an error when the request was rejected
    AcceptVlink(Result<u16, String>),
    /// Kill the current Hypha closing the port connection
    KillVlink,
}
