// serde_json::to_string(&crate::restful::Response::<()>::new(false, (), format!("{}: {}", $fatal, e))).unwrap()

// a macro which is used to extract the response from nested Result types
macro_rules! extract_response {
    ($resp: expr,
     $fatal: expr,
     $err: expr) => {
        match $resp {
            Err(e) => {
                return Ok(warp::reply::with_status(
                    // format!("{}: {}", $fatal, e),
                    serde_json::to_string(&crate::restful::Response::<()>::new(false, (), format!("{}: {}", $fatal, e))).unwrap(),
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
            Ok(query_resp) => match query_resp {
                Ok(inner) => inner,
                Err(err_msg) => {
                    return Ok(warp::reply::with_status(
                        // format!("{}: {}", $err, err_msg),
                        serde_json::to_string(&crate::restful::Response::<()>::new(false, (), format!("{}: {}", $err, err_msg))).unwrap(),
                        http::StatusCode::BAD_REQUEST,
                    ));
                }
            },
        }
    };
}
