use std::error::Error as StdError;

use warp::Filter;

use mocknet::new_database::*;
use mocknet::new_emunet::*;
use mocknet::new_restful::*;

const LOCAL_ADDR: [u8; 4] = [172,27,220,175];
const LOCAL_PORT: u16 = 3031;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {
    let connector = new_connector("grpc://127.0.0.1:27615").await?;
    let mut cluster = cluster::ClusterInfo::new();
    cluster
        .add_server_info("192.168.0.1", 1, "djp", "djp")
        .unwrap();
    cluster
        .add_server_info("192.168.0.2", 2, "djp", "djp")
        .unwrap();
    cluster
        .add_server_info("192.168.0.3", 1, "djp", "djp")
        .unwrap();
    cluster
        .add_server_info("192.168.0.4", 2, "djp", "djp")
        .unwrap();

    let res = init(&connector, cluster).await?;
    match res {
        Ok(_) => {
            println!("successfully initialize the database");
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    let routes = user_registration::build_filter(connector.clone());
    let routes = routes.or(emunet_creation::build_filter(connector.clone()));
    let routes = routes.or(list_all::build_filter(connector.clone()));
    let routes = routes.or(list_emunet::build_filter(connector.clone()));
    let routes = routes.or(user_deletion::build_filter(connector.clone()));
    let routes = routes.or(emunet_init::build_filter(connector.clone()));
    let routes = routes.or(emunet_deletion::build_filter(connector.clone()));
    let routes = routes.or(get_emunet_info::build_filter(connector.clone()));
    warp::serve(routes).run((LOCAL_ADDR, LOCAL_PORT)).await;
    Ok(())
}
