mod message_queue;
mod indradb_backend;
mod client;
mod errors;

// this is made public to remove warnings
pub mod indradb_util;

pub use errors::BackendError as ClientError;
pub use errors::BackendErrorKind as ClientErrorKind;
pub use client::Client;
pub use client::ClientLauncher;