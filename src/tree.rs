use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

type NodeRef<T> = Rc<RefCell<Node<T>>>;

pub struct Node<T> {
    pub value: T,
    pub parent: Option<NodeRef<T>>,
    pub children: Vec<NodeRef<T>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> NodeRef<T> {
        let node = Self {
            value,
            parent: None,
            children: Vec::new(),
        };
        Rc::new(RefCell::new(node))
    }
}

pub struct Tree<T> {
    root: NodeRef<T>,
}

impl<T> Tree<T>
where
    T: Ord,
{
    pub fn new(val: T) -> Self {
        Self {
            root: Node::new(val),
        }
    }

    pub fn insert(parent_ref: &NodeRef<T>, node_ref: &NodeRef<T>) {
        node_ref.borrow_mut().parent = Some(parent_ref.clone());
        parent_ref.borrow_mut().children.push(node_ref.clone());
    }
}
