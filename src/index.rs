use std::collections::BTreeMap;

use crate::{
    node::{NodeRef, TreeNode},
    NodeId, Tree,
};

pub trait Index<'index, Data, Id>: std::fmt::Debug {
    //fn new() -> Self;
    fn from_node(start: &'index TreeNode<'index, Data, Id>) -> Self;
    fn from_tree(root: &Tree<'index, Data, Id>) -> Self;
    fn insert(&mut self, id: Id, node: NodeRef<'index, Data, Id>);
}

#[derive(Debug)]
pub struct BTreeIndex<'index, Data, Id = NodeId>
where
    Data: Default,
    Id: Default,
{
    index: BTreeMap<Id, NodeRef<'index, Data, Id>>,
    //index: BTreeMap<Id, &'index TreeNode<'index, Data, Id>>,
}

impl<'index, Data, Id> Index<'index, Data, Id> for BTreeIndex<'index, Data, Id>
where
    Id: std::fmt::Debug + Clone + Ord + Default + 'static,
    Data: std::fmt::Debug + Default + Clone + 'static,
{
    fn from_node(start: &'index TreeNode<'index, Data, Id>) -> Self {
        let mut index = BTreeMap::new();

        /*
        for node in start {
            index.insert(node.id(), node);
        }
        */

        Self { index }
    }

    fn from_tree(tree: &Tree<'index, Data, Id>) -> Self {
        let mut index = BTreeMap::new();

        for node_ref in tree.root() {
            index.insert(node_ref.node().id(), node_ref.clone());
        }

        Self { index }
    }

    fn insert(&mut self, id: Id, node: NodeRef<'index, Data, Id>) {
        self.index.insert(id, node);
    }
}
