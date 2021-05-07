// use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::algo::UndirectedGraph;
use crate::database::{helpers, Client, Connector};
// use crate::emunet::{EmunetAccessInfo, OutputDevice, OutputLink};

#[derive(Deserialize, Serialize)]
struct Request {
    emunet_uuid: Uuid,
    source: u64,
    destination: u64,
}

async fn route_command(req: Request, client: &mut Client) -> Result<Response<()>, ClientError> {
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
    let (devs, links) = emunet.release_output_emunet();
    let nodes: Vec<(u64, ())> = devs.iter().map(|odev| (odev.id, ())).collect();
    let edges: Vec<((u64, u64), ())> = links.iter().map(|olink| (olink.link_id, ())).collect();
    let graph = UndirectedGraph::new(nodes, edges).unwrap();
    let path = graph.shortest_path(req.source, req.destination).unwrap();

    let forward_route_commands = emunet.release_route_command(&path[..], true);
    println!("forward commands");
    for e in forward_route_commands {
        println!("{}: {}", e.0, e.1);
    }

    let reverse_path: Vec<u64> = path.into_iter().rev().collect();
    let backward_route_commands = emunet.release_route_command(&reverse_path[..], true);
    println!("backward commands");
    for e in backward_route_commands {
        println!("{}: {}", e.0, e.1);
    }

    Ok(Response::success(()))
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
