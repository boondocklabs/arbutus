use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash as _, Hasher as _},
};

use crate::{
    iterator::NodePosition, noderef::NodeRefId, IndexedTree, TreeNode, TreeNodeRef, UniqueGenerator,
};

/// NodeHash represents a hash value for a [`Node`] in a [`Tree`]
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum NodeHash {
    // Hash value depends on the position of the node in the tree
    Positional {
        // Position of the node
        position: NodePosition,
        // The hash value of this node
        hash: u64,
    },

    // Hash value indepdenent of the position in the tree
    Independent {
        // The hash value
        hash: u64,
    },
}

impl NodeHash {
    /// Get the DefaultHasher hash value for this NodeHash
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl std::fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeHash::Positional { position, hash } => f
                .debug_struct("NodeHash::Positional")
                .field("position", &position)
                .field("hash", &format_args!("0x{:X}", hash))
                .finish(),
            NodeHash::Independent { hash } => f
                .debug_struct("NodeHash::Indepdenent")
                .field("hash", &format_args!("0x{:X}", hash))
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct TreeHashIndex<R: TreeNodeRef> {
    forward: HashMap<NodeRefId<R>, NodeHash>,
    inverted: HashMap<NodeHash, NodeRefId<R>>,

    // Unique hashes in this tree
    unique: HashSet<NodeHash>,

    // Node position index
    position: HashMap<NodePosition, NodeRefId<R>>,
}

impl<R: TreeNodeRef> Default for TreeHashIndex<R> {
    fn default() -> Self {
        Self {
            forward: HashMap::new(),
            inverted: HashMap::new(),
            unique: HashSet::new(),
            position: HashMap::new(),
        }
    }
}

impl<R: TreeNodeRef> TreeHashIndex<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tree<G>(tree: &IndexedTree<R, G>) -> Self
    where
        G: UniqueGenerator<Output = NodeRefId<R>> + 'static,
    {
        let mut index = Self::new();

        for node in tree.root() {
            let node_id = node.node().id();
            let node_position = node.position();

            // Get a hash of the node
            let node_hash = node.node().xxhash();

            // Wrap the hash in a NodeHash::Positional, to make this hash value
            // positional depdentent. The hash of the NodeHash with the same hash field
            // will subsequently hash to a different value than the same node contents
            // at a different position in the tree.
            let nodehash = NodeHash::Positional {
                position: *node_position,
                hash: node_hash,
            };

            // Insert into indexes
            index.inverted.insert(nodehash.clone(), node_id.clone());
            index.forward.insert(node_id.clone(), nodehash.clone());
            index.unique.insert(nodehash.clone());

            index.position.insert(*node_position, node_id);
        }

        /*
        let inverted: HashMap<NodeHash, NodeRefId<R>> = tree
            .root()
            .into_iter()
            .map(|node| {
                let h = node.node().xxhash_with(&[&node.depth(), &node.index()]);
                //.xxhash_children_with(&[&node.depth(), &node.index()]);

                let nodehash = NodeHash::Positional {
                    position: *node.position(),
                    hash: h,
                };
                (nodehash, node.node().id())
            })
            .collect();
        */

        /*
        let forward: HashMap<NodeRefId<R>, NodeHash> = inverted
            .iter()
            .map(|(hash, id)| (id.clone(), hash.clone()))
            .collect();
        */

        /*
        let unique: HashSet<NodeHash> =
            inverted.keys().map(|node_hash| node_hash.clone()).collect();
        */

        index
    }

    /// Get the NodeId for the provided [`NodeHash`]
    pub fn get_id(&self, hash: &NodeHash) -> Option<NodeRefId<R>> {
        self.inverted.get(hash).map(|id| *id)
    }

    /// Get the [`NodeId`] for the provided [`NodePosition`]
    pub fn get_position_id(&self, position: &NodePosition) -> Option<NodeRefId<R>> {
        self.position.get(position).map(|id| *id)
    }

