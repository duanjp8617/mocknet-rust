use lazy_static::lazy_static;
// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const BYTES_SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: uuid::Uuid = uuid::Uuid::from_bytes(BYTES_SEED);
}

mod client;
pub mod errors;
pub(crate) mod helpers;
mod message_queue;

pub use client::{new_connector, Client, Connector, QueryResult};
