use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::user::User;

#[derive(Deserialize)]
struct Request {
    name: String,
}

async fn user_registration(
    req: Request,
    client: &mut Client,
) -> Result<Response<String>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    if user_map.get(&req.name).is_some() {
        return Ok(Response::fail(format!(
            "user {} has already registered",
            req.name
        )));
    }

    let user = User::new(&req.name);
    user_map.insert(req.name.clone(), user);
    helpers::set_user_map(&mut guarded_tran, user_map).await?;

    Ok(Response::success(req.name))
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
    super::filter_template("register_user".to_string(), connector, guard)
}
