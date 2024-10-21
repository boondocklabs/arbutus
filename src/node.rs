use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use tracing::debug;

use crate::{
    id::UniqueId,
    noderef::{NodeRef, NodeRefRc, NodeRefRef},
};

pub trait Node: Sized {
    type Data: std::fmt::Display;
    type Id: UniqueId;
    type DataRef<'b>: Deref<Target = Self::Data>
    where
        Self: 'b;
    type DataRefMut<'b>: DerefMut<Target = Self::Data>
    where
        Self: 'b;
    type NodeRef: NodeRef<Inner = Self>;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<Self::NodeRef>>) -> Self;

    fn with_parent(self, parent: Self::NodeRef) -> Self;

    fn id(&self) -> &Self::Id;

    fn data<'b>(&'b self) -> Self::DataRef<'b>;
    fn data_mut<'b>(&'b mut self) -> Self::DataRefMut<'b>;

    fn parent<'b>(&'b self) -> Option<&'b Self::NodeRef>;

    fn children<'b>(&'b self) -> Option<Ref<'b, Vec<Self::NodeRef>>>;

    /// Return the number of child nodes for this node
    fn num_children(&self) -> usize {
        self.children().map(|v| v.len()).unwrap_or(0)
    }

    fn add_child(&mut self, node: Self::NodeRef);
}

/// TreeNodeSimple provides no interior mutability
#[derive(Debug)]
pub struct TreeNodeSimple<Data, Id = crate::NodeId>
where
    Id: UniqueId + 'static,
    Data: std::fmt::Display + 'static,
{
    id: Id,
    data: Data,
    parent: Box<Option<NodeRefRef<Self>>>,
    children: Option<Vec<NodeRefRef<Self>>>,
}

impl<Data, Id> Node for TreeNodeSimple<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::fmt::Display + 'static,
{
    type Data = Data;
    type Id = Id;
    type DataRef<'b> = &'b Data;
    type DataRefMut<'b> = &'b mut Data;

    type NodeRef = NodeRefRef<Self>;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<NodeRefRef<Self>>>) -> Self {
        Self {
            id,
            data,
            children,
            parent: Box::new(None),
        }
    }

    fn with_parent(mut self, parent: Self::NodeRef) -> Self {
        self.parent = Box::new(Some(parent));
        self
    }

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn data<'b>(&'b self) -> Self::DataRef<'b> {
        todo!()
    }

    fn data_mut<'b>(&'b mut self) -> Self::DataRefMut<'b> {
        &mut self.data
    }

    fn children<'b>(&'b self) -> Option<Ref<'b, Vec<Self::NodeRef>>> {
        todo!()
    }

    fn add_child(&mut self, node: Self::NodeRef) {
        if let Some(children) = &mut self.children {
            children.push(node)
        }
    }

    fn parent<'b>(&'b self) -> Option<&'b Self::NodeRef> {
        (&*self.parent).as_ref()
    }
}

/// TreeNodeRefCell wraps each node in Rc and RefCell providing interior mutability
#[derive(Debug)]
pub struct TreeNodeRefCell<Data, Id = crate::NodeId>
where
    Id: UniqueId + 'static,
    Data: std::fmt::Display + 'static,
{
    id: Id,
    data: Rc<RefCell<Data>>,
    parent: Option<NodeRefRc<Self>>,
    children: Rc<Option<RefCell<Vec<NodeRefRc<Self>>>>>,
}

