///! Hypha (pl. hyphae) are the backbone of the port forwarding that emberry uses
///! to tunnel arbitrary application data from one client to another.
use serde::{Deserialize, Serialize};
use vlink::Action;

/// Container for all possible messages that are being sent from via Hyphae
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Signal {
    /// "u16" is vport
    Connect(u16),
    /// "u16" is vport, "Vec<u8>" is data
    Data(u16, Vec<u8>),
    /// "u16" is vport, "String" is stringified `io::Error`
    Error(u16, String),
    /// "String" is stringified `io::Error`
    AcceptError(String),
}

impl Signal {
    pub fn as_vlink(&self) -> Action<'_> {
        match self {
            Signal::Data(vport, data) => Action::Data(*vport, data),
            Signal::Connect(vport) => Action::Connect(*vport),
            Signal::Error(vport, err) => Action::Error(
                *vport,
                std::io::Error::new(std::io::ErrorKind::Other, err.clone()),
            ),
            Signal::AcceptError(err) => {
                Action::AcceptError(std::io::Error::new(std::io::ErrorKind::Other, err.clone()))
            }
        }
    }

    pub fn from_vlink(vlink: &Action) -> Signal {
        match vlink {
            Action::Data(vport, data) => Signal::Data(*vport, Vec::from(*data)),
            Action::Connect(vport) => Signal::Connect(*vport),
            Action::Error(vport, err) => Signal::Error(*vport, err.to_string()),
            Action::AcceptError(err) => Signal::AcceptError(err.to_string()),
        }
    }
}
