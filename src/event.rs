use crate::TreeNodeRef;

#[derive(Debug)]
pub enum TreeEvent<R>
where
    R: TreeNodeRef,
{
    /// Node removed from tree
    NodeRemoved { node: R },

    /// Node data was replaced. The node retains it's original ID and inner node container,
    /// but the inner data was replaced.
    NodeReplaced { node: R },

    /// A subtree was inserted at this node_id
    SubtreeInserted { node: R },

    /// Single child removed from a parent
    ChildRemoved { parent: R, index: usize },

    /// Multiple children removed from a parent
    ChildrenRemoved { parent: R, children: Vec<R> },

    /// Multiple children added to a parent
    ChildrenAdded { parent: R, children: Vec<R> },

    /// Child node replaced
    ChildReplaced { parent: R, index: usize },

    /// Child inserted into a parent at index
    ChildInserted { parent: R, index: usize },
}
