use serde::{Serialize, Deserialize};
use std::cmp::Eq;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct RoomId(pub [u8; 32]);