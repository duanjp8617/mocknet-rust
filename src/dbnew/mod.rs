mod message;
mod message_queue;
mod client;
mod errors;
mod request;

// this is made public to remove warnings
mod backend;
pub use backend::indradb_util;

pub use message::Succeed;
pub use message::Fail;
pub use message::QueryResult;
pub use errors::BackendError as ClientError;
pub use errors::BackendErrorKind as ClientErrorKind;
pub use client::Client;
pub use client::ClientLauncher;