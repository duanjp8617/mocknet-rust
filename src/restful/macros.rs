// a macro which is used to extract the response from nested Result types
macro_rules! extract_response {
    ($resp: expr,
     $fatal: expr,
     $err: expr) => {
        match $resp {
            Err(e) => {
                return Ok(
                    warp::reply::json(&crate::restful::Response::<()>::new(
                        false,
                        (),
                        format!("{}: {}", $fatal, e),
                    ))
                );
            }
            Ok(query_resp) => match query_resp {
                Ok(inner) => inner,
                Err(err_msg) => {
                    return Ok(
                        warp::reply::json(&crate::restful::Response::<()>::new(
                            false,
                            (),
                            format!("{}: {}", $err, err_msg),
                        ))
                    );
                }
            },
        }
    };
}
