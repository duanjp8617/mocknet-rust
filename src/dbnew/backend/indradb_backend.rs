// An implementation of Indradb storage backend
use std::future::Future;

use capnp_rpc::rpc_twoparty_capnp::Side;
use indradb::{SpecificVertexQuery, VertexQueryExt, VertexQuery};
use indradb::{Vertex, Type};
use uuid::Uuid;
use serde::{de::DeserializeOwned, Serialize};

use crate::dbnew::message_queue::{Queue};
use crate::dbnew::message::{Request, Response};
use super::indradb_util::ClientTransaction;
use crate::dbnew::errors::{BackendError};
use super::CORE_INFO_ID;

pub struct IndradbClientBackend {
    tran_worker: crate::autogen::service::Client,
    disconnector: capnp_rpc::Disconnector<Side>,
}

impl IndradbClientBackend {
    // create a vertex with an optional uuid
    pub async fn create_vertex(&self, id: Option<Uuid>) -> Result<Option<Uuid>, BackendError> {
        let trans = self.tran_worker.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let t = Type::new("t").unwrap();
        let v = match id {
            Some(id) => Vertex::with_id(id, t),
            None => Vertex::with_id(indradb::util::generate_uuid_v1(), t),
        };

        let succeed = ct.async_create_vertex(&v).await?;
        if succeed {
            Ok(Some(v.id))
        }
        else {
            Ok(None)
        }
    }

    // get json property with name `property_name` from vertex with id `vid`
    pub async fn get_vertex_json_value(&self, vid: Uuid, property_name: &str) -> Result<Option<serde_json::Value>, BackendError> {
        let trans = self.tran_worker.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid.clone()).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Ok(None);
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        let mut property_list = ct.async_get_vertex_properties(q).await?;
        if property_list.len() == 0 {
            return Ok(None);
        }

        Ok(Some(property_list.pop().unwrap().value))
    }

    // set json property with name `property_name` for vertex with id `vid`
    pub async fn set_vertex_json_value(&self, vid: Uuid, property_name: &str, json: &serde_json::Value) -> Result<bool, BackendError> {
        let trans = self.tran_worker.transaction_request().send().pipeline.get_transaction();
        let ct = ClientTransaction::new(trans);
        
        let q: VertexQuery = SpecificVertexQuery::single(vid).into();
        let vertex_list = ct.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Ok(false);
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v|{v.id}).collect()).property(property_name);
        ct.async_set_vertex_properties(q, json).await?;
        Ok(true)
    }
}

impl IndradbClientBackend {
    pub fn new(client: crate::autogen::service::Client, disconnector: capnp_rpc::Disconnector<Side>) -> Self {
        Self{
            tran_worker: client, 
            disconnector
        }
    }

    // helper functions:
    pub async fn get_core_property<T: DeserializeOwned>(&self, property: &str) -> Result<T, BackendError> {
        let res = self.get_vertex_json_value(super::CORE_INFO_ID.clone(), property).await?;
        match res {
            Some(jv) => Ok(serde_json::from_value(jv).unwrap()),
            None => panic!("database is not correctly initialized"),
        }
    }

    pub async fn set_core_property<T: Serialize>(&self, property: &str, t: T) -> Result<(), BackendError> {
        let jv = serde_json::to_value(t).unwrap();
        let res = self.set_vertex_json_value(CORE_INFO_ID.clone(), property, &jv).await?;
        if !res {
            panic!("database is not correctly initialized");
        }
        Ok(())
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
                let mut req = msg.try_get_msg().unwrap();
                let resp_result = req.execute(&backend).await;
                let _ = msg.callback(resp_result);
            }
        }
        
        backend.disconnector.await.map_err(|e|{e.into()})
    }
}
