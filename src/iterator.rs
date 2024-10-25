use std::collections::HashMap;
use std::{collections::VecDeque, ops::Deref};

use crate::node::TreeNode;
use crate::TreeNodeRef;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct NodePosition {
    // Vertical depth
    pub depth: usize,

    // Horizontal index at each depth.
    pub index: usize,

    // Index of the child relative to the parent
    pub child_index: usize,
}

impl NodePosition {
    /// Get the depth of this node
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the horizontal position of this node relative to all siblings at this depth
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the child position relative to the parent
    pub fn child_index(&self) -> usize {
        self.child_index
    }
}

pub struct IterNode<R>
where
    R: TreeNodeRef,
{
    position: NodePosition,
    node: R,
}

impl<R> IterNode<R>
where
    R: TreeNodeRef,
{
    /// The index along the horizontal at the current depth
    pub fn index(&self) -> usize {
        self.position.index
    }

    /// The vertical depth of this node
    pub fn depth(&self) -> usize {
        self.position.depth
    }

    pub fn position(&self) -> &NodePosition {
        &self.position
    }
}

impl<R> Deref for IterNode<R>
where
    R: TreeNodeRef,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

pub struct NodeRefIter<R>
where
    R: TreeNodeRef,
{
    stack: VecDeque<(usize, usize, usize, R)>,
    index: HashMap<usize, usize>,
}

impl<R> NodeRefIter<R>
where
    R: TreeNodeRef,
{
    pub fn new(node: R) -> Self {
        Self {
            stack: VecDeque::from([(0, 0, 0, node)]),
            index: HashMap::new(),
        }
    }
}

impl<R> Iterator for NodeRefIter<R>
where
    R: TreeNodeRef,
{
    type Item = IterNode<R>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|(child_index, index, depth, node)| {
            node.node().children().map(|children| {
                let index = self.index.entry(depth).or_insert(0);

                // Increment the horizontal index in the iterator state by the number of children we have.
                // This increases the index by how many children we're about to push onto the stack,
                // and will be retained as the iterator moves between different depths
                *index += children.len();

                children
                    .iter()
                    // Enumerate to get the child index
                    .enumerate()
                    // Reverse the iterator as we're pushing onto a stack
                    // that will pop the nodes back off in reverse order
                    .rev()
                    .for_each(|(child_index, child)| {
                        self.stack.push_front((
                            child_index,
                            // *index is positive offset by the number of children we're adding to the VecDeque,
                            // so we need to decrement the next index by 1 as we're iterating backwards
                            // The child index is also decreasing, so we can subtract the difference from the
                            // total nodes to get an index offset
                            *index - (children.len() - child_index),
                            // Each child being pushed will have it's depth set to the current node depth + 1
                            depth + 1,
                            (*child).clone(),
                        ));
                    })
            });

            IterNode {
                position: NodePosition {
                    depth,
                    index,
                    child_index,
                },
                node,
            }
        })
    }
}
