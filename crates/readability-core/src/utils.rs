//! Utility functions for Readability

use crate::constants::NORMALIZE_SPACES;

/// Normalize whitespace in a string
pub fn normalize_spaces(text: &str) -> String {
    NORMALIZE_SPACES.replace_all(text.trim(), " ").to_string()
}

/// Get the text length of a string, normalized
pub fn text_length(text: &str) -> usize {
    normalize_spaces(text).len()
}

/// Check if a string contains any words
pub fn has_any_words(text: &str) -> bool {
    text.split_whitespace().next().is_some()
}

/// Extract the domain from a URL
pub fn get_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}

/// Make a relative URL absolute
pub fn to_absolute_url(base: &str, relative: &str) -> Option<String> {
    let base_url = url::Url::parse(base).ok()?;
    base_url.join(relative).ok().map(|u| u.to_string())
}

/// Check if a node is whitespace-only
pub fn is_whitespace(text: &str) -> bool {
    text.trim().is_empty()
}

/// Count the number of commas in a string (used for heuristics)
pub fn count_commas(text: &str) -> usize {
    text.matches(',').count()
}

/// Unescape HTML entities
pub fn unescape_html_entities(text: &str) -> String {
    // Common HTML entities
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_spaces() {
        assert_eq!(normalize_spaces("  hello   world  "), "hello world");
    }

    #[test]
    fn test_get_domain() {
        assert_eq!(get_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(get_domain("invalid"), None);
    }

    #[test]
    fn test_unescape_html_entities() {
        assert_eq!(unescape_html_entities("&amp;&lt;&gt;"), "&<>");
    }
}
