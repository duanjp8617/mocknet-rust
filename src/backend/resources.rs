// The underlying resource used by EmuNet.
// We should have Vec<CtServer> stored in the database. Each stored item should have cur_ct sets to 0.
struct CtServer {
    id: String, // an unique ID of th server
    wan_ip: String, // wan-facing IP address for accessing the Internet
    manage_ip: String, // management IP address for transmitting control messages
    data_ip: String, // data IP address for transmitting emulation data traffic.`
    max_capacity: u32, // maximum number of emulation containers that this server is allowed to create
    cur_ct: u32, // current number of emulation containers
}

struct CtServerPool {

}

impl CtServerPool {
    async fn allocate(&mut self, n: u32) -> Result<Vec<CtServer>, std::io::Error> {
        unimplemented!();
    }
}

enum EmuNetStatus {
    Uninit, // The initial state of a EmuNet. The EmuNet is created with a local resource pool, but without a working topology
    Ready, // The EmuNet is ready for deployment
    Launching, // The EmuNet is busy creating new EmuDev and EmuLink
    Error, // The EmuNet is experiencing an error, you must 
}

struct EmuNet {
    id: String,
    status: EmuNetStatus,
    servers: Vec<CtServer>,
    emu_dev_uuid: std::collections::HashMap<String, i32>,
    emu_link_uuid: std::collections::HashMap<String, i32>,
}

// We should have Vec<EmuNetUser> stored in the database, or HashMap<String, EmuNetUser> to accelerate indexing
struct EmuNetUser {
    id: String, // an unique ID of the EmuNetUser
    name: String, // name of the user
    enet_id: Option<String>,
}

// 1. Register a new User with a user name, assign a unique ID for the user, initialize the enet_id to None. 
//    Return necessary errors

// 2. The client side then sends the total number of required CtServers by a certain user to the backend. 
// 2.1 Check whether the user has already has an enet, if so, return error.
// 2.2 Start building the enet for the user.
// 1. Making an allocation from the CtServerPool. If there are not enough servers, return error.
// 2. Check whether the CtServers all work normally, if not, return error.
// 3. Create the EmuNet, set the state of the EmuNet to Ready, synchronize the database, and return confirmation message to the frontend

// 3. The client side sends a EmuNet initialization request to the backend. The initialization request contains two list. 
struct EmuDev {
    name: String,
    total_ports: u32,
}
// 1. The first list is a list of EmuDev device to be created.
struct EmuLink {
    in_bound: String,
    out_bound: String,
    name: String,
}
// 2. The second list is a list of EmuLink to be created.
// 3. The backend then confirms that the EmuNet is still uninitialized, if not, return an error.
// 4. Then we start doing the most important part, which is to construct the EmuNet topology in the graph database.
// 4.1: For each EmuDev, create a new vertice in the graph database, and store the vertice uuid in the EmuNet structure.
// 4.2: For each EmuLink, retrieve the in_bound and out_bound EmuDev name, and create a new edge in the graph database, store the 
// edge id in Emunet structure
// 4.3: In case there are errors, return errors.
// 5. After creating the EmuNet structure, set its status to store it in the database, and return OK.



// 2.3 First, check whether we have enough number of CtServer to allocate to this EmuNet. If it's available, 
//    2.1 The backend first checks whether the user exists and whether the user has already received an allocation. 
//        If so, the backend returns error message
//    2.2 The backend then retrieves several CtServers from the database. If the number of available CtServers can not 
//        satisfy the allocation, then return an error message.
//    2.3 The backend then performs a connection check to decide whether all the CtServers can be correctly conncted. If the
//        connection check fails, the backend retursn an error message back to the front end. The failed CtServers will be 
//        deleted from the resource pool, and the succeeded CtServers will be re-added back to the resource pool.
//    2.4 The CtServers are then added to the corresponding user, and the database is updated. 
//    2.5 
