mod message_queue;
mod indradb_backend;

pub mod errors;
pub mod indradb_util;

pub type IndradbClientError = errors::MsgQError<errors::BackendError>;

pub mod client;