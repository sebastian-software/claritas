//! Constants and regex patterns used by Readability
//!
//! These are ported from Readability.js

use lazy_static::lazy_static;
use regex::Regex;

// Unlikely candidates - elements unlikely to be content
lazy_static! {
    pub static ref UNLIKELY_CANDIDATES: Regex = Regex::new(
        r"(?i)-ad-|ai2html|banner|breadcrumbs|combx|comment|community|cover-wrap|disqus|extra|footer|gdpr|header|legends|menu|related|remark|replies|rss|shoutbox|sidebar|skyscraper|social|sponsor|supplemental|ad-break|agegate|pagination|pager|popup|yom-hierarchicalnavigation"
    ).unwrap();

    pub static ref OK_MAYBE_CANDIDATE: Regex = Regex::new(
        r"(?i)and|article|body|column|content|main|shadow"
    ).unwrap();

    pub static ref POSITIVE: Regex = Regex::new(
        r"(?i)article|body|content|entry|hentry|h-entry|main|page|pagination|post|text|blog|story"
    ).unwrap();

    pub static ref NEGATIVE: Regex = Regex::new(
        r"(?i)-ad-|hidden|^hid$|hid$|hid |^hid |banner|combx|comment|com-|contact|foot|footer|footnote|gdpr|masthead|media|meta|outbrain|promo|related|scroll|share|shoutbox|sidebar|skyscraper|sponsor|shopping|tags|tool|widget"
    ).unwrap();

    pub static ref EXTRANEOUS: Regex = Regex::new(
        r"(?i)print|archive|comment|discuss|e[\-]?mail|share|reply|all|login|sign|single|utility"
    ).unwrap();

    pub static ref BYLINE: Regex = Regex::new(
        r"(?i)byline|author|dateline|writtenby|p-author"
    ).unwrap();

    pub static ref NORMALIZE_SPACES: Regex = Regex::new(r"\s{2,}").unwrap();

    pub static ref VIDEOS: Regex = Regex::new(
        r"(?i)//(www\.)?((dailymotion|youtube|youtube-nocookie|player\.vimeo|v\.qq)\.com|(archive|upload\.wikimedia)\.org|player\.twitch\.tv)"
    ).unwrap();

    pub static ref SHARE_ELEMENTS: Regex = Regex::new(
        r"(?i)(\b|_)(share|sharedaddy)(\b|_)"
    ).unwrap();

    pub static ref HAS_CONTENT: Regex = Regex::new(r"\S").unwrap();
}

/// Tags that are considered dividers
pub const DIV_TO_P_ELEMS: &[&str] = &[
    "blockquote", "dl", "div", "img", "ol", "p", "pre", "table", "ul",
];

/// Tags that should be preserved
pub const PRESENTATIONAL_ATTRIBUTES: &[&str] = &[
    "align", "background", "bgcolor", "border", "cellpadding", "cellspacing",
    "frame", "hspace", "rules", "style", "valign", "vspace",
];

/// Deprecated size attribute elements
pub const DEPRECATED_SIZE_ATTRIBUTE_ELEMS: &[&str] = &["table", "th", "td", "hr", "pre"];

/// Tags that contribute to content score
pub const PHRASING_ELEMS: &[&str] = &[
    "abbr", "audio", "b", "bdo", "br", "button", "cite", "code", "data",
    "datalist", "dfn", "em", "embed", "i", "img", "input", "kbd", "label",
    "mark", "math", "meter", "noscript", "object", "output", "progress", "q",
    "ruby", "samp", "script", "select", "small", "span", "strong", "sub",
    "sup", "textarea", "time", "var", "wbr",
];

/// Tags to remove
pub const TAGS_TO_REMOVE: &[&str] = &["script", "noscript", "style"];

/// Void elements (self-closing)
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
];

/// Default tags to score
pub const DEFAULT_TAGS_TO_SCORE: &[&str] = &[
    "section", "h2", "h3", "h4", "h5", "h6", "p", "td", "pre",
];

/// Minimum content length for an article
pub const DEFAULT_CHAR_THRESHOLD: usize = 500;

/// Minimum score for a top candidate
pub const MIN_SCORE: f64 = 20.0;

/// Maximum number of elements to traverse
pub const MAX_ELEMS_TO_PARSE: usize = 100000;

/// Classes that indicate probable visibility
pub const VISIBILITY_CLASSES: &[&str] = &["page", "print"];
