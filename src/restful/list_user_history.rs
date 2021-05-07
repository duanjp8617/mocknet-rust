use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{Retired, User};

#[derive(Deserialize, Serialize)]
pub struct Request {
    pub name: String,
}

#[derive(Serialize)]
pub struct Data {
    network_names: Vec<String>,
    retired_networks: Vec<Retired>,
}

async fn list_user_history(
    req: Request,
    client: &mut Client,
) -> Result<Response<Data>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    let user = user_map.remove(&req.name);
    if user.is_none() {
        return Ok(Response::fail(format!("user {} does not exist", req.name)));
    }
    let user = user.unwrap();
    let retired_networks = user.get_retired_emunets();
    let data = Data {
        network_names: user
            .into_uuid_map()
            .keys()
            .map(|name| name.clone())
            .collect(),
        retired_networks,
    };

    Ok(Response::success(data))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = list_user_history(req, &mut client).await;
    match res {
        Ok(resp) => Ok(resp.into()),
        Err(e) => {
            client.notify_failure();
            let resp: Response<_> = e.into();
            Ok(resp.into())
        }
    }
}

pub fn build_filter(
    connector: Connector,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + Send {
    super::filter_template("list_user_history".to_string(), connector, guard)
}
