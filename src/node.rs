use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    ops::Deref,
    rc::Rc,
};

use tracing::debug;

use crate::{display::TreeDisplay, iterator::NodeRefIter, NodeId};

#[derive(Debug)]
pub struct TreeNode<'tree, Data, Id = NodeId>
where
    Id: Clone + std::fmt::Display + 'tree,
    Data: 'tree,
{
    id: Id,
    data: Rc<RefCell<Data>>,
    children: Rc<Option<RefCell<Vec<NodeRef<'tree, Data, Id>>>>>,
}

impl<'tree, Data, Id> Clone for TreeNode<'tree, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    fn clone(&self) -> Self {
        TreeNode {
            id: self.id.clone(),
            data: self.data.clone(),
            children: self.children.clone(),
        }
    }
}

impl<'tree, Data, Id> TreeNode<'tree, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    pub fn new(id: Id, data: Data, children: Option<Vec<TreeNode<'tree, Data, Id>>>) -> Self {
        let children = children.map(|children| {
            RefCell::new(
                children
                    .into_iter()
                    .map(|child| NodeRef::new(child))
                    .collect(),
            )
        });

        debug!("Created Node ID {}", id);

        TreeNode {
            id,
            data: Rc::new(RefCell::new(data)),
            children: Rc::new(children),
        }
    }

    pub fn id(&self) -> Id {
        self.id.clone()
    }

    pub fn children(&self) -> Option<&RefCell<Vec<NodeRef<'tree, Data, Id>>>> {
        (*self.children).as_ref()
    }

    pub fn data<'b>(&'b self) -> Ref<'b, Data> {
        self.data.borrow()
    }

    pub fn data_mut<'b>(&'b self) -> RefMut<'b, Data> {
        self.data.borrow_mut()
    }

    pub fn add_child(&mut self, node: NodeRef<'tree, Data, Id>)
    where
        Id: 'static,
    {
        if self.children.is_none() {
            debug!(
                "Adding first child {} to node {}",
                node.borrow().id(),
                self.id()
            );
            self.children = Rc::new(Some(RefCell::new(vec![node])))
        } else {
            debug!("Adding child {} to node {}", node.borrow().id(), self.id());
            self.children().unwrap().borrow_mut().push(node);
        }
    }
}

pub struct NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display + 'node,
    Data: 'node,
{
    node_ref: Rc<RefCell<TreeNode<'node, Data, Id>>>,
}

impl<'node, Data, Id> std::fmt::Display for NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display + 'node,
    Data: std::fmt::Debug + std::fmt::Display + 'node,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TreeDisplay::format(self, f, |data, f| write!(f, "{}", data))
    }
}

impl<'node, Data, Id> std::fmt::Debug for NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display + 'node,
    Data: std::fmt::Debug + 'node,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        TreeDisplay::format(self, f, |data, f| write!(f, "{:?}", data))
    }
}

impl<'node, Data, Id> Clone for NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    fn clone(&self) -> Self {
        Self {
            node_ref: self.node_ref.clone(),
        }
    }
}

impl<'node, Data, Id> NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    pub fn new(node: TreeNode<'node, Data, Id>) -> Self {
        Self {
            node_ref: Rc::new(RefCell::new(node)),
        }
    }

    pub fn node<'a>(&'a self) -> Ref<'a, TreeNode<'node, Data, Id>> {
        self.node_ref.borrow()
    }

    pub fn with_data<'a, R, E, F>(&'a self, f: F) -> Result<R, E>
    where
        F: FnOnce(Ref<Data>) -> Result<R, E>,
    {
        let node = self.node_ref.borrow();
        let data = node.data.borrow();
        f(data)
    }

    pub fn with_data_mut<'a, R, E, F>(&'a self, f: F) -> Result<R, E>
    where
        F: FnOnce(RefMut<Data>) -> Result<R, E>,
    {
        let node = self.node_ref.borrow();
        let data = node.data.borrow_mut();
        f(data)
    }

    pub fn for_each<E, F>(&self, f: F) -> Result<(), E>
    where
        F: Fn(usize, NodeRef<Data, Id>) -> Result<(), E>,
    {
        // Create a stack with depth 0, and the initial node
        let mut stack: VecDeque<(usize, NodeRef<Data, Id>)> = VecDeque::from([(0, self.clone())]);

        loop {
            let current = stack.pop_front();
            if let None = current {
                break;
            };
            let node = current.map(|node| {
                node.1.node().children().map(|children| {
                    children
                        .borrow()
                        .iter()
                        .rev()
                        .for_each(|child| stack.push_front((node.0 + 1, (*child).clone())))
                });
                node
            });

            if let Some(node) = node {
                f(node.0, node.1)?
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> NodeRefIter<'node, Data, Id> {
        NodeRefIter::new((*self).clone())
    }
}

impl<'node, Data, Id> Deref for NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display + 'static,
{
    type Target = RefCell<TreeNode<'node, Data, Id>>;

    fn deref(&self) -> &Self::Target {
        &*self.node_ref
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;
    use tracing_test::traced_test;

    use crate::{
        index::{BTreeIndex, TreeIndex},
        NodeId, Tree, TreeBuilder,
    };

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

    impl Default for TestData {
        fn default() -> Self {
            TestData::Foo
        }
    }

    /// Create a simple tree for tests
    fn simple_tree<'a>() -> Result<Option<Tree<'a, TestData, NodeId>>, TestError> {
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

    #[derive(Debug)]
    struct App<'tree, 'index>
    where
        'tree: 'index,
    {
        _tree: Tree<'tree, TestData, NodeId>,
        _index: BTreeIndex<'index, TestData>,
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
