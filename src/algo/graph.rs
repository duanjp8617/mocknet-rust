use std::collections::{BTreeMap, HashMap};

type VertexType = i32;
type EdgeType = i32;

pub struct InMemoryGraph {
    vertexes: HashMap<u64, VertexType>,
    edges: BTreeMap<(u64, u64), EdgeType>, // outgoing->incoming,
    reverse_edges: BTreeMap<(u64, u64), ()> // incoming <- outgoing
}

// TODO: update the interfaces, now this is quite dum.
impl InMemoryGraph {
    pub fn new() -> Self {
        Self {
            vertexes: HashMap::new(),
            edges: BTreeMap::new(),
            reverse_edges: BTreeMap::new(),
        }
    }

    pub fn add_vertexes_from_json(&mut self, jv: serde_json::Value) -> Result<bool, i32> {
        let vec: Vec<(u64, VertexType)> = serde_json::from_value(jv).unwrap();
        for (vid, v) in vec.into_iter() {
            if self.vertexes.contains_key(&vid) {
                panic!("repeated vertex id");
            }
            else {
                self.vertexes.insert(vid, v).unwrap();
            }            
        }
        Ok(true)
    }

    pub fn add_edges_from_json(&mut self, jv: serde_json::Value) -> Result<bool, i32> {
        let vec: Vec<((u64, u64), EdgeType)> = serde_json::from_value(jv).unwrap();
        for ((outgoing_id, incoming_id), e) in vec.into_iter() {
            if !self.vertexes.contains_key(&outgoing_id) {
                panic!("vertex {} not exist", &outgoing_id);
            }
            if !self.vertexes.contains_key(&incoming_id) {
                panic!("vertex {} not exist", &incoming_id);
            }
            if self.edges.contains_key(&(outgoing_id, incoming_id)) || self.edges.contains_key(&(incoming_id, outgoing_id)) {
                panic!("edge ({}, {}) already exists", outgoing_id, incoming_id);
            }

            self.edges.insert((outgoing_id, incoming_id), e).unwrap();
            self.reverse_edges.insert((incoming_id, outgoing_id), ()).unwrap();
        }
        Ok(true)
    }

    pub fn partition(&self, mut server_list: Vec<(u32, u32)>) -> Option<HashMap<u64, u32>> {
        let mut res = HashMap::new();
        for vid in self.vertexes.keys() {
            res.insert(vid, server_list[0].0);
            server_list.get_mut(0).unwrap().1 -= 1;
            if server_list[0].1 == 0 {
                server_list.pop().unwrap();
            }
        }
        Some(res)
    }
}