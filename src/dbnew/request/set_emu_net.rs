use std::mem::replace;

use crate::dbnew::message::{Response, ResponseFuture, DatabaseMessage, Succeed, Fail};
use crate::dbnew::errors::BackendError;
use crate::dbnew::backend::IndradbClientBackend;
use crate::emunet::net;

use Response::SetEmuNet as Resp;

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
                false => Ok(Resp(Fail("EmuNet not exist".to_string()))),
                true => Ok(Resp(Succeed(()))),
            }
        })
    }
}