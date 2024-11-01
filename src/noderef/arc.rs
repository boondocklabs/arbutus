use std::{
    cell::{BorrowError, Ref, RefCell, RefMut},
    collections::VecDeque,
    ops::Deref,
    sync::Arc,
};

use crate::{
    iterator::{IterNode, NodeRefIter},
    TreeNode,
};

use super::{internal::NodeRefInternal, TreeFormat as _, TreeNodeRef};

pub struct NodeRef<T>
where
    T: TreeNode<NodeRef = Self>,
{
    node_ref: Arc<RefCell<T>>,
}

impl<T> std::hash::Hash for NodeRef<T>
where
    T: TreeNode<NodeRef = Self> + std::fmt::Debug + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node().hash(state)
    }
}

impl<T> Clone for NodeRef<T>
where
    T: TreeNode<NodeRef = Self>,
{
    fn clone(&self) -> Self {
        Self {
            node_ref: self.node_ref.clone(),
        }
    }
}

impl<T> NodeRef<T>
where
    T: TreeNode<NodeRef = Self>,
{
    pub fn new(node: T) -> Self {
        Self {
            node_ref: Arc::new(RefCell::new(node)),
        }
    }
}

impl<T> std::fmt::Display for NodeRef<T>
where
    T: TreeNode<NodeRef = Self> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tree_format_display(f)
    }
}

impl<T> std::fmt::Debug for NodeRef<T>
where
    T: TreeNode<NodeRef = Self> + std::fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tree_format_debug(f)
    }
}

impl<T> NodeRefInternal<T> for NodeRef<T> where T: TreeNode<NodeRef = Self> + 'static {}

impl<T> TreeNodeRef for NodeRef<T>
where
    T: TreeNode<NodeRef = Self> + std::fmt::Debug + 'static,
{
    type Inner = T;
    type InnerRef<'b> = Ref<'b, Self::Inner>;
    type InnerRefMut<'b> = RefMut<'b, Self::Inner>;
    type Data = T::Data;

    fn new<N>(node: N) -> Self
    where
        N: Into<Self::Inner>,
    {
        Self {
            node_ref: Arc::new(RefCell::new(node.into())),
        }
    }

    fn node<'b>(&'b self) -> Self::InnerRef<'b> {
        let r: &'b RefCell<T> = &self.node_ref;
        r.borrow()
    }

    fn try_node<'b>(&'b self) -> Result<Self::InnerRef<'b>, BorrowError> {
        let r: &'b RefCell<T> = &self.node_ref;
        r.try_borrow()
    }

    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b> {
        (&*self.node_ref).borrow_mut()
    }

    fn try_node_mut<'b>(&'b self) -> Result<Self::InnerRefMut<'b>, std::cell::BorrowMutError> {
        (&*self.node_ref).try_borrow_mut()
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
}

impl<T> Deref for NodeRef<T>
where
    T: TreeNode<NodeRef = Self>,
{
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &*self.node_ref
    }
}

impl<N> IntoIterator for NodeRef<N>
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

impl<'a, N> IntoIterator for &'a NodeRef<N>
where
    N: TreeNode<NodeRef = NodeRef<N>> + 'static,
{
    type Item = IterNode<NodeRef<N>>;
    type IntoIter = NodeRefIter<NodeRef<N>>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        NodeRefIter::new(self.clone())
    }
}
