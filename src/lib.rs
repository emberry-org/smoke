pub mod messages;
mod user;

#[cfg(feature = "client")]
pub use messages::signal::Signal;
pub use user::User;
use std::time::Duration;

pub const ROOM_REQ_TIMEOUT: Duration = Duration::from_secs(10);
