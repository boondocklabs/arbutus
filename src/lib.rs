//! # Arbutus
//!
//! A tree data structure library for Rust.
//!
//! ## Overview
//!
//! Arbutus provides a high-level API for constructing and manipulating trees,
//! along with support for indexing and querying. The library focuses on simplicity,
//! flexibility, and performance.

mod builder;
mod compare;
mod diff;
mod display;
mod edit;
mod hash;
mod id;
mod index;
mod iterator;
mod tree;

#[cfg(test)]
pub(crate) mod test;

pub mod node;
pub mod noderef;

pub use builder::*;
pub use id::*;
pub use iterator::NodePosition;
pub use tree::IndexedTree;
pub use tree::Tree;

pub use node::TreeNode;
pub use noderef::TreeNodeRef;

pub use iterator::leaf;

pub use diff::TreeDiff;

pub type NodeDepth = usize;
pub type NodeIndex = usize;

pub type IdGenerator = id::AtomicU64Generator;
pub type NodeId = <IdGenerator as UniqueGenerator>::Output;
