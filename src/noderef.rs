use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    ops::Deref,
    rc::Rc,
};

use crate::{display::TreeDisplay, iterator::IterNode, node::Node};

pub trait NodeRef: Clone + IntoIterator<Item = IterNode<Self>> {
    // The inner type of this NodeRef is a Node trait, that
    // has a NodeRef type of ourselves
    type Inner: Node<NodeRef = Self>;
    //where
    //for<'b> <<Self as NodeRef>::Inner as Node>::DataRef<'b>: std::fmt::Display;

    // InnerRef must impl Deref, with a Target of our Inner Node
    type InnerRef<'b>: Deref<Target = Self::Inner>
    where
        Self: 'b;

    type InnerRefMut<'b>: Deref<Target = Self::Inner>
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
impl<T: NodeRef> TreeFormat for T
where
    T: NodeRef,
{
    fn tree_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TreeDisplay::format(self, f, |data, f| write!(f, "{}", *data))
    }
}

/// Simple reference noderef. Does not allow cloning.
#[derive(Debug)]
pub struct NodeRefRef<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    node: T,
}

impl<T> NodeRef for NodeRefRef<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    type Inner = T;
    type InnerRef<'b> = &'b Self::Inner;
    type InnerRefMut<'b> = &'b mut Self::Inner;
    type Data = T::Data;
    type DataRef<'b> = T::DataRef<'b>;
    type DataRefMut<'b> = T::DataRefMut<'b>;

    fn new(node: T) -> Self {
        Self { node }
    }

    fn node<'b>(&'b self) -> Self::InnerRef<'b> {
        &self.node
    }

    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b> {
        &mut self.node
    }

    fn with_data<'b, R, E, F>(&'b self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRef<'_>) -> Result<R, E>,
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
}

impl<T> Deref for NodeRefRef<T>
where
    T: Node<NodeRef = Self>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> Clone for NodeRefRef<T>
where
    T: Node<NodeRef = Self>,
{
    fn clone(&self) -> Self {
        panic!("Cloning of node references is not supported with NodeRefRef nodes. Use one of the Rc/Arc smart pointer NodeRef types.");
    }
}

pub struct NodeRefRc<T>
where
    T: Node<NodeRef = Self>,
{
    node_ref: Rc<RefCell<T>>,
}

impl<T> Clone for NodeRefRc<T>
where
    T: Node<NodeRef = Self>,
{
    fn clone(&self) -> Self {
        Self {
            node_ref: self.node_ref.clone(),
        }
    }
}

impl<T> NodeRefRc<T>
where
    T: Node<NodeRef = Self>,
{
    pub fn new(node: T) -> Self {
        Self {
            node_ref: Rc::new(RefCell::new(node)),
        }
    }
}

impl<T> std::fmt::Display for NodeRefRc<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tree_format(f)
    }
}

impl<T> std::fmt::Debug for NodeRefRc<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tree_format(f)
    }
}

impl<T> NodeRef for NodeRefRc<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    type Inner = T;
    type InnerRef<'b> = Ref<'b, Self::Inner>;
    type InnerRefMut<'b> = RefMut<'b, Self::Inner>;
    type Data = T::Data;
    type DataRef<'b> = T::DataRef<'b>;
    type DataRefMut<'b> = T::DataRefMut<'b>;

    fn new(node: T) -> Self {
        Self {
            node_ref: Rc::new(RefCell::new(node)),
        }
    }

    fn node<'b>(&'b self) -> Self::InnerRef<'b> {
        (&*self.node_ref).borrow()
    }

    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b> {
        (&*self.node_ref).borrow_mut()
    }

    fn with_data<'b, R, E, F>(&'b self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRef<'_>) -> Result<R, E>,
    {
        let node = self.node_ref.borrow();
        let data = node.data();
        f(data)
    }

    fn with_data_mut<'b, R, E, F>(&'b mut self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRefMut<'_>) -> Result<R, E>,
    {
        let mut node = self.node_ref.borrow_mut();
        let data = node.data_mut();
        f(data)
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

impl<T> Deref for NodeRefRc<T>
where
    T: Node<NodeRef = Self>,
{
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &*self.node_ref
    }
}
