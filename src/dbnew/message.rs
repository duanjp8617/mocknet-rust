use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use std::mem::replace;

use super::indradb_backend::IndradbClientBackend;
use super::errors::BackendError;
use crate::emunet::server;
use crate::emunet::user;
use super::message_queue;
use super::CORE_INFO_ID;

type QueryResult<T> = Result<T, String>;
use Result::Ok as QueryOk;
use Result::Err as QueryFail;

pub enum Response {
    InitResp(QueryResult<()>),
}

pub type ResponseFuture<'a> = Pin<Box<dyn Future<Output = Result<Response, BackendError>> + 'a>>;
pub trait DatabaseMessage<Response, Error> {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a>;
}

pub type Request = Box<dyn DatabaseMessage<Response, BackendError> + Send + 'static>;

pub struct InitDatabase {
    server_infos: Vec<server::ServerInfo>,
}

impl InitDatabase {
    fn take(&mut self) -> Vec<server::ServerInfo> {
        replace(&mut self.server_infos, Vec::new())
    }
}

impl DatabaseMessage<Response, BackendError> for InitDatabase {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {
        let server_info_list = self.take();
        Box::pin(async move {
            let res = backend.create_vertex(Some(CORE_INFO_ID.clone())).await?;
            match res {
                Some(_) => {
                    // initialize user map
                    backend.set_core_property("user_map", HashMap::<String, user::EmuNetUser>::new()).await?;

                    // initialize server list                
                    backend.set_core_property("server_info_list", server_info_list).await?;
                            
                    Ok(Response::InitResp(QueryOk(())))
                },
                None => Ok(Response::InitResp(QueryFail("database has already been initialized".to_string()))),
            }
        })
    }
}