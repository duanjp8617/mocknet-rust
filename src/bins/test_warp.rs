use std::net::ToSocketAddrs;
use std::io::{Error, ErrorKind};

use warp::Filter;
use tokio::time::{timeout, Duration};

use mocknet::dbnew::{self};
use mocknet::emunet::server;
use mocknet::restful::{*};

use mocknet::algo::in_memory_graph::{InMemoryGraph};
use mocknet::algo::Partition;

const LOCAL_ADDR: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 3030;

const DB_ADDR: [u8; 4] = [127, 0, 0, 1];
const DB_PORT: u16 = 27615;

use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    err_reason: String,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    println!("{}", serde_json::to_string(&ErrorResponse{ err_reason: "wtf".to_string()}).unwrap());
    
    // build up the database address
    let db_addr_str = DB_ADDR.iter().enumerate().fold(String::new(), |mut s, (idx, part)| {
        if idx < DB_ADDR.len()-1 {
            s = s + &format!("{}.", part);
        }
        else {
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
    let res = timeout(Duration::from_secs(2), dbnew::ClientLauncher::connect(&db_addr)).await.map_err(|_| {
        let err_msg: &str = &format!("connection to {} timeout", &db_addr_str);
        Box::new(Error::new(ErrorKind::Other, err_msg)) as Box<dyn std::error::Error + Send>
    })?;
    let launcher = res.map_err(|e| {
        let err_msg: &str = &format!("connection to {} fails: {}", &db_addr_str, e);
        Box::new(Error::new(ErrorKind::Other, err_msg)) as Box<dyn std::error::Error + Send>
    })?;
    
    launcher.with_db_client(|client| {
        async move {
            // an initial server pool for testing purpose, the server pool should 
            // be initialized from program inputs
            let mut sp = server::ServerInfoList::new(); 
            sp.add_server_info("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5).unwrap();
            sp.add_server_info("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7).unwrap();
            sp.add_server_info("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9).unwrap();
            sp.add_server_info("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10).unwrap();
            
            // try to initialize the database
            let res = client.init(sp.into_vec()).await?;
            match res {
                Ok(_) => {
                    println!("successfully initialize the database")
                },
                Err(s) => {
                    // the database has been initialized, just print the error message
                    println!("{}", &s);
                }
            };
            
            // build up the warp filters
            let ru = register_user::build_filter(client.clone());
            let ce = create_emunet::build_filter(client.clone());
            // let ie = init_emunet::build_filter(client.clone());
            let routes = ru.or(ce);//.or(ie);

            // launch the warp server
            warp::serve(routes).run((LOCAL_ADDR, LOCAL_PORT)).await; 

            Ok(())
        }
    }).await.unwrap();
    
    Ok(())
}