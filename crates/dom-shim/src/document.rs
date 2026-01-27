//! Document type - represents an HTML document

use crate::element::Element;
use crate::error::DomError;
use scraper::{Html, Selector};
use std::cell::RefCell;
use std::rc::Rc;

/// An HTML Document
#[derive(Debug, Clone)]
pub struct Document {
    inner: Rc<RefCell<Html>>,
}

impl Document {
    /// Parse an HTML string into a Document
    pub fn parse(html: &str) -> Self {
        let parsed = Html::parse_document(html);
        Document {
            inner: Rc::new(RefCell::new(parsed)),
        }
    }

    /// Get the document element (html)
    pub fn document_element(&self) -> Option<Element> {
        let inner = self.inner.borrow();
        let selector = Selector::parse("html").ok()?;
        inner.select(&selector).next().map(|el| Element::new(el.id(), self.inner.clone()))
    }

    /// Get the body element
    pub fn body(&self) -> Option<Element> {
        let inner = self.inner.borrow();
        let selector = Selector::parse("body").ok()?;
        inner.select(&selector).next().map(|el| Element::new(el.id(), self.inner.clone()))
    }

    /// Get the head element
    pub fn head(&self) -> Option<Element> {
        let inner = self.inner.borrow();
        let selector = Selector::parse("head").ok()?;
        inner.select(&selector).next().map(|el| Element::new(el.id(), self.inner.clone()))
    }

    /// Query for a single element matching the selector
    pub fn query_selector(&self, selector: &str) -> Result<Option<Element>, DomError> {
        let inner = self.inner.borrow();
        let sel = Selector::parse(selector)
            .map_err(|e| DomError::SelectorError(format!("{:?}", e)))?;
        Ok(inner.select(&sel).next().map(|el| Element::new(el.id(), self.inner.clone())))
    }

    /// Query for all elements matching the selector
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<Element>, DomError> {
        let inner = self.inner.borrow();
        let sel = Selector::parse(selector)
            .map_err(|e| DomError::SelectorError(format!("{:?}", e)))?;
        Ok(inner
            .select(&sel)
            .map(|el| Element::new(el.id(), self.inner.clone()))
            .collect())
    }

    /// Get elements by tag name
    pub fn get_elements_by_tag_name(&self, tag: &str) -> Vec<Element> {
        self.query_selector_all(tag).unwrap_or_default()
    }

    /// Get element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<Element> {
        self.query_selector(&format!("#{}", id)).ok().flatten()
    }

    /// Serialize the document to HTML
    pub fn serialize(&self) -> String {
        self.inner.borrow().html()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_html() {
        let doc = Document::parse("<html><head></head><body><p>Hello</p></body></html>");
        assert!(doc.body().is_some());
        assert!(doc.head().is_some());
    }

    #[test]
    fn test_query_selector() {
        let doc = Document::parse("<html><body><div id='test'>Content</div></body></html>");
        let div = doc.query_selector("#test").unwrap();
        assert!(div.is_some());
    }
}
