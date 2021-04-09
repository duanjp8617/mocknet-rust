use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
#[derive(Deserialize)]
struct Request {
    server_ips: Vec<String>,
}

async fn server_ping(req: Request) -> Result<Response<HashMap<String, bool>>, ClientError> {
    let res: Option<Vec<std::net::Ipv4Addr>> = req
        .server_ips
        .into_iter()
        .map(|ip_str| ip_str.parse::<std::net::Ipv4Addr>().ok())
        .collect();

    let server_ips = match res {
        Some(inner) => inner,
        None => {
            return Ok(Response::fail(format!("invalid IPv4 address")));
        }
    };

    let mut res = HashMap::new();
    for server_ip in server_ips {
        res.insert(server_ip.to_string(), true);
    }

    Ok(Response::success(res))
}

async fn guard(req: Request) -> Result<warp::reply::Json, warp::Rejection> {
    let res = server_ping(req).await;
    match res {
        Ok(resp) => Ok(resp.into()),
        Err(e) => {
            let resp: Response<_> = e.into();
            Ok(resp.into())
        }
    }
}

pub fn build_filter(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + Send {
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path("server_ping"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and_then(guard)
}
