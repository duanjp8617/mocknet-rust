use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::cluster::ClusterInfo;
use crate::new_emunet::user::User;

#[derive(Deserialize)]
struct Request {
    user: String,
    emunet: String,
    capacity: u64,
}

async fn create_emunet(req: Request, client: &mut Client) -> Result<Response<Uuid>, ClientError> {
    let mut tran = client.tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut tran).await?;
    if user_map.get(&req.user).is_none() {
        return Ok(Response::fail("invalid user name".to_string()));
    }
    let user_mut = user_map.get_mut(&req.user).unwrap();

    if user_mut.emunet_exist(&req.emunet) {
        return Ok(Response::fail("invalid emunet name".to_string()));
    }

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
    self.fe.set_server_info_list(sp.into_vec()).await?;

    // create a new emu net node
    let emu_net_id = self
        .fe
        .create_vertex(None)
        .await?
        .expect("vertex ID already exists");
    // create a new emu net
    let mut emu_net = net::EmuNet::new(user, net.clone(), emu_net_id.clone(), capacity);
    emu_net.add_servers(allocation);
    // initialize the EmuNet in the database
    let jv = serde_json::to_value(emu_net).unwrap();
    let res = self
        .fe
        .set_vertex_json_value(emu_net_id, "default", jv)
        .await?;
    if !res {
        panic!("vertex not exist");
    }

    // add the new emunet to user map
    user_mut.add_emu_net(net, emu_net_id.clone());
    self.fe.set_user_map(user_map).await?;

    succeed!(emu_net_id)
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = user_registration(req, &mut client).await;
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