impl<Data, Id> Node for TreeNodeRefCell<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::fmt::Display + 'static,
{
    type Data = Data;
    type Id = Id;
    type DataRef<'b> = Ref<'b, Self::Data>;
    type DataRefMut<'b> = RefMut<'b, Self::Data>;
    type NodeRef = NodeRefRc<Self>;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<Self::NodeRef>>) -> Self {
        let children = children
            .map(|children| RefCell::new(children.into_iter().map(|child| child).collect()));

        debug!("Created Node ID {}", id);

        TreeNodeRefCell {
            id,
            data: Rc::new(RefCell::new(data)),
            children: Rc::new(children),
            parent: None,
        }
    }

    fn with_parent(mut self, parent: Self::NodeRef) -> Self {
        self.parent = Some(parent);
        self
    }

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn data<'b>(&'b self) -> Self::DataRef<'b> {
        (*self.data).borrow()
    }

    fn data_mut<'b>(&'b mut self) -> Self::DataRefMut<'b> {
        self.data.try_borrow_mut().unwrap()
    }

    fn children<'b>(&'b self) -> Option<Ref<'b, Vec<Self::NodeRef>>> {
        if let Some(children) = &*self.children {
            Some(children.borrow())
        } else {
            None
        }
    }

    fn add_child(&mut self, node: NodeRefRc<Self>)
    where
        Id: 'static,
        Data: 'static,
    {
        if self.children.is_none() {
            debug!(
                "Adding first child {} to node {}",
                node.node().id(),
                self.id()
            );
            self.children = Rc::new(Some(RefCell::new(vec![node])))
        } else {
            debug!("Adding child {} to node {}", node.node().id(), self.id());
            let r = self.children.as_ref();
            let r = r.as_ref().unwrap();
            let mut r = r.borrow_mut();
            r.push(node);
        }
    }

    fn parent<'b>(&'b self) -> Option<&'b Self::NodeRef> {
        self.parent.as_ref()
    }
}

impl<Data, Id> Clone for TreeNodeRefCell<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::fmt::Display + 'static,
{
    fn clone(&self) -> Self {
        TreeNodeRefCell {
            id: self.id.clone(),
            data: self.data.clone(),
            children: self.children.clone(),
            parent: self.parent.clone(),
        }
    }
}

impl<Data, Id> TreeNodeRefCell<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::fmt::Display,
{
}

#[cfg(test)]
mod tests {
    use tracing::info;
    use tracing_test::traced_test;

    use crate::{
        index::{BTreeIndex, TreeIndex},
        NodeId, NodeRefRc, Tree, TreeBuilder,
    };

    use super::TreeNodeRefCell;

    #[derive(Debug)]
    #[allow(unused)]
    enum TestError {
        Fail,
    }

    #[derive(Debug, Clone)]
    #[allow(unused)]
    enum TestData {
        Foo,
        Bar,
        Baz,
        String(String),
    }

    impl std::fmt::Display for TestData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Default for TestData {
        fn default() -> Self {
            TestData::Foo
        }
    }

    /// Create a simple tree for tests
    fn simple_tree<'a>(
    ) -> Result<Option<Tree<NodeRefRc<TreeNodeRefCell<TestData, NodeId>>>>, TestError> {
        TreeBuilder::<TestData, TestError>::new()
            .root(TestData::Foo, |foo| {
                foo.child(TestData::Bar, |bar| {
                    bar.child(TestData::String("Hello".into()), |s| {
                        s.child(TestData::String("World".into()), |_| Ok(()))?;
                        Ok(())
                    })?;
                    bar.child(TestData::Baz, |_baz| Ok(()))?;
                    Ok(())
                })?;

                foo.child(TestData::Baz, |_baz| Ok(()))?;
                foo.child(TestData::Baz, |_baz| Ok(()))?;
                foo.child(TestData::Baz, |_baz| Ok(()))?;
                foo.child(TestData::Baz, |_baz| Ok(()))?;

                Ok(())
            })?
            .done()
    }

    type R = NodeRefRc<TreeNodeRefCell<TestData, NodeId>>;

    #[derive(Debug)]
    struct App {
        _tree: Tree<R>,
        _index: BTreeIndex<R>,
    }

    #[traced_test]
    #[test]
    fn test_tree() {
        let tree = simple_tree().unwrap().unwrap();

        let index = BTreeIndex::from_tree(&tree);

        let app = App {
            _tree: tree,
            _index: index,
        }; // index };
        info!("{app:#?}");
    }
}
