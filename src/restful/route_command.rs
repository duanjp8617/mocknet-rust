use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::EmunetState;

#[derive(Deserialize, Serialize)]
pub(crate) struct Request {
    pub(crate) emunet_uuid: Uuid,
    pub(crate) path: Vec<u64>,
    pub(crate) is_add: bool,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct RespData {
    pub(crate) forward_route_commands: Vec<(u64, String)>,
    pub(crate) backward_route_commands: Vec<(u64, String)>,
    pub(crate) src_idx: u64,
    pub(crate) src_ip: String,
    pub(crate) dest_idx: u64,
    pub(crate) dest_ip: String,
    pub(crate) api_server_addr: String,
}

// this must only be called by the mnctl_network_connect, which provides
// three preconditions that always holds.
// 1. req.path contains a valid path inside the specified emunet.
// 2. req.path is longer than 2.
// 3. the emunet is available
async fn route_command(
    req: Request,
    client: &mut Client,
) -> Result<Response<RespData>, ClientError> {
    let mut tran = client.guarded_tran().await?;

    let emunet = helpers::get_emunet(&mut tran, req.emunet_uuid.clone())
        .await?
        .unwrap();
    match emunet.state() {
        EmunetState::Normal => {}
        _ => {
            return Ok(Response::fail(format!(
                "emunet {} is not in normal state",
                req.emunet_uuid
            )))
        }
    };

    let path = req.path;
    let (forward_route_commands, (dest_idx, dest_ip)) =
        emunet.release_route_command(&path[..], req.is_add);

    let reverse_path: Vec<u64> = path.into_iter().rev().collect();
    let (backward_route_commands, (src_idx, src_ip)) =
        emunet.release_route_command(&reverse_path[..], req.is_add);

    Ok(Response::success(RespData {
        forward_route_commands,
        backward_route_commands,
        src_idx,
        src_ip,
        dest_idx,
        dest_ip,
        api_server_addr: emunet.api_server_addr().to_string(),
    }))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = route_command(req, &mut client).await;
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
    super::filter_template("route_command".to_string(), connector, guard)
}
