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

pub mod create_emunet;
pub mod get_emunet;
pub mod init_emunet;
pub mod list_emunet;
pub mod register_user;
