use mocknet::restful::*;

use clap::{App, Arg};

const WARP_ADDR: &str = "WARP_ADDR";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    let warp_addr_arg = Arg::with_name(WARP_ADDR)
        .help("Warp server listening address")
        .long("warp-addr")
        .value_name(WARP_ADDR)
        .takes_value(true)
        .default_value("127.0.0.1:4040");

    let matches = App::new("server-check").arg(&warp_addr_arg).get_matches();

    let warp_addr = matches.value_of(WARP_ADDR).unwrap().to_string();
    let warp_socket_addr = warp_addr
        .parse::<std::net::SocketAddr>()
        .expect("invalid warp listening address");

    let server_ping = server_ping::build_filter();
    warp::serve(server_ping).run(warp_socket_addr).await;

    Ok(())
}
