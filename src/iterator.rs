/*
use std::collections::VecDeque;

use crate::node::TreeNode;


impl<'iter, 'tree, Data, Id> IntoIterator for &'iter TreeNode<'tree, Data, Id>
where
    Id: Clone + 'static,
    Data: 'static,
{
    type Item = &'iter TreeNode<'iter, Data, Id>;

    type IntoIter = TreeNodeIterRef<'iter, 'tree, Data, Id>;

    fn into_iter(self) -> Self::IntoIter {
        TreeNodeIterRef {
            stack: VecDeque::from([self]),
        }
    }
}

pub struct TreeNodeIterRef<'iter, 'tree, Data, Id> {
    stack: VecDeque<&'iter TreeNode<'tree, Data, Id>>,
}

impl<'iter, 'tree, Data, Id> Iterator for TreeNodeIterRef<'iter, 'tree, Data, Id>
where
    Id: Clone + 'static,
    Data: 'static,
{
    type Item = &'iter TreeNode<'iter, Data, Id>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            node.children()
                .map(|children| children.iter().map(|child| self.stack.push_front(child)));
            node
        })
    }
}

*/
