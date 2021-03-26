use std::collections::HashMap;

use indradb::BulkInsertItem;
use indradb::Type;
use indradb::{RangeVertexQuery, SpecificVertexQuery, VertexQueryExt};
use indradb::{Vertex, VertexQuery};
use indradb::{VertexProperty, VertexPropertyQuery};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use super::message::{Request, Response};
use super::message_queue;
use crate::database_new::errors::BackendError;
use crate::database_new::CORE_INFO_ID;
use crate::emunet::{server, user};

pub struct Frontend {
    sender: message_queue::Sender<Request, Response, BackendError>,
}

impl Frontend {
    pub fn new(sender: message_queue::Sender<Request, Response, BackendError>) -> Self {
        Self { sender }
    }
}

impl Clone for Frontend {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

macro_rules! request_wrapper {
    ( $method_name: ident,
      $request_name: ident,
      $( $variable: ident : $t: ty ,)+
      => $rt: ty
    ) => {
        async fn $method_name(&self, $( $variable:$t ,)+) -> Result<$rt, BackendError> {
            let res = self.sender.send(Request::$request_name( $( $variable ,)+ )).await?;
            match res {
                Response::$request_name(res) => Ok(res),
                _ => panic!("invalid response!")
            }
        }
    }
}

impl Frontend {
    request_wrapper!(async_create_vertex, AsyncCreateVertex, v: Vertex, => bool);
    request_wrapper!(async_get_vertices, AsyncGetVertices, q: VertexQuery,  => Vec<Vertex>);
    request_wrapper!(async_delete_vertices, AsyncDeleteVertices, q: VertexQuery, => ());
    request_wrapper!(async_get_vertex_properties, AsyncGetVertexProperties, q: VertexPropertyQuery, => Vec<VertexProperty>);
    request_wrapper!(async_set_vertex_properties, AsyncSetVertexProperties, q: VertexPropertyQuery, value: serde_json::Value, => ());
}

impl Frontend {
    // create a vertex with an optional uuid
    pub async fn create_vertex(&self, id: Option<Uuid>) -> Result<Option<Uuid>, BackendError> {
        let t = Type::new("t").unwrap();
        let v = match id {
            Some(id) => Vertex::with_id(id, t),
            None => Vertex::with_id(indradb::util::generate_uuid_v1(), t),
        };

        let succeed = self.async_create_vertex(v.clone()).await?;
        if succeed {
            Ok(Some(v.id))
        } else {
            Ok(None)
        }
    }

    // get json property with name `property_name` from vertex with id `vid`
    pub async fn get_vertex_json_value(
        &self,
        vid: Uuid,
        property_name: &str,
    ) -> Result<Option<serde_json::Value>, BackendError> {
        let q: VertexQuery = SpecificVertexQuery::single(vid.clone()).into();
        let vertex_list = self.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Ok(None);
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v| v.id).collect())
            .property(property_name);
        let mut property_list = self.async_get_vertex_properties(q).await?;
        if property_list.len() == 0 {
            return Ok(None);
        }

        Ok(Some(property_list.pop().unwrap().value))
    }

    // set json property with name `property_name` for vertex with id `vid`
    pub async fn set_vertex_json_value(
        &self,
        vid: Uuid,
        property_name: &str,
        json: serde_json::Value,
    ) -> Result<bool, BackendError> {
        let q: VertexQuery = SpecificVertexQuery::single(vid).into();
        let vertex_list = self.async_get_vertices(q).await?;
        if vertex_list.len() == 0 {
            return Ok(false);
        }

        let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v| v.id).collect())
            .property(property_name);
        self.async_set_vertex_properties(q, json).await?;
        Ok(true)
    }

    // perform a bulk insertion
    pub async fn bulk_insert(&self, qs: Vec<BulkInsertItem>) -> Result<(), BackendError> {
        let res = self.sender.send(Request::AsyncBulkInsert(qs)).await?;
        match res {
            Response::AsyncBulkInsert(()) => Ok(()),
            _ => panic!("invalid response!"),
        }
    }

    // get all the vertexes
    pub async fn get_vertex_properties(
        &self,
        q: RangeVertexQuery,
    ) -> Result<Vec<serde_json::Value>, BackendError> {
        let q = q.property("default".to_string());
        self.async_get_vertex_properties(q).await.map(|vp| {
            vp.into_iter().fold(Vec::new(), |mut vec, vp| {
                vec.push(vp.value);
                vec
            })
        })
    }

    pub async fn delete_vertex(&self, vid: Uuid) -> Result<(), BackendError> {
        let q: VertexQuery = SpecificVertexQuery::single(vid).into();
        self.async_delete_vertices(q).await
    }
}

impl Frontend {
    // helpers function:
    async fn get_core_property<T: DeserializeOwned>(
        &self,
        property: &str,
    ) -> Result<T, BackendError> {
        let res = self
            .get_vertex_json_value(CORE_INFO_ID.clone(), property)
            .await?;
        match res {
            Some(jv) => Ok(serde_json::from_value(jv).unwrap()),
            None => panic!("database is not correctly initialized"),
        }
    }

    async fn set_core_property<T: Serialize>(
        &self,
        property: &str,
        t: T,
    ) -> Result<(), BackendError> {
        let jv = serde_json::to_value(t).unwrap();
        let res = self
            .set_vertex_json_value(CORE_INFO_ID.clone(), property, jv)
            .await?;
        if !res {
            panic!("database is not correctly initialized");
        }
        Ok(())
    }
}

impl Frontend {
    // public interfaces for accessing core information
    pub async fn get_server_info_list(&self) -> Result<Vec<server::ServerInfo>, BackendError> {
        self.get_core_property("server_info_list").await
    }

    pub async fn set_server_info_list(
        &self,
        server_info_list: Vec<server::ServerInfo>,
    ) -> Result<(), BackendError> {
        self.set_core_property("server_info_list", server_info_list)
            .await
    }

    pub async fn get_user_map(&self) -> Result<HashMap<String, user::EmuNetUser>, BackendError> {
        self.get_core_property("user_map").await
    }

    pub async fn set_user_map(
        &self,
        user_map: HashMap<String, user::EmuNetUser>,
    ) -> Result<(), BackendError> {
        self.set_core_property("user_map", user_map).await
    }
}
