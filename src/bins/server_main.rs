use std::io::{Error, ErrorKind};
use std::net::ToSocketAddrs;

use tokio::time::{delay_for, timeout, Duration};
use warp::Filter;

use mocknet::database;
use mocknet::emunet::server;
use mocknet::restful::*;

const LOCAL_ADDR: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 3030;

const DB_ADDR: [u8; 4] = [127, 0, 0, 1];
const DB_PORT: u16 = 27615;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ErrorResponse {
    err_reason: String,
}

#[derive(Deserialize, Serialize)]
pub struct EdgeInfo {
    edge_id: (u64, u64), // client side edge id in the form of (u64, u64)
    description: String, // a description string to hold the place
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    // build up the database address
    let db_addr_str = DB_ADDR
        .iter()
        .enumerate()
        .fold(String::new(), |mut s, (idx, part)| {
            if idx < DB_ADDR.len() - 1 {
                s = s + &format!("{}.", part);
            } else {
                s = s + &format!("{}:{}", part, DB_PORT);
            }
            s
        });
    println!("{}", &db_addr_str);
    let db_addr = &db_addr_str
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address");

    // create the database launcher
    let mut loop_counter: u32 = 15;
    let launcher = loop {
        if loop_counter == 0 {
            let err_msg: &str = &format!(
                "connection to {} exceeds maximum retry limits",
                &db_addr_str
            );
            return Err(Box::new(Error::new(ErrorKind::Other, err_msg))
                as Box<dyn std::error::Error + Send>);
        }
        let res = timeout(
            Duration::from_secs(2),
            database::ClientLauncher::connect(&db_addr),
        )
        .await;
        match res {
            Ok(conn_res) => match conn_res {
                Ok(conn) => {
                    break conn;
                }
                Err(err) => {
                    println!("connection to {} fails: {}, retrying", &db_addr_str, err);
                }
            },
            Err(_) => {
                println!("connection timeout, retrying");
            }
        };
        loop_counter -= 1;
        delay_for(Duration::from_secs(2)).await;
    };

    launcher
        .with_db_client(|client| {
            async move {
                // an initial server pool for testing purpose, the server pool should
                // be initialized from program inputs
                let mut sp = server::ServerInfoList::new();
                sp.add_server_info("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5)
                    .unwrap();
                sp.add_server_info("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7)
                    .unwrap();
                sp.add_server_info("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9)
                    .unwrap();
                sp.add_server_info("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10)
                    .unwrap();

                // try to initialize the database
                let res = client.init(sp.into_vec()).await?;
                match res {
                    Ok(_) => println!("successfully initialize the database"),
                    Err(s) => {
                        // the database has been initialized, just print the error message
                        println!("{}", &s);
                    }
                };

                // build up the warp filters
                let ru = register_user::build_filter(client.clone());
                let ce = create_emunet::build_filter(client.clone());
                let le = list_emunet::build_filter(client.clone());
                let ge_info = get_emunet_info::build_filter(client.clone());
                let ge_topo = get_emunet_topo::build_filter(client.clone());
                let ie = init_emunet::build_filter(client.clone());
                let destruct_e = destruct_emunet::build_filter(client.clone());
                let delete_e = delete_emunet::build_filter(client.clone());
                let routes = ru.or(ce).or(le).or(ge_info).or(ge_topo).or(ie).or(destruct_e).or(delete_e);

                // launch the warp server
                warp::serve(routes).run((LOCAL_ADDR, LOCAL_PORT)).await;

                Ok(())
            }
        })
        .await
        .unwrap();

    Ok(())
}
