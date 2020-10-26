
use std::net::ToSocketAddrs;
use mocknet::database::build_client_fut;
use mocknet::emunet::server;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "127.0.0.1:27615"
//     .to_socket_addrs()
//     .unwrap()
//     .next()
//     .expect("could not parse address");
    
//     let (db_client, db_loop) = indradb::build(&addr).await?;
//     println!("good");

//     let fut = tokio::spawn(async move {
//         let res = db_client.ping().await;
//         res
//     });

//     let ls = tokio::task::LocalSet::new();
//     let db_loop_end_fut = ls.run_until(async move {
//         println!("running");
//         db_loop.await
//     });
//     db_loop_end_fut.await?;

//     let res = fut.await?;
//     println!("Ping response is {}", res.unwrap());
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:27615"
    .to_socket_addrs()
    .unwrap()
    .next()
    .expect("could not parse address");
    
    let stream = tokio::net::TcpStream::connect(&addr).await?;
    stream.set_nodelay(true)?;
    let ls = tokio::task::LocalSet::new();

    let (indradb_client, backend_fut) = build_client_fut(stream, &ls);


    let jh = tokio::spawn(async move {
        let res = indradb_client.ping().await.unwrap();
        println!("response is {}", res);


        let mut sp = server::ServerPool::new();
        sp.add_server("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5);
        sp.add_server("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7);
        sp.add_server("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9);
        sp.add_server("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10);
        let res = indradb_client.init(sp.into_vec()).await.unwrap();
        if res {
            println!("successfully initializing the database");
        }
        else {
            println!("the database has already been initialized");
        }

        let res = indradb_client.register_user("wtf".to_string()).await.unwrap();
        if res {
            println!("successfully register a new user");
        }
        else {
            println!("the user with a similar name is presented");
        }
    });

    backend_fut.await?;
    jh.await.unwrap();
    
    Ok(())
}