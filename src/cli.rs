use clap::{App, Arg, SubCommand};

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
#[derive(Debug)]
pub struct CtlArg {
    user: String,
    subcmd: UserSubcmd,
}
#[derive(Debug)]
pub enum UserSubcmd {
    History,
    Restore(u64),
    Update(String),
    NetworkOp(String, NetworkSubcmd),
}
#[derive(Debug)]
pub enum NetworkSubcmd {
    Info,
    Dev(u64),
    Path(u64, u64),
    Connect(u64, u64),
    Disconnect(u64, u64),
    ConnectionHistory,
}

const USERNAME: &str = "USERNAME";
const DEVID: &str = "DEVID";
const SRCID: &str = "SRCID";
const DSTID: &str = "DSTID";
const HISTORYIDX: &str = "HISTORYIDX";
const FILEPATH: &str = "FILEPATH";
const NETWORKNAME: &str = "NETWORKNAME";

pub fn parse_ctl_arg() -> CtlArg {
    let username = Arg::with_name(USERNAME)
        .help("user name to operate on")
        .short("u")
        .value_name(USERNAME)
        .takes_value(true);

    // network subcommand
    let info = SubCommand::with_name("info").about("show information about the emulation network");
    let dev = SubCommand::with_name("dev")
        .about("show device information using device id")
        .arg(
            Arg::with_name(DEVID)
                .value_name(DEVID)
                .help("ID of the device to show")
                .takes_value(true),
        );
    let path = SubCommand::with_name("path")
        .about("show the path between two devices")
        .arg(
            Arg::with_name(SRCID)
                .value_name(SRCID)
                .help("source device ID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(DSTID)
                .value_name(DSTID)
                .help("destination device ID")
                .takes_value(true),
        );
    let connect = SubCommand::with_name("connect")
        .about("connect the path for the two devices")
        .arg(
            Arg::with_name(SRCID)
                .value_name(SRCID)
                .help("source device ID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(DSTID)
                .value_name(DSTID)
                .help("destination device ID")
                .takes_value(true),
        );
    let disconnect = SubCommand::with_name("disconnect")
        .about("disconnect the path for the two devices")
        .arg(
            Arg::with_name(SRCID)
                .value_name(SRCID)
                .help("source device ID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(DSTID)
                .value_name(DSTID)
                .help("destination device ID")
                .takes_value(true),
        );
    let conn_history = SubCommand::with_name("conn-history")
        .about("show connection history of this emulation network");

    // user subcommand
    let history =
        SubCommand::with_name("history").about("show the history of this emulation network");
    let restore = SubCommand::with_name("restore")
        .about("restore the emulation network from a history index")
        .arg(
            Arg::with_name(HISTORYIDX)
                .value_name(HISTORYIDX)
                .help("index of the history to restore")
                .takes_value(true),
        );
    let update = SubCommand::with_name("update")
        .about("update the emulation network using an input file")
        .arg(
            Arg::with_name(FILEPATH)
                .value_name(FILEPATH)
                .help("file path that stores the input network format")
                .takes_value(true),
        );
    let network_op = SubCommand::with_name("network")
        .about("operations on the emulation network")
        .arg(
            Arg::with_name(NETWORKNAME)
                .value_name(NETWORKNAME)
                .help("name of the emulation network to operate on")
                .takes_value(true),
        )
        .subcommand(info)
        .subcommand(dev)
        .subcommand(path)
        .subcommand(connect)
        .subcommand(disconnect)
        .subcommand(conn_history);

    let matches = App::new("ctl-cli")
        .arg(&username)
        .subcommand(history)
        .subcommand(restore)
        .subcommand(update)
        .subcommand(network_op)
        .get_matches();

    CtlArg {
        user: matches.value_of(USERNAME).expect("missing user name").to_string(),
        subcmd: if let Some(_) = matches.subcommand_matches("history") {
            UserSubcmd::History
        } else if let Some(matches) = matches.subcommand_matches("restore") {
            UserSubcmd::Restore(
                matches
                    .value_of(HISTORYIDX)
                    .unwrap()
                    .to_string()
                    .parse::<u64>()
                    .unwrap(),
            )
        } else if let Some(matches) = matches.subcommand_matches("update") {
            UserSubcmd::Update(matches.value_of("FILEPATH").unwrap().to_string())
        } else if let Some(matches) = matches.subcommand_matches("network") {
            let network_subcmd = if let Some(_) = matches.subcommand_matches("info") {
                NetworkSubcmd::Info
            } else if let Some(matches) = matches.subcommand_matches("dev") {
                NetworkSubcmd::Dev(
                    matches
                        .value_of(DEVID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                )
            } else if let Some(matches) = matches.subcommand_matches("path") {
                NetworkSubcmd::Path(
                    matches
                        .value_of(SRCID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                    matches
                        .value_of(DSTID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                )
            } else if let Some(matches) = matches.subcommand_matches("connect") {
                NetworkSubcmd::Connect(
                    matches
                        .value_of(SRCID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                    matches
                        .value_of(DSTID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                )
            } else if let Some(matches) = matches.subcommand_matches("disconnect") {
                NetworkSubcmd::Disconnect(
                    matches
                        .value_of(SRCID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                    matches
                        .value_of(DSTID)
                        .unwrap()
                        .to_string()
                        .parse::<u64>()
                        .unwrap(),
                )
            } else if let Some(_) = matches.subcommand_matches("conn-history") {
                NetworkSubcmd::ConnectionHistory
            } else {
                panic!("wtf is this?");
            };
            UserSubcmd::NetworkOp(
                matches.value_of(NETWORKNAME).unwrap().to_string(),
                network_subcmd,
            )
        } else {
            panic!("wtf is this");
        },
    }
}
