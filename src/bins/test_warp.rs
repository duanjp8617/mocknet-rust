// use std::net::ToSocketAddrs;
// use mocknet::database::build_client_fut;
// use mocknet::emunet::server;

// use mocknet::restful::{register_user, create_emunet, init_emunet};
// use warp::Filter;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "127.0.0.1:27615"
//     .to_socket_addrs()
//     .unwrap()
//     .next()
//     .expect("could not parse address");
    
//     let stream = tokio::net::TcpStream::connect(&addr).await?;
//     stream.set_nodelay(true)?;
//     let ls = tokio::task::LocalSet::new();

//     let (indradb_client, backend_fut) = build_client_fut(stream, &ls);


//     let jh = tokio::spawn(async move {
//         let res = indradb_client.ping().await.unwrap();
//         println!("response is {}", res);


//         let mut sp = server::ServerPool::new();
//         sp.add_server("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5);
//         sp.add_server("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7);
//         sp.add_server("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9);
//         sp.add_server("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10);
//         let res = indradb_client.init(sp.into_vec()).await.unwrap();
//         if res {
//             println!("successfully initializing the database");
//         }
//         else {
//             println!("the database has already been initialized");
//         }

//         let res = indradb_client.register_user("wtf").await.unwrap();
//         if res {
//             println!("successfully register a new user");
//         }
//         else {
//             println!("the user with a similar name is presented");
//         }

//         let ru = register_user::build_filter(indradb_client.clone());
//         let ce = create_emunet::build_filter(indradb_client.clone());
//         let ie = init_emunet::build_filter(indradb_client.clone());
//         let routes = ru.or(ce).or(ie);
//         warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
//     });


//     backend_fut.await?;
//     jh.await.unwrap();

//     // // Match any request and return hello world!
//         // let routes = register_user::build_filter(&indradb_client);

//         // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
//     Ok(())
// }

use std::net::ToSocketAddrs;
use mocknet::dbnew::client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:27615"
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address");

    let launcher = client::ClientLauncher::new(&addr).await?;
    launcher.with_db_client(|client| {
        async move {
            Ok(())
        }
    }).await?;
    
    // let stream = tokio::net::TcpStream::connect(&addr).await?;
    // stream.set_nodelay(true)?;
    // let ls = tokio::task::LocalSet::new();

    // let (indradb_client, backend_fut) = build_client_fut(stream, &ls);


    // let jh = tokio::spawn(async move {
    //     let res = indradb_client.ping().await.unwrap();
    //     println!("response is {}", res);


    //     let mut sp = server::ServerPool::new();
    //     sp.add_server("127.0.0.1", 128, "128.0.0.2", "129.0.0.5", 5);
    //     sp.add_server("127.0.0.2", 128, "128.0.0.3", "129.0.0.4", 7);
    //     sp.add_server("137.0.0.1", 128, "138.0.0.2", "139.0.0.5", 9);
    //     sp.add_server("137.0.0.2", 128, "138.0.0.3", "139.0.0.4", 10);
    //     let res = indradb_client.init(sp.into_vec()).await.unwrap();
    //     if res {
    //         println!("successfully initializing the database");
    //     }
    //     else {
    //         println!("the database has already been initialized");
    //     }

    //     let res = indradb_client.register_user("wtf").await.unwrap();
    //     if res {
    //         println!("successfully register a new user");
    //     }
    //     else {
    //         println!("the user with a similar name is presented");
    //     }

    //     let ru = register_user::build_filter(indradb_client.clone());
    //     let ce = create_emunet::build_filter(indradb_client.clone());
    //     let ie = init_emunet::build_filter(indradb_client.clone());
    //     let routes = ru.or(ce).or(ie);
    //     warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    // });


    // backend_fut.await?;
    // jh.await.unwrap();

    // // Match any request and return hello world!
        // let routes = register_user::build_filter(&indradb_client);

        // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}