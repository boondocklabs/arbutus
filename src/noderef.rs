use std::ops::{Deref, DerefMut};

pub mod rc;
pub mod simple;

/// Type alias to get associated type of Id from the Inner node of a NodeRef
pub type NodeRefId<R> = <<R as TreeNodeRef>::Inner as TreeNode>::Id;

use crate::{display::TreeDisplay, iterator::IterNode, node::TreeNode};

pub(crate) mod internal {
    use crate::TreeNode as _;

    use super::TreeNodeRef;
    use std::collections::VecDeque;

    pub trait NodeRefInternal<Inner> {
        /// Iterate through each node from the specified NodeRef. Calls a closure with a mutable reference to each NodeRef
        fn for_each_mut<E, F>(&mut self, mut f: F) -> Result<(), E>
        where
            Self: Sized + TreeNodeRef,
            F: FnMut(&mut Self) -> Result<(), E>,
        {
            let mut stack: VecDeque<Self> = VecDeque::from([self.clone()]);

            loop {
                let current = stack.pop_front();
                if let None = current {
                    break;
                };
                let node = current.map(|mut node| {
                    node.node_mut().children_mut().map(|mut children| {
                        children
                            .iter_mut()
                            .rev()
                            .for_each(|child| stack.push_front(child.clone()))
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
}

pub trait TreeNodeRef:
    internal::NodeRefInternal<<Self as TreeNodeRef>::Inner>
    + Clone
    + std::hash::Hash
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

    // A reference to the Inner Node's data
    type DataRef<'b>: Deref<Target = Self::Data>
    where
        Self: 'b;

    // A mutable reference
    type DataRefMut<'b>: 'b;

    // Create a new NodeRef with the supplied Inner node
    fn new(node: Self::Inner) -> Self;

    /// Get a reference to the inner node
    fn node<'b>(&'b self) -> Self::InnerRef<'b>;

    /// Get a reference to the inner node
    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b>;

    /// Calls the provided closure with a reference to the node's data
    fn with_data<'b, R, E, F>(&'b self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRef<'_>) -> Result<R, E>;

    /// Calls the provided closure with a mutable reference to the node's data
    fn with_data_mut<'b, R, E, F>(&'b mut self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRefMut<'_>) -> Result<R, E>;

    /// Calls the provided closure for each node in the tree.
    /// Includes depth of the node in the first parameter of the closure
    fn for_each<E, F>(&self, f: F) -> Result<(), E>
    where
        F: Fn(usize, Self) -> Result<(), E>;
}

trait TreeFormat {
    fn tree_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

// Implement TreeFormat on anything implementing NodeRef and Display
impl<T: TreeNodeRef> TreeFormat for T
where
    T: TreeNodeRef,
{
    fn tree_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TreeDisplay::format(self, f, |data, f| write!(f, "{}", *data))
    }
}
