use colored::Colorize;
use tracing::{debug, debug_span};

use crate::{
    edit::{vec_edits, Edit},
    hash::update_subtree_hash,
    noderef::NodeRefId,
    IndexedTree, TreeNode, TreeNodeRef, UniqueGenerator,
};

#[derive(Debug, Clone)]
pub enum TreePatchOperation<R>
where
    R: TreeNodeRef + 'static,
{
    InsertChild { dest: R, index: usize, source: R },
    DeleteChild { dest: R, index: usize },
    ReplaceChild { dest: R, index: usize, source: R },
    RemoveChildren { dest: R },
    SetChildren { dest: R, nodes: Vec<R> },
    ReplaceNode { dest: R, source: R },
}

#[derive(Debug)]
pub struct TreePatch<R>
where
    R: TreeNodeRef + 'static,
{
    patches: Vec<TreePatchOperation<R>>,
}

impl<R> TreePatch<R>
where
    R: TreeNodeRef + std::fmt::Debug + 'static,
{
    pub fn new(patches: Vec<TreePatchOperation<R>>) -> Self {
        Self { patches }
    }

    pub fn len(&self) -> usize {
        self.patches.len()
    }

    pub fn patch_tree<G>(&self, tree: &mut IndexedTree<R, G>)
    where
        R::Data: Clone,
        G: UniqueGenerator<Output = NodeRefId<R>>,
    {
        debug_span!("patch").in_scope(|| {
            for patch in self.patches.clone().into_iter() {
                debug!("{} {:#?}", "Patching".bright_purple(), patch);
                match patch {
                    TreePatchOperation::InsertChild {
                        mut dest,
                        index,
                        source,
                    } => {
                        tree.insert_subtree(&mut dest, index, source);
                        update_subtree_hash(dest);
                    }
                    TreePatchOperation::DeleteChild { mut dest, index } => {
                        tree.remove_child(&mut dest, index);
                        update_subtree_hash(dest);
                    }
                    TreePatchOperation::ReplaceChild {
                        mut dest,
                        index,
                        source,
                    } => {
                        tree.replace_child(&mut dest, index, source);
                        update_subtree_hash(dest);
                    }
                    TreePatchOperation::RemoveChildren { mut dest } => {
                        //dest.node_mut().set_children(None);
                        tree.remove_children(&mut dest);
                        update_subtree_hash(dest);
                    }
                    TreePatchOperation::SetChildren { mut dest, nodes } => {
                        tree.set_children(&mut dest, nodes);
                        update_subtree_hash(dest);
                    }
                    TreePatchOperation::ReplaceNode { mut dest, source } => {
                        tree.replace_node(&mut dest, &source);
                        update_subtree_hash(dest);
                    }
                };
            }
        })
    }
}

pub struct TreeDiff<R>
where
    R: TreeNodeRef + 'static,
{
    dest_tree: R,
    source_tree: R,
}

