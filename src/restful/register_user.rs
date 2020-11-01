use crate::database::{IndradbClient};

use warp::{http, Filter};
use serde::Deserialize;

// path/register_user/:user_name
async fn register_user(user_name: String, db_client: IndradbClient) -> Result<impl warp::Reply, warp::Rejection> {
    let res = db_client.register_user(&user_name).await;

    res.map_err(|_| {
        warp::reject::not_found()
    }).and_then(|succeed| {
        if succeed {
            Ok(warp::reply::with_status(format!("User {} successfully registers.", &user_name), http::StatusCode::CREATED))
        }
        else {
            Ok(warp::reply::with_status(format!("User {} has already registered.", &user_name), http::StatusCode::CONFLICT))
        }
    })

}

#[derive(Deserialize)]
struct Json {
    name: String,
}

fn parse_json_body() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .map(|req_body: Json|{req_body.name})
}

pub fn build_filter(db_client: IndradbClient) 
    -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + Send + Sync + 'static
{
    let db_filter = warp::any().map(move || {
        let clone = db_client.clone();
        clone
    });
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path("register_user"))
        .and(warp::path::end())
        .and(parse_json_body())
        .and(db_filter)
        .and_then(register_user)
}