//! A module providing builders for constructing trees and nodes.
//!
//! The `NodeBuilder` and `TreeBuilder` types enable building tree structures in a composable way.
//!

use std::{cell::RefMut, marker::PhantomData};

use tracing::{debug, debug_span};

use crate::{
    id::UniqueGenerator,
    node::{NodeRef, TreeNode},
    Tree,
};

/// A builder for constructing children from a parent node.
///
/// The `NodeBuilder` type provides methods for adding child nodes to the current parent node.
/// It is designed to be used with the `TreeBuilder` type.
///
#[derive(Debug)]
pub struct NodeBuilder<'a, 'tree, Data, E, IdGen = crate::IdGenerator>
where
    IdGen: UniqueGenerator,
{
    node: &'a mut TreeNode<'tree, Data, IdGen::Output>,
    idgen: &'a mut IdGen,
    _phantom: (PhantomData<Data>, PhantomData<E>),
}

impl<'a, 'tree, Data, E, IdGen> NodeBuilder<'a, 'tree, Data, E, IdGen>
where
    IdGen: UniqueGenerator,
{
    /// Creates a new `NodeBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `node`: The parent node to build children for.
    /// * `idgen`: The ID generator to use for child nodes.
    pub fn new(node: &'a mut TreeNode<'tree, Data, IdGen::Output>, idgen: &'a mut IdGen) -> Self {
        debug!("Created new NodeBuilder for {}", node.id());
        Self {
            node,
            idgen,
            _phantom: (PhantomData, PhantomData),
        }
    }

    /// Adds a child to the current parent node.
    ///
    /// # Arguments
    ///
    /// * `data`: The data to associate with the child node.
    /// * `f`: A closure that takes the child builder and adds its own children.
    pub fn child<F>(&mut self, data: Data, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut NodeBuilder<'_, 'tree, Data, E, IdGen>) -> Result<(), E>,
    {
        let id = self.idgen.generate();
        let mut node = TreeNode::<Data, IdGen::Output>::new(id, data, None);
        let mut node_builder = NodeBuilder::<Data, E, IdGen>::new(&mut node, &mut self.idgen);

        // Call the supplied closure with the NodeBuilder to add this node's children
        f(&mut node_builder)?;

        self.node.add_child(NodeRef::new(node));
        Ok(())
    }

    /// Get a mutable reference to the data in this node
    pub fn data_mut<'b>(&'b mut self) -> RefMut<'b, Data> {
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
/// use arbutus::{TreeBuilder, TreeNode};
/// let mut builder = TreeBuilder::<MyData, MyError>::new();
/// let root_builder = builder.root("Root".to_string(), |root| { /* add children */ Ok(()) });
///
/// // Unwrap out of the error. Typically you would use `builder?.done()` to propagate errors up
/// let done = root_builder.unwrap().done();
/// ```
#[derive(Debug)]
pub struct TreeBuilder<'tree, Data, E, IdGen = crate::IdGenerator>
where
    IdGen: UniqueGenerator,
{
    idgen: IdGen,
    root: Option<NodeRef<'tree, Data, IdGen::Output>>,
    current: Option<NodeRef<'tree, Data, IdGen::Output>>,
    debug_span: tracing::Span,
    _phantom: PhantomData<E>,
}

impl<'tree, Data, E, IdGen> TreeBuilder<'tree, Data, E, IdGen>
where
    IdGen: UniqueGenerator,
{
    /// Creates a new `TreeBuilder` instance.
    pub fn new() -> Self {
        let debug_span = debug_span!("TreeBuilder");
        let _debug = debug_span.enter();

        debug!("Created new TreeBuilder");

        drop(_debug);

        Self {
            idgen: IdGen::default(),
            root: None,
            current: None,
            debug_span,
            _phantom: PhantomData,
        }
    }

    /// Returns the constructed tree when finished building it.
    pub fn done(self) -> Result<Option<Tree<'tree, Data, IdGen::Output>>, E> {
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
    pub fn root<F>(mut self, data: Data, f: F) -> Result<Self, E>
    where
        Data: std::fmt::Debug + 'tree,
        F: FnOnce(&mut NodeBuilder<'_, 'tree, Data, E, IdGen>) -> Result<(), E>,
    {
        let id = self.idgen.generate();

        self.debug_span.in_scope(|| {
            let mut node = TreeNode::<Data, IdGen::Output>::new(id, data, None);

            let mut node_builder = NodeBuilder::<Data, E, IdGen>::new(&mut node, &mut self.idgen);

            // Call the supplied closure with the NodeBuilder to add this node's children
            f(&mut node_builder)?;

            let node = NodeRef::new(node);

            if self.root.is_none() {
                debug!("Added root {node:#?}");
                self.current = Some(node.clone());
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
