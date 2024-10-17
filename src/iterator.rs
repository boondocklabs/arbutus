use std::{collections::VecDeque, ops::Deref};

use crate::node::Node;
use crate::noderef::{NodeRef, NodeRefRef};
use crate::NodeRefRc;

pub struct IterNode<R>
where
    R: NodeRef,
{
    depth: usize,
    node: R,
}

impl<R> IterNode<R>
where
    R: NodeRef,
{
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
    stack: VecDeque<(usize, R)>,
}

impl<R> NodeRefIter<R>
where
    R: NodeRef,
{
    pub fn new(node: R) -> Self {
        Self {
            stack: VecDeque::from([(0, node)]),
        }
    }
}

impl<R> Iterator for NodeRefIter<R>
where
    R: NodeRef,
    //<<R as NodeRef>::Inner as Node>::NodeRef
{
    type Item = IterNode<R>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            node.1.node().children().map(|children| {
                children
                    .iter()
                    .rev()
                    .for_each(|child| self.stack.push_front((node.0 + 1, (*child).clone())))
            });

            IterNode {
                depth: node.0,
                node: node.1,
            }
        })
    }
}
