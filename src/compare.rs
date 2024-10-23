use std::hash::{Hash as _, Hasher};

use crate::{IndexedTree, Node, NodeRef, Tree};
use xxhash_rust::xxh64::Xxh64;

/// Tree Comparison

impl<R> PartialEq for Tree<R>
where
    R: NodeRef + 'static,
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

impl<R> PartialEq for IndexedTree<R>
where
    R: NodeRef,
{
    fn eq(&self, other: &Self) -> bool {
        self.tree() == other.tree()
    }
}

impl<R> Eq for IndexedTree<R> where R: NodeRef + std::hash::Hash + PartialEq + 'static {}
