use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use super::RestfulError;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::user::User;

#[derive(Deserialize)]
struct Request {
    name: String,
}

async fn user_registration(
    req: Request,
    client: &mut Client,
) -> Result<Response<String>, RestfulError> {
    let mut tran = client.tran().await?;

    let mut user_map: HashMap<String, User> = helpers::get_user_map(&mut tran).await?;
    if user_map.get(&req.name).is_some() {
        return Ok(Response::fail(format!(
            "user {} has already registered",
            req.name
        )));
    }

    let user = User::new(&req.name);
    user_map.insert(req.name.clone(), user);
    helpers::set_user_map(&mut tran, user_map).await?;

    Ok(Response::success(req.name))
}

async fn wrapper(req: Request, connector: Connector) -> Result<impl warp::Reply, warp::Rejection> {
    let mut client = connector.connect().await.unwrap();
    let res = user_registration(req, &mut client).await;
    match res {
        Ok(resp) => Ok(warp::reply::Json::from(resp)),
        Err(e) => Ok(warp::reply::Json::from(e)),
    }
}

pub fn build_filter(
    connector: Connector,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + Send + Sync + 'static
{
    let connector_filter = warp::any().map(move || {
        let clone = connector.clone();
        clone
    });
    let res = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("register_user"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(connector_filter);
        
    res.and_then(wrapper)
}
