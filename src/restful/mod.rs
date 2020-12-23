macro_rules! extract_response {
    ($resp: expr,
     $fatal: expr,
     $err: expr) => {
        match $resp {
            Err(e) => {
                return Ok(warp::reply::with_status(format!("{}: {}", $fatal, e), http::StatusCode::INTERNAL_SERVER_ERROR));
            },
            Ok(query_resp) => {
                match query_resp {
                    Ok(inner) => inner,
                    Err(err_msg) => {
                        return Ok(warp::reply::with_status(format!("{}: {}", $err, err_msg), http::StatusCode::BAD_REQUEST));
                    },
                }
            }
        }
    };
}

pub mod register_user;
pub mod create_emunet;
pub mod init_emunet;