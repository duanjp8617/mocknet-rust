use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{Emunet, EmunetState, InputDevice, InputLink, MAX_DIRECTED_LINK_POWER};
use crate::{algo::*, emunet::User};

#[derive(Deserialize)]
struct Request<String> {
    emunet_uuid: uuid::Uuid,        // uuid of the emunet object on the database
    devs: Vec<InputDevice<String>>, // a list of devices to be created
    links: Vec<InputLink<String>>,  // a list of links to be created
}

#[derive(Serialize)]
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
