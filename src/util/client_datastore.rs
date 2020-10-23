use crate::autogen;
use super::converters;

use capnp::Error as CapnpError;
use uuid::Uuid;
use serde_json::value::Value as JsonValue;

pub struct ClientTransaction {
    trans: autogen::transaction::Client,
}

impl ClientTransaction {
    pub fn new(trans: autogen::transaction::Client) -> Self {
        ClientTransaction {
            trans,
        }
    }
}

impl ClientTransaction {
    pub async fn async_create_vertex(&self, v: &indradb::Vertex) -> Result<bool, CapnpError> {
        let mut req = self.trans.create_vertex_request();
        converters::from_vertex(v, req.get().init_vertex());
        let res = req.send().promise.await?;
        Ok(res.get()?.get_result())
    }

    pub async fn async_create_vertex_from_type(&self, t: indradb::Type) -> Result<Uuid, CapnpError> {
        let mut req = self.trans.create_vertex_from_type_request();
        req.get().set_t(&t.0);
        let res = req.send().promise.await?;
        let bytes = res.get()?.get_result()?;
        Ok(Uuid::from_slice(bytes).unwrap())
    }

    pub async fn async_get_vertices<Q: Into<indradb::VertexQuery>>(
        &self,
        q: Q,
    ) -> Result<Vec<indradb::Vertex>, CapnpError> {
        let mut req = self.trans.get_vertices_request();
        converters::from_vertex_query(&q.into(), req.get().init_q());
        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::Vertex>, CapnpError> =
            list.into_iter().map(|reader| converters::to_vertex(&reader)).collect();
        list
    }

    pub async fn async_delete_vertices<Q: Into<indradb::VertexQuery>>(&self, q: Q) -> Result<(), CapnpError> {
        let mut req = self.trans.delete_vertices_request();
        converters::from_vertex_query(&q.into(), req.get().init_q());
        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }

    pub async fn async_get_vertex_count(&self) -> Result<u64, CapnpError> {
        let req = self.trans.get_vertex_count_request();
        let res = req.send().promise.await?;
        Ok(res.get()?.get_result())
    }

    pub async fn async_create_edge(&self, e: &indradb::EdgeKey) -> Result<bool, CapnpError> {        
        let mut req = self.trans.create_edge_request();
        converters::from_edge_key(e, req.get().init_key());
        let res = req.send().promise.await?;
        Ok(res.get()?.get_result())
    }

    pub async fn async_get_edges<Q: Into<indradb::EdgeQuery>>(&self, q: Q) -> Result<Vec<indradb::Edge>, CapnpError> {        
        let mut req = self.trans.get_edges_request();
        converters::from_edge_query(&q.into(), req.get().init_q());

        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::Edge>, CapnpError> =
            list.into_iter().map(|reader| converters::to_edge(&reader)).collect();
        list
    }

    pub async fn async_delete_edges<Q: Into<indradb::EdgeQuery>>(&self, q: Q) -> Result<(), CapnpError> {        
        let mut req = self.trans.delete_edges_request();
        converters::from_edge_query(&q.into(), req.get().init_q());
        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }

    pub async fn async_get_edge_count(
        &self,
        id: Uuid,
        t: Option<&indradb::Type>,
        direction: indradb::EdgeDirection,
    ) -> Result<u64, CapnpError> {        
        let mut req = self.trans.get_edge_count_request();
        req.get().set_id(id.as_bytes());

        if let Some(t) = t {
            req.get().set_t(&t.0);
        }

        req.get().set_direction(converters::from_edge_direction(direction));

        let res = req.send().promise.await?;
        Ok(res.get()?.get_result())
    }

    pub async fn async_get_vertex_properties(
        &self,
        q: indradb::VertexPropertyQuery,
    ) -> Result<Vec<indradb::VertexProperty>, CapnpError> {        
        let mut req = self.trans.get_vertex_properties_request();
        converters::from_vertex_property_query(&q, req.get().init_q());

        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::VertexProperty>, CapnpError> = list
            .into_iter()
            .map(|reader| converters::to_vertex_property(&reader))
            .collect();
        list
    }

    pub async fn async_get_all_vertex_properties<Q: Into<indradb::VertexQuery>>(
        &self,
        q: Q,
    ) -> Result<Vec<indradb::VertexProperties>, CapnpError> {        
        let mut req = self.trans.get_all_vertex_properties_request();
        converters::from_vertex_query(&q.into(), req.get().init_q());

        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::VertexProperties>, CapnpError> = list
            .into_iter()
            .map(|reader| converters::to_vertex_properties(&reader))
            .collect();
        list
    }

    pub async fn async_set_vertex_properties(
        &self,
        q: indradb::VertexPropertyQuery,
        value: &JsonValue,
    ) -> Result<(), CapnpError> {
        let mut req = self.trans.set_vertex_properties_request();
        converters::from_vertex_property_query(&q, req.get().init_q());
        req.get().set_value(&value.to_string());

        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }

    pub async fn async_delete_vertex_properties(&self, q: indradb::VertexPropertyQuery) -> Result<(), CapnpError> {
        let mut req = self.trans.delete_vertex_properties_request();
        converters::from_vertex_property_query(&q, req.get().init_q());

        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }

    pub async fn async_get_edge_properties(
        &self,
        q: indradb::EdgePropertyQuery,
    ) -> Result<Vec<indradb::EdgeProperty>, CapnpError> {
        let mut req = self.trans.get_edge_properties_request();
        converters::from_edge_property_query(&q, req.get().init_q());

        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::EdgeProperty>, CapnpError> = list
            .into_iter()
            .map(|reader| converters::to_edge_property(&reader))
            .collect();
        list
    }

    pub async fn async_get_all_edge_properties<Q: Into<indradb::EdgeQuery>>(
        &self,
        q: Q,
    ) -> Result<Vec<indradb::EdgeProperties>, CapnpError> {
        let mut req = self.trans.get_all_edge_properties_request();
        converters::from_edge_query(&q.into(), req.get().init_q());

        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::EdgeProperties>, CapnpError> = list
            .into_iter()
            .map(|reader| converters::to_edge_properties(&reader))
            .collect();
        list
    }

    pub async fn async_set_edge_properties(
        &self,
        q: indradb::EdgePropertyQuery,
        value: &JsonValue,
    ) -> Result<(), CapnpError> {
        let mut req = self.trans.set_edge_properties_request();
        converters::from_edge_property_query(&q, req.get().init_q());
        req.get().set_value(&value.to_string());

        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }

    pub async fn async_delete_edge_properties(&self, q: indradb::EdgePropertyQuery) -> Result<(), CapnpError> {
        let mut req = self.trans.delete_edge_properties_request();
        converters::from_edge_property_query(&q, req.get().init_q());

        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }
}