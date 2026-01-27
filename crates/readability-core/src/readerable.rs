//! Check if a document is probably readable
//!
//! This is a port of isProbablyReaderable from Readability.js

use crate::constants::{UNLIKELY_CANDIDATES, OK_MAYBE_CANDIDATE};
use dom_shim::Document;

/// Default minimum score for isProbablyReaderable
const DEFAULT_MIN_SCORE: f64 = 20.0;

/// Default minimum content length
const DEFAULT_MIN_CONTENT_LENGTH: usize = 140;

/// Options for isProbablyReaderable
pub struct ReaderableOptions {
    pub min_score: f64,
    pub min_content_length: usize,
}

impl Default for ReaderableOptions {
    fn default() -> Self {
        ReaderableOptions {
            min_score: DEFAULT_MIN_SCORE,
            min_content_length: DEFAULT_MIN_CONTENT_LENGTH,
        }
    }
}

/// Check if a document is probably readable
pub fn is_probably_readerable(html: &str) -> bool {
    is_probably_readerable_with_options(html, ReaderableOptions::default())
}

/// Check if a document is probably readable with custom options
pub fn is_probably_readerable_with_options(html: &str, options: ReaderableOptions) -> bool {
    let doc = Document::parse(html);

    // Get all paragraphs and collect candidates
    let paragraphs = doc.get_elements_by_tag_name("p");
    let mut score = 0.0;

    for p in paragraphs {
        // Check if element is visible (simplified check)
        let class = p.class_name().unwrap_or_default();
        let id = p.id().unwrap_or_default();
        let match_string = format!("{} {}", class, id);

        // Skip unlikely candidates
        if UNLIKELY_CANDIDATES.is_match(&match_string)
            && !OK_MAYBE_CANDIDATE.is_match(&match_string)
        {
            continue;
        }

        // Check text content length
        let text = p.text_content();
        let text_len = text.trim().len();

        if text_len < options.min_content_length {
            continue;
        }

        // Add to score based on content
        score += (text_len as f64).sqrt();
    }

    // Also check article, main, and div elements
    for tag in &["article", "main"] {
        for element in doc.get_elements_by_tag_name(tag) {
            let text = element.text_content();
            let text_len = text.trim().len();

            if text_len >= options.min_content_length {
                score += (text_len as f64).sqrt() * 0.5;
            }
        }
    }

    score >= options.min_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_article_is_readable() {
        let html = r#"
            <html>
            <body>
                <article>
                    <p>This is a very long paragraph that contains a lot of text.
                    It should be long enough to pass the minimum content length check.
                    We need to make sure it has enough words to be considered readable content.
                    Adding more text here to ensure we meet the threshold requirements.</p>
                </article>
            </body>
            </html>
        "#;

        assert!(is_probably_readerable(html));
    }

    #[test]
    fn test_empty_page_is_not_readable() {
        let html = "<html><body><p>Short</p></body></html>";
        assert!(!is_probably_readerable(html));
    }
}
