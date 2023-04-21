///! Hypha (pl. hyphae) are the backbone of the port forwarding that emberry uses
///! to tunnel arbitrary application data from one client to another.
use serde::{Deserialize, Serialize};
use vlink::Action;

/// wraps [Action] for usage with smoke
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Signal {
    /// refer to [Action::Connect]
    Connect(u16),
    /// refer to [Action::Data]
    Data(u16, Vec<u8>),
    /// refer to [Action::Error]
    /// The error is stingified for serialization
    Error(u16, String),
    /// refer to [Action::AcceptError]
    /// The error is stingified for serialization
    AcceptError(String),
}

impl Signal {
    /// Converts [Signal] to [Action] using borrowing for the Data variant
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

    /// Converts [Action] to [Signal] cloning all the contained data
    pub fn from_vlink(vlink: &Action) -> Signal {
        match vlink {
            Action::Data(vport, data) => Signal::Data(*vport, Vec::from(*data)),
            Action::Connect(vport) => Signal::Connect(*vport),
            Action::Error(vport, err) => Signal::Error(*vport, err.to_string()),
            Action::AcceptError(err) => Signal::AcceptError(err.to_string()),
        }
    }
}
