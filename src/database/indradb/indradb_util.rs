// this mod is taken from indradb code base

use crate::autogen;

use capnp::Error as CapnpError;
use serde_json::value::Value as JsonValue;
use uuid::Uuid;

pub mod converters {
    use crate::autogen;
    use capnp::Error as CapnpError;
    use chrono::{TimeZone, Utc};
    use std::fmt::Display;
    use uuid::Uuid;

    pub fn from_bulk_insert_items<'a>(
        items: &[indradb::BulkInsertItem],
        mut builder: capnp::struct_list::Builder<'a, autogen::bulk_insert_item::Owned>,
    ) -> Result<(), CapnpError> {
        for (i, item) in items.iter().enumerate() {
            let builder = builder.reborrow().get(i as u32);

            match item {
                indradb::BulkInsertItem::Vertex(vertex) => {
                    let builder = builder.init_vertex();
                    from_vertex(vertex, builder.get_vertex()?);
                }
                indradb::BulkInsertItem::Edge(edge) => {
                    let builder = builder.init_edge();
                    from_edge_key(edge, builder.get_key()?);
                }
                indradb::BulkInsertItem::VertexProperty(id, name, value) => {
                    let mut builder = builder.init_vertex_property();
                    builder.set_id(id.as_bytes());
                    builder.set_name(name);
                    builder.set_value(&value.to_string());
                }
                indradb::BulkInsertItem::EdgeProperty(key, name, value) => {
                    let mut builder = builder.init_edge_property();
                    builder.set_name(name);
                    builder.set_value(&value.to_string());
                    from_edge_key(key, builder.get_key()?);
                }
            }
        }

        Ok(())
    }

    pub fn from_vertex<'a>(vertex: &indradb::Vertex, mut builder: autogen::vertex::Builder<'a>) {
        builder.set_id(vertex.id.as_bytes());
        builder.set_t(&vertex.t.0);
    }

    pub fn from_edge_query<'a>(q: &indradb::EdgeQuery, builder: autogen::edge_query::Builder<'a>) {
        match q {
            indradb::EdgeQuery::Specific(specific) => {
                let mut builder = builder
                    .init_specific()
                    .init_keys(specific.keys.len() as u32);

                for (i, key) in specific.keys.iter().enumerate() {
                    from_edge_key(key, builder.reborrow().get(i as u32));
                }
            }
            indradb::EdgeQuery::Pipe(pipe) => {
                let mut builder = builder.init_pipe();
                builder.set_direction(from_edge_direction(pipe.direction));

                if let Some(t) = &pipe.t {
                    builder.set_t(&t.0);
                }

                if let Some(high) = pipe.high {
                    builder.set_high(high.timestamp_nanos() as u64);
                }

                if let Some(low) = pipe.low {
                    builder.set_low(low.timestamp_nanos() as u64);
                }

                builder.set_limit(pipe.limit);
                from_vertex_query(&pipe.inner, builder.init_inner());
            }
        }
    }

    pub fn from_edge_key<'a>(key: &indradb::EdgeKey, mut builder: autogen::edge_key::Builder<'a>) {
        builder.set_outbound_id(key.outbound_id.as_bytes());
        builder.set_t(&key.t.0);
        builder.set_inbound_id(key.inbound_id.as_bytes());
    }

    pub fn from_vertex_query<'a>(
        q: &indradb::VertexQuery,
        builder: autogen::vertex_query::Builder<'a>,
    ) {
        match q {
            indradb::VertexQuery::Range(q) => {
                let mut builder = builder.init_range();

                if let Some(start_id) = q.start_id {
                    builder.set_start_id(start_id.as_bytes());
                }

                if let Some(ref t) = q.t {
                    builder.set_t(&t.0);
                }

                builder.set_limit(q.limit);
            }
            indradb::VertexQuery::Specific(q) => {
                let mut builder = builder.init_specific().init_ids(q.ids.len() as u32);

                for (i, id) in q.ids.iter().enumerate() {
                    builder.set(i as u32, id.as_bytes());
                }
            }
            indradb::VertexQuery::Pipe(q) => {
                let mut builder = builder.init_pipe();
                builder.set_direction(from_edge_direction(q.direction));
                builder.set_limit(q.limit);

                if let Some(ref t) = q.t {
                    builder.set_t(&t.0);
                }

                from_edge_query(&q.inner, builder.init_inner());
            }
        }
    }

    pub fn from_edge_direction(direction: indradb::EdgeDirection) -> autogen::EdgeDirection {
        match direction {
            indradb::EdgeDirection::Outbound => autogen::EdgeDirection::Outbound,
            indradb::EdgeDirection::Inbound => autogen::EdgeDirection::Inbound,
        }
    }

    pub fn map_capnp_err<T, E: Display>(result: Result<T, E>) -> Result<T, capnp::Error> {
        result.map_err(|err| capnp::Error::failed(format!("{}", err)))
    }

    pub fn to_vertex<'a>(
        reader: &autogen::vertex::Reader<'a>,
    ) -> Result<indradb::Vertex, CapnpError> {
        let id = map_capnp_err(Uuid::from_slice(reader.get_id()?))?;
        let t = map_capnp_err(indradb::Type::new(reader.get_t()?))?;
        Ok(indradb::Vertex::with_id(id, t))
    }

    pub fn to_edge<'a>(reader: &autogen::edge::Reader<'a>) -> Result<indradb::Edge, CapnpError> {
        let key = to_edge_key(&reader.get_key()?)?;
        let created_datetime = Utc.timestamp(reader.get_created_datetime() as i64, 0);
        Ok(indradb::Edge::new(key, created_datetime))
    }

    pub fn to_edge_key<'a>(
        reader: &autogen::edge_key::Reader<'a>,
    ) -> Result<indradb::EdgeKey, CapnpError> {
        let outbound_id = map_capnp_err(Uuid::from_slice(reader.get_outbound_id()?))?;
        let t = map_capnp_err(indradb::Type::new(reader.get_t()?))?;
        let inbound_id = map_capnp_err(Uuid::from_slice(reader.get_inbound_id()?))?;
        Ok(indradb::EdgeKey::new(outbound_id, t, inbound_id))
    }

    pub fn from_vertex_property_query<'a>(
        q: &indradb::VertexPropertyQuery,
        mut builder: autogen::vertex_property_query::Builder<'a>,
    ) {
        builder.set_name(&q.name);
        from_vertex_query(&q.inner, builder.init_inner());
    }

    pub fn to_vertex_property<'a>(
        reader: &autogen::vertex_property::Reader<'a>,
    ) -> Result<indradb::VertexProperty, CapnpError> {
        let id = map_capnp_err(Uuid::from_slice(reader.get_id()?))?;
        let value = map_capnp_err(serde_json::from_str(reader.get_value()?))?;
        Ok(indradb::VertexProperty::new(id, value))
    }

    pub fn to_vertex_properties<'a>(
        reader: &autogen::vertex_properties::Reader<'a>,
    ) -> Result<indradb::VertexProperties, CapnpError> {
        let vertex = map_capnp_err(to_vertex(&reader.get_vertex()?))?;
        let named_props: Result<Vec<indradb::NamedProperty>, CapnpError> = reader
            .get_props()?
            .into_iter()
            .map(to_named_property)
            .collect();
        Ok(indradb::VertexProperties::new(vertex, named_props?))
    }

    pub fn to_named_property(
        reader: autogen::property::Reader,
    ) -> Result<indradb::NamedProperty, CapnpError> {
        let name = map_capnp_err(reader.get_name())?.to_string();
        let value = map_capnp_err(serde_json::from_str(reader.get_value()?))?;
        Ok(indradb::NamedProperty::new(name, value))
    }

    pub fn from_edge_property_query<'a>(
        q: &indradb::EdgePropertyQuery,
        mut builder: autogen::edge_property_query::Builder<'a>,
    ) {
        builder.set_name(&q.name);
        from_edge_query(&q.inner, builder.init_inner());
    }

    pub fn to_edge_property<'a>(
        reader: &autogen::edge_property::Reader<'a>,
    ) -> Result<indradb::EdgeProperty, CapnpError> {
        let key = to_edge_key(&reader.get_key()?)?;
        let value = map_capnp_err(serde_json::from_str(reader.get_value()?))?;
        Ok(indradb::EdgeProperty::new(key, value))
    }

    pub fn to_edge_properties<'a>(
        reader: &autogen::edge_properties::Reader<'a>,
    ) -> Result<indradb::EdgeProperties, CapnpError> {
        let edge = map_capnp_err(to_edge(&reader.get_edge()?))?;
        let named_props: Result<Vec<indradb::NamedProperty>, CapnpError> = reader
            .get_props()?
            .into_iter()
            .map(to_named_property)
            .collect();
        Ok(indradb::EdgeProperties::new(edge, named_props?))
    }
}

