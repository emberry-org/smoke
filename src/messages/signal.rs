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
    /// THE NAME "Vlan" IS TEMPORARY AND SUBJECT TO CHANGE
    ///
    /// "Vec<u8>" is the raw bytes to be transfered to the other peer
    ///
    /// "String" is the stringified version of the io::Error the other side might encounter
    Vlan(Result<Vec<u8>, String>),
    /// THE NAME "Vlan" IS TEMPORARY AND SUBJECT TO CHANGE
    ///
    /// Requests opening a tcp tunnel to the peer
    /// "u16" is the port on the remotes local host to which you want to establish a connection
    VlanRequest(u16),
    /// THE NAME "Vlan" IS TEMPORARY AND SUBJECT TO CHANGE
    ///
    /// "u16" is the port to which all Vlan traffic will be routed
    ///
    /// "String" is the stringified io::Error in case opening the socket failed
    /// it will also contain an error when the request was rejected
    VlanAccept(Result<u16, String>),
}
