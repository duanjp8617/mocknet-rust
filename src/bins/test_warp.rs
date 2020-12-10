use std::net::ToSocketAddrs;

use warp::Filter;

use mocknet::database::{self, QueryOk, QueryFail};
use mocknet::emunet::server;
use mocknet::restful::{*};

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
            sp.add_server_info(server::ServerAddress::new("127.0.0.1", 128, "128.0.0.2", "129.0.0.5").unwrap(), 5).unwrap();
            sp.add_server_info(server::ServerAddress::new("127.0.0.2", 128, "128.0.0.3", "129.0.0.4").unwrap(), 7).unwrap();
            sp.add_server_info(server::ServerAddress::new("137.0.0.1", 128, "138.0.0.2", "139.0.0.5").unwrap(), 9).unwrap();
            sp.add_server_info(server::ServerAddress::new("137.0.0.2", 128, "138.0.0.3", "139.0.0.4").unwrap(), 10).unwrap();
            
            // try to initialize the database
            let res = client.init(sp.into_vec()).await?;
            match res {
                QueryOk(_) => {},
                QueryFail(s) => {
                    println!("{}", &s);
                }
            };
            
            // register a test user "wtf"
            let _res = client.register_user("wtf").await.unwrap();


            let ru = register_user::build_filter(client.clone());
            let ce = create_emunet::build_filter(client.clone());
            let ie = init_emunet::build_filter(client.clone());
            let routes = ru.or(ce).or(ie);
            warp::serve(routes).run(([127, 0, 0, 1], 3030)).await; 

            Ok(())
        }
    }).await.unwrap();
    
    Ok(())
}