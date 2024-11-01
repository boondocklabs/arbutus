use std::{
    cell::{BorrowError, BorrowMutError},
    ops::{Deref, DerefMut},
};

pub mod arc;
pub mod rc;

/// Type alias to get associated type of Id from the Inner node of a NodeRef
pub type NodeRefId<R> = <<R as TreeNodeRef>::Inner as TreeNode>::Id;

use crate::{display::TreeDisplay, iterator::IterNode, node::TreeNode};

pub(crate) mod internal {
    pub trait NodeRefInternal<Inner> {}
}

pub trait TreeNodeRef:
    internal::NodeRefInternal<<Self as TreeNodeRef>::Inner>
    + Clone
    + std::hash::Hash
    + std::fmt::Debug
    + IntoIterator<Item = IterNode<Self>>
{
    // The inner type of this NodeRef is a Node trait, that
    // has a NodeRef type of ourselves
    type Inner: TreeNode<NodeRef = Self>;

    // InnerRef must impl Deref, with a Target of our Inner Node
    type InnerRef<'b>: Deref<Target = Self::Inner>
    where
        Self: 'b;

    type InnerRefMut<'b>: DerefMut<Target = Self::Inner>
    where
        Self: 'b;

    // The Data type contained within the Inner Node
    type Data;

    // Create a new NodeRef with the supplied Inner node
    fn new<T>(node: T) -> Self
    where
        T: Into<Self::Inner>;

    /// Get a reference to the inner node
    fn node<'b>(&'b self) -> Self::InnerRef<'b>;

    /// Try to get a reference to the inner node
    fn try_node<'b>(&'b self) -> Result<Self::InnerRef<'b>, BorrowError>;

    /// Get a reference to the inner node
    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b>;

    /// Try to get a mutable reference to the inner node
    fn try_node_mut<'b>(&'b self) -> Result<Self::InnerRefMut<'b>, BorrowMutError>;

    /// Calls the provided closure with a reference to the Node's data
    fn with_data<'b, R, E, F>(&'b self, f: F) -> Result<R, E>
    where
        F: FnOnce(<Self::Inner as TreeNode>::DataRef<'_>) -> Result<R, E> + 'b,
    {
        let node = self.node();
        let data = node.data();
        f(data)
    }

    /// Calls the provided closure with a mutable reference to the Node's data
    fn with_data_mut<'b, R, E, F>(&'b mut self, f: F) -> Result<R, E>
    where
        F: FnOnce(<Self::Inner as TreeNode>::DataRefMut<'_>) -> Result<R, E> + 'b,
    {
        let mut node = self.node_mut();
        let data = node.data_mut();
        f(data)
    }

    /// Calls the provided closure for each node in the tree.
    /// Includes depth of the node in the first parameter of the closure
    fn for_each<E, F>(&self, f: F) -> Result<(), E>
    where
        F: Fn(usize, Self) -> Result<(), E>,
    {
        // Create a stack with depth 0, and the initial node
        let mut stack: Vec<(usize, Self)> = Vec::from([(0, self.clone())]);

        loop {
            let current = stack.pop();
            if let None = current {
                break;
            };
            let node = current.map(|node| {
                node.1.node().children().map(|children| {
                    children
                        .iter()
                        .rev()
                        .for_each(|child| stack.push((node.0 + 1, child.clone())))
                });
                node
            });

            if let Some(node) = node {
                f(node.0, node.1)?
            }
        }
        Ok(())
    }

    /// Iterate through each node from the specified NodeRef. Calls a closure with a mutable reference to each NodeRef
    fn for_each_mut<E, F>(&mut self, mut f: F) -> Result<(), E>
    where
        Self: Sized + TreeNodeRef,
        F: FnMut(&mut Self) -> Result<(), E>,
    {
        let mut stack: Vec<Self> = Vec::from([self.clone()]);

        loop {
            let current = stack.pop();
            if let None = current {
                break;
            };
            let node = current.map(|mut node| {
                node.node_mut().children_mut().map(|mut children| {
                    children
                        .iter_mut()
                        .rev()
                        .for_each(|child| stack.push(child.clone()))
                });
                node
            });

            if let Some(mut node) = node {
                f(&mut node)?
            }
        }
        Ok(())
    }
}

trait TreeFormat {
    fn tree_format_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn tree_format_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

// Implement TreeFormat on anything implementing NodeRef
impl<T: TreeNodeRef> TreeFormat for T
where
    T: TreeNodeRef,
    for<'x> T::InnerRef<'x>: std::fmt::Debug,
{
    fn tree_format_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TreeDisplay::format(self, f, |data, f| write!(f, "{}", *data))
    }
    fn tree_format_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRef")
            .field("node", &self.try_node())
            .finish()
    }
}
