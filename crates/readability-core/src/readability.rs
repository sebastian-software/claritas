//! Readability - Rust port of Mozilla Readability
//!
//! This is a port of the core Readability algorithm that extracts
//! readable content from web pages.

use crate::constants::*;
use crate::utils;
use dom_shim::{Document, Element};

/// Extracted article content
#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub byline: Option<String>,
    pub content: String,
    pub text_content: Option<String>,
    pub length: i32,
    pub excerpt: Option<String>,
    pub site_name: Option<String>,
    pub dir: Option<String>,
    pub lang: Option<String>,
    pub published_time: Option<String>,
}

/// Options for the Readability parser
#[derive(Debug, Clone)]
pub struct Options {
    /// Maximum number of elements to parse (0 = unlimited)
    pub max_elems_to_parse: usize,
    /// Number of top candidates to consider
    pub n_top_candidates: usize,
    /// Minimum character threshold for article
    pub char_threshold: usize,
    /// Classes to preserve in output
    pub classes_to_preserve: Vec<String>,
    /// Keep all classes
    pub keep_classes: bool,
    /// Disable JSON-LD parsing
    pub disable_json_ld: bool,
    /// Link density modifier
    pub link_density_modifier: f64,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            max_elems_to_parse: 0,
            n_top_candidates: 5,
            char_threshold: DEFAULT_CHAR_THRESHOLD,
            classes_to_preserve: vec!["page".to_string()],
            keep_classes: false,
            disable_json_ld: false,
            link_density_modifier: 0.0,
            debug: false,
        }
    }
}

/// Flags that control parsing behavior
#[derive(Debug, Clone, Copy)]
struct Flags {
    strip_unlikelys: bool,
    weight_classes: bool,
    clean_conditionally: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Flags {
            strip_unlikelys: true,
            weight_classes: true,
            clean_conditionally: true,
        }
    }
}

/// Metadata extracted from the document
#[derive(Debug, Default, Clone)]
struct Metadata {
    title: Option<String>,
    byline: Option<String>,
    excerpt: Option<String>,
    site_name: Option<String>,
    published_time: Option<String>,
}

/// Content score for an element
#[derive(Debug, Clone, Default)]
struct ContentScore {
    score: f64,
}

/// Candidate element with its score
#[derive(Debug, Clone)]
struct Candidate {
    element: Element,
    score: f64,
}

/// Unlikely roles that should be removed
const UNLIKELY_ROLES: &[&str] = &[
    "menu",
    "menubar",
    "complementary",
    "navigation",
    "alert",
    "alertdialog",
    "dialog",
];

/// The main Readability parser
pub struct Readability {
    doc: Document,
    url: Option<String>,
    options: Options,
    flags: Flags,
    article_title: Option<String>,
    article_byline: Option<String>,
    article_dir: Option<String>,
    article_lang: Option<String>,
    article_site_name: Option<String>,
    metadata: Metadata,
    attempts: Vec<(String, f64)>,
}

impl Readability {
    /// Create a new Readability parser
    pub fn new(html: &str, url: Option<&str>) -> Self {
        Self::with_options(html, url, Options::default())
    }

    /// Create a new Readability parser with custom options
    pub fn with_options(html: &str, url: Option<&str>, options: Options) -> Self {
        Readability {
            doc: Document::parse(html),
            url: url.map(|s| s.to_string()),
            options,
            flags: Flags::default(),
            article_title: None,
            article_byline: None,
            article_dir: None,
            article_lang: None,
            article_site_name: None,
            metadata: Metadata::default(),
            attempts: Vec::new(),
        }
    }

    /// Parse the document and extract the article
    pub fn parse(&mut self) -> Option<Article> {
        // Check element count limit
        if self.options.max_elems_to_parse > 0 {
            let all_elements = self.doc.query_selector_all("*").unwrap_or_default();
            if all_elements.len() > self.options.max_elems_to_parse {
                return None;
            }
        }

        // Prep document
        self.prep_document();

        // Get metadata
        self.metadata = self.get_article_metadata();
        self.article_title = self.metadata.title.clone();

        // Grab article content
        let article_content = self.grab_article()?;

        // Post-process content
        let content = self.post_process_content(&article_content);

        // Get excerpt if not in metadata
        let excerpt = self.metadata.excerpt.clone().or_else(|| {
            article_content
                .query_selector("p")
                .map(|p| p.text_content().trim().to_string())
                .filter(|s| !s.is_empty())
        });

        let text_content = article_content.text_content();
        let length = text_content.len() as i32;

        Some(Article {
            title: self.article_title.clone().unwrap_or_default(),
            byline: self.metadata.byline.clone().or(self.article_byline.clone()),
            content,
            text_content: Some(text_content),
            length,
            excerpt,
            site_name: self.metadata.site_name.clone().or(self.article_site_name.clone()),
            dir: self.article_dir.clone(),
            lang: self.article_lang.clone(),
            published_time: self.metadata.published_time.clone(),
        })
    }

