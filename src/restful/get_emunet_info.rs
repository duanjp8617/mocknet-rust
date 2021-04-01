use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::cluster;

#[derive(Serialize)] 
struct DeviceInfo {
    id: u64,
    meta: String,
}

#[derive(Serialize)]
struct ServerInfo {
    dev_infos: Vec<DeviceInfo>,
    server_info: cluster::ServerInfo,
}

#[derive(Serialize)]
struct LinkInfo {
    link_id: (u64, u64),
    meta: String
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
    link_infos: Vec<LinkInfo>
}

#[derive(Deserialize)]
struct Request {
    emunet_uuid: Uuid,
}

async fn get_emunet_info(req: Request, client: &mut Client) -> Result<Response<ResponseData>, ClientError> {
    let mut tran = client.guarded_tran().await?;

    let emunet = match helpers::get_emunet(&mut tran, req.emunet_uuid.clone()).await? {
        None => return Ok(Response::fail(format!("emunet {} does not exist", req.emunet_uuid))),
        Some(emunet) => emunet,
    };
    let graph = emunet.release_emunet_graph();
    
    let mut link_infos = Vec::new();
    for ((s, d), edge) in graph.edges() {
        link_infos.push(LinkInfo {
            link_id: (*s, *d),
            meta: edge.clone()
        });
    }

    let mut server_infos = Vec::new();
    for (_, cs) in emunet.servers().iter() {
        let mut dev_infos = Vec::new();
        for dev_id in cs.devs().iter() {
            dev_infos.push( DeviceInfo {
                id: *dev_id,
                meta: graph.get_node(*dev_id).unwrap().clone()
            });
        }
        server_infos.push(ServerInfo {
            dev_infos,
            server_info: cs.server_info().clone()
        });
    }

    let emunet_info  = EmunetInfo{
        emunet_name: emunet.emunet_name().clone(),
        emunet_uuid: emunet.emunet_uuid().clone(),
        max_capacity: emunet.max_capacity(),
        user_name: emunet.emunet_user().clone(),
        state: emunet.state().into(),
        dev_count: emunet.dev_count(),
    };
    

    Ok(Response::success(ResponseData {
        emunet_info,
        server_infos,
        link_infos
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
