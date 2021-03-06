use std::convert::From;
use std::future::Future;

use indradb_proto::ClientError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use warp::Filter;

use crate::database::{Client, Connector};

fn parse_json_body<T: DeserializeOwned + Send>(
) -> impl warp::Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub(crate) success: bool,
    pub(crate) data: Option<T>,
    pub(crate) message: String,
}

impl<T> Response<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: String::new(),
        }
    }

    fn fail(err_msg: String) -> Self {
        Self {
            success: false,
            data: None,
            message: err_msg,
        }
    }
}

impl<T: Serialize> From<Response<T>> for warp::reply::Json {
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

pub mod emunet_creation;
pub mod emunet_deletion;
pub mod emunet_init;
pub mod emunet_update;
pub mod execute_command;
pub mod get_emunet_info;
pub mod get_emunet_state;
pub mod list_all;
pub mod list_emunet;
pub mod list_user_history;
pub mod route_command;
pub mod user_deletion;
pub mod user_registration;

pub mod server_ping;

// maintainance utilities
pub mod add_nodes;
pub mod clear_garbage_servers;

// mnctl utilities
pub mod mnctl_util;