    /// Get the [`NodeHash`] for the provided [`NodeId`]
    pub fn get_hash(&self, id: &NodeRefId<R>) -> Option<NodeHash> {
        self.forward.get(id).map(|hash| hash.clone())
    }

    /// Get a set of unique hash values in the tree
    pub fn unique(&self) -> &HashSet<NodeHash> {
        &self.unique
    }

    /// Get the NodeId's from this tree which don't exist in some other tree
    pub fn diff_id(&self, other: &Self) -> HashSet<NodeRefId<R>> {
        let diff = self.unique.difference(other.unique());
        diff.map(|hash| self.get_id(hash).unwrap()).collect()
    }

    /// Get NodeHashes from this tree which don't exist in some other tree
    pub fn diff_hash<'b>(&'b self, other: &'b Self) -> HashSet<NodeHash> {
        self.unique
            .difference(other.unique())
            .map(|hash| hash.clone())
            .collect()
    }

    /// Insert a node into the index
    pub fn insert(&mut self, hash: NodeHash, id: NodeRefId<R>) -> Option<NodeRefId<R>> {
        self.inverted.insert(hash, id)
    }

    /// Remove a hash from the index
    pub fn remove_hash(&mut self, hash: &NodeHash) -> Option<NodeRefId<R>> {
        self.inverted.remove(hash)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use tracing_test::traced_test;

    use crate::{NodeHash, NodePosition};

    #[traced_test]
    #[test]
    fn node_hash() {
        let a = NodeHash::Positional {
            position: NodePosition {
                depth: 4,
                index: 3,
                child_index: 0,
            },
            hash: 0x863C27D43B7A0945,
        };

        let b = NodeHash::Positional {
            position: NodePosition {
                depth: 3,
                index: 1,
                child_index: 0,
            },
            hash: 0x7A107AE0F851BF94,
        };

        // Same node data hash (0x7A107...) but at a different location will hash to a different value
        let c = NodeHash::Positional {
            position: NodePosition {
                depth: 2,
                index: 1,
                child_index: 0,
            },
            hash: 0x7A107AE0F851BF94,
        };

        let ha = a.get_hash();
        let hb = b.get_hash();
        let hc = c.get_hash();

        assert_ne!(ha, hb);
        assert_ne!(ha, hc);
        assert_ne!(hb, ha);
        assert_ne!(hb, hc);
        assert_ne!(hc, ha);
        assert_ne!(hc, hb);
    }

    #[traced_test]
    #[test]
    fn node_hash_same() {
        let a = NodeHash::Positional {
            position: NodePosition {
                depth: 2,
                index: 1,
                child_index: 0,
            },
            hash: 0x7A107AE0F851BF94,
        };
        let b = NodeHash::Positional {
            position: NodePosition {
                depth: 2,
                index: 1,
                child_index: 0,
            },
            hash: 0x7A107AE0F851BF94,
        };

        // The same positional node hashes should produce the same hash value
        assert_eq!(a.get_hash(), b.get_hash());
    }

    #[traced_test]
    #[test]
    fn node_hash_child_index() {
        let a = NodeHash::Positional {
            position: NodePosition {
                depth: 2,
                index: 1,
                child_index: 0,
            },
            hash: 0x7A107AE0F851BF94,
        };

        let b = NodeHash::Positional {
            position: NodePosition {
                depth: 2,
                index: 1,
                child_index: 1,
            },
            hash: 0x7A107AE0F851BF94,
        };

        // Different child index should produce distinct hashes
        assert_ne!(a.get_hash(), b.get_hash());
    }

    #[traced_test]
    #[test]
    fn node_hash_position() {
        let mut check: HashSet<NodeHash> = HashSet::new();

        let mut count = 0;
        for depth in 0..100 {
            for index in 0..100 {
                for child_index in 0..10 {
                    let a = NodeHash::Positional {
                        position: NodePosition {
                            depth,
                            index,
                            child_index,
                        },
                        hash: 0x7A107AE0F851BF94,
                    };

                    assert!(check.insert(a) == true);
                    count += 1;
                }
            }
        }

        assert_eq!(count, check.len());
    }
}
