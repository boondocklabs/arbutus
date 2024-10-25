//! A module providing builders for constructing trees and nodes.
//!
//! The `NodeBuilder` and `TreeBuilder` types enable building tree structures in a composable way.
//!

use std::marker::PhantomData;

use tracing::{debug, debug_span};

use crate::{
    id::UniqueGenerator,
    node::{refcell, TreeNode},
    NodeDepth, NodeIndex, Tree, TreeNodeRef,
};

type DefaultNodeRef<T> = crate::noderef::rc::NodeRef<T>;
type DefaultNode<Data, IdGen> = refcell::Node<Data, <IdGen as UniqueGenerator>::Output>;

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
    N: TreeNode<Id = G::Output, NodeRef = R>,
    R: TreeNodeRef<Inner = N>,
{
    // NodeRef of this node
    node_ref: &'a mut R,
    // UniqueGenerator handle
    idgen: &'a mut G,
    // Tree depth of this node
    depth: NodeDepth,
    // Horizontal index of this node
    index: NodeIndex,

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
    N: TreeNode<Id = G::Output, NodeRef = R>,
    R: TreeNodeRef<Inner = N>,
{
    /// Creates a new `NodeBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `node`: The parent node to build children for.
    /// * `idgen`: The ID generator to use for child nodes.
    pub fn new(node_ref: &'a mut R, idgen: &'a mut G, depth: NodeDepth, index: NodeIndex) -> Self {
        Self {
            node_ref,
            idgen,
            depth,
            index,
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
    {
        // Get the current number of children of this node to determine the node index
        let child_index = self.node_ref.node().num_children();

        // Generate a new ID for this child
        let id = self.idgen.generate();

        // Create a new node for this child
        let node = N::new(id, data, None).with_parent(self.node_ref.clone());
        let mut child_node_ref = R::new(node);
        let mut node_builder = NodeBuilder::<D, E, G, N, R>::new(
            &mut child_node_ref,
            &mut self.idgen,
            self.depth + 1,
            self.index + child_index,
        );

        // Call the supplied closure with the NodeBuilder to add this node's children
        f(&mut node_builder)?;

        // Create a new NodeRef for this child node

        self.node_ref.node_mut().push_child(child_node_ref);
        Ok(())
    }

    pub fn node<'b>(&'b mut self) -> &'b R {
        &self.node_ref
    }

    pub fn node_mut<'b>(&'b mut self) -> &'b mut R {
        &mut self.node_ref
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
/// use arbutus::{TreeBuilder, node::refcell::Node};
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
    N: TreeNode<Id = G::Output, NodeRef = R>,
    R: TreeNodeRef<Inner = N>,
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
    N: TreeNode<Id = G::Output, NodeRef = R>,
    R: TreeNodeRef<Inner = N>,
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
            debug_span,
            _phantom: (PhantomData, PhantomData, PhantomData),
        }
    }

    /// Returns the constructed tree when finished building it.
    pub fn done(self) -> Result<Option<Tree<R, G>>, E> {
        self.debug_span.in_scope(|| {
            debug!("Finished build tree");

            if let Some(root) = self.root {
                Ok(Some(Tree::from_node(root, Some(self.idgen))))
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
        N: TreeNode<NodeRef = R, Id = G::Output>,
        R: TreeNodeRef<Inner = N> + std::fmt::Debug,
    {
        let id = self.idgen.generate();

        self.debug_span.in_scope(|| {
            let node = TreeNode::new(id, data, None);
            let mut node_ref = TreeNodeRef::new(node);

            let mut node_builder =
                NodeBuilder::<D, E, G, N, R>::new(&mut node_ref, &mut self.idgen, 0, 0);

            // Call the supplied closure with the NodeBuilder to add this node's children
            f(&mut node_builder)?;

            if self.root.is_none() {
                debug!("Added root");
                self.root = Some(node_ref);
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

        #[derive(Debug, Clone, Hash)]
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
                foo.child(TestData::Bar, |bar| bar.child(TestData::Baz, |_| Ok(())))?;
                foo.child(TestData::String("Hello".into()), |_| Ok(()))?;

                Ok(())
            })
            .unwrap()
            .done()
            .unwrap()
            .unwrap();

        println!("{}", tree.root());
    }

    #[test]
    fn test_indices() {
        #[derive(Debug)]
        #[allow(unused)]
        enum MyError {
            Fail(String),
        }

        #[derive(Debug, Clone, Hash)]
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
                assert_eq!(foo.depth, 0);
                assert_eq!(foo.index, 0);
                foo.child(TestData::Bar, |bar| {
                    assert_eq!(bar.depth, 1);
                    assert_eq!(bar.index, 0);
                    bar.child(TestData::Baz, |baz| {
                        assert_eq!(baz.depth, 2);
                        assert_eq!(baz.index, 0);
                        Ok(())
                    })
                })?;
                foo.child(TestData::String("Hello".into()), |s| {
                    assert_eq!(s.depth, 1);
                    assert_eq!(s.index, 1);
                    Ok(())
                })?;

                Ok(())
            })
            .unwrap()
            .done()
            .unwrap()
            .unwrap();

        println!("{}", tree.root());
    }
}
