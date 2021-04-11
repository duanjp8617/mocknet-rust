use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::ServerInfo;

#[derive(Deserialize)]
struct Request {
    k8s_nodes: Vec<ServerInfo>,
}

async fn add_nodes(req: Request, client: &mut Client) -> Result<Response<()>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;

    let mut cluster_info = helpers::get_cluster_info(&mut guarded_tran).await?;
    for server in req.k8s_nodes {
        let res = cluster_info.add_server_info(server.node_name.clone(), server.max_capacity);
        if res == false {
            return Ok(Response::fail(format!(
                "invalid node name {}",
                server.node_name
            )));
        }
    }

    helpers::set_cluster_info(&mut guarded_tran, cluster_info)
        .await
        .unwrap();

    Ok(Response::success(()))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = add_nodes(req, &mut client).await;
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
    super::filter_template("add_nodes".to_string(), connector, guard)
}
