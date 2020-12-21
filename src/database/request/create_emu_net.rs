use std::mem::replace;
use std::collections::HashMap;

use crate::emunet::user;
use crate::emunet::server;
use crate::database::message::{Response, ResponseFuture, DatabaseMessage};
use crate::database::errors::BackendError;
use crate::database::backend::IndradbClientBackend;
use crate::emunet::net;

pub struct CreateEmuNet {
    user: String,
    emu_net: String,
    capacity: u32,
}

impl CreateEmuNet {
    pub fn new(user: String, emu_net: String, capacity: u32) -> Self {
        Self { user, emu_net, capacity }
    }
}

impl DatabaseMessage<Response, BackendError> for CreateEmuNet {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let msg = replace(self, CreateEmuNet {
            user: String::new(),
            emu_net: String::new(),
            capacity: 0,
        });

        Box::pin(async move {
            let msg = msg;
            
            // get the user
            let mut user_map: HashMap<String, user::EmuNetUser> = backend.get_user_map().await?;
            if user_map.get(&msg.user).is_none() {
                return fail!(CreateEmuNet, "invalid user name".to_string());
            }
            let user_mut = user_map.get_mut(&msg.user).unwrap();

            // check whether the emunet has existed
            if user_mut.emu_net_exist(&msg.emu_net) {
                return fail!(CreateEmuNet, "invalid emu-net name".to_string())
            }

            // get the allocation of servers
            let server_info_list: Vec<server::ServerInfo> = backend.get_server_info_list().await?;
            let mut sp = server::ServerInfoList::from_iterator(server_info_list.into_iter()).unwrap();
            let allocation = match sp.allocate_servers(msg.capacity) {
                Some(alloc) => alloc,
                None => return fail!(CreateEmuNet, "invalid capacity".to_string()),
            };
            backend.set_server_info_list(sp.into_vec()).await?;
            
            // create a new emu net node
            let emu_net_id = backend.create_vertex(None).await?.expect("vertex ID already exists");
            // create a new emu net
            let mut emu_net = net::EmuNet::new(msg.emu_net.clone(), emu_net_id.clone(), msg.capacity);
            emu_net.add_servers(allocation);
            // initialize the EmuNet in the database
            let jv = serde_json::to_value(emu_net).unwrap();
            let res = backend.set_vertex_json_value(emu_net_id, "default", &jv).await?;
            if !res {
                panic!("vertex not exist");
            }

            // add the new emunet to user map
            user_mut.add_emu_net(msg.emu_net, emu_net_id.clone());
            backend.set_user_map(user_map).await?;

            succeed!(CreateEmuNet, emu_net_id,)
        })
    }
}