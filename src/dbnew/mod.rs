mod message_queue;
mod indradb_backend;
mod client;

// this is made public to remove warnings
pub mod indradb_util;

pub mod errors;
pub type ClientError = errors::BackendError;
pub type ClientErrorKind = errors::BackendErrorKind;
pub use client::Client;
pub use client::ClientLauncher;