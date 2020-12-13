use std::future::Future;
use std::pin::Pin;

use super::indradb_backend::IndradbClientBackend;
use super::errors::BackendError;
use crate::emunet::server;

type QueryResult<T> = Result<T, String>;
use Result::Ok as QueryOk;
use Result::Err as QueryFail;

enum Response {
    InitResp(QueryResult<()>),
}

pub trait DatabaseMessage<Response, Error> {
    type RespFut: Future<Output = Result<Response, Error>>;

    fn execute(self, backend: &IndradbClientBackend) -> Self::RespFut;
}

pub struct InitDatabase {
    server_infos: Vec<server::ServerInfo>,
}

impl DatabaseMessage<Response, BackendError> for InitDatabase {
    type RespFut = Pin<Box<dyn Future<Output = Result<Response, BackendError>> + Send + 'static>>;

    fn execute(self, backend: &IndradbClientBackend) -> Self::RespFut {
        Box::pin(async move {
            Ok(Response::InitResp(QueryOk(())))
        })
    }
}