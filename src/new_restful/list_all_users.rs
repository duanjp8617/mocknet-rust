use std::collections::HashMap;

use indradb_proto::ClientError;
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::new_database::{helpers, Client, Connector};
use crate::new_emunet::user::User;

type RespType = HashMap<String, HashMap<String, Uuid>>;

async fn list_all_users_temp(client: &mut Client) -> Result<Response<RespType>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let user_map: HashMap<String, User> = helpers::get_user_map(&mut guarded_tran).await?;
    let res = user_map
        .into_iter()
        .fold(HashMap::new(), |mut hm, (user_name, user)| {
            hm.insert(user_name, user.into_uuid_map());
            hm
        });

    Ok(Response::success(res))
}

async fn guard(mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = list_all_users_temp(&mut client).await;
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
        .and(warp::path("list_all_users"))
        .and(warp::path::end())
        .and(connector_filter)
        .and_then(guard)
}
