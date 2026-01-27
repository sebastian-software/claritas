//! Node types and references

use std::cell::RefCell;
use std::rc::Rc;

/// A reference-counted node
pub type NodeRef = Rc<RefCell<Node>>;

/// DOM Node representation
#[derive(Debug, Clone)]
pub enum Node {
    Document,
    Element {
        tag_name: String,
        attributes: Vec<(String, String)>,
        children: Vec<NodeRef>,
    },
    Text(String),
    Comment(String),
}
