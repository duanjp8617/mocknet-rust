// An implementation of Indradb storage backend
use std::future::Future;

use capnp_rpc::rpc_twoparty_capnp::Side;
use indradb::{Vertex, VertexQuery};
use indradb::{VertexProperty, VertexPropertyQuery};
use indradb::BulkInsertItem;
use capnp::Error as CapnpError;

use super::indradb_util::ClientTransaction;
use super::indradb_util::converters;
use super::message_queue::Queue;
use super::message::{Request, Response};
use crate::database::errors::BackendError;

pub struct Backend {
    tran_worker: crate::autogen::service::Client,
    disconnector: capnp_rpc::Disconnector<Side>,
}

impl Backend {
    pub fn new(client: crate::autogen::service::Client, disconnector: capnp_rpc::Disconnector<Side>) -> Self {
        Self{
            tran_worker: client, 
            disconnector
        }
    }
}

macro_rules! transaction_wrapper {
    ( $method_name: ident, 
      $( $variable: ident : $t: ty ,)+
      => $rt: ty
    ) => {
        async fn $method_name(&self, $( $variable:$t ,)+) -> Result<$rt, CapnpError> {
            let trans = self.tran_worker.transaction_request().send().pipeline.get_transaction();
            let ct = ClientTransaction::new(trans);
            ct.$method_name( $( $variable ,)+ ).await
        }
    }
}

impl Backend {
    transaction_wrapper!(async_create_vertex, v: &Vertex, => bool);
    transaction_wrapper!(async_get_vertices, q: VertexQuery,  => Vec<Vertex>);
    transaction_wrapper!(async_get_vertex_properties, q: VertexPropertyQuery, => Vec<VertexProperty>);
    transaction_wrapper!(async_set_vertex_properties, q: VertexPropertyQuery, value: &serde_json::Value, => ());

    async fn async_bulk_insert(&self, qs: Vec<BulkInsertItem>) -> Result<(), CapnpError> {
        let mut req = self.tran_worker.bulk_insert_request();
        converters::from_bulk_insert_items(&qs, req.get().init_items(qs.len() as u32)).unwrap();

        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }

    async fn dispatch_request(&self, req: Request) -> Result<Response, BackendError> {
        match req {
            Request::AsyncCreateVertex(v) => {
                Ok(Response::AsyncCreateVertex(self.async_create_vertex(&v).await?))
            },
            Request::AsyncGetVertices(q) => {
                Ok(Response::AsyncGetVertices(self.async_get_vertices(q).await?))
            },
            Request::AsyncGetVertexProperties(q) => {
                Ok(Response::AsyncGetVertexProperties(self.async_get_vertex_properties(q).await?))
            },
            Request::AsyncSetVertexProperties(q, value) => {
                Ok(Response::AsyncSetVertexProperties(self.async_set_vertex_properties(q, &value).await?))
            },
            Request::AsyncBulkInsert(qs) => {
                Ok(Response::AsyncBulkInsert(self.async_bulk_insert(qs).await?))
            }
        }
    }
}

pub fn build_backend_fut(backend: Backend, mut queue: Queue<Request, Response, BackendError>) 
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
