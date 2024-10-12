//! A module providing builders for constructing trees and nodes.
//!
//! The `NodeBuilder` and `TreeBuilder` types enable building tree structures in a composable way.
//!

use std::marker::PhantomData;

use tracing::{debug, debug_span};

use crate::{
    id::UniqueGenerator,
    node::{NodeRef, TreeNode},
};

/// A builder for constructing children from a parent node.
///
/// The `NodeBuilder` type provides methods for adding child nodes to the current parent node.
/// It is designed to be used with the `TreeBuilder` type.
///
/// # Examples
///
/// ```
/// use arbutus::{NodeBuilder, TreeNode};
/// let mut builder = NodeBuilder::new(node, idgen);
/// let child_builder = builder.child(data, |child| { /* add children */ });
/// ```
#[derive(Debug)]
pub struct NodeBuilder<'a, 'tree, Data, IdGen>
where
    IdGen: UniqueGenerator,
{
    node: &'a mut TreeNode<'tree, Data, IdGen::Output>,
    idgen: &'a mut IdGen,
    _phantom: PhantomData<Data>,
}

impl<'a, 'tree, Data, IdGen> NodeBuilder<'a, 'tree, Data, IdGen>
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
            _phantom: PhantomData,
        }
    }

    /// Adds a child to the current parent node.
    ///
    /// # Arguments
    ///
    /// * `data`: The data to associate with the child node.
    /// * `f`: A closure that takes the child builder and adds its own children.
    pub fn child<F>(&mut self, data: Data, f: F) -> &Self
    where
        F: Fn(&mut NodeBuilder<'_, 'tree, Data, IdGen>),
    {
        let id = self.idgen.generate();
        let mut node = TreeNode::<Data, IdGen::Output>::new(id, data, None);
        let mut node_builder = NodeBuilder::<Data, IdGen>::new(&mut node, &mut self.idgen);

        // Call the supplied closure with the NodeBuilder to add this node's children
        f(&mut node_builder);

        self.node.add_child(NodeRef::new(node));
        self
    }
}

/// A builder for constructing trees.
///
/// The `TreeBuilder` type provides methods for adding nodes and children to the tree structure.
///
/// # Examples
///
/// ```
/// use arbutus::{TreeBuilder, TreeNode};
/// let mut builder = TreeBuilder::new();
/// let root_builder = builder.root(data, |root| { /* add children */ });
/// let done = root_builder.done();
/// ```
#[derive(Debug)]
pub struct TreeBuilder<'tree, Data, IdGen = crate::IdGenerator>
where
    IdGen: UniqueGenerator,
{
    idgen: IdGen,
    root: Option<NodeRef<'tree, Data, IdGen::Output>>,
    current: Option<NodeRef<'tree, Data, IdGen::Output>>,
    debug_span: tracing::Span,
}

impl<'tree, Data, IdGen> TreeBuilder<'tree, Data, IdGen>
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
        }
    }

    /// Returns the constructed tree when finished building it.
    pub fn done(self) -> Option<NodeRef<'tree, Data, IdGen::Output>> {
        self.debug_span.in_scope(|| {
            debug!("Finished build tree");

            self.root
        })
    }

    /// Adds a root node to the tree and returns the updated builder.
    ///
    /// # Arguments
    ///
    /// * `data`: The data to associate with the root node.
    /// * `f`: A closure that takes the root builder and adds its own children.
    pub fn root<F>(mut self, data: Data, f: F) -> Self
    where
        Data: 'tree,
        F: Fn(&mut NodeBuilder<'_, 'tree, Data, IdGen>),
    {
        let id = self.idgen.generate();

        self.debug_span.in_scope(|| {
            let mut node = TreeNode::<Data, IdGen::Output>::new(id, data, None);

            let mut node_builder = NodeBuilder::<Data, IdGen>::new(&mut node, &mut self.idgen);

            // Call the supplied closure with the NodeBuilder to add this node's children
            f(&mut node_builder);

            let node = NodeRef::new(node);

            if self.root.is_none() {
                self.current = Some(node.clone());
                self.root = Some(node);
                debug!("Added root node");
            } else {
                debug!("Adding node as child of current")
                //self.current.unwrap().node().children().
            }
        });
        self
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
        enum TestData {
            Foo,
            Bar,
            String(String),
            Baz,
        }

        let tree = TreeBuilder::<TestData>::new()
            .root(TestData::Foo, |foo| {
                debug!("Foo builder closure");
                foo.child(TestData::Bar, |bar| {
                    debug!("Bar builder closure");
                    bar.child(TestData::Baz, |_| {});
                });
                foo.child(TestData::String("Hello".into()), |_| {});
            })
            .done();
        info!("{tree:#?}");
    }
}
