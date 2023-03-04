use serde::{Deserialize, Serialize};

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
}