impl<R> TreeDiff<R>
where
    R: TreeNodeRef + std::fmt::Debug + std::fmt::Display + 'static,
{
    pub fn new(dest_tree: R, source_tree: R) -> Self {
        Self {
            dest_tree,
            source_tree,
        }
    }

    pub fn diff(&mut self) -> TreePatch<R> {
        debug_span!("diff").in_scope(|| {
            let mut patches = Vec::new();

            // Stack of pending nodes to compare. Each is initialized with the root tree nodes from each tree
            let mut dest_stack: Vec<R> = Vec::from([self.dest_tree.clone()]);
            let mut source_stack: Vec<R> = Vec::from([self.source_tree.clone()]);

            while let (Some(dest), Some(source)) = (dest_stack.pop(), source_stack.pop()) {
                let dhash = dest.node().get_subtree_hash();
                let shash = source.node().get_subtree_hash();

                debug!("Pop dest: 0x{dhash:X} source: 0x{shash:X}");

                // Only consider nodes which have mismatched subtree hashes
                if dhash != shash {
                    debug!(
                        "Subtree mismatch at {} ",
                        dest.node().get_position().unwrap()
                    );
                    debug!(
                        "Subtree Hashes Dest: {} Source: {}",
                        format!("0x{:X}", dest.node().get_subtree_hash()).bright_green(),
                        format!("0x{:X}", source.node().get_subtree_hash()).bright_green()
                    );

                    // If the data hashes don't match, issue a ReplaceNode op
                    if source.node().data_xxhash() != dest.node().data_xxhash() {
                        patches.push(TreePatchOperation::ReplaceNode {
                            dest: dest.clone(),
                            source: source.clone(),
                        });
                    }

                    match (dest.node().children(), source.node().children()) {
                        (None, None) => {
                            debug!("Node is a leaf node. Diffing parents.");

                            let dnode = dest.node();
                            let snode = source.node();

                            let dest_parent = dnode.parent().unwrap();
                            let source_parent = snode.parent().unwrap();

                            patches.extend(Self::diff_children(dest_parent, source_parent));
                        }
                        (None, Some(source_children)) => {
                            debug!("Only source has children. Adding all source children to dest");

                            let children: Vec<R> =
                                source_children.iter().map(|child| child.clone()).collect();
                            patches.push(TreePatchOperation::SetChildren {
                                dest: dest.clone(),
                                nodes: children,
                            });

                            patches.push(TreePatchOperation::ReplaceNode {
                                dest: dest.clone(),
                                source: source.clone(),
                            });
                        }
                        (Some(_dest_children), None) => {
                            debug!("Only dest has children. Removing all children from dest");
                            patches.push(TreePatchOperation::RemoveChildren { dest: dest.clone() })
                        }
                        (Some(dest_children), Some(source_children)) => {
                            let dest_child_hashes: Vec<u64> = dest_children
                                .iter()
                                .map(|child| child.node().get_subtree_hash())
                                .collect();

                            let source_child_hashes: Vec<u64> = source_children
                                .iter()
                                .map(|child| child.node().get_subtree_hash())
                                .collect();

                            if dest_child_hashes == source_child_hashes {
                                debug!("Child hashes are identical. Parent mismatch.");
                                continue;
                            }

                            if dest_children.len() == source_children.len() {
                                for (dest_child, source_child) in
                                    dest_children.iter().zip(source_children.iter())
                                {
                                    let dest_child_hash = dest_child.node().get_subtree_hash();
                                    let source_child_hash = source_child.node().get_subtree_hash();

                                    if dest_child_hash != source_child_hash {
                                        // Check if this child subtree matches the destination subtree.
                                        if source_child_hash == dhash {
                                            debug!(
                                                "{} 0x{dhash:X}",
                                                "Source child subtree matches dest subtree"
                                                    .yellow()
                                            );

                                            let children: Vec<R> = source_children
                                                .iter()
                                                .map(|child| child.clone())
                                                .collect();
                                            patches.push(TreePatchOperation::SetChildren {
                                                dest: dest.clone(),
                                                nodes: children,
                                            });

                                            patches.push(TreePatchOperation::ReplaceNode {
                                                dest: dest.clone(),
                                                source: source.clone(),
                                            });
                                        } else {
                                            debug!("{}", "Pushing children".green());
                                            dest_stack.push(dest_child.clone());
                                            source_stack.push(source_child.clone());
                                        }
                                    } else {
                                        debug!("{}", "Skipping subtree".cyan());
                                    }
                                }
                                continue;
                            } else {
                                debug!("{}", "Child length mismatch".bright_blue());
                                patches.extend(Self::diff_children(&dest, &source));
                            }
                        }
                    }
                }
            }
            TreePatch::new(patches)
        })
    }

    fn diff_children(dest: &R, source: &R) -> Vec<TreePatchOperation<R>> {
        let mut patches = Vec::new();

        let dest_node = dest.node();
        let source_node = source.node();

        let dest_children = dest_node.children().unwrap();
        let source_children = source_node.children().unwrap();

        let dest_child_hashes: Vec<u64> = dest_children
            .iter()
            .map(|child| child.node().get_subtree_hash())
            .collect();

        let source_child_hashes: Vec<u64> = source_children
            .iter()
            .map(|child| child.node().get_subtree_hash())
            .collect();

        // Get the edits between the vec of child hashes
        let edits = vec_edits(&dest_child_hashes, &source_child_hashes);

        for edit in edits {
            let patch = match edit {
                Edit::Delete { dest_index } => TreePatchOperation::DeleteChild {
                    dest: dest.clone(),
                    index: dest_index,
                },
                Edit::Replace {
                    dest_index,
                    source_index,
                } => TreePatchOperation::ReplaceChild {
                    dest: dest.clone(),
                    index: dest_index,
                    source: source_children[source_index].clone(),
                },

                Edit::Insert {
                    dest_index,
                    source_index,
                } => TreePatchOperation::InsertChild {
                    dest: dest.clone(),
                    index: dest_index,
                    source: source_children[source_index].clone(),
                },
            };

            patches.push(patch);
        }

        patches
    }
}

#[cfg(test)]
mod tests {
    use colored::Colorize as _;
    use tracing_test::traced_test;

    use crate::test::{
        test_tree, test_tree_deep, test_tree_nested, test_tree_node, test_tree_vec, TestNode,
    };

    use super::TreeDiff;

