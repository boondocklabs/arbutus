use std::{collections::VecDeque, ops::Deref};

use crate::node::NodeRef;

pub struct IterNode<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    depth: usize,
    node: NodeRef<'iter, Data, Id>,
}

impl<'iter, Data, Id> IterNode<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    pub fn depth(&self) -> usize {
        self.depth
    }
}

impl<'iter, Data, Id> Deref for IterNode<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    type Target = NodeRef<'iter, Data, Id>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'iter, Data, Id> IntoIterator for NodeRef<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display + 'iter,
    Data: Clone + std::fmt::Debug + 'iter,
{
    type Item = IterNode<'iter, Data, Id>;
    type IntoIter = NodeRefIter<'iter, Data, Id>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        NodeRefIter::new(self)
    }
}

pub struct NodeRefIter<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display + 'iter,
{
    stack: VecDeque<(usize, NodeRef<'iter, Data, Id>)>,
}

impl<'iter, Data, Id> NodeRefIter<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display + 'iter,
{
    pub fn new(node: NodeRef<'iter, Data, Id>) -> Self {
        Self {
            stack: VecDeque::from([(0, node)]),
        }
    }
}

impl<'iter, Data, Id> Iterator for NodeRefIter<'iter, Data, Id>
where
    Data: std::fmt::Debug + 'iter,
    Id: Clone + std::fmt::Display + 'iter,
{
    type Item = IterNode<'iter, Data, Id>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            node.1.node().children().map(|children| {
                children
                    .borrow()
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
