use std::{
    cell::{Ref, RefCell},
    collections::VecDeque,
    ops::Deref,
    rc::Rc,
};

use crate::NodeId;

#[derive(Debug, Clone)]
pub struct TreeNode<'tree, Data, Id = NodeId> {
    id: Id,
    data: Rc<RefCell<Data>>,
    children: Option<Vec<NodeRef<'tree, Data, Id>>>,
    //_phantom: PhantomData<&'tree Data>,
}

impl<'tree, Data, Id> TreeNode<'tree, Data, Id>
where
    Id: Clone + 'static,
    Data: Clone + 'static,
{
    pub fn new(id: Id, data: Data, children: Option<Vec<TreeNode<'tree, Data, Id>>>) -> Self {
        let children = children.map(|children| {
            children
                .into_iter()
                .map(|child| NodeRef::new(child))
                .collect()
        });

        TreeNode {
            id,
            data: Rc::new(RefCell::new(data)),
            children,
        }
    }

    pub fn id(&self) -> Id {
        self.id.clone()
    }

    pub fn children(&self) -> Option<&Vec<NodeRef<'tree, Data, Id>>> {
        self.children.as_ref()
    }

    pub fn data<'b>(&'b self) -> Ref<'b, Data> {
        self.data.borrow()
    }
}

#[derive(Debug, Clone)]
pub struct NodeRef<'node, Data, Id> {
    node_ref: Rc<RefCell<TreeNode<'node, Data, Id>>>,
}

impl<'node, Data, Id> NodeRef<'node, Data, Id>
where
    Id: Clone + 'static,
    Data: Clone + 'static,
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
                node.node()
                    .children()
                    .map(|children| children.iter().map(|child| stack.push_front(child.clone())));
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
    Id: 'static,
{
    type Target = RefCell<TreeNode<'node, Data, Id>>;

    fn deref(&self) -> &Self::Target {
        &*self.node_ref
    }
}

impl<'iter, Data, Id> IntoIterator for NodeRef<'iter, Data, Id>
where
    Id: Clone + 'static,
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

pub struct TreeNodeIter<'iter, Data, Id> {
    stack: VecDeque<NodeRef<'iter, Data, Id>>,
}

impl<'iter, Data, Id> Iterator for TreeNodeIter<'iter, Data, Id>
where
    Data: Clone + std::fmt::Debug + 'static,
    Id: Clone + 'static,
{
    type Item = NodeRef<'iter, Data, Id>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            node.node().children().map(|children| {
                for child in children.clone() {
                    self.stack.push_front(child.clone())
                }
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
        id::{AtomicU64Generator, UniqueGenerator},
        index::{BTreeIndex, TreeIndex},
        NodeId, Tree,
    };

    use super::TreeNode;

    #[derive(Debug, Clone)]
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
    fn simple_tree<'a>() -> TreeNode<'a, TestData> {
        let mut gen = AtomicU64Generator::default();

        let a = TreeNode::new(gen.generate(), TestData::Foo, None);
        let b = TreeNode::new(gen.generate(), TestData::Bar, Some(vec![a]));
        let root = TreeNode::new(
            gen.generate(),
            TestData::String("Hello".into()),
            Some(vec![b]),
        );
        root
    }

    #[derive(Debug)]
    struct App<'tree, 'index>
    where
        'tree: 'index,
    {
        tree: Tree<'tree, TestData, NodeId>,
        index: BTreeIndex<'index, TestData>,
    }

    #[traced_test]
    #[test]
    fn test_tree() {
        let nodes = simple_tree();

        let tree = Tree::new(nodes);
        let index = BTreeIndex::from_tree(&tree);

        let app = App { tree, index }; // index };
        info!("{app:#?}");
    }
}
