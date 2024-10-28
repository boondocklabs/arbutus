use std::hash::Hasher;

use xxhash_rust::xxh64::Xxh64;

use crate::{TreeNode as _, TreeNodeRef};

/// Recursively update the subtree hashes, starting from an inner node down to the root
pub fn update_subtree_hash<R>(mut node: R)
where
    R: TreeNodeRef + std::fmt::Debug + 'static,
{
    let mut hasher = Xxh64::new(0);

    if let Some(children) = node.node().children() {
        for child in children.iter() {
            let hash = child.node().get_subtree_hash();
            hasher.write_u64(hash);
        }
    }

    node.hash(&mut hasher);

    let new_hash = hasher.finish();

    node.node_mut().set_subtree_hash(new_hash);

    // If this node has a parent, recursively update the subtree hash of the parent
    if let Some(parent) = node.node().parent() {
        update_subtree_hash(parent.clone());
    }
}
