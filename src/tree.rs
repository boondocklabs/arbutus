use std::{
    collections::HashSet,
    hash::{Hash as _, Hasher},
    ops::{Deref, DerefMut},
};

use tracing::{debug, error, warn};
use xxhash_rust::xxh64::Xxh64;

use crate::{
    index::{BTreeIndex, TreeIndex},
    node::TreeNode,
    noderef::{NodeRefId, TreeNodeRef},
    UniqueGenerator,
};

use crate::node::internal::NodeInternal as _;

pub struct Tree<R, G = crate::IdGenerator>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    // Root node of this tree
    root: Option<R>,

    // Unique ID Generator
    idgen: Option<G>,
}

impl<R, G> std::fmt::Debug for Tree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field(
                "subtree_hash",
                &format_args!("0x{:X}", self.root().node().get_subtree_hash()),
            )
            .field("depth", &self.depth())
            .field("width", &self.width())
            .finish()
    }
}

impl<R, G> Tree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    pub fn new() -> Self {
        Self {
            root: None,
            idgen: None,
        }
    }

    pub fn generator(&self) -> &G {
        self.idgen.as_ref().unwrap()
    }

    /// Allocate a new node ID
    pub fn generate_id(&self) -> G::Output {
        self.idgen
            .as_ref()
            .expect("ID Generator is not defined")
            .generate()
    }

    /// Convert this tree into an [`IndexedTree`]
    pub fn index(self) -> IndexedTree<R, G> {
        IndexedTree::from_tree(self)
    }

    /// Get the maximum depth of the tree
    pub fn depth(&self) -> usize {
        // The iterator yields IterNode's which have a depth() method,
        // so we .map() to yield the depth as usize, and .max()
        // to get the maximum depth.
        self.root().into_iter().map(|f| f.depth()).max().unwrap()
    }

    /// Get the maximum width of the tree (iterator index())
    pub fn width(&self) -> usize {
        self.root().into_iter().map(|f| f.index()).max().unwrap()
    }

    /// Get the positional xxh64 hash of the tree. This includes the index, depth, and data of each node
    pub fn xxhash_positional(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        for node in self.root() {
            // Include the node index and depth in the hash
            node.index().hash(&mut hasher);
            node.depth().hash(&mut hasher);
            node.node().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Create a [`Tree`] container from a root [`NodeRef`]
    pub fn from_node(root: R, idgen: Option<G>) -> Self {
        Self {
            root: Some(root),
            idgen,
        }
    }

    /// Get the root [`NodeRef`] of the tree
    pub fn root(&self) -> R {
        self.root.as_ref().unwrap().clone()
    }

    /// Get a reference to the root [`NodeRef`] of the tree
    pub fn root_ref<'a>(&'a self) -> &'a R {
        self.root.as_ref().unwrap()
    }

    /// Get a mutable reference to the root [`NodeRef`] of the tree
    pub fn root_ref_mut<'a>(&'a mut self) -> &'a mut R {
        self.root.as_mut().unwrap()
    }

    /// Remove the provided [`NodeRef`] from the tree
    pub fn remove_node(&mut self, node: &R) {
        let node_id = node.node().id().clone();
        debug!("Removing node id {node_id}");

        let mut index = None;

        // Remove the node from the parent children vec
        if let Some(parent) = node.clone().node().parent() {
            if let Some(children) = parent.node().children() {
                for child in (&*children).iter().enumerate() {
                    if child.1.node().id() == node_id {
                        debug!("Found child node at index {}", child.0);
                        // Found index of node to remove
                        index = Some(child.0);
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

    /// Remove a child from a node at the given index
    pub fn remove_child(&mut self, parent: &mut R, index: usize) -> Option<R> {
        let parent_id = parent.node().id();
        if let Some(removed) = parent.node_mut().remove_child_index(index) {
            debug!("Child {index} removed from {parent_id}");
            Some(removed)
        } else {
            warn!("Child not found attempting to remove child at index {index}");
            None
        }
    }

    /// Remove all children from the specified parent node
    pub fn remove_children(&mut self, parent: &mut R) {
        let parent_id = parent.node().id();
        parent.node_mut().set_children(None);
        debug!("All children removed from {parent_id}");
    }

    pub fn set_children(&mut self, parent: &mut R, mut children: Vec<R>) {
        for child in &mut children {
            let new_id = self.generate_id();
            child.node_mut().set_id(new_id);
            child.node_mut().set_parent(parent.clone());
        }
        parent.node_mut().set_children(Some(children));
    }

    /// Replace a child in a node with a new child at the given index
    pub fn replace_child(&mut self, parent: &mut R, index: usize, mut new: R) {
        new.node_mut().set_id(self.generate_id());

        if let Some(mut children) = new.node_mut().children_mut() {
            for child in children.iter_mut() {
                let new_id = self.generate_id();
                child.node_mut().set_id(new_id);
            }
        }

        new.node_mut().set_parent(parent.clone());
        parent.node_mut().replace_child(new, index);
    }

    /// Insert a child into a parent at the given index
    pub fn insert_child(&mut self, parent: &mut R, index: usize, mut new: R) -> Option<()> {
        new.node_mut().set_parent(parent.clone());
        parent.node_mut().insert_child(new, index)
    }

    pub fn replace_node(&mut self, dest: &mut R, source: &R) {
        *dest.node_mut().data_mut() = source.node().data().clone();
    }

    /// Create a new node from the provided data. Does not insert into the tree, but allocates a new ID
    pub fn create_node(&self, data: <<R as TreeNodeRef>::Inner as TreeNode>::Data) -> Option<R> {
        // Generate a new Node ID
        if let Some(gen) = &self.idgen {
            let id = gen.generate();
            debug!("Allocated new node ID {id}");

            // Create a new Inner Node
            let node = <R as TreeNodeRef>::Inner::new(id, data, None);

            // Create and return a new NodeRef wrapping this node
            Some(R::new(node))
        } else {
            error!("ID generator not available attempting to create new node from Tree");
            None
        }
    }

    /// Insert a subtree as a child of the specified parent at a given child index
    pub fn insert_subtree(&mut self, parent: &mut R, index: usize, mut subtree: R) -> Option<()>
    where
        R::Data: Clone,
        <<R as TreeNodeRef>::Inner as TreeNode>::Data: Clone,
    {
        println!("INSERT SUBTREE");
        //let mut nodes = Vec::new();

        subtree
            .for_each_mut(|r| {
                let new_id = self.generate_id();
                r.node_mut().set_id(new_id);

                Ok::<(), ()>(())
            })
            .unwrap();

        // Set the parent of the subtree
        subtree.node_mut().set_parent(parent.clone());

        // Insert the root of the cloned subtree into the parent node at the provided index
        parent.node_mut().insert_child(subtree, index);

        Some(())
    }
}

impl<R, G> Deref for Tree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.root.as_ref().unwrap()
    }
}

pub struct IndexedTree<R, G = crate::IdGenerator>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    tree: Tree<R, G>,
    leaves: Vec<R>,
    index: BTreeIndex<R>,
}

impl<R, G> std::fmt::Debug for IndexedTree<R, G>
where
    R: TreeNodeRef + std::fmt::Debug + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let leaf_ids: Vec<<<R as TreeNodeRef>::Inner as TreeNode>::Id> =
            self.leaves.iter().map(|leaf| leaf.node().id()).collect();
        let ids = self.index.get_ids();

        f.debug_struct("IndexedTree")
            .field("tree", &self.tree)
            .field("leaf_ids", &leaf_ids)
            .field("index_ids", &ids)
            .finish()
    }
}

