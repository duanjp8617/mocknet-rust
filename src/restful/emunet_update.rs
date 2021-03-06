use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use uuid::Uuid;
use warp::Filter;

use super::list_user_history::Data;
use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{Emunet, EmunetState, InputDevice, InputLink, MAX_DIRECTED_LINK_POWER};
use crate::{algo::*, emunet::User};

#[derive(Deserialize, Serialize)]
struct Request<String> {
    emunet_uuid: uuid::Uuid,        // uuid of the emunet object on the database
    devs: Vec<InputDevice<String>>, // a list of devices to be created
    links: Vec<InputLink<String>>,  // a list of links to be created
}

#[derive(Serialize, Deserialize)]
struct ResponseData {
    status: String,
}

async fn background_task_guard(
    emunet: Emunet,
    input_graph: UndirectedGraph<u64, InputDevice<String>, InputLink<String>>,
    mut client: Client,
) {
    let api_server_addr = emunet.api_server_addr().to_string();
    let emunet_req = emunet.release_init_grpc_request();
    let pods = emunet.release_pods();

    let res =
        super::emunet_deletion::delete_background_task(api_server_addr.clone(), emunet_req, pods)
            .await;
    match res {
        Err(err_str) => {
            emunet.set_state(EmunetState::Error(err_str));

            let mut guarded_tran = client.guarded_tran().await.unwrap();
            let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
            assert!(fut.await.unwrap() == true);
            return;
        }
        _ => {}
    };

    {
        let mut guarded_tran = client.guarded_tran().await.unwrap();
        let user_map: HashMap<String, User> =
            helpers::get_user_map(&mut guarded_tran).await.unwrap();
        user_map
            .get(emunet.emunet_user())
            .unwrap()
            .add_retired(&emunet);
        helpers::set_user_map(&mut guarded_tran, user_map)
            .await
            .unwrap();

        emunet.clear_emunet_resource();
        emunet.build_emunet_graph(&input_graph);

        let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
        assert!(fut.await.unwrap() == true);
    }

    let emunet_req = emunet.release_init_grpc_request();
    let pods = emunet.release_pods();
    let res = super::emunet_init::init_background_task(api_server_addr, emunet_req, pods).await;

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

async fn update_check(
    req: Request<String>,
    client: &mut Client,
) -> Result<
    Result<
        (
            Emunet,
            UndirectedGraph<u64, InputDevice<String>, InputLink<String>>,
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
        EmunetState::Normal => {}
        _ => {
            return Ok(Err(format!(
                "emunet {} is not in normal state",
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
    if graph.edges_num() * 2 > (2 as usize).pow(MAX_DIRECTED_LINK_POWER) {
        return Ok(Err(format!(
            "input graph can only have at most {} edges",
            (2 as usize).pow(MAX_DIRECTED_LINK_POWER - 1)
        )));
    }

    emunet.set_state(EmunetState::Working);
    emunet.clear_device_login_info();
    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
    assert!(fut.await? == true);

    Ok(Ok((emunet, graph)))
}

async fn guard(
    req: Request<String>,
    mut client: Client,
) -> Result<warp::reply::Json, warp::Rejection> {
    let res = update_check(req, &mut client).await;
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
    super::filter_template("update_emunet".to_string(), connector, guard)
}

#[derive(Deserialize)]
struct InputNetworkGraph {
    devs: Vec<InputDevice<String>>,
    links: Vec<InputLink<String>>,
}

pub async fn mnctl_network_update(
    user: &str,
    emunet: &str,
    input_file: &str,
    warp_addr: &str,
) -> Result<(), String> {
    // read input network graph
    let json_str = read_to_string(&input_file)
        .await
        .map_err(|_| format!("can't open input file at: {}", input_file))?;
    let input_graph: InputNetworkGraph =
        serde_json::from_str(&json_str).map_err(|_| "invalid network graph format".to_string())?;

    // query emunet_uuid
    let req = super::list_emunet::Request {
        user: user.to_string(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/list_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<HashMap<String, Uuid>> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;
    let map = if response.success {
        response.data.unwrap()
    } else {
        return Err(response.message);
    };
    let emunet_uuid = map
        .get(emunet)
        .ok_or(format!("emunet {} does not exist", emunet))?;

    // send update request
    let req = Request {
        emunet_uuid: emunet_uuid.clone(),
        devs: input_graph.devs,
        links: input_graph.links,
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/update_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<ResponseData> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;

    if response.success == true {
        println!("update success: {}", response.data.unwrap().status);
        Ok(())
    } else {
        Err(response.message)
    }
}

pub async fn mnctl_network_restore(
    user: &str,
    emunet: &str,
    restore_index: usize,
    warp_addr: &str,
) -> Result<(), String> {
    // read the history
    let req = super::list_user_history::Request {
        name: user.to_string(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/list_user_history", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<Data> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;
    let retired_networks = if response.success {
        response.data.unwrap().retired_networks
    } else {
        return Err(response.message);
    };

    // perform an early check
    if restore_index > retired_networks.len() {
        return Err("invalid history index".to_string());
    }

    // query emunet_uuid
    let req = super::list_emunet::Request {
        user: user.to_string(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/list_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<HashMap<String, Uuid>> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;
    let map = if response.success {
        response.data.unwrap()
    } else {
        return Err(response.message);
    };
    let emunet_uuid = map
        .get(emunet)
        .ok_or(format!("emunet {} does not exist", emunet))?;

    // send update request
    let retired_network = &retired_networks[restore_index];
    let req = Request {
        emunet_uuid: emunet_uuid.clone(),
        devs: retired_network
            .nodes
            .iter()
            .map(|nid| InputDevice {
                id: *nid,
                description: String::new(),
            })
            .collect(),
        links: retired_network
            .edges
            .iter()
            .map(|eid| InputLink {
                edge_id: *eid,
                description: String::new(),
            })
            .collect(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/update_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<ResponseData> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;

    if response.success == true {
        println!("update success: {}", response.data.unwrap().status);
        Ok(())
    } else {
        Err(response.message)
    }
}
