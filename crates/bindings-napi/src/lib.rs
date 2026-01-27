//! Node.js bindings for Claritas via napi-rs

#![deny(clippy::all)]

use napi_derive::napi;

#[napi(object)]
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
}

#[napi]
pub fn extract_readability(html: String, url: Option<String>) -> Option<Article> {
    let result = readability_core::Readability::new(&html, url.as_deref()).parse();
    result.map(|a| Article {
        title: a.title,
        byline: a.byline,
        content: a.content,
        text_content: a.text_content,
        length: a.length,
        excerpt: a.excerpt,
        site_name: a.site_name,
        dir: a.dir,
        lang: a.lang,
    })
}

#[napi]
pub fn is_probably_readerable(html: String) -> bool {
    readability_core::is_probably_readerable(&html)
}