impl<R, G> IndexedTree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    // Create a new empty indexed tree
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            leaves: Vec::new(),
            index: BTreeIndex::new(),
        }
    }

    pub fn from_tree(tree: Tree<R, G>) -> Self {
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

    pub fn tree(&self) -> &Tree<R, G> {
        &self.tree
    }

    pub fn index(&self) -> &BTreeIndex<R> {
        &self.index
    }

    pub fn get_node(&self, id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id) -> Option<&R> {
        self.index.get(id)
    }

    pub fn get_node_mut(
        &mut self,
        id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id,
    ) -> Option<&mut R> {
        self.index.get_mut(id)
    }

    pub fn remove_node(&mut self, node: &R) -> Option<()> {
        let node_id = node.node().id().clone();

        // Remove the node from the tree
        self.tree.remove_node(node);

        let mut remove_ids: HashSet<<<R as TreeNodeRef>::Inner as TreeNode>::Id> =
            HashSet::from([node_id]);

        // Remove node and descendents from the index
        for node in node.clone().into_iter() {
            remove_ids.insert(node.node().id().clone());
        }

        for id in remove_ids {
            // Remove from the index
            let _removed = self.index.remove(&id)?;

            // Remove from leaves
            self.leaves.retain(|node| node.node().id() != id);
        }

        Some(())
    }

    pub fn insert_noderef(&mut self, parent: &mut R, index: usize, node: R) -> Option<()> {
        self.tree.insert_child(parent, index, node.clone())?;

        for node in node.into_iter() {
            let id = node.node().id().clone();
            self.index.insert(id, node.clone());
            if node.node().num_children() == 0 {
                self.leaves.push(node.clone());
            }
        }

        Some(())
    }

    pub fn insert_child(
        &mut self,
        parent_id: NodeRefId<R>,
        index: usize,
        data: <<R as TreeNodeRef>::Inner as TreeNode>::Data,
    ) -> Option<()> {
        let mut parent = self.get_node_mut(&parent_id)?.clone();

        let node = self.tree.create_node(data)?;

        self.tree.insert_child(&mut parent, index, node.clone())?;

        for node in node.into_iter() {
            let id = node.node().id().clone();
            self.index.insert(id, node.clone());
            if node.node().num_children() == 0 {
                self.leaves.push(node.clone());
            }
        }

        Some(())
    }

    pub fn remove_node_id(
        &mut self,
        id: &<<R as TreeNodeRef>::Inner as TreeNode>::Id,
    ) -> Option<()> {
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

/// Deref IndexedTree into Tree
impl<R, G> Deref for IndexedTree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    type Target = Tree<R, G>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

/// DerefMut IndexedTree into Tree
impl<R, G> DerefMut for IndexedTree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}
