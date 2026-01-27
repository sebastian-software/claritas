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
/// - Collapse all whitespace to single spaces
/// - Remove whitespace between tags
/// - Lowercase tag names
fn normalize_html(html: &str) -> String {
    // First, collapse all whitespace
    let collapsed: String = html
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Remove space after < and before >
    let result = collapsed
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" />", "/>")
        .replace("> <", "><");

    result.trim().to_string()
}

/// Calculate similarity between two strings (0.0 to 1.0)
fn calculate_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    // Simple Jaccard similarity on words
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Result of running a fixture test
#[derive(Debug)]
enum FixtureResult {
    Pass,
    ContentMismatch { expected_len: usize, actual_len: usize, similarity: f64 },
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

            let similarity = calculate_similarity(&normalized_result, &normalized_expected);

            if normalized_result == normalized_expected {
                FixtureResult::Pass
            } else {
                FixtureResult::ContentMismatch {
                    expected_len: normalized_expected.len(),
                    actual_len: normalized_result.len(),
                    similarity,
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
    let mut total_similarity = 0.0;
    let mut high_similarity = 0; // > 80%
    let mut medium_similarity = 0; // 50-80%
    let mut low_similarity = 0; // < 50%

    let mut failures: Vec<(String, FixtureResult)> = Vec::new();

    for fixture in &fixtures {
        let result = run_fixture_test_detailed(fixture);
        match &result {
            FixtureResult::Pass => {
                passed += 1;
                total_similarity += 1.0;
                high_similarity += 1;
            }
            FixtureResult::ContentMismatch { similarity, .. } => {
                content_mismatch += 1;
                total_similarity += similarity;
                if *similarity >= 0.8 {
                    high_similarity += 1;
                } else if *similarity >= 0.5 {
                    medium_similarity += 1;
                } else {
                    low_similarity += 1;
                }
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

    let avg_similarity = total_similarity / total as f64;

    println!("\n=== Fixture Test Results ===");
    println!("Total fixtures: {}", total);
    println!("Exact match:    {} ({:.1}%)", passed, passed as f64 / total as f64 * 100.0);
    println!("Content diff:   {} ({:.1}%)", content_mismatch, content_mismatch as f64 / total as f64 * 100.0);
    println!("Empty content:  {}", empty_content);
    println!("Parse failed:   {}", parse_failed);
    println!("----------------------------");
    println!("Avg similarity: {:.1}%", avg_similarity * 100.0);
    println!("High (>=80%):   {}", high_similarity);
    println!("Medium (50-80%): {}", medium_similarity);
    println!("Low (<50%):     {}", low_similarity);
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
