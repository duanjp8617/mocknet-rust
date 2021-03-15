use serde::Deserialize;
use warp::Filter;
use tokio::time;

use crate::restful::Response;

#[derive(Deserialize)]
struct Json {
    server_ip: String,
}

async fn server_ping(
    json_msg: Json,
) -> Result<impl warp::Reply, warp::Rejection> {
    time::delay_for(time::Duration::from_millis(500)).await;

    let resp = Response::new(true, (), format!("server {} responds in {}ms", &json_msg.server_ip, 500));

    Ok(warp::reply::json(&resp))
}

/// This filter pings the server as specified in the arguments
pub fn build_filter() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + Send + Sync + 'static
{
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path("server_ping"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and_then(server_ping)
}