    /// Prepare the document by removing scripts, styles, etc.
    fn prep_document(&mut self) {
        // Get language and direction from html element
        if let Some(html) = self.doc.document_element() {
            self.article_lang = html.get_attribute("lang");
            self.article_dir = html.get_attribute("dir");
        }
    }

    /// Extract article metadata from meta tags
    fn get_article_metadata(&self) -> Metadata {
        let mut metadata = Metadata::default();

        // Try to get title
        metadata.title = self.get_article_title();

        // Get meta tags
        let metas = self.doc.get_elements_by_tag_name("meta");
        for meta in metas {
            let name = meta
                .get_attribute("name")
                .or_else(|| meta.get_attribute("property"))
                .unwrap_or_default()
                .to_lowercase();
            let content = meta.get_attribute("content");

            if let Some(content) = content {
                match name.as_str() {
                    "author" | "dc.creator" | "og:author" => {
                        if metadata.byline.is_none() {
                            metadata.byline = Some(content);
                        }
                    }
                    "description" | "og:description" | "twitter:description" => {
                        if metadata.excerpt.is_none() {
                            metadata.excerpt = Some(content);
                        }
                    }
                    "og:site_name" => {
                        if metadata.site_name.is_none() {
                            metadata.site_name = Some(content);
                        }
                    }
                    "article:published_time" | "og:article:published_time" => {
                        if metadata.published_time.is_none() {
                            metadata.published_time = Some(content);
                        }
                    }
                    "og:title" | "twitter:title" => {
                        if metadata.title.is_none() {
                            metadata.title = Some(content);
                        }
                    }
                    _ => {}
                }
            }
        }

        metadata
    }

    /// Get the article title
    fn get_article_title(&self) -> Option<String> {
        // First try og:title or twitter:title
        let metas = self.doc.get_elements_by_tag_name("meta");
        for meta in &metas {
            let property = meta.get_attribute("property").unwrap_or_default();
            let name = meta.get_attribute("name").unwrap_or_default();

            if property == "og:title" || name == "twitter:title" {
                if let Some(content) = meta.get_attribute("content") {
                    if !content.trim().is_empty() {
                        return Some(content.trim().to_string());
                    }
                }
            }
        }

        // Try <title> tag
        if let Ok(Some(title_el)) = self.doc.query_selector("title") {
            let title = title_el.text_content().trim().to_string();
            if !title.is_empty() {
                // Clean up title (remove site name suffix)
                return Some(self.clean_title(&title));
            }
        }

        // Try h1
        if let Ok(Some(h1)) = self.doc.query_selector("h1") {
            let title = h1.text_content().trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }

        None
    }

    /// Clean up article title by removing site name suffixes
    fn clean_title(&self, title: &str) -> String {
        // Common separators: | - : » /
        let separators = [" | ", " - ", " : ", " » ", " / "];

        for sep in &separators {
            if let Some(idx) = title.rfind(sep) {
                let before = &title[..idx];
                let after = &title[idx + sep.len()..];

                // Use the longer part (usually the actual title)
                if before.len() > after.len() && before.split_whitespace().count() >= 3 {
                    return before.trim().to_string();
                }
            }
        }

        title.to_string()
    }

    /// Check if an element is probably visible
    fn is_probably_visible(&self, element: &Element) -> bool {
        // Check for hidden attribute
        if element.has_attribute("hidden") {
            return false;
        }

        // Check aria-hidden
        if element.get_attribute("aria-hidden") == Some("true".to_string()) {
            return false;
        }

        // Check style for display:none or visibility:hidden
        if let Some(style) = element.get_attribute("style") {
            let style_lower = style.to_lowercase();
            if style_lower.contains("display:none") || style_lower.contains("display: none") {
                return false;
            }
            if style_lower.contains("visibility:hidden") || style_lower.contains("visibility: hidden") {
                return false;
            }
        }

        true
    }

