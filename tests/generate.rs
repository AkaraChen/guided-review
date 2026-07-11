use std::str::FromStr;

use guided_review::{GuidedReview, PullRequestRef, parse_review, render_review};
use serde_json::{Value, json};

fn pull_request() -> PullRequestRef {
    PullRequestRef::from_str("owner/repository#7").expect("valid pull request reference")
}

fn supported_claim(text: &str) -> Value {
    json!({
        "text": text,
        "basis": "observed",
        "evidence": [{
            "name": "Relevant code",
            "path": "src/core.rs",
            "start_line": 10,
            "end_line": 11,
            "language": "rust",
            "code": "let first = 1;\nlet second = 2;"
        }]
    })
}

fn synthetic_review() -> GuidedReview {
    let review = json!({
        "title": "A review title",
        "metadata": {
            "files": "3 changed",
            "additions": "+20",
            "deletions": "-4",
            "tests": "12 passed",
            "merge": "CLEAN"
        },
        "thesis": supported_claim("The core behavior changes."),
        "reading_order": [{
            "path": "src/core.rs",
            "reason": supported_claim("Start at the core decision.")
        }],
        "line_map": [{
            "line": "visible",
            "claim": supported_claim("Users can observe the new behavior.")
        }],
        "risks": [],
        "verification": [{
            "status": "verified",
            "claim": supported_claim("The focused test proves the behavior.")
        }],
        "questions": [],
        "recommendation": {
            "decision": "comment_only",
            "summary": supported_claim("The change is ready for human review."),
            "blockers": [],
            "follow_ups": []
        }
    });

    parse_review(&review.to_string()).expect("valid synthetic review")
}

fn render_synthetic_review() -> String {
    render_review(&pull_request(), &synthetic_review()).expect("rendered review")
}

fn section_between<'a>(html: &'a str, section: &str, next_section: &str) -> &'a str {
    html.split_once(&format!("<section id=\"{section}\">"))
        .unwrap_or_else(|| panic!("missing {section} section"))
        .1
        .split_once(&format!("<section id=\"{next_section}\">"))
        .unwrap_or_else(|| panic!("missing {next_section} section after {section}"))
        .0
}

#[test]
fn renders_the_pr_title_and_required_review_sections() {
    let html = render_synthetic_review();

    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<title>A review title</title>"));
    assert!(html.contains("<h1>A review title</h1>"));
    for section in [
        "thesis",
        "reading",
        "lines",
        "verification",
        "risks",
        "questions",
        "recommendation",
    ] {
        assert!(
            html.contains(&format!("<section id=\"{section}\">")),
            "missing required {section} section"
        );
    }
}

#[test]
fn renders_metadata_from_the_review_input() {
    let html = render_synthetic_review();

    for value in ["3 changed", "+20", "-4", "12 passed", "CLEAN"] {
        assert!(html.contains(value), "missing metadata value {value}");
    }
}

#[test]
fn reading_order_does_not_render_code_evidence() {
    let html = render_synthetic_review();
    let reading_order = section_between(&html, "reading", "lines");

    assert!(reading_order.contains("src/core.rs"));
    assert!(reading_order.contains("Start at the core decision."));
    assert!(!reading_order.contains("code-card"));
}

#[test]
fn reading_order_links_files_to_their_pull_request_diff() {
    let html = render_synthetic_review();
    let reading_order = section_between(&html, "reading", "lines");

    // sha256("src/core.rs"): GitHub's per-file anchor in the "Files changed" view.
    let expected = "href=\"https://github.com/owner/repository/pull/7/files#diff-\
7534aab5e9a7a7e69e0ac7e0c3dfe6b38c28ed17ff74289d268738251e1042a3\"";
    assert!(
        reading_order.contains(expected),
        "reading order should link the file to its diff anchor"
    );
    assert!(
        reading_order.contains("target=\"_blank\""),
        "file links should open in a new tab"
    );
}

#[test]
fn code_evidence_rows_do_not_create_blank_preformatted_lines() {
    let html = render_synthetic_review();

    assert!(html.contains("<span class=\"ln\">10</span>"));
    assert!(html.contains("</span></span><span class=\"code-line\">"));
    assert!(!html.contains("</span></span>\n<span class=\"code-line\">"));
}

#[test]
fn escapes_review_copy_before_rendering_html() {
    let mut review = synthetic_review();
    review.title = "<em>unsafe</em>".to_owned();

    let html = render_review(&pull_request(), &review).expect("rendered review");

    assert!(!html.contains("<em>unsafe</em>"));
    assert!(html.contains("&lt;em&gt;unsafe"));
}

#[test]
fn fixed_dev_preview_payload_remains_valid() {
    parse_review(include_str!("../examples/review.json")).expect("valid example review body");
}
