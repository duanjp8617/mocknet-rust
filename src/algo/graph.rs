use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::ops::RangeInclusive;

use super::traits::{Max, Min};

struct InMemoryGraph<Nid, Node, Edge> {
    nodes: HashMap<Nid, Node>,
    edges: BTreeMap<(Nid, Nid), Edge>,
    reverse_edges: BTreeMap<(Nid, Nid), ()>,
}

pub(crate) struct UndirectedGraph<Nid, Node, Edge> {
    inner: InMemoryGraph<Nid, Node, Edge>,
}

impl<Nid, Node, Edge> UndirectedGraph<Nid, Node, Edge>
where
    Nid: Ord + Hash + Copy,
{
    pub(crate) fn new(nodes: Vec<(Nid, Node)>, edges: Vec<((Nid, Nid), Edge)>) -> Option<Self> {
        let mut node_map = HashMap::new();
        let _ = nodes
            .into_iter()
            .map(|(nid, node)| match node_map.insert(nid, node) {
                None => Some(()),
                Some(_) => None,
            })
            .collect::<Option<Vec<_>>>()?;

        let mut edge_map = BTreeMap::new();
        let mut reverse_edge_map = BTreeMap::new();
        let _ = edges
            .into_iter()
            .map(|(eid, edge)| {
                // report error for invalid edges
                if !node_map.contains_key(&eid.0) || !node_map.contains_key(&eid.1) {
                    return None;
                }
                // insert edges into the map, report error on edge id collision
                let reverse_eid = (eid.1, eid.0);
                if !edge_map.contains_key(&eid) && !edge_map.contains_key(&reverse_eid) {
                    edge_map.insert(eid, edge);
                    reverse_edge_map.insert(reverse_eid, ());
                }
                Some(())
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Self {
            inner: InMemoryGraph {
                nodes: node_map,
                edges: edge_map,
                reverse_edges: reverse_edge_map,
            },
        })
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = (&Nid, &Node)> {
        self.inner.nodes.iter()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = (&(Nid, Nid), &Edge)> {
        self.inner.edges.iter()
    }

    pub(crate) fn nodes_num(&self) -> usize {
        self.inner.nodes.len()
    }

    pub(crate) fn get_node(&self, nid: Nid) -> Option<&Node> {
        self.inner.nodes.get(&nid)
    }

    pub(crate) fn get_edge(&self, eid: (Nid, Nid)) -> Option<&Edge> {
        self.inner.edges.get(&eid)
    }
}

impl<Nid, Node, Edge> UndirectedGraph<Nid, Node, Edge>
where
    Nid: Min + Max + Ord + Hash + Copy,
{
    pub(crate) fn edges_by_nid<'a>(
        &'a self,
        nid: Nid,
    ) -> (
        impl Iterator<Item = (Nid, Nid)> + 'a,
        impl Iterator<Item = (Nid, Nid)> + 'a,
    ) {
        let outgoing = self.inner.edges.range(RangeInclusive::new(
            (nid, Nid::minimum()),
            (nid, Nid::maximum()),
        ));
        let res1 = outgoing.map(move |((s, d), _)| {
            assert!(*s == nid, "FATAL!");
            (*s, *d)
        });

        let incoming = self.inner.reverse_edges.range(RangeInclusive::new(
            (nid, Nid::minimum()),
            (nid, Nid::maximum()),
        ));
        let res2 = incoming.map(move |((d, s), _)| {
            assert!(*d == nid, "FATAL!");
            (*s, *d)
        });

        (res1, res2)
    }
}
