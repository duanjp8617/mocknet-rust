mod message_queue;
mod indradb_client;
mod indradb_backend;

pub mod errors;
pub mod indradb_util;

pub use self::indradb_client::IndradbClient;
pub use self::indradb_client::build_client_fut;

pub type IndradbClientError = errors::MsgQError<errors::BackendError>;

mod client;