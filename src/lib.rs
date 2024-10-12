//! # Arbutus
//!
//! A tree data structure library for Rust.
//!
//! ## Overview
//!
//! Arbutus provides a high-level API for constructing and manipulating trees,
//! along with support for indexing and querying. The library focuses on simplicity,
//! flexibility, and performance.

use id::UniqueGenerator;

mod builder;
mod id;
mod index;
mod iterator;
mod node;
mod tree;

pub use builder::*;
pub use tree::IndexedTree;
pub use tree::Tree;

pub type IdGenerator = id::AtomicU64Generator;
pub type NodeId = <IdGenerator as UniqueGenerator>::Output;
