//! DOM Shim - A browser-like DOM API over html5ever/scraper
//!
//! This crate provides a DOM API similar to the browser's Document/Element
//! interface, built on top of the scraper crate (which uses html5ever).

mod document;
mod element;
mod error;
mod node;
mod serializer;

pub use document::Document;
pub use element::Element;
pub use error::DomError;
pub use node::{Node, NodeRef};
