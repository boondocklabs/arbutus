use std::collections::{HashMap, HashSet, VecDeque};

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
    queue: VecDeque<R>,

    // Queue of nodes at the next depth. This will move into the outer queue
    // after it has drained indicating all nodes at the current depth have
    // been processed.
    next: VecDeque<R>,
}

impl<R> LeafIter<R>
where
    R: TreeNodeRef + std::fmt::Debug + 'static,
{
    pub fn new(leaves: &Vec<R>) -> Self {
        let mut leaves = leaves.clone();

        leaves.reverse();

        // Track which nodes have been visited, initialized with initial set of leaf nodes
        let visited: HashSet<NodeRefId<R>> =
            leaves.iter().map(|leaf| leaf.node().id().clone()).collect();

        Self {
            visited,
            children_visited: HashMap::new(),
            queue: VecDeque::from(leaves.clone()),
            next: VecDeque::new(),
        }
    }

    fn pop_next(&mut self) -> Option<R> {
        if self.queue.is_empty() && !self.next.is_empty() {
            std::mem::swap(&mut self.queue, &mut self.next);
        }
        self.queue.pop_front()
    }

    fn defer_node(&mut self, node: R) {
        // Move next nodes into queue if the outer queue is empty
        if self.queue.is_empty() {
            std::mem::swap(&mut self.queue, &mut self.next);
        }
        self.next.push_back(node);
    }

    fn mark_child_visited(&mut self, parent_id: NodeRefId<R>, child_id: NodeRefId<R>) {
        self.children_visited
            .entry(parent_id)
            .or_insert_with(HashSet::new)
            .insert(child_id);
    }

    /// Iterate through the tree backwards from a Vec of leaf NodeRefs, calling the provided closure with a reference to each node
    pub fn for_each<F, E>(&mut self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut R) -> Result<(), E>,
    {
        while let Some(mut node) = self.pop_next() {
            let node_id = node.node().id();

            // Get the expected number of children for this node
            let expected_children = node.node().num_children();

            // Get the children visited HashSet for this node
            let children_visited = self
                .children_visited
                .entry(node_id)
                .or_insert(HashSet::new());

            // Get the number of children we have visited for this node
            let have_children = children_visited.len();

            if expected_children != have_children {
                // Put the node into the next depth queue, as not all children
                // of this node have been resolved yet.

                self.defer_node(node);

                // Continue the loop to pop the next node from the queue
                continue;
            }

            // All children for this node have been resolved
            f(&mut node)?;

            if let Some(parent) = node.node().parent() {
                let parent_id = parent.node().id();
                self.mark_child_visited(parent_id, node_id);

                if self.visited.insert(parent_id) {
                    self.next.push_back(parent.clone());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{
        test::{test_tree_node, TestNode},
        TreeNode as _, TreeNodeRef as _,
    };

    #[traced_test]
    #[test]

    fn leaf_for_each() {
        let tree = test_tree_node(vec![
            TestNode(
                "a",
                vec![
                    TestNode("1", vec![]),
                    TestNode("2", vec![]),
                    TestNode("3", vec![]),
                ],
            ),
            TestNode(
                "b",
                vec![
                    TestNode("1", vec![]),
                    TestNode("2", vec![]),
                    TestNode("3", vec![]),
                ],
            ),
            TestNode(
                "c",
                vec![
                    TestNode("1", vec![]),
                    TestNode("2", vec![]),
                    TestNode("3", vec![]),
                ],
            ),
            TestNode(
                "d",
                vec![
                    TestNode("x", vec![]),
                    TestNode("y", vec![]),
                    TestNode(
                        "z",
                        vec![
                            TestNode("1", vec![]),
                            TestNode("2", vec![]),
                            TestNode("3", vec![]),
                        ],
                    ),
                ],
            ),
        ]);

        println!("{}", tree.root());

        let mut min_depth = usize::MAX;

        tree.leaf_iter()
            .for_each(|node| {
                let inner = node.node();

                let pos = inner.get_position().unwrap();
                println!("Node {:?} {:?}", node, pos);

                assert!(pos.depth() <= min_depth);
                min_depth = pos.depth();

                Ok::<(), ()>(())
            })
            .ok();
    }
}
