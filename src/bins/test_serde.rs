use mocknet::algo::in_memory_graph::{*};

fn main() {
    // print_an_address().unwrap();
    println!("mf");
    // let vertexes: Vec<_> = vec!(1,2,3,4,5).into_iter().map(|e| {
    //     (e, 0)
    // }).collect();
    // let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,5), (2,3)).into_iter().map(|e| {
    //     (e, 0)
    // }).collect();

    // let vertexes: Vec<_> = vec!(1,2,3,3,4,5).into_iter().map(|e| {
    //     (e, 0)
    // }).collect();
    // let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,5), (2,3)).into_iter().map(|e| {
    //     (e, 0)
    // }).collect();

    let vertexes: Vec<_> = vec!(1,2,3,4,5).into_iter().map(|e| {
        (e, 0)
    }).collect();
    let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,4), (1,5), (2,3), (7,8)).into_iter().map(|e| {
        (e, 0)
    }).collect();

    let graph: InMemoryGraph<u64, u64, u64> = InMemoryGraph::from_vecs(vertexes, edges).unwrap();
    graph.dump();
}