use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::usize;

use colored::Colorize;

use crate::node::TreeNode;
use crate::TreeNodeRef;

pub mod leaf;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodePosition {
    // Vertical depth
    pub depth: usize,

    // Horizontal index at each depth.
    pub index: usize,

    // Index of the child relative to the parent
    pub child_index: usize,
}

impl std::fmt::Display for NodePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} {}: {} {}: {}",
            "depth".bright_cyan(),
            self.depth().to_string().bright_purple(),
            "index".bright_cyan(),
            self.index().to_string().bright_purple(),
            "child_index".bright_cyan(),
            self.child_index().to_string().bright_purple(),
        )
    }
}

impl NodePosition {
    pub fn zero() -> Self {
        NodePosition {
            depth: 0,
            index: 0,
            child_index: 0,
        }
    }

    // Return a NodePosition with all positions set to usize::MAX
    pub fn max() -> Self {
        NodePosition {
            depth: usize::MAX,
            index: usize::MAX,
            child_index: usize::MAX,
        }
    }

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

    pub fn child_index(&self) -> usize {
        self.position.child_index
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

impl<R> DerefMut for IterNode<R>
where
    R: TreeNodeRef,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

pub struct NodeRefIter<R>
where
    R: TreeNodeRef,
{
    stack: Vec<(usize, usize, usize, R)>,
    index: HashMap<usize, usize>,
}

impl<R> NodeRefIter<R>
where
    R: TreeNodeRef,
{
    pub fn new(node: R) -> Self {
        Self {
            stack: Vec::from([(0, 0, 0, node)]),
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
        let current = self.stack.pop();

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
                        self.stack.push((
                            child_index,
                            // *index is positive offset by the number of children we're adding to the Vec,
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
