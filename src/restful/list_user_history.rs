use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{Retired, User};

#[derive(Deserialize)]
struct Request {
    name: String,
}

async fn list_user_history(
    req: Request,
    client: &mut Client,
) -> Result<Response<Vec<Retired>>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    let user = user_map.get(&req.name);
    if user.is_none() {
        return Ok(Response::fail(format!("user {} does not exist", req.name)));
    }

    Ok(Response::success(user.unwrap().get_retired_emunets()))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = list_user_history(req, &mut client).await;
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
    super::filter_template("list_user_history".to_string(), connector, guard)
}
