use std::collections::{HashMap, HashSet};

use crate::{noderef::NodeRefId, TreeNode as _, TreeNodeRef};

pub struct LeafIter<R>
where
    R: TreeNodeRef + std::fmt::Debug + 'static,
{
    // HashSet of visited Node Ids
    visited: HashSet<NodeRefId<R>>,

    // Map of sets of of visited children for each parent node
    children_visited: HashMap<NodeRefId<R>, HashSet<NodeRefId<R>>>,

    // Queue of nodes at the current depth
    queue: Vec<R>,

    // Queue of nodes at the next depth. This will move into the outer queue
    // after it has drained indicating all nodes at the current depth have
    // been processed.
    next: Vec<R>,
}

impl<R> LeafIter<R>
where
    R: TreeNodeRef + std::fmt::Debug + 'static,
{
    pub fn new(leaves: &Vec<R>) -> Self {
        // Track which nodes have been visited, initialized with initial set of leaf nodes
        let visited: HashSet<NodeRefId<R>> =
            leaves.iter().map(|leaf| leaf.node().id().clone()).collect();

        Self {
            visited,
            children_visited: HashMap::new(),
            queue: Vec::from(leaves.clone()),
            next: Vec::new(),
        }
    }

    /// Iterate through the tree backwards from a Vec of leaf NodeRefs
    pub fn for_each<F>(&mut self, f: F)
    where
        F: Fn(&R),
    {
        while let Some(node) = self.queue.pop() {
            let node_id = node.node().id().clone();
            println!("Pop {}", node_id);

            // Get the expected number of children for this node
            let expected_children = node.node().num_children();

            // Get the number of children we have visited for this node
            let have_children = self
                .children_visited
                .get(&node_id)
                .map(|v| v.len())
                .unwrap_or(0);

            if expected_children != have_children {
                // Put the node into the next depth queue, as not all children
                // of this node have been resolved yet.

                // Move next nodes into queue if the outer queue is empty
                if self.queue.is_empty() {
                    self.queue.append(&mut self.next);
                }

                self.next.push(node);

                // Continue the loop to pop the next node from the queue
                continue;
            } else {
                // All children have been resolved for the current node
                // Remove the children hashset from children_visited
                // We can only use this set if we don't need to traverse in a deterministic order
                // For now, just ignore it
                let _children = self.children_visited.remove(&node_id);

                /*
                if let Some(child_nodes) = node.node().children() {
                    for child in &*child_nodes {
                        //f(&child)
                    }
                }
                */

                // Yield ourselves. All descendents of the children of this node have been previously yielded
                // Note that it's up to the caller to iterate through the yielded node.node().children()
                // in order to do what they please with the children
                f(&node)
            }

            // Check if this node has a parent
            if let Some(parent) = node.node().parent() {
                let parent_node = parent.node();
                let parent_id = parent_node.id();

                // Insert this node into children_visisted for the parent
                self.children_visited
                    .entry(parent_id)
                    .or_insert(HashSet::new())
                    .insert(node_id);

                if self.visited.insert(parent_id) {
                    // Node has not been visited before, add to queue
                    //info!("PUSH NEXT {}", parent_id);
                    self.next.push(parent.clone());
                }
            } else {
                // No parent. This is the root node.
                break;
            }

            if self.queue.is_empty() {
                self.queue.append(&mut self.next);
            }
        }
    }
}
