use indradb::{Vertex, VertexQuery, VertexPropertyQuery, VertexProperty};

#[derive(Clone)]
pub enum Request {
    Init,
    AsyncCreateVertex(Vertex),
    AsyncGetVertices(VertexQuery),
    AsyncGetVertexProperties(VertexPropertyQuery),
    AsyncSetVertexProperties(VertexPropertyQuery, serde_json::Value),
}

#[derive(Clone)]
pub enum Response {
    Init,
    AsyncCreateVertex(bool),
    AsyncGetVertices(Vec<Vertex>),
    AsyncGetVertexProperties(Vec<VertexProperty>),
    AsyncSetVertexProperties(()),
}