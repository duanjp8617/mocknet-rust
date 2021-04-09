use std::future::Future;

use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector, GuardedTransaction};
use crate::emunet::{Emunet, EmunetState};
use crate::k8s_api::{mocknet_client, EmunetReq, QueryReq};

#[derive(Deserialize)]
struct Request {
    emunet_uuid: uuid::Uuid,
}

fn delete_emunet_from_db<'a>(
    emunet: &Emunet,
    guarded_tran: &'a mut GuardedTransaction,
) -> impl Future<Output = ()> + Send + 'a {
    let servers_opt = match emunet.state() {
        EmunetState::Error(_) => None,
        _ => {
            emunet.clear_emunet_resource();
            Some(emunet.release_emunet_servers())
        }
    };
    let emunet_uuid = emunet.emunet_uuid();
    let emunet_user = emunet.emunet_user().to_string();
    let emunet_name = emunet.emunet_name().to_string();
    let emunet_id = emunet.emunet_id();

    async move {
        match servers_opt {
            Some(servers) => {
                let mut cluster_info = helpers::get_cluster_info(guarded_tran).await.unwrap();
                cluster_info.rellocate_servers(servers);
                helpers::set_cluster_info(guarded_tran, cluster_info)
                    .await
                    .unwrap();
            }
            None => {}
        }

        helpers::delete_vertex(guarded_tran, emunet_uuid)
            .await
            .unwrap();

        let mut user_map = helpers::get_user_map(guarded_tran).await.unwrap();
        assert!(
            user_map
                .get_mut(&emunet_user)
                .unwrap()
                .delete_emunet(&emunet_name)
                .is_some()
                == true
        );
        helpers::set_user_map(guarded_tran, user_map).await.unwrap();

        let mut id_allocator = helpers::get_emunet_id_allocator(guarded_tran)
            .await
            .unwrap();
        assert!(id_allocator.realloc(emunet_id) == true);
        helpers::set_emunet_id_allocator(guarded_tran, id_allocator)
            .await
            .unwrap();
    }
}

pub(crate) async fn delete_background_task(
    api_server_addr: String,
    emunet_req: EmunetReq,
    pod_names: Vec<String>,
) -> Result<(), String> {
    let mut k8s_api_client = mocknet_client::MocknetClient::connect(api_server_addr.clone())
        .await
        .map_err(|_| format!("can't connect to k8s api server at {}", api_server_addr))?;

    let grpc_req = tonic::Request::new(emunet_req);
    let response = k8s_api_client
        .delete(grpc_req)
        .await
        .map_err(|_| {
            format!(
                "can't finish delete grpc call at api server {}",
                api_server_addr
            )
        })?
        .into_inner();
    if response.status != true {
        return Err("k8s cluster can't delete this emunet".to_string());
    }

    let total_query_attemps = 300;
    for i in 0..total_query_attemps {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let query = tonic::Request::new(QueryReq {
            is_init: false,
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
            return Ok(());
        }
    }

    Err(format!(
        "k8s cluster can't finish initialize this emunet querying {} times",
        total_query_attemps
    ))
}

async fn emunet_delete(
    emunet_uuid: uuid::Uuid,
    client: &mut Client,
) -> Result<Response<()>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;
    let emunet = match helpers::get_emunet(&mut guarded_tran, emunet_uuid.clone()).await? {
        None => {
            return Ok(Response::fail(format!(
                "emunet {} does not exist",
                emunet_uuid
            )))
        }
        Some(emunet) => emunet,
    };

    let state = emunet.state();
    match state {
        EmunetState::Working => {
            return Ok(Response::fail(format!(
                "emunet {} is in working state, can't be deleted",
                emunet_uuid
            )))
        }
        EmunetState::Uninit | EmunetState::Error(_) => {
            let fut = delete_emunet_from_db(&emunet, &mut guarded_tran);
            fut.await;
            return Ok(Response::success(()));
        }
        EmunetState::Normal => {
            emunet.set_state(EmunetState::Working);
            let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
            assert!(fut.await? == true);
            drop(guarded_tran);

            let api_server_addr = emunet.api_server_addr().to_string();
            let emunet_req = emunet.release_init_grpc_request();
            let pod_names = emunet.release_pod_names();

            let res = delete_background_task(api_server_addr, emunet_req, pod_names).await;
            let mut guarded_tran = client.guarded_tran().await.unwrap();
            match res {
                Ok(_) => {
                    let fut = delete_emunet_from_db(&emunet, &mut guarded_tran);
                    fut.await;
                    return Ok(Response::success(()));
                }
                Err(err_str) => {
                    emunet.set_state(EmunetState::Error(err_str.clone()));

                    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
                    assert!(fut.await.unwrap() == true);

                    return Ok(Response::fail(err_str));
                }
            }
        }
    }
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = emunet_delete(req.emunet_uuid, &mut client).await;
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
    super::filter_template("delete_emunet".to_string(), connector, guard)
}
