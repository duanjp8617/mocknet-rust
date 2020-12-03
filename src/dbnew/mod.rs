mod message_queue;
mod indradb_backend;
mod client;

mod indradb_util;

pub mod errors;
pub type ClientError = errors::MsgQError<errors::BackendError>;
pub use client::Client;
pub use client::ClientLauncher;