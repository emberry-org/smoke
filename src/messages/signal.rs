use serde::{Deserialize, Serialize};

pub const MAX_SIGNAL_BUF_SIZE: usize = 4096;
pub const MAX_FILE_PART_DATA_SIZE: usize = MAX_SIGNAL_BUF_SIZE - 18;

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
    /// "String" is the file identifier,
    /// "u64" is pointing to the byte index at which the data from "Vec<u8>" starts
    FilePart(String, u64, Vec<u8>),
}
