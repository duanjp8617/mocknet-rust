use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use warp::Filter;

use super::Response;
use crate::algo::*;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{DeviceInfo, Emunet, EmunetState, LinkInfo};
use crate::k8s_api::*;

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

async fn background_task(
    emunet: Emunet,
    graph: UndirectedGraph<u64, DeviceInfo<String>, LinkInfo<String>>,
    client: &mut Client,
) -> Result<(), ClientError> {
    emunet.build_emunet_graph(&graph);
    {
        let mut guarded_tran = client.guarded_tran().await?;
        let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
        assert!(fut.await.unwrap() == true);
    }

    let api_server_addr = emunet.api_server_addr();
    let mut k8s_api_client =
        match mocknet_client::MocknetClient::connect(api_server_addr.clone()).await {
            Ok(inner) => inner,
            Err(_) => {
                emunet.set_state(EmunetState::Error(format!(
                    "can't connect to k8s api server at {}",
                    api_server_addr
                )));
                {
                    let mut guarded_tran = client.guarded_tran().await?;
                    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
                    assert!(fut.await.unwrap() == true);
                }
                return Ok(());
            }
        };

    let grpc_req = tonic::Request::new(emunet.release_grpc_messages());
    let _response = k8s_api_client.init(grpc_req).await;

    // emulate creation work
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    emunet.set_state(EmunetState::Normal);
    let mut guarded_tran = client.guarded_tran().await?;
    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
    assert!(fut.await.unwrap() == true);

    Ok(())
}

async fn background_task_guard(
    emunet: Emunet,
    graph: UndirectedGraph<u64, DeviceInfo<String>, LinkInfo<String>>,
    mut client: Client,
) {
    let res = background_task(emunet, graph, &mut client).await;
    match res {
        Ok(_) => {}
        Err(_) => {
            client.notify_failure();
        }
    }
}

async fn emunet_init(
    req: Request<String>,
    client: &mut Client,
) -> Result<
    Result<
        (
            Emunet,
            UndirectedGraph<u64, DeviceInfo<String>, LinkInfo<String>>,
        ),
        String,
    >,
    ClientError,
> {
    let mut guarded_tran = client.guarded_tran().await?;

    let emunet: Emunet =
        match helpers::get_emunet(&mut guarded_tran, req.emunet_uuid.clone()).await? {
            None => return Ok(Err(format!("emunet {} does not exist", req.emunet_uuid))),
            Some(emunet) => emunet,
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
    // this check is not necessary? consider delete it
    if emunet.dev_count() > 0 {
        return Ok(Err(
            "FATAL: emunet has active devices, this should never happen".to_string(),
        ));
    }

    let filter_dev_id: Option<Vec<DeviceInfo<String>>> = req
        .devs
        .into_iter()
        .map(|dinfo| {
            if dinfo.id() < (2 as u64).pow(32) {
                Some(dinfo)
            } else {
                None
            }
        })
        .collect();
    if filter_dev_id.is_none() {
        return Ok(Err("invalid device id".to_string()));
    }

    let graph = match UndirectedGraph::new(
        filter_dev_id
            .unwrap()
            .into_iter()
            .map(|v| (v.id(), v))
            .collect(),
        req.links.into_iter().map(|e| (e.link_id(), e)).collect(),
    ) {
        None => return Ok(Err("invalid input graph".to_string())),
        Some(graph) => graph,
    };
    if graph.nodes_num() > emunet.max_capacity() as usize {
        return Ok(Err("input graph exceeds capacity limitation".to_string()));
    }

    emunet.set_state(EmunetState::Working);
    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
    assert!(fut.await.unwrap() == true);

    Ok(Ok((emunet, graph)))
}

async fn guard(
    req: Request<String>,
    mut client: Client,
) -> Result<warp::reply::Json, warp::Rejection> {
    let res = emunet_init(req, &mut client).await;
    match res {
        Ok(res) => match res {
            Ok((emunet, graph)) => {
                let state_str = emunet.state().into();
                tokio::spawn(background_task_guard(emunet, graph, client));

                Ok(Response::success(ResponseData { status: state_str }).into())
            }
            Err(s) => {
                let resp: Response<String> = Response::fail(s);
                Ok(resp.into())
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
