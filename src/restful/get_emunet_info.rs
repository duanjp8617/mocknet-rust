use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{EmunetAccessInfo, OutputDevice, OutputLink};

#[derive(Serialize)]
struct EmunetInfo {
    emunet_name: String,
    emunet_uuid: Uuid,
    max_capacity: u64,
    user_name: String,
    access_info: EmunetAccessInfo,
    state: String,
    dev_count: u64,
}

#[derive(Serialize)]
struct ResponseData {
    emunet_info: EmunetInfo,
    devices: Vec<OutputDevice>,
    links: Vec<OutputLink>,
}

#[derive(Deserialize)]
struct Request {
    emunet_uuid: Uuid,
}

async fn get_emunet_info(
    req: Request,
    client: &mut Client,
) -> Result<Response<ResponseData>, ClientError> {
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
    let (devices, links) = emunet.release_output_emunet();

    let access_info = emunet.access_info();
    let emunet_info = EmunetInfo {
        emunet_name: emunet.emunet_name().to_string(),
        emunet_uuid: emunet.emunet_uuid().clone(),
        max_capacity: emunet.max_capacity(),
        user_name: emunet.emunet_user().to_string(),
        access_info: EmunetAccessInfo {
            login_server_addr: access_info.login_server_addr.clone(),
            login_server_user: access_info.login_server_user.clone(),
            login_server_pwd: access_info.login_server_pwd.clone(),
        },
        state: emunet.state().into(),
        dev_count: emunet.dev_count(),
    };

    Ok(Response::success(ResponseData {
        emunet_info,
        devices,
        links,
    }))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = get_emunet_info(req, &mut client).await;
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
    super::filter_template("get_emunet_info".to_string(), connector, guard)
}
