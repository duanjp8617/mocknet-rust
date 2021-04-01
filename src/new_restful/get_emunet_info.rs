use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::cluster;
use crate::new_emunet::user::User;

#[derive(Serialize)] 
struct VertexInfo {
    id: u64,
    meta: String,
}

#[derive(Serialize)]
struct ServerInfo {
    vertex_infos: Vec<VertexInfo>,
    server_info: cluster::ServerInfo,
}

#[derive(Serialize)]
struct EdgeInfo {
    edge_id: (u64, u64),
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
    edge_infos: Vec<EdgeInfo>
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
    
    let mut edge_infos = Vec::new();
    for ((s, d), edge) in graph.edges() {
        edge_infos.push(EdgeInfo {
            edge_id: (*s, *d),
            meta: edge.clone()
        });
    }

    let mut server_infos = Vec::new();
    for (_, cs) in emunet.servers().iter() {
        let mut vertex_infos = Vec::new();
        for dev_id in cs.devs().iter() {
            vertex_infos.push( VertexInfo {
                id: *dev_id,
                meta: graph.get_node(*dev_id).unwrap().clone()
            });
        }
        server_infos.push(ServerInfo {
            vertex_infos,
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
        edge_infos
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
