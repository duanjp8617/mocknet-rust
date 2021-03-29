use std::convert::From;
use std::future::Future;

use indradb_proto::ClientError;
use serde::{de::DeserializeOwned, Serialize};
use warp::Filter;

use crate::new_database::errors::ConnectorError;
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
pub enum RestfulError {
    ClientError { inner: ClientError },
    ConnectorError { inner: ConnectorError },
}

impl From<ClientError> for RestfulError {
    fn from(e: ClientError) -> RestfulError {
        Self::ClientError { inner: e }
    }
}

impl From<ConnectorError> for RestfulError {
    fn from(e: ConnectorError) -> RestfulError {
        Self::ConnectorError { inner: e }
    }
}

impl<T: Serialize + Default> From<Response<T>> for warp::reply::Json {
    fn from(resp: Response<T>) -> Self {
        warp::reply::json(&resp)
    }
}

impl From<RestfulError> for warp::reply::Json {
    fn from(e: RestfulError) -> Self {
        match e {
            RestfulError::ClientError { inner } => {
                Response::<()>::fail(format!("fatal: {}", inner)).into()
            }
            RestfulError::ConnectorError { inner } => {
                Response::<()>::fail(format!("fatal: {}", inner)).into()
            }
        }
    }
}

async fn handle_req<Req, T, F, R>(
    req: Req,
    connector: Connector,
    handle: F,
) -> Result<impl warp::Reply, warp::Rejection>
where
    T: Serialize + Default,
    F: Fn(Req, Client) -> R + Send + Sync,
    R: Future<Output = Result<Response<T>, RestfulError>>,
{
    let client = connector.connect().await.unwrap();
    let res = handle(req, client).await;
    match res {
        Ok(resp) => Ok(warp::reply::Json::from(resp)),
        Err(e) => Ok(warp::reply::Json::from(e)),
    }
}

// pub fn build_pre_filters (
//     connector: Connector,
// )  -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + Send + Sync + 'static 
// {
//     let connector_filter = warp::any().map(move || {
//         let clone = connector.clone();
//         clone
//     });
//     let res = warp::post()
//         .and(warp::path("v1"))
//         .and(warp::path("register_user"))
//         .and(warp::path::end())
//         .and(parse_json_body())
//         .and(connector_filter)
//         .and(handle_filter);
// }

mod user_registration;
mod new_user_registration;