
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

    let ls = tokio::task::LocalSet::new();
    let db_loop_end_fut = ls.run_until(db_loop);

    let resp = db_client.ping().await?;
    println!("Ping indradb returns {}", resp);

    db_loop_end_fut.await?;
    Ok(())
}