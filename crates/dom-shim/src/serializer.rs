//! HTML serialization utilities

/// Normalize HTML for comparison
pub fn normalize_html(html: &str) -> String {
    // Remove extra whitespace between tags
    let mut result = String::new();
    let mut in_tag = false;
    let mut last_was_space = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                last_was_space = false;
                result.push(c);
            }
            '>' => {
                in_tag = false;
                last_was_space = false;
                result.push(c);
            }
            c if c.is_whitespace() => {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            }
            c => {
                last_was_space = false;
                result.push(c);
            }
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_html() {
        let input = "<div>   hello   world   </div>";
        let expected = "<div> hello world </div>";
        assert_eq!(normalize_html(input), expected);
    }
}
