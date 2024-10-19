use std::ops::Deref;

use crate::{
    index::{BTreeIndex, TreeIndex},
    node::Node,
    noderef::NodeRef,
};

#[derive(Debug)]
pub struct Tree<R>
where
    R: NodeRef + 'static,
{
    root: Option<R>,
}

impl<R> Tree<R>
where
    R: NodeRef + 'static,
{
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn from_nodes(root: R) -> Self {
        Self { root: Some(root) }
    }

    pub fn root(&self) -> R {
        self.root.as_ref().unwrap().clone()
    }

    pub fn root_ref<'a>(&'a self) -> &'a R {
        self.root.as_ref().unwrap()
    }

    pub fn root_ref_mut<'a>(&'a mut self) -> &'a mut R {
        self.root.as_mut().unwrap()
    }
}

impl<R> Deref for Tree<R>
where
    R: NodeRef + 'static,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.root.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub struct IndexedTree<R>
where
    R: NodeRef + 'static,
{
    tree: Tree<R>,
    index: BTreeIndex<R>,
}

impl<R> IndexedTree<R>
where
    R: NodeRef + 'static,
{
    // Create a new empty indexed tree
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            index: BTreeIndex::new(),
        }
    }

    pub fn from_tree(tree: Tree<R>) -> Self {
        let index = BTreeIndex::from_tree(&tree);

        Self { tree, index }
    }

    pub fn tree(&self) -> &Tree<R> {
        &self.tree
    }

    pub fn index(&self) -> &BTreeIndex<R> {
        &self.index
    }

    pub fn get_node(&self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&R> {
        self.index.get(id)
    }

    pub fn get_node_mut(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<&mut R> {
        self.index.get_mut(id)
    }
}

impl<R> Deref for IndexedTree<R>
where
    R: NodeRef + 'static,
{
    type Target = Tree<R>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
