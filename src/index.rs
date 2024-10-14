use std::collections::BTreeMap;

use crate::{node::NodeRef, NodeId, Tree};

pub trait TreeIndex<'index, Data, Id>: std::fmt::Debug
where
    Id: Clone + std::fmt::Display + 'static,
{
    fn new() -> Self;
    //fn from_node(start: &'index TreeNode<'index, Data, Id>) -> Self;
    fn from_tree(root: &Tree<'index, Data, Id>) -> Self;
    fn insert(&mut self, id: Id, node: NodeRef<'index, Data, Id>);
    fn get(&self, id: &Id) -> Option<&NodeRef<'index, Data, Id>>;
}

#[derive(Debug)]
pub struct BTreeIndex<'index, Data, Id = NodeId>
where
    Id: Default + Clone + std::fmt::Display + 'static,
{
    index: BTreeMap<Id, NodeRef<'index, Data, Id>>,
}

impl<'index, Data, Id> TreeIndex<'index, Data, Id> for BTreeIndex<'index, Data, Id>
where
    Id: std::fmt::Debug + std::fmt::Display + Clone + Ord + Default + 'static,
    Data: std::fmt::Debug + 'static,
{
    fn new() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }

    fn from_tree(tree: &Tree<'index, Data, Id>) -> Self {
        let mut index = Self::new();

        for node in tree.root().iter() {
            index.insert(node.node().id(), node.clone());
        }

        index
    }

    fn insert(&mut self, id: Id, node: NodeRef<'index, Data, Id>) {
        self.index.insert(id, node);
    }

    fn get(&self, id: &Id) -> Option<&NodeRef<'index, Data, Id>> {
        self.index.get(id)
    }
}
