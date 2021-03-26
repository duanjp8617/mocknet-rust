use lazy_static::lazy_static;
// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const BYTES_SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: uuid::Uuid = uuid::Uuid::from_bytes(BYTES_SEED);
}

mod errors;
pub use errors::BackendError as ClientError;
pub use errors::BackendErrorKind as ClientErrorKind;

mod indradb;
pub use self::indradb::indradb_util;

mod client;
pub use client::Client;
pub use client::ClientLauncher;