    /// Check if an element has no content
    fn is_element_without_content(&self, element: &Element) -> bool {
        let text = element.text_content();
        let text_trimmed = text.trim();

        if !text_trimmed.is_empty() {
            return false;
        }

        // Check for images, videos, iframes
        let children = element.children();
        for child in &children {
            let tag = child.tag_name();
            if tag == "img" || tag == "video" || tag == "iframe" || tag == "object" || tag == "embed" {
                return false;
            }
        }

        // Check for nested elements with content
        !children.iter().any(|c| !self.is_element_without_content(c))
    }

    /// Check if a byline is valid
    fn is_valid_byline(&self, text: &str) -> bool {
        let trimmed = text.trim();
        trimmed.len() > 0 && trimmed.len() < 100
    }

    /// Get the inner text of an element (normalized)
    fn get_inner_text(&self, element: &Element) -> String {
        utils::normalize_spaces(&element.text_content())
    }

    /// Initialize a node with its base content score
    fn initialize_node(&self, element: &Element) -> f64 {
        let tag_name = element.tag_name();
        let mut score = self.get_class_weight(element);

        match tag_name.as_str() {
            "div" => score += 5.0,
            "pre" | "td" | "blockquote" => score += 3.0,
            "address" | "ol" | "ul" | "dl" | "dd" | "dt" | "li" | "form" => score -= 3.0,
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "th" => score -= 5.0,
            "article" => score += 10.0,
            "section" => score += 5.0,
            _ => {}
        }

        score
    }

    /// Get node ancestors up to a maximum depth
    fn get_node_ancestors(&self, element: &Element, max_depth: usize) -> Vec<Element> {
        let mut ancestors = Vec::new();
        let mut current = element.parent_element();
        let mut depth = 0;

        while let Some(parent) = current {
            if depth >= max_depth {
                break;
            }
            ancestors.push(parent.clone());
            current = parent.parent_element();
            depth += 1;
        }

        ancestors
    }

    /// Check if element has an ancestor with the given tag
    fn has_ancestor_tag(&self, element: &Element, tag: &str) -> bool {
        let mut current = element.parent_element();
        while let Some(parent) = current {
            if parent.tag_name().eq_ignore_ascii_case(tag) {
                return true;
            }
            current = parent.parent_element();
        }
        false
    }

