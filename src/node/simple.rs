use crate::{NodePosition, UniqueId};

use super::{internal::NodeInternal, TreeNode};

/// TreeNodeSimple provides no interior mutability
#[derive(Debug, Clone)]
pub struct Node<Data, Id = crate::NodeId>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    id: Id,
    data: Data,
    parent: Box<Option<<Node<Data, Id> as TreeNode>::NodeRef>>,
    children: Option<Vec<<Node<Data, Id> as TreeNode>::NodeRef>>,
    position: Option<NodePosition>,
    subtree_hash: u64,
}

impl<Data, Id> NodeInternal<Self> for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn set_parent(&mut self, parent: <Self as TreeNode>::NodeRef) {
        self.parent = Box::new(Some(parent));
    }
}

impl<Data, Id> std::hash::Hash for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.num_children().hash(state);
        self.data().hash(state);
    }
}

impl<Data, Id> TreeNode for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    type NodeRef = crate::noderef::simple::NodeRef<Self>;
    type Data = Data;
    type Id = Id;
    type DataRef<'b> = &'b Data;
    type DataRefMut<'b> = &'b mut Data;
    type ChildrenRef<'b> = &'b Vec<Self::NodeRef>;
    type ChildrenRefMut<'b> = &'b mut Vec<Self::NodeRef>;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<Self::NodeRef>>) -> Self {
        Self {
            id,
            data,
            children,
            parent: Box::new(None),
            position: None,
            subtree_hash: 0,
        }
    }

    fn with_parent(mut self, parent: Self::NodeRef) -> Self {
        self.parent = Box::new(Some(parent));
        self
    }

    fn with_position(mut self, position: NodePosition) -> Self {
        self.position = Some(position);
        self
    }

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn data<'b>(&'b self) -> Self::DataRef<'b> {
        &self.data
    }

    fn data_mut<'b>(&'b mut self) -> Self::DataRefMut<'b> {
        &mut self.data
    }

    fn children<'b>(&'b self) -> Option<Self::ChildrenRef<'b>> {
        self.children.as_ref()
    }

    fn children_mut<'b>(&'b mut self) -> Option<Self::ChildrenRefMut<'b>> {
        self.children.as_mut()
    }

    fn parent<'b>(&'b self) -> Option<&'b Self::NodeRef> {
        (&*self.parent).as_ref()
    }

    fn parent_mut<'b>(&'b mut self) -> Option<&'b mut Self::NodeRef> {
        (&mut *self.parent).as_mut()
    }

    fn set_children(&mut self, children: Option<Vec<Self::NodeRef>>) {
        self.children = children
    }

    fn get_position(&self) -> Option<&NodePosition> {
        self.position.as_ref()
    }

    fn set_subtree_hash(&mut self, tree_hash: u64) {
        self.subtree_hash = tree_hash;
    }

    fn get_subtree_hash(&self) -> u64 {
        self.subtree_hash
    }
}
