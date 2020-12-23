use indradb::{Vertex, VertexQuery};
use indradb::{VertexProperty, VertexPropertyQuery};

#[derive(Clone)]
pub enum Request {
    AsyncCreateVertex(Vertex),
    AsyncGetVertices(VertexQuery),
    AsyncGetVertexProperties(VertexPropertyQuery),
    AsyncSetVertexProperties(VertexPropertyQuery, serde_json::Value),
}

#[derive(Clone)]
pub enum Response {
    AsyncCreateVertex(bool),
    AsyncGetVertices(Vec<Vertex>),
    AsyncGetVertexProperties(Vec<VertexProperty>),
    AsyncSetVertexProperties(()),
}