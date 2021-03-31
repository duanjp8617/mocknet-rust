use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::emunet::{self, EmuNet};
use crate::new_emunet::user::User;

#[derive(Deserialize)]
struct Request {
    user: String,
    emunet: String,
    capacity: u64,
}

async fn create_emunet(req: Request, client: &mut Client) -> Result<Response<Uuid>, ClientError> {
    let mut tran = client.guarded_tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut tran).await?;
    if user_map.get(&req.user).is_none() {
        return Ok(Response::fail("invalid user name".to_string()));
    }

    let user_mut = user_map.get_mut(&req.user).unwrap();
    let emunet_uuid = match user_mut.register_emunet(&req.emunet) {
        Some(uuid) => uuid,
        None => {
            return Ok(Response::fail(format!(
                "invalid emunet name {}",
                req.emunet
            )));
        }
    };

    let mut cluster_info = helpers::get_cluster_info(&mut tran).await?;
    let allocation = match cluster_info.allocate_servers(req.capacity) {
        Ok(alloc) => alloc,
        Err(remaining) => {
            return Ok(Response::fail(format!(
                "not enough capacity at backend, remaining capacity: {}",
                remaining
            )));
        }
    };

    if !(helpers::create_vertex(&mut tran, emunet_uuid.clone()).await?) {
        panic!(format!("invalid emunet uuid {}", emunet_uuid));
    }

    let emunet = EmuNet::new(req.emunet, emunet_uuid.clone(), req.user, allocation);
    let jv = serde_json::to_value(emunet).unwrap();
    let res = helpers::set_vertex_json_value(
        &mut tran,
        emunet_uuid.clone(),
        emunet::EMUNET_NODE_PROPERTY,
        &jv,
    )
    .await?;
    if !res {
        panic!("vertex not exist");
    }

    helpers::set_user_map(&mut tran, user_map)
        .await
        .expect("can't fail");
    helpers::set_cluster_info(&mut tran, cluster_info)
        .await
        .expect("can't fail");

    Ok(Response::success(emunet_uuid))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = create_emunet(req, &mut client).await;
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
    super::filter_template("create_emunet".to_string(), connector, guard)
}
