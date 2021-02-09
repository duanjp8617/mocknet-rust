// Remove all the existing vertexes from a emunet, and turn the emunet into uninit state.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::time;
use warp::Filter;

use crate::database::Client;
use crate::emunet::net::*;
use crate::restful::Response;

use super::emunet_error;

// format of the incoming json message
#[derive(Deserialize)]
struct Json {
    emunet_uuid: uuid::Uuid, // uuid of the emunet object on the database
}

// the actual work is done in a background task
async fn background_task(client: Client, mut emunet: EmuNet) {
    // emulate the background task of destructing containers and connections
    time::delay_for(time::Duration::new(5, 0)).await;
    // potentially perform an update on the vertexes

    let res = client.delete_emunet_vertexes(&emunet).await;
    match res {
        Err(err) => {
            let err = EmuNetError::DatabaseFail(format!("{:?}", err));
            emunet_error(client, emunet, err).await;
            return;
        }
        Ok(_) => {}
    };

    // delete the vertexes and set state to normal
    emunet.delete_vertexes();
    emunet.uninit();

    // store the state in the database, panic the server program on failure
    let res = client.set_emu_net(emunet).await.unwrap();
    if res.is_err() {
        panic!("this should never happen");
    }
}

// path/create_emunet/
async fn destruct_emunet(
    json: Json,
    db_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    // retrieve the emunet object from the database
    let mut emunet = extract_response!(
        db_client.get_emu_net(json.emunet_uuid.clone()).await,
        "internal_server_error",
        "operation_fail"
    );
    if !emunet.is_normal() {
        // we can only destruct an emunet that is in normal state
        return Ok(warp::reply::json(&Response::<()>::new(
            false,
            (),
            "operation_fail: we can only destruct Emunet in normal state".to_string(),
        )));
    };

    // update the state of the emunet object into working
    emunet.working();
    let _ = extract_response!(
        db_client.set_emu_net(emunet.clone()).await,
        "internal_server_error",
        "fatal(this-should-never-happen)"
    );

    // do the actual destruction in the background
    tokio::spawn(background_task(db_client, emunet));

    // reply to the client
    #[derive(Serialize)]
    struct ResponseData {
        status: String,
    }
    Ok(warp::reply::json(&Response::<ResponseData>::new(
        true,
        ResponseData {
            status: "working".to_string(),
        },
        String::new(),
    )))
}

/// This filter initializes the emunet by creating the vertexes and edges of the emulation network.
pub fn build_filter(
    db_client: Client,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + Send + Sync + 'static
{
    let db_filter = warp::any().map(move || {
        let clone = db_client.clone();
        clone
    });
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path("destruct_emunet"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(destruct_emunet)
}
