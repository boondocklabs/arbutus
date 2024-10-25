/// TreeNodeSimple provides no interior mutability
#[derive(Debug, Clone)]
pub struct TreeNodeSimple<Data, Id = crate::NodeId>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    id: Id,
    data: Data,
    parent: Box<Option<NodeRefRef<Self>>>,
    children: Option<Vec<NodeRefRef<Self>>>,
}

impl<Data, Id> internal::NodeInternal<Data, Id> for TreeNodeSimple<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn test_internal(&mut self) {
        todo!()
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<Data, Id> std::hash::Hash for TreeNodeSimple<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data().hash(state)
    }
}

impl<Data, Id> Node for TreeNodeSimple<Data, Id>
where
    Id: UniqueId + 'static,
    Data: std::hash::Hash + std::fmt::Display + Clone + 'static,
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

    fn id(&self) -> Self::Id {
        self.id.clone()
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

    fn children_mut<'b>(&'b self) -> Option<RefMut<'b, Vec<Self::NodeRef>>> {
        todo!()
    }

    fn add_child(&mut self, node: Self::NodeRef) {
        if let Some(children) = &mut self.children {
            children.push(node)
        }
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
