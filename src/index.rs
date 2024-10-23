use std::collections::BTreeMap;

use crate::{node::Node, noderef::NodeRef, Tree, UniqueGenerator};

pub trait TreeIndex<R>
where
    R: NodeRef,
{
    fn new() -> Self;
    fn from_tree<G: UniqueGenerator>(root: &Tree<R, G>) -> Self;
    fn from_node(node: &R) -> Self;
    fn insert(&mut self, id: <<R as NodeRef>::Inner as Node>::Id, node: R);
    fn get(&self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&R>;
    fn get_mut(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&mut R>;
    fn remove(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<R>;
    fn get_ids(&self) -> Vec<<<R as NodeRef>::Inner as Node>::Id>;
}

#[derive(Debug)]
pub struct BTreeIndex<R>
where
    R: NodeRef,
{
    index: BTreeMap<<<R as NodeRef>::Inner as Node>::Id, R>,
}

impl<R> TreeIndex<R> for BTreeIndex<R>
where
    R: NodeRef + IntoIterator + Clone,
{
    fn new() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }

    fn from_tree<G: UniqueGenerator>(tree: &Tree<R, G>) -> Self {
        Self::from_node(&tree.root())
    }

    fn from_node(node: &R) -> Self {
        let mut index = Self::new();
        for node in node.clone().into_iter() {
            index.insert(node.node().id().clone(), node.clone());
        }
        index
    }

    fn insert(&mut self, id: <<R as NodeRef>::Inner as Node>::Id, node: R) {
        self.index.insert(id, node);
    }

    fn get(&self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&R> {
        self.index.get(id)
    }

    fn get_mut(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&mut R> {
        self.index.get_mut(id)
    }

    fn remove(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<R> {
        self.index.remove(id)
    }

    fn get_ids(&self) -> Vec<<<R as NodeRef>::Inner as Node>::Id> {
        self.index.keys().map(|k| *k).collect()
    }
}
