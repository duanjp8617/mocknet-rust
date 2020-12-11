use std::net::ToSocketAddrs;

use warp::Filter;

use mocknet::database::{self, QueryOk, QueryFail};
use mocknet::emunet::server;
use mocknet::restful::{*};

use mocknet::algo::in_memory_graph::{InMemoryGraph};
use mocknet::algo::Partition;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    let addr = "127.0.0.1:27615"
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address");

    let launcher = match database::ClientLauncher::connect(&addr).await {
        Ok(l) => l,
        Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send>),
    };
    launcher.with_db_client(|client| {
        async move {
            // an initial server pool
            let mut sp = server::ServerInfoList::new(); 
            sp.add_server_info("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5).unwrap();
            sp.add_server_info("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7).unwrap();
            sp.add_server_info("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9).unwrap();
            sp.add_server_info("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10).unwrap();
            
            // try to initialize the database
            let res = client.init(sp.into_vec()).await?;
            match res {
                QueryOk(_) => {
                    println!("Successfully initialize the database.")
                },
                QueryFail(s) => {
                    println!("{}", &s);
                }
            };
            
            // register a test user "wtf"
            let res = client.register_user("wtf").await?;
            match res {
                QueryOk(_) => {
                    println!("successfully register user wtf");
                }
                QueryFail(s) => {
                    println!("{}", &s);
                }
            };

            // create an emunet
            let res = client.create_emu_net("wtf".to_string(), "wtf".to_string(), 12).await?;
            match res {
                QueryOk(uuid) => {
                    println!("successfulily create an emunet with id {}", uuid);
                }
                QueryFail(s) => {
                    println!("{}", &s);
                }
            };

            // get the emunet wtf from user wtf
            let res = client.list_emu_net_uuid("wtf".to_string()).await?;
            let mut wtf_emunet_uuid = indradb::util::generate_uuid_v1();
            match res {
                QueryOk(hmap) => {
                    wtf_emunet_uuid = hmap.get("wtf").unwrap().clone();
                    println!("wtf emunet uuid is {}", &wtf_emunet_uuid);
                },
                QueryFail(s) => {
                    panic!("{}", &s);
                }
            }

            let res = client.get_emu_net(wtf_emunet_uuid).await?;
            let mut emu_net = res.unwrap();
            println!("{:?}", &emu_net);

            let vertexes: Vec<_> = vec!(1,2,3,4,5).into_iter().map(|e| {
                (e, 0)
            }).collect();
            let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,5), (2,3)).into_iter().map(|e| {
                (e, 0)
            }).collect();
        
            let vertex_json = serde_json::to_value(vertexes).unwrap();
            let edge_json = serde_json::to_value(edges).unwrap();
        
            let graph: InMemoryGraph<u64, u64, u64> = InMemoryGraph::from_jsons(vertex_json, edge_json).unwrap();
            graph.dump();

            let partition_result = graph.partition(emu_net.servers_mut()).unwrap();
            // println!("{:?}", &partition_result);

            // let ru = register_user::build_filter(client.clone());
            // let ce = create_emunet::build_filter(client.clone());
            // let ie = init_emunet::build_filter(client.clone());
            // let routes = ru.or(ce).or(ie);
            // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await; 

            Ok(())
        }
    }).await.unwrap();
    
    Ok(())
}