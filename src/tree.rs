use std::{collections::HashSet, ops::Deref};

use tracing::debug;

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

    pub fn index(self) -> IndexedTree<R> {
        IndexedTree::from_tree(self)
    }

    pub fn remove_node(&mut self, node: &R) {
        let node_id = node.node().id().clone();
        debug!("Removing node id {node_id}");

        let mut index = None;

        // Remove the node from the parents children vec
        if let Some(parent) = node.clone().node().parent() {
            if let Some(children) = parent.node().children() {
                for child in (&*children).iter().enumerate() {
                    if *child.1.node().id() == node_id {
                        debug!("Found child node at index {}", child.0);
                        // Found index of node to remove
                        index = Some(child.0);
                        //parent.node_mut().remove_child_index(child.0);
                    }
                }
            }
        }

        if let Some(index) = index {
            node.clone()
                .node_mut()
                .parent_mut()
                .unwrap()
                .node_mut()
                .remove_child_index(index);
        }
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
    leaves: Vec<R>,
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
            leaves: Vec::new(),
            index: BTreeIndex::new(),
        }
    }

    pub fn from_tree(tree: Tree<R>) -> Self {
        let index = BTreeIndex::from_tree(&tree);

        let mut leaves = Vec::new();

        // Find all leaves
        for node in tree.root() {
            if node.node().children().is_none() {
                leaves.push(node.clone())
            }
        }

        Self {
            tree,
            index,
            leaves,
        }
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

    pub fn remove_node(&mut self, node: &R) -> Option<()> {
        let node_id = node.node().id().clone();

        // Remove the node from the tree
        self.tree.remove_node(node);

        let mut remove_ids: HashSet<<<R as NodeRef>::Inner as Node>::Id> = HashSet::from([node_id]);

        // Remove node and descendents from the index
        for node in node.clone().into_iter() {
            remove_ids.insert(node.node().id().clone());
        }

        for id in remove_ids {
            // Remove from the index
            let _removed = self.index.remove(&id)?;

            // Remove from leaves
            self.leaves.retain(|node| *node.node().id() != id);
        }

        Some(())
    }

    pub fn remove_node_id(&mut self, id: &<<R as NodeRef>::Inner as Node>::Id) -> Option<()> {
        debug!("Removing node ID {id}");
        let node = self.get_node(id)?.clone();
        self.remove_node(&node)
    }

    pub fn leaves<'b>(&'b self) -> &'b Vec<R> {
        &self.leaves
    }

    pub fn reindex(&mut self) {
        if let Some(root) = &self.root {
            self.index = BTreeIndex::from_node(root);
        }

        let mut leaves = Vec::new();
        // Find all leaves
        for node in self.root() {
            if node.node().children().is_none() {
                leaves.push(node.clone())
            }
        }
        self.leaves = leaves;
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
