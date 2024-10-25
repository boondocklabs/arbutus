/// Simple reference noderef. Does not allow cloning.
#[derive(Debug, Hash)]
pub struct NodeRefRef<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    node: T,
}

impl<T> NodeRefInternal<T> for NodeRefRef<T> where T: Node<NodeRef = Self> + 'static {}

impl<T> NodeRef for NodeRefRef<T>
where
    T: Node<NodeRef = Self> + 'static,
{
    type Inner = T;
    type InnerRef<'b> = &'b Self::Inner;
    type InnerRefMut<'b> = &'b mut Self::Inner;
    type Data = T::Data;
    type DataRef<'b> = T::DataRef<'b>;
    type DataRefMut<'b> = T::DataRefMut<'b>;

    fn new(node: T) -> Self {
        Self { node }
    }

    fn node<'b>(&'b self) -> Self::InnerRef<'b> {
        &self.node
    }

    fn node_mut<'b>(&'b mut self) -> Self::InnerRefMut<'b> {
        &mut self.node
    }

    fn with_data<'b, R, E, F>(&'b self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRef<'_>) -> Result<R, E>,
    {
        f(self.node.data())
    }

    fn with_data_mut<'b, R, E, F>(&'b mut self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self::DataRefMut<'_>) -> Result<R, E>,
    {
        f(self.node.data_mut())
    }

    fn for_each<E, F>(&self, f: F) -> Result<(), E>
    where
        F: Fn(usize, Self) -> Result<(), E>,
    {
        // Create a stack with depth 0, and the initial node
        let mut stack: VecDeque<(usize, Self)> = VecDeque::from([(0, self.clone())]);

        loop {
            let current = stack.pop_front();
            if let None = current {
                break;
            };
            let node = current.map(|node| {
                node.1.node().children().map(|children| {
                    children
                        .iter()
                        .rev()
                        .for_each(|child| stack.push_front((node.0 + 1, child.clone())))
                });
                node
            });

            if let Some(node) = node {
                f(node.0, node.1)?
            }
        }
        Ok(())
    }
}

impl<T> Deref for NodeRefRef<T>
where
    T: Node<NodeRef = Self>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> Clone for NodeRefRef<T>
where
    T: Node<NodeRef = Self>,
{
    fn clone(&self) -> Self {
        panic!("Cloning of node references is not supported with NodeRefRef nodes. Use one of the Rc/Arc smart pointer NodeRef types.");
    }
}

impl<N> IntoIterator for NodeRefRef<N>
where
    N: Node<NodeRef = Self> + 'static,
{
    type Item = IterNode<Self>;
    type IntoIter = NodeRefIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        NodeRefIter::new(self)
    }
}
