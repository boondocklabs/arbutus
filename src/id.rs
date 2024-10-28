use std::{rc::Rc, sync::atomic::AtomicU64};

pub trait UniqueId:
    Copy + Clone + Ord + PartialEq + std::fmt::Debug + std::fmt::Display + std::hash::Hash
{
    type Output;
}

impl UniqueId for u64 {
    type Output = Self;
}

pub trait UniqueGenerator: Default + std::fmt::Debug + Clone + 'static {
    type Output: UniqueId;

    /// Generate a unique value
    fn generate(&self) -> Self::Output;
}

#[derive(Default, Debug, Clone)]
pub struct AtomicU64Generator {
    next_id: Rc<AtomicU64>,
}

impl UniqueGenerator for AtomicU64Generator {
    type Output = u64;

    fn generate(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Default, Debug, Clone)]
pub struct UuidGenerator;

#[derive(Copy, Clone, Debug, Hash)]
pub struct Uuid(uuid::Uuid);

impl UniqueId for Uuid {
    type Output = Self;
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Ord for Uuid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Uuid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for Uuid {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Uuid {}

impl UniqueGenerator for UuidGenerator {
    type Output = Uuid;

    fn generate(&self) -> Uuid {
        Uuid(uuid::Uuid::new_v4())
    }
}
