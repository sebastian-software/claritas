//! Error types for DOM operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Selector error: {0}")]
    SelectorError(String),

    #[error("Node not found")]
    NodeNotFound,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
