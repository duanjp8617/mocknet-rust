use std::mem::replace;

use uuid::Uuid;

use crate::database::message::{Response, ResponseFuture, DatabaseMessage};
use crate::database::errors::BackendError;
use crate::database::backend::IndradbClientBackend;

pub struct GetEmuNet {
    emu_net_id: Uuid,
}

impl GetEmuNet {
    pub fn new(emu_net_id: Uuid) -> Self {
        Self{ emu_net_id }
    }
}

impl DatabaseMessage<Response, BackendError> for GetEmuNet {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let emu_net_id = replace(&mut self.emu_net_id, indradb::util::generate_uuid_v1());

        Box::pin(async move {
            let res = backend.get_vertex_json_value(emu_net_id, "default").await?;
            match res {
                None => fail!(GetEmuNet, "emunet not exist".to_string()),
                Some(jv) => succeed!(GetEmuNet, serde_json::from_value(jv).unwrap(),),
            }
        })
    }
}