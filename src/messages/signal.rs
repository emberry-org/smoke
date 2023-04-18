use serde::{Deserialize, Serialize};

use super::vlink;

pub const MAX_SIGNAL_BUF_SIZE: usize = 4096;

/// Container for all possible messages that are being sent from Emberry (client) to Emberry (client) (p2p)
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Signal {
    /// Keep alive message
    Kap,
    /// End of conversation. Prompts to close the connection
    EOC,
    /// `Username( ... ).0` - sets the `username` of the peer
    Username(String),
    /// `Vlink( ... ).0` - Transfers a wrapped [vlink::Action] to the currently active [vlink::TcpBridge]
    Vlink(vlink::Signal),
    /// `VlinkOpen( ... ).0` - `vlinkid`
    /// 
    /// Informs the peer that a vlink with the name of `vlinkid` has been opened to accept connections
    VlinkOpen(String),
    /// Cuts the current Vlink connection
    /// 
    /// This is only valid information when the client sending this previously sent VlinkOpen
    VlinkCut,
    /// `ChangeContext( ... ).0` - changes the `context` to which all following context sensitive [Signal]s are adressed
    ChangeContext(String),
    /// CONTEXT SENSITIVE
    /// 
    /// `Message( ... ).0` - inputs `message` to the current `context`
    /// 
    /// `message` is unsalitized UTF-8 user input
    Message(String),
}
