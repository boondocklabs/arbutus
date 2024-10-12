use id::UniqueGenerator;

mod id;
mod index;
mod iterator;
mod node;

pub use node::Tree;

pub type IdGenerator = id::AtomicU64Generator;
pub type NodeId = <IdGenerator as UniqueGenerator>::Output;
