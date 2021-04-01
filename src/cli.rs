use clap::{App, Arg};

pub struct CliArg {
    pub warp_addr: String,
    pub indradb_addr: String,
    pub cluster_config_path: Option<String>,
}

const WARP_ADDR: &str = "WARP_ADDR";
const INDRADB_ADDR: &str = "INDRADB_ADDR";
const CLUSTER_CONFIG_PATH: &str = "CLUSTER_CONFIG_PATH";

pub fn parse_cli_arg() -> CliArg {
    let warp_addr_arg = Arg::with_name(WARP_ADDR)
        .help("Warp server listening address")
        .long("warp-addr")
        .value_name(WARP_ADDR)
        .takes_value(true)
        .default_value("127.0.0.1:3030");

    let indradb_addr_arg = Arg::with_name(INDRADB_ADDR)
        .help("Indradb server address")
        .long("indradb-addr")
        .value_name(INDRADB_ADDR)
        .takes_value(true)
        .default_value("127.0.0.1:27615");

    let cluster_config_path_arg = Arg::with_name(CLUSTER_CONFIG_PATH)
        .help("Cluster configure file path")
        .long("cluster-config")
        .value_name(CLUSTER_CONFIG_PATH)
        .takes_value(true);

    let matches = App::new("mocknet-server")
        .arg(&warp_addr_arg)
        .arg(&indradb_addr_arg)
        .arg(&cluster_config_path_arg)
        .get_matches();

    CliArg {
        warp_addr: matches.value_of(WARP_ADDR).unwrap().to_string(),
        indradb_addr: matches.value_of(INDRADB_ADDR).unwrap().to_string(),
        cluster_config_path: matches.value_of(CLUSTER_CONFIG_PATH).map(|s| s.to_string()),
    }
}
