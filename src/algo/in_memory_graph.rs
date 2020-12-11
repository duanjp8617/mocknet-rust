use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::fmt;

use serde::{de::DeserializeOwned, Serialize};

use super::traits::PartitionBin;

pub type Result<T> = std::result::Result<T, String>;

// outgoing -> incoming
pub type EdgeId<T> = (T, T); 

// incoming <- outgoing
type ReverseEdgeId<T> = (T, T); 
fn reverse_edge_id<T: Clone>(edge_id: &EdgeId<T>) -> ReverseEdgeId<T> {
    (edge_id.1.clone(), edge_id.0.clone())
}

pub struct InMemoryGraph<Vid, Vertex, Edge> {
    vertexes: HashMap<Vid, Vertex>,
    edges: BTreeMap<EdgeId<Vid>, Edge>, 
    reverse_edges: BTreeMap<ReverseEdgeId<Vid>, ()> 
}

// // TODO: update the interfaces, now this is quite dum.
impl<Vid, Vertex, Edge> InMemoryGraph<Vid, Vertex, Edge>
where
    Vid: Eq + Ord + Hash + DeserializeOwned + Clone,
    Vertex: DeserializeOwned,
    Edge: DeserializeOwned

{
    pub fn from_jsons(vertexes_json: serde_json::Value, edges_json: serde_json::Value) -> Result<Self> {
        let mut vertex_map = HashMap::new();
        let vertex_vec: Vec<(Vid, Vertex)> = serde_json::from_value(vertexes_json).map_err(|e|{
            format!("fail to convert vertex json message: {}", &e)
        })?;
        let insert_res: Result<Vec<_>> = vertex_vec.into_iter().map(|(vid, v)| {
            // insert vertex into map, report error on id collision
            match vertex_map.insert(vid, v) {
                None => Ok(()),
                Some(_) => Err("repeated vertex id".to_string()),
            }
        }).collect();
        let _ = insert_res?;

        let mut edge_map = BTreeMap::new();
        let mut reverse_edge_map = BTreeMap::new();
        let edge_vec: Vec<(EdgeId<Vid>, Edge)> = serde_json::from_value(edges_json).map_err(|e| {
            format!("fail to convert edge json message: {}", &e)
        })?;
        let insert_res: Result<Vec<_>> = edge_vec.into_iter().map(|(eid, e)| {
            // report error for invalid edges
            if !vertex_map.contains_key(&eid.0) || !vertex_map.contains_key(&eid.1) {
                return Err("edge is not connected to a valid vertex".to_string());
            }
            // insert edges into the map, report error on edge id collision
            let reverse_eid = reverse_edge_id(&eid);
            if edge_map.contains_key(&eid) || edge_map.contains_key(&reverse_eid) {
                Err("repreated edge id".to_string())
            }
            else {
                edge_map.insert(eid, e);
                reverse_edge_map.insert(reverse_eid, ());
                Ok(())
            }
        }).collect();
        let _ = insert_res?;

        Ok(Self {
            vertexes: vertex_map,
            edges: edge_map,
            reverse_edges: reverse_edge_map,
        })
    }
}

impl<Vid, Vertex, Edge> InMemoryGraph<Vid, Vertex, Edge> 
where 
    Vid: Eq + Hash + Clone
{
    pub fn partition<'a, I, T>(&self, mut bins: I) -> Result<HashMap<Vid, <T as PartitionBin>::Id>> 
    where
        T: 'a + PartitionBin<Size = u32>,
        I: Iterator<Item = &'a mut T>
    {
        // acquire an iterator of vids
        let mut vids = self.vertexes.keys();
        // retrieve the first bin
        let mut curr_bin = bins.next().ok_or("not enough resource".to_string())?;
        // initialize the resulting HashMap
        let mut res = HashMap::new();
        println!("wtf");
        
        // iterate through all the vids and make assignment
        while let Some(vid) = vids.next() {
            if curr_bin.fill(1) {
                res.insert(vid.clone(), curr_bin.bin_id());
            }
            else {
                if let Some(new_bin) = bins.next() {
                    curr_bin = new_bin;
                }
                else {
                    return Err("not enough resource".to_string());
                }
            }
        }

        Ok(res)
    }
}

impl<Vid, Vertex, Edge> InMemoryGraph<Vid, Vertex, Edge>
where
    Vid: fmt::Debug,
    Vertex: fmt::Debug,
    Edge: fmt::Debug,
{
    pub fn dump(&self) {
        println!("{:?}", self.vertexes);
        println!("{:?}", self.edges);
        println!("{:?}", self.reverse_edges);
    }
}