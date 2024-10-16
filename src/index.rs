use std::collections::BTreeMap;

use crate::{node::Node, noderef::NodeRef, Tree};

pub trait TreeIndex<R>
where
    R: NodeRef,
{
    fn new() -> Self;
    fn from_tree(root: &Tree<R>) -> Self;
    fn insert(&mut self, id: <<R as NodeRef>::Inner as Node>::Id, node: R);
    fn get(&self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&R>;
    fn get_mut(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&mut R>;
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

    fn from_tree(tree: &Tree<R>) -> Self {
        let mut index = Self::new();

        for node in tree.root() {
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
}
