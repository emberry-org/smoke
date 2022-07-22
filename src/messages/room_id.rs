use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RoomId(pub [u8; 32]);