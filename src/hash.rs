use std::collections::{HashMap, HashSet};

use crate::{iterator::NodePosition, noderef::NodeRefId, IndexedTree, Node, NodeRef};

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
pub struct TreeHashIndex<R: NodeRef> {
    forward: HashMap<NodeRefId<R>, NodeHash>,
    inverted: HashMap<NodeHash, NodeRefId<R>>,

    // Unique hashes in this tree
    unique: HashSet<NodeHash>,
}

impl<R: NodeRef> Default for TreeHashIndex<R> {
    fn default() -> Self {
        Self {
            forward: HashMap::new(),
            inverted: HashMap::new(),
            unique: HashSet::new(),
        }
    }
}

impl<R: NodeRef> TreeHashIndex<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tree(tree: &IndexedTree<R>) -> Self {
        let mut inverted: HashMap<NodeHash, NodeRefId<R>> = HashMap::new();
        let mut forward: HashMap<NodeRefId<R>, NodeHash> = HashMap::new();
        let mut unique: HashSet<NodeHash> = HashSet::new();

        for node in tree.root() {
            let node_id = node.node().id();
            let position = node.position();

            // Get a hash of the node
            let node_hash = node.node().xxhash();

            // Wrap the hash in a NodeHash::Positional, to make this hash value
            // positional depdentent. The hash of the NodeHash with the same hash field
            // will subsequently hash to a different value than the same node contents
            // at a different position in the tree.
            let nodehash = NodeHash::Positional {
                position: *position,
                hash: node_hash,
            };

            // Insert into indexes
            inverted.insert(nodehash.clone(), node_id.clone());
            forward.insert(node_id.clone(), nodehash.clone());
            unique.insert(nodehash.clone());
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

        Self {
            inverted,
            unique,
            forward,
        }
    }

    /// Get the NodeId for the provided [`NodeHash`]
    pub fn get_id(&self, hash: &NodeHash) -> Option<NodeRefId<R>> {
        self.inverted.get(hash).map(|id| *id)
    }

    /// Get the [`NodeHash`] for the provided [`NodeId`]
    pub fn get_hash(&self, id: &NodeRefId<R>) -> Option<NodeHash> {
        self.forward.get(id).map(|hash| hash.clone())
    }

    /// Get unique set of hash values in the tree
    pub fn unique(&self) -> &HashSet<NodeHash> {
        &self.unique
    }

    /// Get the NodeId's from this tree which don't exist in some other tree
    pub fn diff_id(&self, other: &Self) -> HashSet<NodeRefId<R>> {
        let diff = self.unique.difference(other.unique());
        diff.map(|hash| self.get_id(hash).unwrap()).collect()
    }

    /// Get NodeHashes from this tree which don't exist in some other tree
    pub fn diff_hash<'b>(&'b self, other: &'b Self) -> HashSet<&'b NodeHash> {
        self.unique.difference(other.unique()).collect()
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
