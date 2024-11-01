use std::{
    cell::{BorrowError, Ref, RefCell, RefMut},
    rc::Rc,
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
    node_ref: Rc<RefCell<T>>,
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
            node_ref: Rc::new(RefCell::new(node)),
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
            node_ref: Rc::new(RefCell::new(node.into())),
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
