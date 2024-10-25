use crate::{noderef::simple::NodeRef, UniqueId};

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
    parent: Box<Option<NodeRef<Self>>>,
    children: Option<Vec<NodeRef<Self>>>,
}

impl<Data, Id> NodeInternal<Data, Id> for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<Data, Id> std::hash::Hash for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data().hash(state)
    }
}

impl<Data, Id> TreeNode for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    type NodeRef = NodeRef<Self>;
    type Data = Data;
    type Id = Id;
    type DataRef<'b> = &'b Data;
    type DataRefMut<'b> = &'b mut Data;
    type ChildrenRef<'b> = &'b Vec<Self::NodeRef>;
    type ChildrenRefMut<'b> = &'b mut Vec<Self::NodeRef>;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<NodeRef<Self>>>) -> Self {
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

    fn remove_child_index(&mut self, index: usize) {
        if let Some(children) = &mut self.children {
            let _removed = children.remove(index);
        }
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
}
