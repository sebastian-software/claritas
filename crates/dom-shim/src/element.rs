//! Element type - represents a DOM element

use scraper::{ElementRef, Html, Node as ScraperNode, Selector};
use ego_tree::NodeId;
use std::cell::RefCell;
use std::rc::Rc;

/// A DOM Element
#[derive(Debug, Clone)]
pub struct Element {
    node_id: NodeId,
    doc: Rc<RefCell<Html>>,
}

impl Element {
    /// Create a new Element from a node ID and document reference
    pub(crate) fn new(node_id: NodeId, doc: Rc<RefCell<Html>>) -> Self {
        Element { node_id, doc }
    }

    /// Get the element reference
    fn element_ref(&self) -> Option<ElementRef<'_>> {
        // SAFETY: We need to work around the borrow checker here
        // This is safe because we're not holding the borrow across yield points
        let doc = unsafe { &*self.doc.as_ptr() };
        let node = doc.tree.get(self.node_id)?;
        ElementRef::wrap(node)
    }

    /// Get the tag name (lowercase)
    pub fn tag_name(&self) -> String {
        self.element_ref()
            .map(|el| el.value().name().to_lowercase())
            .unwrap_or_default()
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.element_ref()
            .and_then(|el| el.value().attr(name).map(|s| s.to_string()))
    }

    /// Check if element has an attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.get_attribute(name).is_some()
    }

    /// Get the id attribute
    pub fn id(&self) -> Option<String> {
        self.get_attribute("id")
    }

    /// Get the class attribute
    pub fn class_name(&self) -> Option<String> {
        self.get_attribute("class")
    }

    /// Get classes as a list
    pub fn class_list(&self) -> Vec<String> {
        self.class_name()
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Get the text content of this element
    pub fn text_content(&self) -> String {
        self.element_ref()
            .map(|el| el.text().collect::<Vec<_>>().join(""))
            .unwrap_or_default()
    }

    /// Get the inner HTML
    pub fn inner_html(&self) -> String {
        self.element_ref()
            .map(|el| el.inner_html())
            .unwrap_or_default()
    }

    /// Get the outer HTML
    pub fn outer_html(&self) -> String {
        self.element_ref()
            .map(|el| el.html())
            .unwrap_or_default()
    }

    /// Get child elements (not text nodes)
    pub fn children(&self) -> Vec<Element> {
        let doc = self.doc.clone();
        self.element_ref()
            .map(|el| {
                el.children()
                    .filter_map(|child| {
                        if matches!(child.value(), ScraperNode::Element(_)) {
                            Some(Element::new(child.id(), doc.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all child nodes including text
    pub fn child_nodes(&self) -> Vec<NodeId> {
        self.element_ref()
            .map(|el| el.children().map(|c| c.id()).collect())
            .unwrap_or_default()
    }

    /// Get the parent element
    pub fn parent_element(&self) -> Option<Element> {
        let doc_ref = unsafe { &*self.doc.as_ptr() };
        let node = doc_ref.tree.get(self.node_id)?;
        let parent = node.parent()?;
        if matches!(parent.value(), ScraperNode::Element(_)) {
            Some(Element::new(parent.id(), self.doc.clone()))
        } else {
            None
        }
    }

    /// Query for a single element matching the selector within this element
    pub fn query_selector(&self, selector: &str) -> Option<Element> {
        let sel = Selector::parse(selector).ok()?;
        self.element_ref()
            .and_then(|el| el.select(&sel).next())
            .map(|el| Element::new(el.id(), self.doc.clone()))
    }

    /// Query for all elements matching the selector within this element
    pub fn query_selector_all(&self, selector: &str) -> Vec<Element> {
        let sel = match Selector::parse(selector) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        self.element_ref()
            .map(|el| {
                el.select(&sel)
                    .map(|matched| Element::new(matched.id(), self.doc.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get elements by tag name within this element
    pub fn get_elements_by_tag_name(&self, tag: &str) -> Vec<Element> {
        self.query_selector_all(tag)
    }

    /// Get the next sibling element
    pub fn next_element_sibling(&self) -> Option<Element> {
        let doc_ref = unsafe { &*self.doc.as_ptr() };
        let node = doc_ref.tree.get(self.node_id)?;
        let mut current = node.next_sibling();
        while let Some(sibling) = current {
            if matches!(sibling.value(), ScraperNode::Element(_)) {
                return Some(Element::new(sibling.id(), self.doc.clone()));
            }
            current = sibling.next_sibling();
        }
        None
    }

    /// Get the previous sibling element
    pub fn prev_element_sibling(&self) -> Option<Element> {
        let doc_ref = unsafe { &*self.doc.as_ptr() };
        let node = doc_ref.tree.get(self.node_id)?;
        let mut current = node.prev_sibling();
        while let Some(sibling) = current {
            if matches!(sibling.value(), ScraperNode::Element(_)) {
                return Some(Element::new(sibling.id(), self.doc.clone()));
            }
            current = sibling.prev_sibling();
        }
        None
    }

    /// Get the first child element
    pub fn first_element_child(&self) -> Option<Element> {
        self.children().into_iter().next()
    }

    /// Get the last child element
    pub fn last_element_child(&self) -> Option<Element> {
        self.children().into_iter().last()
    }

    /// Check if element matches a selector
    pub fn matches(&self, selector: &str) -> bool {
        let sel = match Selector::parse(selector) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.element_ref()
            .map(|el| sel.matches(&el))
            .unwrap_or(false)
    }

    /// Get the closest ancestor matching a selector
    pub fn closest(&self, selector: &str) -> Option<Element> {
        let sel = Selector::parse(selector).ok()?;
        let mut current = Some(self.clone());
        while let Some(el) = current {
            if el.element_ref().map(|e| sel.matches(&e)).unwrap_or(false) {
                return Some(el);
            }
            current = el.parent_element();
        }
        None
    }

    /// Get the node ID
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id && Rc::ptr_eq(&self.doc, &other.doc)
    }
}

impl Eq for Element {}

#[cfg(test)]
mod tests {
    use crate::Document;

    #[test]
    fn test_element_basics() {
        let doc = Document::parse("<html><body><div id='test' class='foo bar'>Hello</div></body></html>");
        let div = doc.query_selector("#test").unwrap().unwrap();

        assert_eq!(div.tag_name(), "div");
        assert_eq!(div.id(), Some("test".to_string()));
        assert_eq!(div.class_name(), Some("foo bar".to_string()));
        assert_eq!(div.class_list(), vec!["foo", "bar"]);
        assert_eq!(div.text_content(), "Hello");
    }

    #[test]
    fn test_element_traversal() {
        let doc = Document::parse("<html><body><ul><li>1</li><li>2</li><li>3</li></ul></body></html>");
        let ul = doc.query_selector("ul").unwrap().unwrap();
        let children = ul.children();

        assert_eq!(children.len(), 3);
        assert_eq!(children[0].text_content(), "1");
        assert_eq!(children[1].text_content(), "2");
        assert_eq!(children[2].text_content(), "3");
    }
}
