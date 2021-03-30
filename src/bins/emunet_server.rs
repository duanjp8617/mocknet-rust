use std::error::Error as StdError;

use mocknet::new_database::*;
use mocknet::new_emunet::*;
use mocknet::new_restful::*;

const LOCAL_ADDR: [u8; 4] = [172, 23, 66, 208];
const LOCAL_PORT: u16 = 3030;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {
    let connector = new_connector("grpc://127.0.0.1:27615").await?;
    let mut cluster = cluster::ClusterInfo::new();
    cluster
        .add_server_info("192.168.0.1", 15, "djp", "djp")
        .unwrap();
    cluster
        .add_server_info("192.168.0.2", 15, "djp", "djp")
        .unwrap();
    cluster
        .add_server_info("192.168.0.3", 15, "djp", "djp")
        .unwrap();
    cluster
        .add_server_info("192.168.0.4", 15, "djp", "djp")
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

    let user_registration_filter = user_registration::build_filter(connector.clone());
    let routes = user_registration_filter;
    warp::serve(routes).run((LOCAL_ADDR, LOCAL_PORT)).await;
    Ok(())
}