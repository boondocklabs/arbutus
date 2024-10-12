use crate::{
    index::{BTreeIndex, TreeIndex},
    node::NodeRef,
    NodeId,
};

#[derive(Debug)]
pub struct Tree<'tree, Data, Id = NodeId>
where
    Id: Clone + std::fmt::Display + 'static,
{
    root: NodeRef<'tree, Data, Id>,
}

impl<'tree, Data, Id> Tree<'tree, Data, Id>
where
    Data: std::fmt::Debug,
    Id: Clone + std::fmt::Debug + std::fmt::Display + 'static,
    Data: Clone + 'static,
{
    pub fn from_nodes(root: NodeRef<'tree, Data, Id>) -> Self {
        Self { root }
    }

    pub fn root(&self) -> NodeRef<'tree, Data, Id> {
        self.root.clone()
    }

    pub fn root_ref<'a>(&'a self) -> &'a NodeRef<'tree, Data, Id> {
        &self.root
    }
}

pub struct IndexedTree<'tree, Data, Id = NodeId>
where
    Id: Default + Clone + std::fmt::Display + 'static,
    Data: Default + std::fmt::Debug,
{
    tree: Tree<'tree, Data, Id>,
    index: BTreeIndex<'tree, Data, Id>,
}

impl<'tree, Data, Id> IndexedTree<'tree, Data, Id>
where
    Id: Default + Clone + Ord + std::fmt::Debug + std::fmt::Display + 'static,
    Data: Default + Clone + std::fmt::Debug + 'static,
{
    pub fn from_tree(tree: Tree<'tree, Data, Id>) -> Self {
        let index = BTreeIndex::from_tree(&tree);

        Self { tree, index }
    }

    pub fn tree(&self) -> &Tree<'tree, Data, Id> {
        &self.tree
    }

    pub fn index(&self) -> &BTreeIndex<'tree, Data, Id> {
        &self.index
    }

    pub fn get_node(&self, id: &Id) -> Option<&NodeRef<'tree, Data, Id>> {
        self.index.get(id)
    }
}
