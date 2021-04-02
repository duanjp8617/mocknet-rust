use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet;

#[derive(Serialize)]
struct DeviceInfo {
    id: u64,
    meta: String,
}

#[derive(Serialize)]
struct ServerInfo {
    dev_infos: Vec<DeviceInfo>,
    server_info: emunet::ServerInfo,
}

#[derive(Serialize)]
struct LinkInfo {
    link_id: (u64, u64),
    meta: String,
}

#[derive(Serialize)]
struct EmunetInfo {
    emunet_name: String,
    emunet_uuid: Uuid,
    max_capacity: u64,
    user_name: String,
    state: String,
    dev_count: u64,
}

#[derive(Serialize)]
struct ResponseData {
    emunet_info: EmunetInfo,
    server_infos: Vec<ServerInfo>,
    link_infos: Vec<LinkInfo>,
}

#[derive(Deserialize)]
struct Request {
    emunet_uuid: Uuid,
}

#[derive(Serialize)]
struct State {
    emunet_uuid: Uuid,
    state: String,
}

async fn get_emunet_state(
    req: Request,
    client: &mut Client,
) -> Result<Response<State>, ClientError> {
    let mut tran = client.guarded_tran().await?;

    let emunet = match helpers::get_emunet(&mut tran, req.emunet_uuid.clone()).await? {
        None => {
            return Ok(Response::fail(format!(
                "emunet {} does not exist",
                req.emunet_uuid
            )))
        }
        Some(emunet) => emunet,
    };
    

    Ok(Response::success(State {
        emunet_uuid: req.emunet_uuid,
        state: emunet.state().into(),
    }))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = get_emunet_state(req, &mut client).await;
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
    super::filter_template("get_emunet_state".to_string(), connector, guard)
}
