//! A module providing builders for constructing trees and nodes.
//!
//! The `NodeBuilder` and `TreeBuilder` types enable building tree structures in a composable way.
//!

use std::marker::PhantomData;

use tracing::{debug, debug_span};

use crate::{
    id::UniqueGenerator,
    node::{Node, TreeNodeRefCell},
    NodeRef, NodeRefRc, Tree,
};

type DefaultNodeRef<T> = NodeRefRc<T>;
type DefaultNode<Data, IdGen> = TreeNodeRefCell<Data, <IdGen as UniqueGenerator>::Output>;

/// A builder for constructing children from a parent node.
///
/// The `NodeBuilder` type provides methods for adding child nodes to the current parent node.
/// It is designed to be used with the `TreeBuilder` type.
///
//#[derive(Debug)]
pub struct NodeBuilder<
    'a,
    D,
    E,
    G = crate::IdGenerator,
    N = DefaultNode<D, G>,
    R = DefaultNodeRef<N>,
> where
    G: UniqueGenerator,
    D: std::fmt::Display + 'static,
    N: Node,
    R: NodeRef,
{
    node: &'a mut N,
    idgen: &'a mut G,
    _phantom: (
        PhantomData<D>,
        PhantomData<E>,
        PhantomData<N>,
        PhantomData<R>,
    ),
}

impl<'a, D, E, G, N, R> NodeBuilder<'a, D, E, G, N, R>
where
    D: std::fmt::Display,
    G: UniqueGenerator,
    N: Node,
    R: NodeRef,
{
    /// Creates a new `NodeBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `node`: The parent node to build children for.
    /// * `idgen`: The ID generator to use for child nodes.
    pub fn new(node: &'a mut N, idgen: &'a mut G) -> Self {
        debug!("Created new NodeBuilder for {}", node.id());
        Self {
            node,
            idgen,
            _phantom: (PhantomData, PhantomData, PhantomData, PhantomData),
        }
    }

    /// Adds a child to the current node.
    ///
    /// # Arguments
    ///
    /// * `data`: The data to associate with the child node.
    /// * `f`: A closure that takes the child builder and adds its own children.
    pub fn child<F>(&mut self, data: N::Data, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut NodeBuilder<'_, D, E, G, N, R>) -> Result<(), E>,
        N: Node<Id = G::Output>,
    {
        let id = self.idgen.generate();
        let mut node = Node::new(id, data, None);
        let mut node_builder = NodeBuilder::<D, E, G, N, R>::new(&mut node, &mut self.idgen);

        // Call the supplied closure with the NodeBuilder to add this node's children
        f(&mut node_builder)?;

        let r = NodeRef::new(node);

        self.node.add_child(r);
        Ok(())
    }

    /// Get a mutable reference to the data in this node
    pub fn data_mut<'b>(&'b mut self) -> N::DataRefMut<'b> {
        self.node.data_mut()
    }
}

/// A builder for constructing trees.
///
/// The `TreeBuilder` type provides methods for adding nodes and children to the tree structure.
///
/// There is a `root` method on the builder to add an initial root node, which calls
/// the provided closure with a NodeBuilder that can be used to recursively build children of
/// the node. The closures expect a Result<(), E> to be returned, where E is your defined error
/// type. This allows errors within your closures to propagate.
///
/// # Examples
///
/// ```
/// type MyData = String;
/// type MyError = String;
///
/// use arbutus::{TreeBuilder, TreeNodeRefCell};
/// let mut builder = TreeBuilder::<MyData, MyError>::new();
/// let root_builder = builder.root("Root".to_string(), |root| { /* add children */ Ok(()) });
///
/// // Unwrap out of the error. Typically you would use `builder?.done()` to propagate errors up
/// let done = root_builder.unwrap().done();
/// ```
#[derive(Debug)]
pub struct TreeBuilder<D, E, G = crate::IdGenerator, N = DefaultNode<D, G>, R = DefaultNodeRef<N>>
where
    G: UniqueGenerator,
{
    idgen: G,
    root: Option<R>,
    debug_span: tracing::Span,
    _phantom: (PhantomData<E>, PhantomData<N>, PhantomData<D>),
}

impl<D, E, G, N, R> TreeBuilder<D, E, G, N, R>
where
    D: std::fmt::Display,
    G: UniqueGenerator,
    N: Node,
    R: NodeRef,
{
    /// Creates a new `TreeBuilder` instance.
    pub fn new() -> Self {
        let debug_span = debug_span!("TreeBuilder");
        let _debug = debug_span.enter();

        debug!("Created new TreeBuilder");

        drop(_debug);

        Self {
            idgen: G::default(),
            root: None,
            //current: None,
            debug_span,
            _phantom: (PhantomData, PhantomData, PhantomData),
        }
    }

    /// Returns the constructed tree when finished building it.
    pub fn done(self) -> Result<Option<Tree<R>>, E> {
        self.debug_span.in_scope(|| {
            debug!("Finished build tree");

            if let Some(root) = self.root {
                Ok(Some(Tree::from_nodes(root)))
            } else {
                Ok(None)
            }
        })
    }

    /// Adds a root node to the tree and returns the updated builder.
    ///
    /// # Arguments
    ///
    /// * `data`: The data to associate with the root node.
    /// * `f`: A closure that takes the root builder and adds its own children.
    pub fn root<F>(mut self, data: N::Data, f: F) -> Result<Self, E>
    where
        D: std::fmt::Debug + 'static,
        F: FnOnce(&mut NodeBuilder<'_, D, E, G, N, R>) -> Result<(), E>,
        N: Node<NodeRef = R, Id = G::Output>,
        R: NodeRef<Inner = N> + std::fmt::Debug,
    {
        let id = self.idgen.generate();

        self.debug_span.in_scope(|| {
            let mut node = Node::new(id, data, None);

            let mut node_builder = NodeBuilder::<D, E, G, N, R>::new(&mut node, &mut self.idgen);

            // Call the supplied closure with the NodeBuilder to add this node's children
            f(&mut node_builder)?;

            let node = NodeRef::new(node);

            if self.root.is_none() {
                debug!("Added root {node:#?}");
                //self.current = Some(node.clone());
                self.root = Some(node);
            } else {
                panic!("Root node already exists");
                //debug!("Adding node as child of current")
                //self.current.unwrap().node().children().
            }
            Ok(())
        })?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;
    use tracing_test::traced_test;

    use super::*;

    #[traced_test]
    #[test]
    fn test_builder() {
        #[derive(Debug)]
        #[allow(unused)]
        enum MyError {
            Fail(String),
        }

        #[derive(Debug)]
        #[allow(unused)]
        enum TestData {
            Foo,
            Bar,
            String(String),
            Baz,
        }

        impl std::fmt::Display for TestData {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        let tree = TreeBuilder::<TestData, MyError>::new()
            .root(TestData::Foo, |foo| {
                debug!("Foo builder closure");

                foo.child(TestData::Bar, |bar| {
                    debug!("Bar builder closure");
                    bar.child(TestData::Baz, |_| Ok(()))
                })?;

                foo.child(TestData::String("Hello".into()), |_| Ok(()))?;

                Ok(())
            })
            .unwrap()
            .done();
        info!("{tree:#?}");
    }
}
