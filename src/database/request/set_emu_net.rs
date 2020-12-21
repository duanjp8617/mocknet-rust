use std::mem::replace;

use crate::database::message::{Response, ResponseFuture, DatabaseMessage};
use crate::database::errors::BackendError;
use crate::database::backend::IndradbClientBackend;
use crate::emunet::net;

pub struct SetEmuNet {
    emu_net: net::EmuNet
}

impl SetEmuNet {
    pub fn new(emu_net: net::EmuNet) -> Self {
        Self{ emu_net }
    }
}

impl DatabaseMessage<Response, BackendError> for SetEmuNet {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let emu_net = replace(&mut self.emu_net, net::EmuNet::new(String::new(), indradb::util::generate_uuid_v1(), 0));

        Box::pin(async move {
            let uuid = emu_net.get_uuid().clone();
            let jv = serde_json::to_value(emu_net).unwrap();
            let res = backend.set_vertex_json_value(uuid, "default", &jv).await?;
            match res {
                false => fail!(SetEmuNet, "EmuNet not exist".to_string()),
                true => succeed!(SetEmuNet, (),),
            }
        })
    }
}