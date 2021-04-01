use std::{error::Error as StdError, io::BufRead};

use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use warp::Filter;

use mocknet::cli::*;
use mocknet::database::*;
use mocknet::emunet::ClusterInfo;
use mocknet::restful::*;

#[derive(Deserialize, Serialize)]
struct ServerInfo {
    conn_addr: String,
    max_capacity: u64,
    username: String,
    password: String,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {
    let arg = parse_cli_arg();

    let warp_socket_addr = arg
        .warp_addr
        .parse::<std::net::SocketAddr>()
        .expect("invalid warp listening address");
    let connector = new_connector(format!("grpc://{}", &arg.indradb_addr)).await?;

    if let Some(config_file) = arg.cluster_config_path {
        let json_str = read_to_string(&config_file).await?;
        let server_infos: Vec<ServerInfo> = serde_json::from_str(&json_str)
            .expect("invalid cluster configuration file format");

        let mut cluster = ClusterInfo::new();
        for server_info in server_infos {
            cluster
                .add_server_info(
                    server_info.conn_addr,
                    server_info.max_capacity,
                    server_info.username,
                    server_info.password,
                )
                .expect("invalid server configuration");
        }

        let res = init(&connector, cluster).await?;
        match res {
            Ok(_) => {
                println!("successfully initialize the database");
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    } else {
        let is_init = init_ok(&connector).await?;
        if !is_init {
            println!("the database is not initialized");
            return Ok(());
        }
    };

    let routes = user_registration::build_filter(connector.clone());
    let routes = routes.or(emunet_creation::build_filter(connector.clone()));
    let routes = routes.or(list_all::build_filter(connector.clone()));
    let routes = routes.or(list_emunet::build_filter(connector.clone()));
    let routes = routes.or(user_deletion::build_filter(connector.clone()));
    let routes = routes.or(emunet_init::build_filter(connector.clone()));
    let routes = routes.or(emunet_deletion::build_filter(connector.clone()));
    let routes = routes.or(get_emunet_info::build_filter(connector.clone()));

    warp::serve(routes).run(warp_socket_addr).await;
    Ok(())
}
