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
mod display;
mod hash;
mod id;
mod index;
mod iterator;
mod node;
mod noderef;
mod tree;

pub use builder::*;
pub use hash::{NodeHash, TreeHashIndex};
pub use id::*;
pub use node::{Node, TreeNodeRefCell, TreeNodeSimple};
pub use noderef::{NodeRef, NodeRefRc, NodeRefRef};
pub use tree::IndexedTree;
pub use tree::Tree;

pub type IdGenerator = id::AtomicU64Generator;
pub type NodeId = <IdGenerator as UniqueGenerator>::Output;
