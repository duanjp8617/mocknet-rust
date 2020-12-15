pub mod indradb_util;

mod indradb_backend;
pub use indradb_backend::IndradbClientBackend;
pub use indradb_backend::build_backend_fut;

mod init;
mod register_user;
mod create_emu_net;

use lazy_static::lazy_static;
// CORE_INFO_ID is a vertex id that stores core inforamtion of mocknet.
const BYTES_SEED: [u8; 16] = [1, 2,  3,  4,  5,  6,  7,  8,
                              9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    static ref CORE_INFO_ID: uuid::Uuid = uuid::Uuid::from_bytes(BYTES_SEED);
}