    /// Grab the article content
    fn grab_article(&mut self) -> Option<Element> {
        let body = self.doc.body()?;

        // First pass: collect elements to score
        let mut elements_to_score: Vec<Element> = Vec::new();
        let mut nodes_to_check: Vec<Element> = vec![body.clone()];
        let mut seen_elements: Vec<Element> = Vec::new();

        while let Some(node) = nodes_to_check.pop() {
            // Check if we've already seen this element
            if seen_elements.iter().any(|e| e == &node) {
                continue;
            }
            seen_elements.push(node.clone());

            // Add children to check
            for child in node.children() {
                nodes_to_check.push(child);
            }

            // Check if element is visible
            if !self.is_probably_visible(&node) {
                continue;
            }

            // Check for unlikely roles
            if let Some(role) = node.get_attribute("role") {
                if UNLIKELY_ROLES.contains(&role.as_str()) {
                    continue;
                }
            }

            // Skip unlikely candidates
            if self.flags.strip_unlikelys {
                let class_name = node.class_name().unwrap_or_default();
                let id = node.id().unwrap_or_default();
                let match_string = format!("{} {}", class_name, id);

                if UNLIKELY_CANDIDATES.is_match(&match_string)
                    && !OK_MAYBE_CANDIDATE.is_match(&match_string)
                    && !self.has_ancestor_tag(&node, "table")
                    && !self.has_ancestor_tag(&node, "code")
                    && node.tag_name() != "body"
                    && node.tag_name() != "a"
                {
                    continue;
                }
            }

            // Skip empty elements
            let tag = node.tag_name();
            if (tag == "div" || tag == "section" || tag == "header"
                || tag == "h1" || tag == "h2" || tag == "h3"
                || tag == "h4" || tag == "h5" || tag == "h6")
                && self.is_element_without_content(&node)
            {
                continue;
            }

            // Add elements that should be scored
            if DEFAULT_TAGS_TO_SCORE.contains(&tag.to_uppercase().as_str()) {
                elements_to_score.push(node.clone());
            }
        }

        // Score the elements - store candidates with their scores
        let mut candidates: Vec<Candidate> = Vec::new();

        for element in &elements_to_score {
            let inner_text = self.get_inner_text(element);

            // Skip if too short
            if inner_text.len() < 25 {
                continue;
            }

            // Get ancestors
            let ancestors = self.get_node_ancestors(element, 5);
            if ancestors.is_empty() {
                continue;
            }

            // Calculate content score
            let mut content_score = 1.0;
            content_score += utils::count_commas(&inner_text) as f64;
            content_score += (inner_text.len() as f64 / 100.0).min(3.0);

            // Score ancestors
            for (level, ancestor) in ancestors.iter().enumerate() {
                // Find or create candidate for this ancestor
                let candidate_idx = candidates.iter().position(|c| c.element == *ancestor);

                let score_divider = if level == 0 {
                    1.0
                } else if level == 1 {
                    2.0
                } else {
                    (level * 3) as f64
                };

                let score_to_add = content_score / score_divider;

                if let Some(idx) = candidate_idx {
                    candidates[idx].score += score_to_add;
                } else {
                    let init_score = self.initialize_node(ancestor);
                    candidates.push(Candidate {
                        element: ancestor.clone(),
                        score: init_score + score_to_add,
                    });
                }
            }
        }

        // Apply link density penalty
        for candidate in &mut candidates {
            let link_density = self.get_link_density(&candidate.element);
            candidate.score *= 1.0 - link_density;
        }

        // Sort candidates by score
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Get top candidate
        let top_candidate = candidates.first().map(|c| c.element.clone());

        // If no good candidate found, try article or main tags
        if top_candidate.is_none() || candidates.first().map(|c| c.score).unwrap_or(0.0) < MIN_SCORE {
            // Try <article> tag first
            if let Ok(Some(article)) = self.doc.query_selector("article") {
                return Some(article);
            }

            // Try <main> tag
            if let Ok(Some(main)) = self.doc.query_selector("main") {
                return Some(main);
            }

            // Fall back to body
            return Some(body);
        }

        top_candidate
    }

    /// Get the class weight for an element
    fn get_class_weight(&self, element: &Element) -> f64 {
        if !self.flags.weight_classes {
            return 0.0;
        }

        let mut weight = 0.0;

        let class_name = element.class_name().unwrap_or_default();
        let id = element.id().unwrap_or_default();
        let match_string = format!("{} {}", class_name, id);

        // Positive patterns
        if POSITIVE.is_match(&match_string) {
            weight += 25.0;
        }

        // Negative patterns
        if NEGATIVE.is_match(&match_string) {
            weight -= 25.0;
        }

        weight
    }

    /// Get link density for an element (ratio of link text to total text)
    fn get_link_density(&self, element: &Element) -> f64 {
        let text_length = element.text_content().len();
        if text_length == 0 {
            return 0.0;
        }

        let links = element.get_elements_by_tag_name("a");
        let link_length: usize = links.iter().map(|a| a.text_content().len()).sum();

        link_length as f64 / text_length as f64
    }

    /// Post-process the article content
    fn post_process_content(&self, element: &Element) -> String {
        let mut html = element.inner_html();

        // Remove unwanted elements via regex
        // This is a simple approach - a proper implementation would use DOM manipulation
        html = self.remove_elements_by_tag(&html, "script");
        html = self.remove_elements_by_tag(&html, "style");
        html = self.remove_elements_by_tag(&html, "link");
        html = self.remove_elements_by_tag(&html, "header");
        html = self.remove_elements_by_tag(&html, "footer");
        html = self.remove_elements_by_tag(&html, "nav");
        html = self.remove_elements_by_tag(&html, "aside");
        html = self.remove_elements_by_tag(&html, "form");
        html = self.remove_elements_by_tag(&html, "noscript");

        // Fix relative URLs if we have a base URL
        if let Some(base_url) = &self.url {
            html = self.fix_relative_urls(&html, base_url);
        }

        // Wrap in a div with readability class
        format!("<div id=\"readability-page-1\" class=\"page\">{}</div>", html.trim())
    }

