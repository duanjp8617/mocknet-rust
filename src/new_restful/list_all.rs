use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Serialize;
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::cluster::ServerInfo;
use crate::new_emunet::emunet::{self, EmuNet};
use crate::new_emunet::user::User;

#[derive(Serialize)]
struct Inner {
    users: HashMap<String, HashMap<String, EmuNet>>,
    usable_servers: HashMap<Uuid, ServerInfo>,
}

async fn list_all(client: &mut Client) -> Result<Response<Inner>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    let user_map = user_map
        .into_iter()
        .fold(HashMap::new(), |mut hm, (user_name, user)| {
            hm.insert(user_name, user.into_uuid_map());
            hm
        });

    let mut users = HashMap::new();
    for (user_name, emunet_map) in user_map.into_iter() {
        let mut emunets = HashMap::new();
        for (emunet_name, emunet_uuid) in emunet_map.into_iter() {
            let res = helpers::get_vertex_json_value(
                &mut guarded_tran,
                emunet_uuid,
                emunet::EMUNET_NODE_PROPERTY,
            )
            .await?;
            match res {
                None => return Ok(Response::fail("FATAL: emunet does not exist".to_string())),
                Some(jv) => {
                    let emunet: EmuNet =
                        serde_json::from_value(jv).expect("FATAL: invalid JSON format");
                    emunets.insert(emunet_name, emunet);
                }
            }
        }
        users.insert(user_name, emunets);
    }

    let servers = helpers::get_cluster_info(&mut guarded_tran)
        .await?
        .into_vec();
    let mut usable_servers = HashMap::new();
    for si in servers.into_iter() {
        usable_servers.insert(si.uuid.clone(), si);
    }

    Ok(Response::success(Inner {
        users,
        usable_servers,
    }))
}

async fn guard(mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = list_all(&mut client).await;
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
    let connector_filter = warp::any()
        .map(move || {
            let clone = connector.clone();
            clone
        })
        .and_then(super::get_client);
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path("list_all"))
        .and(warp::path::end())
        .and(connector_filter)
        .and_then(guard)
}
