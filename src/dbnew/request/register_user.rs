use std::mem::replace;
use std::collections::HashMap;

use crate::emunet::user;
use crate::dbnew::message::{Response, ResponseFuture, DatabaseMessage, Succeed, Fail};
use crate::dbnew::errors::BackendError;
use crate::dbnew::backend::IndradbClientBackend;

use Response::RegisterUser as Resp;

pub struct RegisterUser {
    user_name: String,
}

impl RegisterUser {
    pub fn new(user_name: String) -> Self {
        Self{ user_name }
    }
}


impl DatabaseMessage<Response, BackendError> for RegisterUser {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let user_name = replace(&mut self.user_name, String::new());
        
        Box::pin(async move {
            // read current user map
            let mut user_map: HashMap<String, user::EmuNetUser> = backend.get_user_map().await?;
            if user_map.get(&user_name).is_some() {
                return Ok(Resp(Fail("user has already registered".to_string())));
            }

            // register the new user
            let user = user::EmuNetUser::new(&user_name);
            user_map.insert(user_name, user);
            
            // sync update in the db
            backend.set_user_map(user_map).await?;
            
            Ok(Resp(Succeed(())))
        })
    }
}
