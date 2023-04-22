pub mod messages;
mod user;

#[cfg(feature = "client")]
pub use messages::signal::Signal;
use std::time::Duration;
pub use user::User;

pub const ROOM_REQ_TIMEOUT: Duration = Duration::from_secs(10);
