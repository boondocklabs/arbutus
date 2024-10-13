use std::{
    cell::{Ref, RefCell},
    collections::VecDeque,
    fmt::Write,
    ops::Deref,
    rc::Rc,
};

use tracing::debug;

use crate::{iterator::NodeRefIter, NodeId};

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
    Id: Clone + std::fmt::Display,
{
    node_ref: Rc<RefCell<TreeNode<'node, Data, Id>>>,
}

impl<'node, Data, Id> std::fmt::Debug for NodeRef<'node, Data, Id>
where
    Id: Clone + std::fmt::Display + 'node,
    Data: std::fmt::Debug + 'node,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\n")?;

        let mut iter = self.iter().peekable();

        let mut root_children = false;

        let column_width = 2;

        loop {
            if let Some(node) = iter.next() {
                // Peek at the next node to see if there are siblings
                let has_siblings = if let Some(next_node) = iter.peek() {
                    node.depth() == next_node.depth()
                } else {
                    false
                };

                let has_children = node.node().children().is_some();

                if node.depth() == 0 {
                    root_children = has_children
                }

                // The position of the first character of the payload from the previous row
                let pos = node.depth() * column_width;

                if node.depth() == 0 {
                    if has_children || has_siblings {
                        f.write_char('┏')?;
                    } else {
                        f.write_char('━')?;
                    }
                } else {
                    for i in 0..pos {
                        if i % column_width == 0 {
                            f.write_char('┃')?;
                        } else {
                            f.write_char(' ')?;
                        }
                    }

                    if has_children || has_siblings {
                        f.write_char('┣')?;
                    } else {
                        f.write_char('┗')?;
                    }
                }

                f.write_fmt(format_args!(" {:?}\n", node.node().data()))?;
            } else {
                // Finished node iteration
                if root_children {
                    f.write_str("┗")?;
                }
                return Ok(());
            }
        }
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

    pub fn node<'b>(&'b self) -> Ref<'b, TreeNode<'node, Data, Id>> {
        self.node_ref.borrow()
    }

    pub fn for_each<E, F>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(usize, NodeRef<Data, Id>) -> Result<(), E>,
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

    use super::NodeRef;

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
    fn simple_tree<'a>() -> Result<Option<NodeRef<'a, TestData, NodeId>>, TestError> {
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
        let nodes = simple_tree().unwrap().unwrap();

        let tree = Tree::from_nodes(nodes);
        let index = BTreeIndex::from_tree(&tree);

        let app = App {
            _tree: tree,
            _index: index,
        }; // index };
        info!("{app:#?}");
    }
}
