use crate::{
    node::arc::Node, noderef::arc::NodeRef, IndexedTree, NodeBuilder, NodeId, TreeBuilder,
};

#[derive(Debug)]
#[allow(unused)]
pub enum MyError {
    Fail(String),
}

#[derive(Debug, Clone, Hash)]
#[allow(unused)]
pub enum TestData {
    Root,
    Nest,
    String(&'static str),
}

impl std::fmt::Display for TestData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TestNode(pub &'static str, pub Vec<Self>);

pub fn test_tree_node(data: Vec<TestNode>) -> IndexedTree<NodeRef<Node<&'static str, NodeId>>> {
    fn add_children(builder: &mut NodeBuilder<&'static str, ()>, children: &Vec<TestNode>) {
        for child in children {
            builder
                .child(child.0, |nb| {
                    add_children(nb, &child.1);
                    Ok(())
                })
                .unwrap();
        }
    }

    let tree = TreeBuilder::<&'static str, ()>::new()
        .root("root", |root| {
            for node in data {
                root.child(node.0, |parent| {
                    add_children(parent, &node.1);
                    Ok(())
                })?;
            }
            Ok(())
        })
        .unwrap()
        .done()
        .unwrap()
        .unwrap()
        .index();

    tree
}

/// Construct a tree from a Vec of tuples of (&str, Vec of children)
pub fn test_tree_vec(
    data: Vec<(&'static str, Vec<&'static str>)>,
) -> IndexedTree<NodeRef<Node<&'static str, NodeId>>> {
    let tree = TreeBuilder::<&'static str, ()>::new()
        .root("root", |root| {
            for (data, children) in data {
                root.child(data, |node| {
                    for child in children {
                        node.child(child, |_| Ok(()))?;
                    }

                    Ok(())
                })?;
            }
            Ok(())
        })
        .unwrap()
        .done()
        .unwrap()
        .unwrap()
        .index();

    tree
}

pub fn test_tree_deep(
    a: Vec<&'static str>,
    b: Vec<&'static str>,
) -> IndexedTree<NodeRef<Node<&'static str, NodeId>>> {
    let tree = TreeBuilder::<&'static str, ()>::new()
        .root("root", |root| {
            root.child("column", |col| {
                col.child("row", |row| {
                    for child in &a {
                        row.child(child, |_| Ok(()))?;
                    }

                    Ok(())
                })?;
                col.child("row", |row| {
                    for child in &b {
                        row.child(child, |_| Ok(()))?;
                    }

                    Ok(())
                })?;
                Ok(())
            })?;
            Ok(())
        })
        .unwrap()
        .done()
        .unwrap()
        .unwrap()
        .index();

    println!("{}", tree.root());

    tree
}

pub fn test_tree_nested(
    depth: usize,
    children: Vec<&'static str>,
) -> IndexedTree<NodeRef<Node<TestData, NodeId>>> {
    let tree = TreeBuilder::<TestData, MyError>::new()
        .root(TestData::Root, |root| {
            for _ in 0..depth {
                root.child(TestData::Nest, |nest| {
                    nest.child(TestData::Nest, |nest| {
                        for child in &children {
                            nest.child(TestData::String(child), |_| Ok(()))?;
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            Ok(())
        })
        .unwrap()
        .done()
        .unwrap()
        .unwrap()
        .index();

    println!("{}", tree.root());

    tree
}

pub fn test_tree(children: Vec<&'static str>) -> IndexedTree<NodeRef<Node<TestData, NodeId>>> {
    let tree = TreeBuilder::<TestData, MyError>::new()
        .root(TestData::Root, |root| {
            for child in children {
                root.child(TestData::String(child), |_| Ok(()))?;
            }
            Ok(())
        })
        .unwrap()
        .done()
        .unwrap()
        .unwrap()
        .index();

    println!("{}", tree.root());

    tree
}
