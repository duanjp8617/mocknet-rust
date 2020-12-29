// a macro which is used to extract the response from nested Result types
macro_rules! extract_response {
    ($resp: expr,
     $fatal: expr,
     $err: expr) => {
        match $resp {
            Err(e) => {
                return Ok(
                    warp::reply::with_status(
                        format!("{{ \"reason\": \"{}: {}\" }}", $fatal, e), 
                        http::StatusCode::INTERNAL_SERVER_ERROR
                    )
                );
            },
            Ok(query_resp) => {
                match query_resp {
                    Ok(inner) => inner,
                    Err(err_msg) => {
                        return Ok(
                            warp::reply::with_status(
                                format!("{{ \"reason\": \"{}: {}\" }}", $err, err_msg), 
                                http::StatusCode::BAD_REQUEST
                            )
                        );
                    },
                }
            }
        }
    };
}

use warp::Filter;
use serde::de::DeserializeOwned;

// parse the input JSON message
//
// Note: when accepting a body, we want a JSON body and reject huge payloads
fn parse_json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}


pub mod register_user;
pub mod create_emunet;
pub mod init_emunet;