use indradb_proto::ClientError;
use serde::Deserialize;
use warp::Filter;

use super::Response;
use crate::database::{helpers, Client, Connector};
use crate::emunet::EmunetState;

#[derive(Deserialize)]
struct Request {
    emunet_uuid: uuid::Uuid,
}

async fn emunet_deletion(req: Request, client: &mut Client) -> Result<Response<()>, ClientError> {
    let mut guarded_tran = client.guarded_tran().await?;
    let emunet = match helpers::get_emunet(&mut guarded_tran, req.emunet_uuid.clone()).await? {
        None => {
            return Ok(Response::fail(format!(
                "emunet {} does not exist",
                req.emunet_uuid
            )))
        }
        Some(emunet) => emunet,
    };

    emunet.set_state(EmunetState::Working);
    let fut = helpers::set_emunet(&mut guarded_tran, &emunet);
    let res = fut.await?;
    if !res {
        panic!("FATAL: this should not happen");
    }
    drop(guarded_tran);

    // emulation deletion work
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let mut guarded_tran = client.guarded_tran().await.expect("FATAL");

    let mut cluster_info = helpers::get_cluster_info(&mut guarded_tran)
        .await
        .expect("FATAL");
    let cs_v = emunet.release_emunet_resource();
    cluster_info.rellocate_servers(cs_v);
    helpers::set_cluster_info(&mut guarded_tran, cluster_info)
        .await
        .expect("FATAL");

    let emunet_uuid = emunet.emunet_uuid();
    helpers::delete_vertex(&mut guarded_tran, emunet_uuid)
        .await
        .expect("FATAL");

    let mut user_map = helpers::get_user_map(&mut guarded_tran)
        .await
        .expect("FATAL");
    assert!(
        user_map
            .get_mut(&emunet.emunet_user())
            .unwrap()
            .delete_emunet(&emunet.emunet_name())
            .is_some()
            == true
    );
    helpers::set_user_map(&mut guarded_tran, user_map)
        .await
        .expect("FATAL");

    Ok(Response::success(()))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = emunet_deletion(req, &mut client).await;
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
    super::filter_template("delete_emunet".to_string(), connector, guard)
}
