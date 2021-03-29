use std::error::Error as StdError;
use std::convert::TryInto;

use indradb::{
    EdgeKey, EdgePropertyQuery, EdgeQuery, SpecificEdgeQuery, SpecificVertexQuery, VertexPropertyQuery, VertexQuery,
};
use indradb_proto as proto;
use mocknet::new_database::*;
use mocknet::new_emunet::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {

    let connector = new_connector("grpc://127.0.0.1:27615").await?;
    let mut cluster = cluster::ClusterInfo::new();
    cluster.add_server_info("192.168.0.1", 15, "djp", "djp").unwrap();
    cluster.add_server_info("192.168.0.2", 15, "djp", "djp").unwrap();
    cluster.add_server_info("192.168.0.3", 15, "djp", "djp").unwrap();
    cluster.add_server_info("192.168.0.4", 15, "djp", "djp").unwrap();
    
    let mut client = connector.connect().await?;
    let res = client.init(cluster).await?;
    match res {
        Ok(_) => {
            println!("successfully initialize the database");
        },
        Err(e) => {
            println!("{}", e);
        }
    }

    Ok(())
}