use std::hash::{Hash as _, Hasher};

use crate::{noderef::NodeRefId, IndexedTree, Tree, TreeNode, TreeNodeRef, UniqueGenerator};
use xxhash_rust::xxh64::Xxh64;

/// Tree Comparison

impl<R, G> PartialEq for Tree<R, G>
where
    R: TreeNodeRef + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        let mut hasher_self = Xxh64::new(0);
        let mut hasher_other = Xxh64::new(0);

        for node in self.root().into_iter() {
            node.node().hash_children(&mut hasher_self);
            node.node().hash(&mut hasher_self);
        }

        for node in other.root().into_iter() {
            node.node().hash_children(&mut hasher_other);
            node.node().hash(&mut hasher_other);
        }

        let self_hash = hasher_self.finish();
        let other_hash = hasher_other.finish();

        self_hash == other_hash
    }
}

impl<R, G> PartialEq for IndexedTree<R, G>
where
    R: TreeNodeRef,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.tree() == other.tree()
    }
}

impl<R, G> Eq for IndexedTree<R, G>
where
    R: TreeNodeRef + std::hash::Hash + PartialEq + 'static,
    G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
{
}
