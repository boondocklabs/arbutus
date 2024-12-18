use std::{
    hash::{Hash as _, Hasher},
    ops::{Deref, DerefMut},
};

use crate::{id::UniqueId, noderef::TreeNodeRef, NodePosition};
use xxhash_rust::xxh64::Xxh64;

pub mod arc;
pub mod rc;

/// Sealed trait for internal Node methods
pub(crate) mod internal {
    use super::TreeNode;

    pub trait NodeInternal<Node>
    where
        Node: TreeNode,
    {
        fn set_id(&mut self, id: Node::Id);
        fn set_parent(&mut self, parent: Node::NodeRef);

        /// Take ownership of the children Vec out of the Option, leaving None in its place
        fn take_children(&mut self) -> Option<Vec<Node::NodeRef>>;
    }
}

pub trait TreeNode:
    internal::NodeInternal<Self> + Clone + std::hash::Hash + std::fmt::Debug
{
    type Data: std::hash::Hash + Clone + std::fmt::Display;
    type Id: UniqueId;
    type DataRef<'b>: Deref<Target = Self::Data>
    where
        Self: 'b;
    type DataRefMut<'b>: DerefMut<Target = Self::Data>
    where
        Self: 'b;

    type NodeRef: TreeNodeRef<Inner = Self>;

    type ChildrenRef<'b>: Deref<Target = Vec<Self::NodeRef>>
    where
        Self: 'b;

    type ChildrenRefMut<'b>: DerefMut<Target = Vec<Self::NodeRef>>
    where
        Self: 'b;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<Self::NodeRef>>) -> Self;

    fn with_parent(self, parent: Self::NodeRef) -> Self;

    fn with_position(self, position: NodePosition) -> Self;

    fn id(&self) -> Self::Id;

    fn get_position(&self) -> Option<&NodePosition>;

    fn set_subtree_hash(&mut self, subtree_hash: u64);
    fn get_subtree_hash(&self) -> u64;

    fn data<'b>(&'b self) -> Self::DataRef<'b>;
    fn data_mut<'b>(&'b mut self) -> Self::DataRefMut<'b>;

    fn parent<'b>(&'b self) -> Option<&'b Self::NodeRef>;
    fn parent_mut<'b>(&'b mut self) -> Option<&'b mut Self::NodeRef>;

    fn children<'b>(&'b self) -> Option<Self::ChildrenRef<'b>>;
    fn children_mut<'b>(&'b mut self) -> Option<Self::ChildrenRefMut<'b>>;

    fn set_children(&mut self, children: Option<Vec<Self::NodeRef>>);

    /// Return the number of child nodes for this node
    fn num_children(&self) -> usize {
        self.children().map(|v| v.len()).unwrap_or(0)
    }

    /// Add a new child node to this node
    fn push_child(&mut self, node: Self::NodeRef) {
        if let Some(mut children) = self.children_mut() {
            children.push(node);
            return;
        }

        self.set_children(Some(Vec::from([node])));
    }

    /// Insert a child node to this node at the specified index
    fn insert_child(&mut self, node: Self::NodeRef, index: usize) -> Option<()> {
        if let Some(mut children) = self.children_mut() {
            if index <= children.len() {
                children.insert(index, node);
                return Some(());
            } else {
                tracing::error!(
                    "Attempted to insert child with index {} into children with length {}",
                    index,
                    children.len()
                );
                return None;
            }
        };

        // No existing children, create a new child vec for this node
        self.set_children(Some(Vec::from([node])));
        Some(())
    }

    /// Remove a child node from this node at the specified index, returing
    /// the NodeRef to the removed child.
    fn remove_child_index(&mut self, index: usize) -> Option<Self::NodeRef> {
        if let Some(mut children) = self.children_mut() {
            Some(children.remove(index))
        } else {
            None
        }
    }

    fn replace_child(&mut self, source: Self::NodeRef, index: usize) {
        if let Some(mut children) = self.children_mut() {
            children[index] = source
        }
    }

    fn hash_children(&self, state: &mut impl std::hash::Hasher) {
        if let Some(children) = self.children() {
            for child in children.iter() {
                child.node().hash(state);
            }
        }
    }

    fn xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn data_xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        self.data().hash(&mut hasher);
        hasher.finish()
    }

    /// Hash the node including immediate children
    fn xxhash_children(&self) -> u64 {
        let mut hasher = Xxh64::new(0);

        self.hash_children(&mut hasher);

        // Hash ourselves
        self.hash(&mut hasher);

        hasher.finish()
    }

    /// Hash the node including immediate children. Additional data to hash can provided in the `with` argument
    fn xxhash_children_with(&self, with: &[&impl std::hash::Hash]) -> u64 {
        let mut hasher = Xxh64::new(0);

        // Hash additional context
        for h in with {
            h.hash(&mut hasher);
        }

        if let Some(children) = self.children() {
            for child in children.iter() {
                child.node().hash(&mut hasher);
            }
        }

        // Hash ourselves
        self.hash(&mut hasher);

        hasher.finish()
    }

    /// Compute the node hash with additional context
    fn xxhash_with(&self, with: &[&impl std::hash::Hash]) -> u64 {
        let mut hasher = Xxh64::new(0);
        for h in with {
            h.hash(&mut hasher);
        }
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{NodeId, Tree, TreeBuilder};

    type NodeRef<T> = crate::noderef::arc::NodeRef<T>;
    type Node<Data, Id> = crate::node::arc::Node<Data, Id>;

    #[derive(Debug)]
    #[allow(unused)]
    enum TestError {
        Fail,
    }

    #[derive(Debug, Clone, Hash)]
    #[allow(unused)]
    enum TestData {
        Foo,
        Bar,
        Baz,
        String(String),
    }

    impl std::fmt::Display for TestData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Default for TestData {
        fn default() -> Self {
            TestData::Foo
        }
    }

    /// Create a simple tree for tests using the TreeBuilder
    fn simple_tree<'a>() -> Result<Option<Tree<NodeRef<Node<TestData, NodeId>>>>, TestError> {
        TreeBuilder::<TestData, TestError>::new()
            .root(TestData::Foo, |foo| {
                foo.child(TestData::Bar, |bar| {
                    bar.child(TestData::String("Hello".into()), |s| {
                        s.child(TestData::String("World".into()), |_| Ok(()))?;
                        Ok(())
                    })?;
                    bar.child(TestData::Baz, |_baz| Ok(()))?;
                    Ok(())
                })?;

                foo.child(TestData::Baz, |_baz| Ok(()))?;
                foo.child(TestData::Baz, |_baz| Ok(()))?;
                foo.child(TestData::Baz, |_baz| Ok(()))?;
                foo.child(TestData::Baz, |_baz| Ok(()))?;

                Ok(())
            })?
            .done()
    }

    #[traced_test]
    #[test]
    fn test_tree() {
        let tree = simple_tree().unwrap().unwrap();

        // Create an indexed tree
        let tree = tree.index();

        println!("{:?}", tree);
    }
}
