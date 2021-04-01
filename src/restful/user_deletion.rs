use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::user::User;

#[derive(Deserialize)]
struct Request {
    name: String,
}

async fn user_deletion(req: Request, client: &mut Client) -> Result<Response<()>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    let user = user_map.remove(&req.name);
    if user.is_none() {
        return Ok(Response::fail(format!("user {} does not exist", req.name)));
    }

    let emunets = user.unwrap().into_uuid_map();
    if emunets.len() > 0 {
        return Ok(Response::fail(format!(
            "user {} still has active emunets",
            req.name
        )));
    }

    helpers::set_user_map(&mut guarded_tran, user_map)
        .await
        .expect("can't fail");

    Ok(Response::success(()))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = user_deletion(req, &mut client).await;
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
    super::filter_template("delete_user".to_string(), connector, guard)
}
