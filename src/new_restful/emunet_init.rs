use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::algo::*;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::cluster::*;
use crate::new_emunet::device::*;
use crate::new_emunet::emunet::{self, EmuNet, EmunetState};
use crate::new_emunet::user::User;

#[derive(Deserialize)]
struct Request<String> {
    emunet_uuid: uuid::Uuid,       // uuid of the emunet object on the database
    devs: Vec<DeviceInfo<String>>, // a list of devices to be created
    links: Vec<LinkInfo<String>>,  // a list of links to be created
}

#[derive(Serialize)]
struct ResponseData {
    status: String,
}

async fn emunet_init(
    req: Request<String>,
    client: &mut Client,
) -> Result<
    Result<
        (
            EmuNet,
            UndirectedGraph<u64, DeviceInfo<String>, LinkInfo<String>>,
        ),
        String,
    >,
    ClientError,
> {
    let mut guarded_tran = client.guarded_tran().await?;

    let res = helpers::get_vertex_json_value(
        &mut guarded_tran,
        req.emunet_uuid.clone(),
        emunet::EMUNET_NODE_PROPERTY,
    )
    .await?;
    let emunet: EmuNet = match res {
        None => return Ok(Err(format!("emunet {} does not exist", req.emunet_uuid))),
        Some(jv) => serde_json::from_value(jv).expect("FATAL: invalid JSON format"),
    };

    match emunet.state() {
        EmunetState::Uninit => {}
        _ => {
            return Ok(Err(format!(
                "emunet {} is already initialized",
                req.emunet_uuid
            )))
        }
    };

    let graph = match UndirectedGraph::new(
        req.devs.into_iter().map(|v| (v.id(), v)).collect(),
        req.links.into_iter().map(|e| (e.link_id(), e)).collect(),
    ) {
        None => return Ok(Err("invalid input graph".to_string())),
        Some(graph) => graph,
    };
    if graph.nodes_num() > emunet.max_capacity() as usize {
        return Ok(Err("input graph exceeds capacity limitation".to_string()));
    }

    emunet.set_state(EmunetState::Working);
    let jv = serde_json::to_value(&emunet).unwrap();
    let res = helpers::set_vertex_json_value(
        &mut guarded_tran,
        req.emunet_uuid.clone(),
        emunet::EMUNET_NODE_PROPERTY,
        &jv,
    )
    .await?;
    if !res {
        panic!("vertex not exist");
    }

    Ok(Ok((emunet, graph)))
}

async fn guard(
    req: Request<String>,
    mut client: Client,
) -> Result<warp::reply::Json, warp::Rejection> {
    let res = emunet_init(req, &mut client).await;
    match res {
        Ok(res) => {
            match res {
                Ok((emunet, graph)) => {
                    Ok(Response::success("working".to_string()).into())
                },
                Err(s ) => {
                    let resp: Response<String> = Response::fail(s);
                    Ok(resp.into())
                }
            }
        },
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
    super::filter_template("init_emunet".to_string(), connector, guard)
}
