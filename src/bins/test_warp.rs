use std::net::ToSocketAddrs;
use std::io::{Error, ErrorKind};

use warp::Filter;
use tokio::time::{timeout, Duration};

use mocknet::dbnew::{self};
use mocknet::emunet::server;
use mocknet::restful::{*};

use mocknet::algo::in_memory_graph::{InMemoryGraph};
use mocknet::algo::Partition;

const LOCAL_ADDR: [u32; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u32 = 3030;

const DB_ADDR: [u32; 4] = [127, 0, 0, 1];
const DB_PORT: u32 = 27615;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
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
                    println!("Successfully initialize the database.")
                },
                Err(s) => {
                    println!("{}", &s);
                }
            };
            
            // // register a test user "wtf"
            // let res = client.register_user("wtf").await?;
            // match res {
            //     Succeed(_) => {
            //         println!("successfully register user wtf");
            //     }
            //     Fail(s) => {
            //         println!("{}", &s);
            //     }
            // };

            // // create an emunet
            // let res = client.create_emu_net("wtf".to_string(), "wtf".to_string(), 12).await?;
            // match res {
            //     Succeed(uuid) => {
            //         println!("successfulily create an emunet with id {}", uuid);
            //     }
            //     Fail(s) => {
            //         println!("{}", &s);
            //     }
            // };

            // // get the emunet wtf from user wtf
            // let res = client.list_emu_net_uuid("wtf".to_string()).await?;
            // let mut wtf_emunet_uuid = indradb::util::generate_uuid_v1();
            // match res {
            //     Succeed(hmap) => {
            //         wtf_emunet_uuid = hmap.get("wtf").unwrap().clone();
            //         println!("wtf emunet uuid is {}", &wtf_emunet_uuid);
            //     },
            //     Fail(s) => {
            //         panic!("{}", &s);
            //     }
            // }

            // let res = client.get_emu_net(wtf_emunet_uuid).await?;
            // let mut emu_net = res.unwrap();
            // println!("{:?}", &emu_net);

            // let vertexes: Vec<_> = vec!(1,2,3,4,5).into_iter().map(|e| {
            //     (e, 0)
            // }).collect();
            // let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,5), (2,3)).into_iter().map(|e| {
            //     (e, 0)
            // }).collect();
        
            // let vertex_json = serde_json::to_value(vertexes).unwrap();
            // let edge_json = serde_json::to_value(edges).unwrap();
        
            // let graph: InMemoryGraph<u64, u64, u64> = InMemoryGraph::from_jsons(vertex_json, edge_json).unwrap();
            // graph.dump();

            // let partition_result = graph.partition(emu_net.servers_mut()).unwrap(); 
            // println!("{:?}", &partition_result);

            let ru = register_user::build_filter(client.clone());
            let ce = create_emunet::build_filter(client.clone());
            // let ie = init_emunet::build_filter(client.clone());
            let routes = ru.or(ce);//.or(ie);
            warp::serve(routes).run(([127, 0, 0, 1], 3030)).await; 

            Ok(())
        }
    }).await.unwrap();
    
    Ok(())
}