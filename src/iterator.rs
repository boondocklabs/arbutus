use std::collections::HashMap;
use std::{collections::VecDeque, ops::Deref};

use crate::node::Node;
use crate::noderef::{NodeRef, NodeRefRef};
use crate::NodeRefRc;

pub struct IterNode<R>
where
    R: NodeRef,
{
    index: usize,
    depth: usize,
    node: R,
}

impl<R> IterNode<R>
where
    R: NodeRef,
{
    /// The index along the horizontal at the current depth
    pub fn index(&self) -> usize {
        self.index
    }

    /// The vertical depth of this node
    pub fn depth(&self) -> usize {
        self.depth
    }
}

impl<R> Deref for IterNode<R>
where
    R: NodeRef,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<N> IntoIterator for NodeRefRc<N>
where
    N: Node<NodeRef = Self> + 'static,
{
    type Item = IterNode<Self>;
    type IntoIter = NodeRefIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        NodeRefIter::new(self)
    }
}

impl<N> IntoIterator for NodeRefRef<N>
where
    N: Node<NodeRef = Self> + 'static,
{
    type Item = IterNode<Self>;
    type IntoIter = NodeRefIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        NodeRefIter::new(self)
    }
}

pub struct NodeRefIter<R>
where
    R: NodeRef,
{
    stack: VecDeque<(usize, usize, R)>,
    index: HashMap<usize, usize>,
}

impl<R> NodeRefIter<R>
where
    R: NodeRef,
{
    pub fn new(node: R) -> Self {
        Self {
            stack: VecDeque::from([(0, 0, node)]),
            index: HashMap::new(),
        }
    }
}

impl<R> Iterator for NodeRefIter<R>
where
    R: NodeRef,
{
    type Item = IterNode<R>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|(index, depth, node)| {
            node.node().children().map(|children| {
                children.iter().rev().for_each(|child| {
                    let index = self.index.entry(depth + 1).or_insert(0);
                    *index = *index + 1;
                    self.stack.push_front((*index, depth + 1, (*child).clone()))
                })
            });

            IterNode { index, depth, node }
        })
    }
}
