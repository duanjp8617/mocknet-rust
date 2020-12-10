// 
//
// 
// 
// 
// 
// 
// 
// 
// 


// struct InMemoryGraph {

// }

// impl InMemoryGraph {
//     // add vertexes to in memory graph using the input json string
//     fn create_vertexes_from_json() {
//         unimplemented!()
//     }

//     // add edges to the inmemory graph using the input json string
//     fn create_edges_from_json() {
//         unimplemented!()
//     }

//     // partition the graph using the algorithm, and the server pool,
//     // return a HashMap, containing vertex id to server mapping
//     fn partition() {
//         unimplemented!()
//     }

//     // sync with the database, how to do it?
//     // 1. create a vertex in the indradb
//     // 2. assign the vertex property to the indradb vertex
//     // 3. add a vertex id to vertex uuid mapping
//     // 4. create an edge in the indradb.
//     // 5. assign the edge property to the indradb edge
//     // 6. add the edge id to edge uuid mapping
//     fn sync_with_db() {
//         unimplemented!()
//     }
// }
// pub type Result<T> = std::result::Result<T, &'static str>;

pub mod in_memory_graph;

mod traits;
pub use traits::PartitionBin;