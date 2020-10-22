use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CtServer {
    name: String,
    id: Uuid,
    conn_addr: SocketAddr, //In the form of tonic address http://xx.xx.xx.xx:xxxxx
    data_ip: IpAddr, // an IPv4 address
    man_ip: IpAddr, // an IPv4 address
    capacity: u32,
}

struct EmuNetUser {
    name: String, 
    id: Uuid, 
    enets: HashMap<String, Uuid>,
}

enum EmuNetStatus {
    Uninit, // The initial state of a EmuNet. The EmuNet is created with a local resource pool, but without a working topology
    Ready, // The EmuNet is ready for deployment
    Launching, // The EmuNet is busy creating new EmuDev and EmuLink
    Error, // The EmuNet is experiencing an error, you must 
}

struct EmuNet {
    name: String,
    id: Uuid,
    status: EmuNetStatus,
    servers: Vec<CtServer>,
    emu_dev_uuid: std::collections::HashMap<String, Uuid>,
    emu_link_uuid: std::collections::HashMap<String, Uuid>,
}

struct EmuDev {
    name: String,
    id: Uuid,
    total_ports: u32,
    link_port_map: HashMap<Uuid, u32>,
}

struct EmuLink {
    in_bound: String,
    out_bound: String,
    name: String,
}


