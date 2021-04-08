use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use warp::Filter;

use super::Response;
use crate::algo::*;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{DeviceInfo, Emunet, EmunetState, LinkInfo};
use crate::k8s_api::{self, mocknet_client, EmunetReq, QueryReq};

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

async fn init_background_task(
    api_server_addr: String,
    emunet_req: EmunetReq,
    pod_names: Vec<String>,
) -> Result<Vec<k8s_api::DeviceInfo>, String> {
    let mut k8s_api_client = mocknet_client::MocknetClient::connect(api_server_addr.clone())
        .await
        .map_err(|_| format!("can't connect to k8s api server at {}", api_server_addr))?;

    let grpc_req = tonic::Request::new(emunet_req);
    let response = k8s_api_client
        .init(grpc_req)
        .await
        .map_err(|_| {
            format!(
                "can't finish init grpc call at api server {}",
                api_server_addr
            )
        })?
        .into_inner();
    if response.status != true {
        return Err("k8s cluster can't initialize this emunet".to_string());
    }

    let total_query_attemps = 300;
    for i in 0..total_query_attemps {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let query = tonic::Request::new(QueryReq {
            is_init: true,
            pod_names: pod_names.clone(),
        });
        let response = k8s_api_client
            .query(query)
            .await
            .map_err(|_| {
                format!(
                    "can't finish the {}-th query call at api server {}",
                    i, api_server_addr
                )
            })?
            .into_inner();

        if response.status {
            return Ok(response.device_infos);
        }
    }

    Err(format!(
        "k8s cluster can't finish initialize this emunet querying {} times",
        total_query_attemps
    ))
}

async fn background_task_guard(
    emunet: Emunet,
    graph: UndirectedGraph<u64, DeviceInfo<String>, LinkInfo<String>>,
    mut client: Client,
) {
    emunet.build_emunet_graph(&graph);
    {
        let mut guarded_tran = client.guarded_tran().await.unwrap();
        let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
        assert!(fut.await.unwrap() == true);
    }

    let api_server_addr = emunet.api_server_addr().to_string();
    let emunet_req = emunet.release_init_grpc_request();
    let pod_names = emunet.release_pod_names();
    let res = init_background_task(api_server_addr, emunet_req, pod_names).await;
    match res {
        Ok(device_infos) => {
            emunet.update_device_login_info(&device_infos);
            emunet.set_state(EmunetState::Normal);
        }
        Err(err_str) => {
            emunet.set_state(EmunetState::Error(err_str));
        }
    }

    let mut guarded_tran = client.guarded_tran().await.unwrap();
    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
    assert!(fut.await.unwrap() == true);
}

async fn init_check(
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
    if graph.edges_num() > (2 as usize).pow(23) {
        return Ok(Err("input graph has too many edges".to_string()));
    }

    emunet.set_state(EmunetState::Working);
    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
    assert!(fut.await? == true);

    Ok(Ok((emunet, graph)))
}

async fn guard(
    req: Request<String>,
    mut client: Client,
) -> Result<warp::reply::Json, warp::Rejection> {
    let res = init_check(req, &mut client).await;
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
