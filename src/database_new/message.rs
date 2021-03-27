use indradb::BulkInsertItem;
use indradb::{Vertex, VertexQuery};
use indradb::{VertexProperty, VertexPropertyQuery};
#[derive(Clone)]
pub enum Request {
    AsyncCreateVertex(Vertex),
    AsyncGetVertices(VertexQuery),
    AsyncDeleteVertices(VertexQuery),
    AsyncGetVertexProperties(VertexPropertyQuery),
    AsyncSetVertexProperties(VertexPropertyQuery, serde_json::Value),
    AsyncBulkInsert(Vec<BulkInsertItem>),
}

#[derive(Clone)]
pub enum Response {
    AsyncCreateVertex(bool),
    AsyncGetVertices(Vec<Vertex>),
    AsyncDeleteVertices(()),
    AsyncGetVertexProperties(Vec<VertexProperty>),
    AsyncSetVertexProperties(()),
    AsyncBulkInsert(()),
}
