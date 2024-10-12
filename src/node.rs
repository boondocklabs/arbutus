use std::{
    cell::{Ref, RefCell},
    collections::VecDeque,
    ops::Deref,
    rc::Rc,
};

use tracing::debug;

use crate::NodeId;

#[derive(Debug)]
pub struct TreeNode<'tree, Data, Id = NodeId>
where
    Id: Clone + std::fmt::Display,
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

#[derive(Debug)]
pub struct NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display,
{
    node_ref: Rc<RefCell<TreeNode<'node, Data, Id>>>,
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

    pub fn node<'b>(&'b self) -> Ref<'b, TreeNode<'node, Data, Id>> {
        self.node_ref.borrow()
    }

    pub fn each<F>(&self, mut f: F)
    where
        F: FnMut(NodeRef<Data, Id>),
    {
        let mut stack: VecDeque<NodeRef<Data, Id>> = VecDeque::from([self.clone()]);

        loop {
            let current = stack.pop_front();
            if let None = current {
                break;
            };
            let node = current.map(|node| {
                node.node().children().map(|children| {
                    _ = children
                        .borrow()
                        .iter()
                        .map(|child| stack.push_front((*child).clone()));
                });
                node
            });

            if let Some(node) = node {
                f(node)
            }
        }
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

impl<'iter, Data, Id> IntoIterator for NodeRef<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display + 'static,
    Data: Clone + std::fmt::Debug + 'static,
{
    type Item = NodeRef<'iter, Data, Id>;
    type IntoIter = TreeNodeIter<'iter, Data, Id>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        TreeNodeIter {
            stack: VecDeque::from([self]),
        }
    }
}

pub struct TreeNodeIter<'iter, Data, Id>
where
    Id: Clone + std::fmt::Display + 'static,
{
    stack: VecDeque<NodeRef<'iter, Data, Id>>,
}

impl<'iter, Data, Id> Iterator for TreeNodeIter<'iter, Data, Id>
where
    Data: std::fmt::Debug + 'static,
    Id: Clone + std::fmt::Display + 'static,
{
    type Item = NodeRef<'iter, Data, Id>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            node.node().children().map(|children| {
                _ = children.borrow().iter().map(|child| {
                    self.stack.push_front((*child).clone());
                });
            });

            node
        })
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

    use super::NodeRef;

    #[derive(Debug, Clone)]
    #[allow(unused)]
    enum TestData {
        Foo,
        Bar,
        String(String),
    }

    impl Default for TestData {
        fn default() -> Self {
            TestData::Foo
        }
    }

    /// Create a simple tree for tests
    fn simple_tree<'a>() -> NodeRef<'a, TestData, NodeId> {
        TreeBuilder::<TestData>::new()
            .root(TestData::Foo, |foo| {
                foo.child(TestData::Bar, |bar| {
                    bar.child(TestData::String("Hello".into()), |_| {});
                });
            })
            .done()
            .unwrap()
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
        let nodes = simple_tree();

        let tree = Tree::from_nodes(nodes);
        let index = BTreeIndex::from_tree(&tree);

        let app = App {
            _tree: tree,
            _index: index,
        }; // index };
        info!("{app:#?}");
    }
}
