use std::collections::BTreeMap;

use crate::{
    node::TreeNode,
    noderef::{NodeRefId, TreeNodeRef},
    Tree, UniqueGenerator,
};

pub trait TreeIndex<R>
where
    R: TreeNodeRef,
{
    fn new() -> Self;
    fn from_tree<G: UniqueGenerator>(tree: &Tree<R, G>) -> Self
    where
        G: UniqueGenerator<Output = NodeRefId<R>> + 'static;
    fn from_node(node: &R) -> Self;
    fn insert(&mut self, id: <<R as TreeNodeRef>::Inner as TreeNode>::Id, node: R);
    fn get(&self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<&R>;
    fn get_mut(&mut self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<&mut R>;
    fn remove(&mut self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<R>;
    fn get_ids(&self) -> Vec<<<R as TreeNodeRef>::Inner as TreeNode>::Id>;
}

#[derive(Debug)]
pub struct BTreeIndex<R>
where
    R: TreeNodeRef,
{
    index: BTreeMap<<<R as TreeNodeRef>::Inner as TreeNode>::Id, R>,
}

impl<R> TreeIndex<R> for BTreeIndex<R>
where
    R: TreeNodeRef + IntoIterator + Clone,
{
    fn new() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }

    fn from_tree<G: UniqueGenerator>(tree: &Tree<R, G>) -> Self
    where
        G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
    {
        Self::from_node(&tree.root())
    }

    fn from_node(node: &R) -> Self {
        let mut index = Self::new();
        for node in node.clone().into_iter() {
            index.insert(node.node().id().clone(), node.clone());
        }
        index
    }

    fn insert(&mut self, id: <<R as TreeNodeRef>::Inner as TreeNode>::Id, node: R) {
        self.index.insert(id, node);
    }

    fn get(&self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<&R> {
        self.index.get(id)
    }

    fn get_mut(&mut self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<&mut R> {
        self.index.get_mut(id)
    }

    fn remove(&mut self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<R> {
        self.index.remove(id)
    }

    fn get_ids(&self) -> Vec<<<R as TreeNodeRef>::Inner as TreeNode>::Id> {
        self.index.keys().map(|k| *k).collect()
    }
}
