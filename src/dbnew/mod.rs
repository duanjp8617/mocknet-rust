mod message_queue;
mod indradb_backend;
mod client;
mod errors;

// this is made public to remove warnings
pub mod indradb_util;

pub type QueryResult<T> = Result<T, String>;
pub use Result::Ok as QueryOk;
pub use Result::Err as QueryFail;
pub use errors::BackendError as ClientError;
pub use errors::BackendErrorKind as ClientErrorKind;
pub use client::Client;
pub use client::ClientLauncher;

mod message;

use lazy_static::lazy_static;
// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const BYTES_SEED: [u8; 16] = [1, 2,  3,  4,  5,  6,  7,  8,
                              9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: uuid::Uuid = uuid::Uuid::from_bytes(BYTES_SEED);
}