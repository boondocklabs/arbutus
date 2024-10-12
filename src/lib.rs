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
