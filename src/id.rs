use std::sync::atomic::AtomicU64;

#[derive(Debug)]
pub struct Ident<T = AtomicU64Generator>(T);

impl<T> Ident<T>
where
    T: UniqueGenerator,
{
    pub fn new(id: T::Output) -> Ident<T::Output> {
        Ident(id)
    }

    pub fn generate(gen: &mut T) -> Ident<T::Output> {
        Ident(gen.generate())
    }
}

pub trait UniqueGenerator: std::fmt::Debug {
    type Output: std::fmt::Debug;
    /// Generate a unique value
    fn generate(&mut self) -> Self::Output;
}

#[derive(Default, Debug)]
pub struct AtomicU64Generator {
    next_id: AtomicU64,
}

impl UniqueGenerator for AtomicU64Generator {
    type Output = u64;

    fn generate(&mut self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Default, Debug)]
pub(crate) struct UuidGenerator;

impl UniqueGenerator for UuidGenerator {
    type Output = uuid::Uuid;

    fn generate(&mut self) -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }
}
