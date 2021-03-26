// use std::io::{Error, ErrorKind};
// use std::net::ToSocketAddrs;

// use tokio::time::{delay_for, timeout, Duration};
// use warp::Filter;

// const LOCAL_ADDR: [u8; 4] = [127, 0, 0, 1];
// const LOCAL_PORT: u16 = 3030;

// const DB_ADDR: [u8; 4] = [127, 0, 0, 1];
// const DB_PORT: u16 = 27615;

// use mocknet::database_new;
// use mocknet::emunet_new::cluster;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
//     let db_addr_str = DB_ADDR
//         .iter()
//         .enumerate()
//         .fold(String::new(), |mut s, (idx, part)| {
//             if idx < DB_ADDR.len() - 1 {
//                 s = s + &format!("{}.", part);
//             } else {
//                 s = s + &format!("{}:{}", part, DB_PORT);
//             }
//             s
//         });
//     let db_addr = &db_addr_str
//         .to_socket_addrs()
//         .unwrap()
//         .next()
//         .expect("could not parse address");

//     let mut loop_counter: u32 = 15;
//     let launcher = loop {
//         if loop_counter == 0 {
//             let err_msg: &str = &format!(
//                 "connection to {} exceeds maximum retry limits",
//                 &db_addr_str
//             );
//             return Err(Box::new(Error::new(ErrorKind::Other, err_msg))
//                 as Box<dyn std::error::Error + Send>);
//         }
//         let res = timeout(
//             Duration::from_secs(2),
//             database_new::ClientLauncher::connect(&db_addr),
//         )
//         .await;
//         match res {
//             Ok(conn_res) => match conn_res {
//                 Ok(conn) => {
//                     break conn;
//                 }
//                 Err(err) => {
//                     println!("connection to {} fails: {}, retrying", &db_addr_str, err);
//                 }
//             },
//             Err(_) => {
//                 println!("connection timeout, retrying");
//             }
//         };
//         loop_counter -= 1;
//         delay_for(Duration::from_secs(2)).await;
//     };

//     launcher
//         .with_db_client(|client| {
//             async move {
//                 // add 8 conseuctive servers
//                 let mut cluster_info = cluster::ClusterInfo::new();
//                 let ip_base = "10.10.5.";
//                 for i in 5..12 {
//                     cluster_info
//                         .add_server_info(
//                             format!("{}{}", ip_base, i),
//                             15,
//                             "djp".into(),
//                             "djp".into(),
//                         )
//                         .unwrap();
//                 }

//                 // let res = client.init(sp.into_vec()).await?;
//                 // match res {
//                 //     Ok(_) => println!("successfully initialize the database"),
//                 //     Err(s) => {
//                 //         // the database has been initialized, just print the error message
//                 //         println!("{}", &s);
//                 //     }
//                 // };

//                 // // build up the warp filters
//                 // let ru = register_user::build_filter(client.clone());
//                 // let ce = create_emunet::build_filter(client.clone());
//                 // let le = list_emunet::build_filter(client.clone());
//                 // let ge_info = get_emunet_info::build_filter(client.clone());
//                 // let ge_topo = get_emunet_topo::build_filter(client.clone());
//                 // let ie = init_emunet::build_filter(client.clone());
//                 // let destruct_e = destruct_emunet::build_filter(client.clone());
//                 // let delete_e = delete_emunet::build_filter(client.clone());
//                 // let delete_u = delete_user::build_filter(client.clone());
//                 // let update_u = update_emunet::build_filter(client.clone());
//                 // let routes = ru
//                 //     .or(ce)
//                 //     .or(le)
//                 //     .or(ge_info)
//                 //     .or(ge_topo)
//                 //     .or(ie)
//                 //     .or(destruct_e)
//                 //     .or(delete_e)
//                 //     .or(delete_u)
//                 //     .or(update_u);

//                 // // launch the warp server
//                 // warp::serve(routes).run((LOCAL_ADDR, LOCAL_PORT)).await;

//                 Ok(())
//             }
//         })
//         .await
//         .unwrap();

//     Ok(())
// }

fn main() {
    println!("fuck!");
}