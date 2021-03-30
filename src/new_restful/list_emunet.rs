use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::user::User;

type RespType = HashMap<String, Uuid>;

#[derive(Deserialize)]
struct Request {
    user: String,
}

async fn list_emunet(req: Request, client: &mut Client) -> Result<Response<RespType>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    let res = user_map.remove(&req.user);
    match res {
        None => Ok(Response::fail(format!("invalid user name {}", req.user))),
        Some(user) => Ok(Response::success(user.into_uuid_map())),
    }
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = list_emunet(req, &mut client).await;
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
    super::filter_template("list_emunet".to_string(), connector, guard)
}
