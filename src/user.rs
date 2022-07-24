use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct User {
    pub key: PubKey,
}

pub type PubKey = [u8; 32];
