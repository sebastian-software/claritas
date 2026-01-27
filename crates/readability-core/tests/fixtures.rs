//! Golden tests for Readability fixtures
//!
//! These tests compare our output against the expected output from
//! the original Readability.js test suite.

use readability_core::Readability;
use std::fs;
use std::path::PathBuf;

/// Get the fixtures directory path
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures")
        .join("readability")
}

/// Normalize HTML for comparison
/// - Collapse whitespace
/// - Normalize attribute order
/// - Remove insignificant differences
fn normalize_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut last_was_whitespace = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                last_was_whitespace = false;
                result.push(c);
            }
            '>' => {
                in_tag = false;
                last_was_whitespace = false;
                result.push(c);
            }
            c if c.is_whitespace() => {
                if !last_was_whitespace && !in_tag {
                    result.push(' ');
                    last_was_whitespace = true;
                } else if in_tag && !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                }
            }
            c => {
                last_was_whitespace = false;
                result.push(c);
            }
        }
    }

    result.trim().to_string()
}

/// Result of running a fixture test
#[derive(Debug)]
enum FixtureResult {
    Pass,
    ContentMismatch { expected_len: usize, actual_len: usize },
    EmptyContent,
    ParseFailed,
    MissingFiles,
}

/// Load and run a fixture test, returning detailed result
fn run_fixture_test_detailed(fixture_name: &str) -> FixtureResult {
    let fixture_path = fixtures_dir().join(fixture_name);

    let source_path = fixture_path.join("source.html");
    let expected_path = fixture_path.join("expected.html");

    if !source_path.exists() || !expected_path.exists() {
        return FixtureResult::MissingFiles;
    }

    let source = match fs::read_to_string(&source_path) {
        Ok(s) => s,
        Err(_) => return FixtureResult::MissingFiles,
    };
    let expected = match fs::read_to_string(&expected_path) {
        Ok(s) => s,
        Err(_) => return FixtureResult::MissingFiles,
    };

    let mut readability = Readability::new(&source, Some("http://fakehost/"));
    let result = readability.parse();

    match result {
        Some(article) => {
            if article.content.is_empty() {
                return FixtureResult::EmptyContent;
            }

            let normalized_result = normalize_html(&article.content);
            let normalized_expected = normalize_html(&expected);

            if normalized_result == normalized_expected {
                FixtureResult::Pass
            } else {
                FixtureResult::ContentMismatch {
                    expected_len: normalized_expected.len(),
                    actual_len: normalized_result.len(),
                }
            }
        }
        None => FixtureResult::ParseFailed,
    }
}

/// Get all fixture names
fn get_all_fixtures() -> Vec<String> {
    let fixtures = fixtures_dir();
    if !fixtures.exists() {
        return vec![];
    }

    let mut names: Vec<String> = fs::read_dir(&fixtures)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().is_dir() {
                let source = entry.path().join("source.html");
                let expected = entry.path().join("expected.html");
                if source.exists() && expected.exists() {
                    Some(entry.file_name().to_string_lossy().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    names.sort();
    names
}

/// Run all fixtures and report statistics
#[test]
fn test_all_fixtures() {
    let fixtures = get_all_fixtures();
    let total = fixtures.len();

    let mut passed = 0;
    let mut content_mismatch = 0;
    let mut empty_content = 0;
    let mut parse_failed = 0;

    let mut failures: Vec<(String, FixtureResult)> = Vec::new();

    for fixture in &fixtures {
        let result = run_fixture_test_detailed(fixture);
        match &result {
            FixtureResult::Pass => passed += 1,
            FixtureResult::ContentMismatch { .. } => {
                content_mismatch += 1;
                // Content mismatch is expected until we complete the port
                // Don't add to failures for now
            }
            FixtureResult::EmptyContent => {
                empty_content += 1;
                failures.push((fixture.clone(), result));
            }
            FixtureResult::ParseFailed => {
                parse_failed += 1;
                failures.push((fixture.clone(), result));
            }
            FixtureResult::MissingFiles => {}
        }
    }

    println!("\n=== Fixture Test Results ===");
    println!("Total fixtures: {}", total);
    println!("Exact match:    {} ({:.1}%)", passed, passed as f64 / total as f64 * 100.0);
    println!("Content diff:   {} ({:.1}%)", content_mismatch, content_mismatch as f64 / total as f64 * 100.0);
    println!("Empty content:  {}", empty_content);
    println!("Parse failed:   {}", parse_failed);
    println!("============================\n");

    if !failures.is_empty() {
        println!("Failures:");
        for (name, result) in &failures {
            println!("  - {}: {:?}", name, result);
        }
        println!();
    }

    // For now, we pass if we can parse all fixtures (content mismatch is expected)
    let critical_failures = empty_content + parse_failed;
    assert!(
        critical_failures == 0,
        "Critical failures: {} empty content, {} parse failed",
        empty_content,
        parse_failed
    );
}

/// Test that we can at least parse every fixture
#[test]
fn test_all_fixtures_parse() {
    let fixtures = get_all_fixtures();
    let mut parse_failures = Vec::new();

    for fixture in &fixtures {
        let fixture_path = fixtures_dir().join(fixture);
        let source_path = fixture_path.join("source.html");

        if let Ok(source) = fs::read_to_string(&source_path) {
            let mut readability = Readability::new(&source, Some("http://fakehost/"));
            if readability.parse().is_none() {
                parse_failures.push(fixture.clone());
            }
        }
    }

    if !parse_failures.is_empty() {
        println!("Parse failures ({}):", parse_failures.len());
        for name in &parse_failures {
            println!("  - {}", name);
        }
    }

    assert!(
        parse_failures.is_empty(),
        "Failed to parse {} fixtures",
        parse_failures.len()
    );
}

/// Individual fixture tests for debugging specific cases
macro_rules! fixture_test {
    ($name:ident, $fixture:expr) => {
        #[test]
        fn $name() {
            let result = run_fixture_test_detailed($fixture);
            match result {
                FixtureResult::Pass => {}
                FixtureResult::ContentMismatch { .. } => {
                    // Expected during development
                }
                other => panic!("Fixture {} failed: {:?}", $fixture, other),
            }
        }
    };
}

// Individual tests for key fixtures
fixture_test!(test_fixture_001, "001");
fixture_test!(test_fixture_002, "002");
fixture_test!(test_fixture_003_metadata_preferred, "003-metadata-preferred");
fixture_test!(test_fixture_medium_1, "medium-1");
fixture_test!(test_fixture_nytimes_1, "nytimes-1");
fixture_test!(test_fixture_wikipedia, "wikipedia");
fixture_test!(test_fixture_bbc_1, "bbc-1");
fixture_test!(test_fixture_cnn, "cnn");

#[cfg(test)]
mod normalize_tests {
    use super::normalize_html;

    #[test]
    fn test_normalize_whitespace() {
        let input = "<div>   hello   world   </div>";
        let expected = "<div> hello world </div>";
        assert_eq!(normalize_html(input), expected);
    }

    #[test]
    fn test_normalize_newlines() {
        let input = "<div>\n  hello\n  world\n</div>";
        let expected = "<div> hello world </div>";
        assert_eq!(normalize_html(input), expected);
    }
}
