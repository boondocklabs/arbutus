use tracing::debug;

use super::{internal, TreeNode};
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::{noderef::rc::NodeRef, TreeNodeRef as _, UniqueId};

/// TreeNodeRefCell wraps each node in Rc and RefCell providing interior mutability
pub struct Node<Data, Id = crate::NodeId>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    id: Id,
    data: Rc<RefCell<Data>>,
    parent: Option<NodeRef<Self>>,
    children: Rc<Option<RefCell<Vec<NodeRef<Self>>>>>,
}

impl<Data, Id> std::fmt::Debug for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Debug + std::fmt::Display + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeNodeRefCell")
            .field("id", &self.id)
            .field("hash", &format_args!("0x{:X}", self.xxhash()))
            .field("data", &format_args!("{}", self.data()))
            .field(
                "parent_id",
                &format_args!("{:?}", self.parent.as_ref().map(|p| p.node().id())),
            )
            .field(
                "child_ids",
                &format_args!(
                    "{:?}",
                    self.children().map(|children| children
                        .iter()
                        .map(|c| c.node().id())
                        .collect::<Vec<Id>>())
                ),
            )
            .finish()
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

impl<Data, Id> internal::NodeInternal<Data, Id> for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn test_internal(&mut self) {
        println!("Internal called");
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<Data, Id> TreeNode for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    type Data = Data;
    type Id = Id;
    type DataRef<'b> = Ref<'b, Self::Data>;
    type DataRefMut<'b> = RefMut<'b, Self::Data>;
    type NodeRef = NodeRef<Self>;

    fn new(id: Self::Id, data: Self::Data, children: Option<Vec<Self::NodeRef>>) -> Self {
        let children = children
            .map(|children| RefCell::new(children.into_iter().map(|child| child).collect()));

        debug!("Created Node ID {}", id);

        Node {
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

    fn id(&self) -> Self::Id {
        self.id
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

    fn children_mut<'b>(&'b self) -> Option<RefMut<'b, Vec<Self::NodeRef>>> {
        if let Some(children) = &*self.children {
            Some(children.borrow_mut())
        } else {
            None
        }
    }

    fn add_child(&mut self, node: NodeRef<Self>)
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

    fn remove_child_index(&mut self, index: usize) {
        if let Some(children) = &*self.children {
            let _removed = children.borrow_mut().remove(index);
        }
    }

    fn parent<'b>(&'b self) -> Option<&'b Self::NodeRef> {
        self.parent.as_ref()
    }

    fn parent_mut<'b>(&'b mut self) -> Option<&'b mut Self::NodeRef> {
        self.parent.as_mut()
    }

    fn set_children(&mut self, children: Option<Vec<Self::NodeRef>>) {
        if let Some(children) = children {
            self.children = Rc::new(Some(RefCell::new(children)))
        } else {
            self.children = Rc::new(None)
        }
    }
}

impl<Data, Id> Clone for Node<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn clone(&self) -> Self {
        Node {
            id: self.id.clone(),
            data: self.data.clone(),
            children: self.children.clone(),
            parent: self.parent.clone(),
        }
    }
}
