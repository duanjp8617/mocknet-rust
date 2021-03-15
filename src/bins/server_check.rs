use mocknet::restful::*;

const LOCAL_ADDR: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 3031;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    // build up the warp filters
    let server_ping = server_ping::build_filter();
    // launch the warp server
    warp::serve(server_ping).run((LOCAL_ADDR, LOCAL_PORT)).await;

    Ok(())

}
