use indradb_proto::ClientError;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::ServerInfo;

async fn clear_garbage_servers(
    client: &mut Client,
) -> Result<Response<Vec<ServerInfo>>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;
    let garbage_servers = helpers::get_garbage_servesr(&mut guarded_tran).await?;
    helpers::set_garbage_servesr(&mut guarded_tran, Vec::new())
        .await
        .unwrap();

    Ok(Response::success(garbage_servers))
}

async fn guard(mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = clear_garbage_servers(&mut client).await;
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
        .and(warp::path("clear_garbage_servers"))
        .and(warp::path::end())
        .and(connector_filter)
        .and_then(guard)
}
