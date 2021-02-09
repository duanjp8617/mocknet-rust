#[macro_use]
mod macros;

use serde::{de::DeserializeOwned, Serialize};
use warp::Filter;

// parse the input JSON message
//
// Note: when accepting a body, we want a JSON body and reject huge payloads
fn parse_json_body<T: DeserializeOwned + Send>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[derive(Serialize)]
struct Response<T: Serialize> {
    success: bool,
    data: T,
    message: String,
}

impl<T: Serialize> Response<T> {
    fn new(success: bool, data: T, message: String) -> Self {
        Self {
            success,
            data,
            message,
        }
    }
}

use crate::database::Client;
use crate::emunet::net::{EmuNet, EmuNetError};
// helper function to update error state on the emunet object
async fn emunet_error(client: Client, mut emunet: EmuNet, err: EmuNetError) {
    emunet.error(err);
    // store the error state in the database, panic the server program on failure
    let res = client
        .set_emu_net(emunet)
        .await
        .expect("this should not happen");
    if res.is_err() {
        panic!("this should never happen");
    }
}

pub mod create_emunet;
pub mod delete_emunet;
pub mod destruct_emunet;
pub mod get_emunet_info;
pub mod get_emunet_topo;
pub mod init_emunet;
pub mod list_emunet;
pub mod register_user;
