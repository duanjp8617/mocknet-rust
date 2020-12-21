use std::mem::replace;
use std::collections::HashMap;

use crate::emunet::server;
use crate::emunet::user;
use crate::database::message::{Response, ResponseFuture, DatabaseMessage};
use crate::database::errors::BackendError;
use crate::database::backend::IndradbClientBackend;
use crate::database::CORE_INFO_ID;

pub struct Init {
    server_infos: Vec<server::ServerInfo>,
}

impl Init {
    pub fn new(server_infos: Vec<server::ServerInfo>) -> Self {
        Self{ server_infos }
    }
}

impl DatabaseMessage<Response, BackendError> for Init {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let server_info_list = replace(&mut self.server_infos, Vec::new());
        
        Box::pin(async move {
            let res = backend.create_vertex(Some(CORE_INFO_ID.clone())).await?;
            match res {
                Some(_) => {
                    // initialize user map
                    backend.set_user_map(HashMap::<String, user::EmuNetUser>::new()).await?;

                    // initialize server list                
                    backend.set_server_info_list(server_info_list).await?;
                            
                    succeed!(Init, (),)
                },
                None => fail!(Init, "database has already been initialized".to_string()),
            }
        })
    }
}