use std::collections::VecDeque;

use crate::node::NodeRef;

pub trait Tree<'a, Data> {}

/*
impl<'a, Data> IntoIterator for &'a dyn Tree<'a, Data> {
    type Item = NodeRef<'a, Data>;
    type IntoIter = TreeNodeIterRef<'a, Data>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        TreeNodeIterRef {
            stack: VecDeque::from([NodeRef::new(self)]),
        }
    }
}

struct TreeNodeIterRef<'a, M> {
    stack: VecDeque<NodeRef<'a, M>>,
}

impl<'a, M> Iterator for TreeNodeIterRef<'a, M> {
    type Item = NodeRef<'a, M>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            match node.inner() {
                    self.stack.push_front(content.borrow());
                _ => {}
            }
            node
        })
    }
}
*/
