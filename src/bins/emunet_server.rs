use std::error::Error as StdError;
use std::convert::TryInto;

use indradb::{
    EdgeKey, EdgePropertyQuery, EdgeQuery, SpecificEdgeQuery, SpecificVertexQuery, VertexPropertyQuery, VertexQuery,
};
use indradb_proto as proto;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {
    let mut client = proto::Client::new(String::from("grpc://127.0.0.1:27615").try_into().unwrap()).await.unwrap();
    let jh = tokio::spawn(async move {
        let client = client;
    });
    jh.await.unwrap();
    Ok(())
}