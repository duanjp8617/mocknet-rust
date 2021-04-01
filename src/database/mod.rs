use lazy_static::lazy_static;

const BYTES_SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: uuid::Uuid = uuid::Uuid::from_bytes(BYTES_SEED);
}

mod client;
mod message_queue;

pub(crate) mod helpers;
pub(crate) use client::Client;

pub mod errors;
pub use client::{init, new_connector, Connector};
