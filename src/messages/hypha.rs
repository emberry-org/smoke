///! Hypha (pl. hyphae) are the backbone of the port forwarding that emberry uses
///! to tunnel arbitrary application data from one client to another.

use serde::{Serialize, Deserialize};

/// Container for all possible messages that are being sent from via Hyphae
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Signal {
    /// THE NAME "Vlan" IS TEMPORARY AND SUBJECT TO CHANGE
    ///
    /// "Vec<u8>" is the raw bytes to be transfered to the other peer
    Data(Vec<u8>),
    /// THE NAME "Vlan" IS TEMPORARY AND SUBJECT TO CHANGE
    ///
    /// Requests opening a tcp tunnel to the peer
    /// "u16" is the port on the remotes local host to which you want to establish a connection
    Request(u16),
    /// THE NAME "Vlan" IS TEMPORARY AND SUBJECT TO CHANGE
    ///
    /// "u16" is the port to which all Vlan traffic will be routed
    ///
    /// "String" is the stringified io::Error in case opening the socket failed
    /// it will also contain an error when the request was rejected
    Accept(Result<u16, String>),
    /// Kill the current Hypha closing the port connection
    Kill,
}
