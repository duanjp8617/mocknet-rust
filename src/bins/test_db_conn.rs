
use std::net::ToSocketAddrs;
use mocknet::database::indradb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:27615"
    .to_socket_addrs()
    .unwrap()
    .next()
    .expect("could not parse address");
    
    let (db_client, db_loop) = indradb::build(&addr).await?;
    println!("good");

    let fut = tokio::spawn(async move {
        let res = db_client.ping().await;
        res
    });

    let ls = tokio::task::LocalSet::new();
    let db_loop_end_fut = ls.run_until(async move {
        println!("running");
        db_loop.await
    });
    db_loop_end_fut.await?;

    let res = fut.await?;
    println!("Ping response is {}", res.unwrap());
    Ok(())
}