pub struct ClientTransaction {
    trans: autogen::transaction::Client,
}

impl ClientTransaction {
    pub fn new(trans: autogen::transaction::Client) -> Self {
        ClientTransaction { trans }
    }
}

impl ClientTransaction {
    pub async fn async_create_vertex(&self, v: &indradb::Vertex) -> Result<bool, CapnpError> {
        let mut req = self.trans.create_vertex_request();
        converters::from_vertex(v, req.get().init_vertex());
        let res = req.send().promise.await?;
        Ok(res.get()?.get_result())
    }

    pub async fn async_create_vertex_from_type(
        &self,
        t: indradb::Type,
    ) -> Result<Uuid, CapnpError> {
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
        let list: Result<Vec<indradb::Vertex>, CapnpError> = list
            .into_iter()
            .map(|reader| converters::to_vertex(&reader))
            .collect();
        list
    }

    pub async fn async_delete_vertices<Q: Into<indradb::VertexQuery>>(
        &self,
        q: Q,
    ) -> Result<(), CapnpError> {
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

    pub async fn async_get_edges<Q: Into<indradb::EdgeQuery>>(
        &self,
        q: Q,
    ) -> Result<Vec<indradb::Edge>, CapnpError> {
        let mut req = self.trans.get_edges_request();
        converters::from_edge_query(&q.into(), req.get().init_q());

        let res = req.send().promise.await?;
        let list = res.get()?.get_result()?;
        let list: Result<Vec<indradb::Edge>, CapnpError> = list
            .into_iter()
            .map(|reader| converters::to_edge(&reader))
            .collect();
        list
    }

    pub async fn async_delete_edges<Q: Into<indradb::EdgeQuery>>(
        &self,
        q: Q,
    ) -> Result<(), CapnpError> {
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

        req.get()
            .set_direction(converters::from_edge_direction(direction));

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

    pub async fn async_delete_vertex_properties(
        &self,
        q: indradb::VertexPropertyQuery,
    ) -> Result<(), CapnpError> {
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

    pub async fn async_delete_edge_properties(
        &self,
        q: indradb::EdgePropertyQuery,
    ) -> Result<(), CapnpError> {
        let mut req = self.trans.delete_edge_properties_request();
        converters::from_edge_property_query(&q, req.get().init_q());

        let res = req.send().promise.await?;
        res.get()?;
        Ok(())
    }
}
