use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::EmunetState;
use crate::k8s_api::{mocknet_client, ExecReq};

#[derive(Deserialize, Serialize)]
pub(crate) struct Request {
    pub(crate) emunet_uuid: Uuid,
    pub(crate) dev_idx: u64,
    pub(crate) cmd: String,
    pub(crate) api_server_addr: String,
}

async fn execute_command(
    req: Request,
    client: &mut Client,
) -> Result<Response<String>, ClientError> {
    let mut tran = client.guarded_tran().await?;

    // make sure that we can execute command in this emunet
    let emunet = match helpers::get_emunet(&mut tran, req.emunet_uuid.clone()).await? {
        None => {
            return Ok(Response::fail(format!(
                "emunet {} does not exist",
                req.emunet_uuid
            )))
        }
        Some(emunet) => emunet,
    };
    match emunet.state() {
        EmunetState::Normal => {}
        _ => {
            return Ok(Response::fail(format!(
                "emunet {} is not in normal state",
                req.emunet_uuid
            )))
        }
    };

    // retrieve the pod_name
    let pod_name = match emunet.get_pod_name(req.dev_idx) {
        Some(inner) => inner,
        None => {
            return Ok(Response::fail(format!(
                "device {} is not presetned in emunet {}",
                req.dev_idx, req.emunet_uuid
            )))
        }
    };

    // run the grpc command
    let mut k8s_api_client =
        match mocknet_client::MocknetClient::connect(req.api_server_addr.clone()).await {
            Ok(inner) => inner,
            Err(_) => {
                return Ok(Response::fail(format!(
                    "can't connect to k8s api server at {}",
                    req.api_server_addr
                )));
            }
        };
    let grpc_req = tonic::Request::new(ExecReq {
        pod_name: pod_name,
        cmd: req.cmd.clone(),
    });
    let response = match k8s_api_client.exec(grpc_req).await {
        Ok(inner) => inner.into_inner(),
        Err(_) => {
            return Ok(Response::fail(format!(
                "fail to execute command '{}' on device {}",
                req.cmd, req.dev_idx
            )));
        }
    };

    Ok(Response::success(response.std_out))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = execute_command(req, &mut client).await;
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
    super::filter_template("execute_command".to_string(), connector, guard)
}
