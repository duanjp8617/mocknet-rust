
use lazy_static::lazy_static;
// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const BYTES_SEED: [u8; 16] = [1, 2,  3,  4,  5,  6,  7,  8,
                              9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: uuid::Uuid = uuid::Uuid::from_bytes(BYTES_SEED);
}

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