    #[traced_test]
    #[test]
    fn deep() {
        let mut a = test_tree_deep(vec!["foo", "a", "bar"], vec!["a", "b", "c"]);
        let b = test_tree_deep(vec!["foo", "b", "bar"], vec!["a", "b", "c"]);

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn nested() {
        let mut a = test_tree_nested(2, vec!["foo", "a", "bar"]);
        let b = test_tree_nested(1, vec!["foo", "a", "bar"]);

        let root = a.root();

        let mut diff = TreeDiff::new(root, b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn insert_child() {
        let mut a = test_tree(vec!["foo", "a", "bar"]);
        let b = test_tree(vec!["foo", "b", "bar"]);

        let root = a.root();

        let mut diff = TreeDiff::new(root, b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn append_child() {
        let mut a = test_tree(vec!["foo", "bar"]);
        let b = test_tree(vec!["foo", "bar", "a"]);

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn prepend_child() {
        let mut a = test_tree(vec!["foo", "bar"]);
        let b = test_tree(vec!["a", "foo", "bar"]);

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn delete_middle_child() {
        let mut a = test_tree(vec!["foo", "a", "bar"]);
        let b = test_tree(vec!["foo", "bar"]);

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn delete_first_child() {
        let mut a = test_tree(vec!["foo", "a", "bar"]);
        let b = test_tree(vec!["a", "bar"]);

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn replace_parent() {
        let mut a = test_tree_vec(vec![("a", vec!["a", "b"])]);
        let b = test_tree_vec(vec![("b", vec!["a", "b"])]);

        println!("A:\n{}", a.root());
        println!("B:\n{}", b.root());

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    /// Test when the dest doesn't have children, and the node data is different
    #[traced_test]
    #[test]
    fn insert_children() {
        let nodes_a = vec![TestNode("a", vec![TestNode("1", vec![])])];

        let nodes_b = vec![TestNode(
            "a",
            vec![TestNode("b", vec![TestNode("1", vec![])])],
        )];

        let mut a = test_tree_node(nodes_a);
        let b = test_tree_node(nodes_b);

        println!("A:\n{}", a.root());
        println!("B:\n{}", b.root());

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }

    /// Test inserting a node, where the same subtree exists as children, as in this case where
    /// in tree B we're inserting node "b" between node "a" and it's child "1"
    ///
    /// A:
    ///
    /// ┏ 0: root [subtree_hash: 0x522555CA2FC30738 hash: 0x15D0E77B7348BFB3 depth:0 index:0 child_index:0]
    /// ┃ ┣ 1: a [subtree_hash: 0x3C45C5486A7CC2E5 hash: 0x7554A0C1CCFB0717 depth:1 index:0 child_index:0]
    /// ┃ ┃ ┣ 2: 1 [subtree_hash: 0x2AD65604E372F4A4 hash: 0x5EC23857437847F0 depth:2 index:0 child_index:0]
    /// ┃ ┃ ┃ ┗ 3: x [subtree_hash: 0xF9F30DD8B72F28BA hash: 0xF9F30DD8B72F28BA depth:3 index:0 child_index:0]
    /// ┗
    /// B:
    ///
    /// ┏ 0: root [subtree_hash: 0xE86FA358F4D2D5D6 hash: 0x43113E803A2EDC56 depth:0 index:0 child_index:0]
    /// ┃ ┣ 1: a [subtree_hash: 0x59B2061BD36A059F hash: 0x866203435F1BBBD4 depth:1 index:0 child_index:0]
    /// ┃ ┃ ┣ 2: b [subtree_hash: 0x634ACF58B06566AA hash: 0x1F774DFECBC0B6A1 depth:2 index:0 child_index:0]
    /// ┃ ┃ ┃ ┣ 3: 1 [subtree_hash: 0x2AD65604E372F4A4 hash: 0x5EC23857437847F0 depth:3 index:0 child_index:0]
    /// ┃ ┃ ┃ ┃ ┗ 4: x [subtree_hash: 0xF9F30DD8B72F28BA hash: 0xF9F30DD8B72F28BA depth:4 index:0 child_index:0]
    /// ┗

    #[traced_test]
    #[test]
    fn move_subtree() {
        let nodes_a = vec![TestNode(
            "a",
            vec![TestNode("1", vec![TestNode("x", vec![])])],
        )];

        let nodes_b = vec![TestNode(
            "a",
            vec![TestNode(
                "b",
                vec![TestNode("1", vec![TestNode("x", vec![])])],
            )],
        )];

        let mut a = test_tree_node(nodes_a);
        let b = test_tree_node(nodes_b);

        println!("A:\n{}", a.root());
        println!("B:\n{}", b.root());

        let mut diff = TreeDiff::new(a.root(), b.root());
        diff.diff().patch_tree(&mut a);

        println!("{}\n{}", "Patched Tree:".green(), a.root());
        assert_eq!(a, b);
    }
}
