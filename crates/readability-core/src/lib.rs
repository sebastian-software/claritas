//! Readability Core - Rust port of Mozilla Readability
//!
//! This crate extracts the main readable content from web pages,
//! removing navigation, ads, and other clutter.

mod constants;
mod readability;
mod readerable;
mod utils;

pub use readability::{Article, Options, Readability};
pub use readerable::is_probably_readerable;
