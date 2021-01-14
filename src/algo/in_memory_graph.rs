use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::fmt;
pub mod affinity_partition;
use std::fmt::{Display, Debug};
use affinity_partition::{Edge_py, Node};

use serde::de::DeserializeOwned;

use super::traits::{PartitionBin, Partition, Weighted};

type Result<T> = std::result::Result<T, String>;

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
    pub fn from_vecs(vertexes: Vec<(Vid, Vertex)>, edges: Vec<(EdgeId<Vid>, Edge)>) -> Result<Self> {
        if vertexes.len() == 0 {
            return Err("no vertexes".to_string());
        }

        let mut vertex_map = HashMap::new();
        let insert_res: Result<Vec<_>> = vertexes.into_iter().map(|(vid, v)| {
            // insert vertex into map, report error on id collision
            match vertex_map.insert(vid, v) {
                None => Ok(()),
                Some(_) => Err("repeated vertex id".to_string()),
            }
        }).collect();
        let _ = insert_res?;

        let mut edge_map = BTreeMap::new();
        let mut reverse_edge_map = BTreeMap::new();
        let insert_res: Result<Vec<_>> = edges.into_iter().map(|(eid, e)| {
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

    pub fn into(self) -> (Vec<Vertex>, Vec<Edge>) {
        let vertexes = self.vertexes.into_iter().map(|(_, vertex)| {vertex});
        let edges = self.edges.into_iter().map(|(_, edge)| {edge});
        (vertexes.collect(), edges.collect())
    }

    pub fn size(&self) -> usize {
        self.vertexes.len()
    }
}

impl<'a, Vid, Vertex, Edge, T, I> Partition<'a, T, I> for InMemoryGraph<Vid, Vertex, Edge>
where
    T: 'a + PartitionBin<Size = u32>,
    I: Iterator<Item = &'a mut T>,
    Vid: Eq + Hash + Clone + Copy + Debug + Display,
    Vertex: Weighted,
    Edge: Weighted
{
    type ItemId = Vid;

    /*fn partition(&self, mut bins: I) -> Result<HashMap<Vid, <T as PartitionBin>::BinId>> 
    {
        // acquire an iterator of vids
        let mut vids = self.vertexes.keys();
        // retrieve the first bin
        let mut curr_bin = bins.next().ok_or("not enough resource".to_string())?;
        // initialize the resulting HashMap
        let mut res = HashMap::new();
        
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
    }*/

    fn partition(&self, mut bins: I, partition_number: usize, rank_swap: bool, rank_swap_mode: String, cluster_threshold: usize) -> Result::<HashMap<Vid, T::BinId>>{
        /*
            // cluster_threshold: usize, the threshold that stop affinity clustering, or stop at most 5 rounds.
            // rank_swap_mode: String ("near" or "rank"). "near" mode pair two partitions nearby together, which can approximately minimize edges cut off,
               while "rank" mode pair the largest interval with the smallest one, may raise edges cut off.
            // rank_swap: weather to implement rank_swap algorithm.
        */
            let mut edges = Vec::<Edge_py<Node<Vid>>>::new();
            
            for ((start, end), edge) in self.edges.iter() {
                let start_weight = self.vertexes.get(start).unwrap().get_weight();
                let end_weight = self.vertexes.get(end).unwrap().get_weight();
                
                edges.push(Edge_py {
                    start: Node {
                        name: start.clone(),
                        weight: start_weight
                    },
                    end: Node {
                        name: end.clone(),
                        weight: end_weight
                    },
                    weight: edge.get_weight()
                });
    
                edges.push(Edge_py {
                    start: Node {
                        name: end.clone(),
                        weight: end_weight
                    },
                    end: Node {
                        name: start.clone(),
                        weight: start_weight
                    },
                    weight: edge.get_weight()
                });
            }
    
            let mut res = HashMap::new();
            let clusters = affinity_partition::partition(edges, partition_number, rank_swap, rank_swap_mode, cluster_threshold);
            let mut curr_bin = bins.next().ok_or("not enough resource".to_string())?;
            for i in 0..clusters.len() {
                for j in 0..clusters[i].len() {
                    res.insert(clusters[i][j].name, curr_bin.bin_id());
                }
                if let Some(new_bin) = bins.next() {
                    curr_bin = new_bin;
                }
                else {
                    return Err("not enough resource".to_string());
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