    /// Remove elements by tag name (simple regex-based approach)
    fn remove_elements_by_tag(&self, html: &str, tag: &str) -> String {
        // Match self-closing tags like <link .../>
        let self_closing = regex::Regex::new(&format!(r"(?is)<{}\s[^>]*/?>", tag)).unwrap();
        let result = self_closing.replace_all(html, "");

        // Match opening and closing tags with content
        let with_content = regex::Regex::new(&format!(r"(?is)<{}\s*[^>]*>.*?</{}>", tag, tag)).unwrap();
        with_content.replace_all(&result, "").to_string()
    }

    /// Fix relative URLs to absolute
    fn fix_relative_urls(&self, html: &str, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');

        // Fix href attributes
        let href_re = regex::Regex::new(r#"href="(/[^"]*)""#).unwrap();
        let result = href_re.replace_all(html, |caps: &regex::Captures| {
            format!("href=\"{}{}\"", base, &caps[1])
        });

        // Fix src attributes
        let src_re = regex::Regex::new(r#"src="(/[^"]*)""#).unwrap();
        src_re.replace_all(&result, |caps: &regex::Captures| {
            format!("src=\"{}{}\"", base, &caps[1])
        }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_article() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Article</title></head>
            <body>
                <article>
                    <h1>Test Article</h1>
                    <p>This is a test paragraph with some content. It needs to be long enough
                    to be considered valid content by the readability algorithm. Let's add
                    more text to make it substantial enough for parsing.</p>
                    <p>Another paragraph here with more content to ensure we have enough
                    text for the algorithm to work with properly.</p>
                </article>
            </body>
            </html>
        "#;

        let mut readability = Readability::new(html, None);
        let result = readability.parse();

        assert!(result.is_some());
        let article = result.unwrap();
        assert_eq!(article.title, "Test Article");
    }

    #[test]
    fn test_get_class_weight() {
        let html = r#"<html><body><div id="content" class="article">Test</div></body></html>"#;
        let readability = Readability::new(html, None);
        let div = readability.doc.query_selector("#content").unwrap().unwrap();

        let weight = readability.get_class_weight(&div);
        assert!(weight > 0.0); // "article" and "content" are positive
    }

    #[test]
    fn test_negative_class_weight() {
        let html = r#"<html><body><div id="sidebar" class="comment">Test</div></body></html>"#;
        let readability = Readability::new(html, None);
        let div = readability.doc.query_selector("#sidebar").unwrap().unwrap();

        let weight = readability.get_class_weight(&div);
        assert!(weight < 0.0); // "sidebar" and "comment" are negative
    }

    #[test]
    fn test_link_density() {
        let html = r##"<html><body><div><a href="#">Link</a> Some text here</div></body></html>"##;
        let readability = Readability::new(html, None);
        let div = readability.doc.query_selector("div").unwrap().unwrap();

        let density = readability.get_link_density(&div);
        assert!(density > 0.0 && density < 1.0);
    }

    #[test]
    fn test_is_probably_visible() {
        let html = r#"<html><body>
            <div id="visible">Visible</div>
            <div id="hidden" hidden>Hidden</div>
            <div id="aria-hidden" aria-hidden="true">Aria Hidden</div>
        </body></html>"#;

        let readability = Readability::new(html, None);

        let visible = readability.doc.query_selector("#visible").unwrap().unwrap();
        assert!(readability.is_probably_visible(&visible));

        let hidden = readability.doc.query_selector("#hidden").unwrap().unwrap();
        assert!(!readability.is_probably_visible(&hidden));

        let aria_hidden = readability.doc.query_selector("#aria-hidden").unwrap().unwrap();
        assert!(!readability.is_probably_visible(&aria_hidden));
    }

    #[test]
    fn test_initialize_node_scores() {
        let html = r#"<html><body>
            <article>Article</article>
            <div>Div</div>
            <h1>H1</h1>
            <form>Form</form>
        </body></html>"#;

        let readability = Readability::new(html, None);

        let article = readability.doc.query_selector("article").unwrap().unwrap();
        assert!(readability.initialize_node(&article) > 0.0);

        let div = readability.doc.query_selector("div").unwrap().unwrap();
        assert!(readability.initialize_node(&div) > 0.0);

        let h1 = readability.doc.query_selector("h1").unwrap().unwrap();
        assert!(readability.initialize_node(&h1) < 0.0);

        let form = readability.doc.query_selector("form").unwrap().unwrap();
        assert!(readability.initialize_node(&form) < 0.0);
    }
}
