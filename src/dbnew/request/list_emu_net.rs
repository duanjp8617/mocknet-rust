use std::mem::replace;
use std::collections::HashMap;

use crate::emunet::user;
use crate::dbnew::message::{Response, ResponseFuture, DatabaseMessage, Succeed, Fail};
use crate::dbnew::errors::BackendError;
use crate::dbnew::backend::IndradbClientBackend;

use Response::ListEmuNet as Resp;

pub struct ListEmuNet {
    user: String,
}

impl ListEmuNet {
    pub fn new(user: String) -> Self {
        Self{ user }
    }
}


impl DatabaseMessage<Response, BackendError> for ListEmuNet {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let msg = replace(self, ListEmuNet {
            user: String::new(),
        });

        Box::pin(async move {
            let msg = msg;
            
            // get user
            let user_map: HashMap<String, user::EmuNetUser> = backend.get_user_map().await?;
            if !user_map.contains_key(&msg.user) {
                return Ok(Resp(Fail("invalid user name".to_string())));
            }
            let user = user_map.get(&msg.user).unwrap();
            
            Ok(Resp(Succeed(user.get_all_emu_nets())))
        })
    }
}