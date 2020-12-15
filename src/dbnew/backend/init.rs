use std::mem::replace;
use std::collections::HashMap;

use crate::emunet::server;
use crate::emunet::user;
use crate::dbnew::message::{Response, ResponseFuture, DatabaseMessage, Succeed, Fail};
use crate::dbnew::errors::BackendError;
use super::IndradbClientBackend;
use super::CORE_INFO_ID;

use Response::Init as InitResp;

pub struct InitDatabase {
    server_infos: Vec<server::ServerInfo>,
}

impl DatabaseMessage<Response, BackendError> for InitDatabase {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let server_info_list = replace(&mut self.server_infos, Vec::new());
        Box::pin(async move {
            let res = backend.create_vertex(Some(CORE_INFO_ID.clone())).await?;
            match res {
                Some(_) => {
                    // initialize user map
                    backend.set_core_property("user_map", HashMap::<String, user::EmuNetUser>::new()).await?;

                    // initialize server list                
                    backend.set_core_property("server_info_list", server_info_list).await?;
                            
                    Ok(InitResp(Succeed(())))
                },
                None => Ok(InitResp(Fail("database has already been initialized".to_string()))),
            }
        })
    }
}