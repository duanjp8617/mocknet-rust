use std::convert::From;
use std::future::Future;

use indradb_proto::ClientError;
use serde::{de::DeserializeOwned, Serialize};
use warp::Filter;

use crate::new_database::{Client, Connector};

fn parse_json_body<T: DeserializeOwned + Send>(
) -> impl warp::Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[derive(Serialize)]
struct Response<T: Serialize + Default> {
    success: bool,
    data: T,
    message: String,
}

impl<T: Serialize + Default> Response<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: String::new(),
        }
    }

    fn fail(err_msg: String) -> Self {
        Self {
            success: false,
            data: T::default(),
            message: err_msg,
        }
    }
}

impl<T: Serialize + Default> From<Response<T>> for warp::reply::Json {
    fn from(resp: Response<T>) -> Self {
        warp::reply::json(&resp)
    }
}

impl From<ClientError> for Response<()> {
    fn from(e: ClientError) -> Self {
        Response::<()>::fail(format!("fatal: {}", e))
    }
}

async fn get_client(connector: Connector) -> Result<Client, warp::Rejection> {
    connector.connect().await.map_err(|_| warp::reject())
}

fn filter_template<Req, F, R>(
    api_prefix: String,
    connector: Connector,
    handle: F,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Send + Clone
where
    Req: DeserializeOwned + Send,
    F: Fn(Req, Client) -> R + Send + Clone,
    R: Future<Output = Result<warp::reply::Json, warp::Rejection>> + Send,
{
    let connector_filter = warp::any()
        .map(move || {
            let clone = connector.clone();
            clone
        })
        .and_then(get_client);
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path(api_prefix))
        .and(warp::path::end())
        .and(parse_json_body())
        .and(connector_filter)
        .and_then(handle)
}

pub mod user_registration;
