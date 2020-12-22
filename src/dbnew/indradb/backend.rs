// An implementation of Indradb storage backend
use std::future::Future;
use std::collections::HashMap;

use capnp_rpc::rpc_twoparty_capnp::Side;
use indradb::{SpecificVertexQuery, VertexQueryExt, VertexQuery};
use indradb::{Vertex, Type};
use uuid::Uuid;
use serde::{de::DeserializeOwned, Serialize};

// use crate::database::message_queue::{Queue};
// use crate::database::message::{Request, Response};
// use super::indradb_util::ClientTransaction;
// use crate::database::errors::{BackendError};
// use crate::database::CORE_INFO_ID;
// use crate::emunet::{server, user};

use super::message_queue::Queue;
use super::message::{Request, Response};

use crate::dbnew::errors::BackendError;

pub struct IndradbClientBackend {
    tran_worker: crate::autogen::service::Client,
    disconnector: capnp_rpc::Disconnector<Side>,
}

impl IndradbClientBackend {
    pub fn new(client: crate::autogen::service::Client, disconnector: capnp_rpc::Disconnector<Side>) -> Self {
        Self{
            tran_worker: client, 
            disconnector
        }
    }
}

impl IndradbClientBackend {
    async fn dispatch_request(&self, req: Request) -> Result<Response, BackendError> {
        match req {
            Request::Init => {
                Ok(Response::Init)
            },
        }
    }
}

pub fn build_backend_fut(backend: IndradbClientBackend, mut queue: Queue<Request, Response, BackendError>) 
    -> impl Future<Output = Result<(), BackendError>> + 'static 
{
    fn drain_queue(mut queue: Queue<Request, Response, BackendError>) {
        queue.close();
        while let Ok(_) = queue.try_recv() {}
    }
    
    async move {        
        while let Some(mut msg) = queue.recv().await {
            if msg.is_close_msg() {
                drain_queue(queue);
                break;
            }
            else {
                let req = msg.try_get_msg().unwrap();
                let resp_result = backend.dispatch_request(req).await;
                let _ = msg.callback(resp_result);
            }
        }
        
        backend.disconnector.await.map_err(|e|{e.into()})
    }
}
