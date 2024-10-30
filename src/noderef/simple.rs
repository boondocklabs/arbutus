use std::{collections::VecDeque, ops::Deref};

use crate::{
    iterator::{IterNode, NodeRefIter},
    TreeNode,
};

use super::{internal::NodeRefInternal, TreeFormat as _, TreeNodeRef, TreeNodeRefRef};

/// Simple reference noderef. Does not allow cloning.
#[derive(Debug, Hash)]
pub struct NodeRef<'a, T>
where
    T: TreeNode<NodeRef = Self> + 'static,
{
    node: &'a mut T,
}

impl<'a, T> NodeRefInternal<T> for NodeRef<'a, T> where T: TreeNode<NodeRef = Self> + 'static {}

impl<'a, T> TreeNodeRefRef<'a> for NodeRef<'a, T>
where
    T: TreeNode<NodeRef = Self> + 'static,
{
    fn new(node: &'a mut Self::Inner) -> Self {
        Self { node }
    }
}

impl<'a, T> TreeNodeRef for NodeRef<'a, T>
where
    T: TreeNode<NodeRef = Self> + std::fmt::Debug + 'static,
{
    type Inner = T;
    type InnerRef<'b> = &'b Self::Inner where Self: 'b;
    type InnerRefMut<'b> = &'b mut Self::Inner where Self: 'b;
    type Data = T::Data;
    type DataRef<'b> = T::DataRef<'b> where Self: 'b;
    type DataRefMut<'b> = T::DataRefMut<'b>;

    fn new<N>(_node: N) -> Self
    where
        N: Into<Self::Inner>,
    {
        panic!("Don't use this...");
    }

    fn node<'b>(&'b self) -> Self::InnerRef<'b> {
        &self.node
    }

    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b> {
        &mut self.node
    }

    fn with_data<'b, R, E, F>(&'b self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRef<'b>) -> Result<R, E> + 'b,
    {
        f(self.node.data())
    }

    fn with_data_mut<'b, R, E, F>(&'b mut self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRefMut<'_>) -> Result<R, E>,
    {
        f(self.node.data_mut())
    }

    fn for_each<E, F>(&self, f: F) -> Result<(), E>
    where
        F: Fn(usize, Self) -> Result<(), E>,
    {
        // Create a stack with depth 0, and the initial node
        let mut stack: VecDeque<(usize, Self)> = VecDeque::from([(0, self.clone())]);

        loop {
            let current = stack.pop_front();
            if let None = current {
                break;
            };
            let node = current.map(|node| {
                node.1.node().children().map(|children| {
                    children
                        .iter()
                        .rev()
                        .for_each(|child| stack.push_front((node.0 + 1, child.clone())))
                });
                node
            });

            if let Some(node) = node {
                f(node.0, node.1)?
            }
        }
        Ok(())
    }

    fn try_node<'b>(&'b self) -> Result<Self::InnerRef<'b>, std::cell::BorrowError> {
        todo!()
    }

    fn try_node_mut<'b>(&'b self) -> Result<Self::InnerRefMut<'b>, std::cell::BorrowMutError> {
        todo!()
    }
}

impl<'a, T> Deref for NodeRef<'a, T>
where
    T: TreeNode<NodeRef = Self>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, T> Clone for NodeRef<'a, T>
where
    T: TreeNode<NodeRef = Self>,
{
    fn clone(&self) -> Self {
        panic!("Cloning of node references is not supported with NodeRefRef nodes. Use one of the Rc/Arc smart pointer NodeRef types.");
    }
}

impl<'a, N> IntoIterator for NodeRef<'a, N>
where
    N: TreeNode<NodeRef = Self> + 'static,
{
    type Item = IterNode<Self>;
    type IntoIter = NodeRefIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        NodeRefIter::new(self)
    }
}

impl<'a, T> std::fmt::Display for NodeRef<'a, T>
where
    T: TreeNode<NodeRef = Self> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tree_format(f)
    }
}
