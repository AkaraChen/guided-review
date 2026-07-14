use garde::Validate;
use schemars::JsonSchema;
use serde::{
    Deserialize, Deserializer, Serialize,
    ser::{SerializeStruct, Serializer},
};
use thiserror::Error;

/// A complete guided review of one pull request. Every claim must cite at
/// least one code excerpt as evidence, and text fields must not be blank.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct GuidedReview {
    /// Review headline shown at the top of the page.
    #[garde(custom(non_blank))]
    pub title: String,
    /// Pull request statistics displayed in the page header.
    #[garde(dive)]
    pub metadata: PullRequestMetadata,
    /// One-paragraph statement of what the change accomplishes.
    #[garde(dive)]
    pub thesis: SupportedClaim,
    /// Suggested order in which a reviewer should read the changed files.
    #[schemars(length(min = 1))]
    #[garde(dive, length(min = 1))]
    pub reading_order: Vec<ReadingStep>,
    /// Claims organized by review line (visible behavior, data, state, ...).
    #[schemars(length(min = 1))]
    #[garde(dive, length(min = 1))]
    pub line_map: Vec<LineMapEntry>,
    /// Identified risks, from blockers down to follow-ups.
    #[serde(default)]
    #[garde(dive)]
    pub risks: Vec<Risk>,
    /// What has been proven, partially proven, or remains unproven.
    #[schemars(length(min = 1))]
    #[garde(dive, length(min = 1))]
    pub verification: Vec<Verification>,
    /// Open questions for the pull request author.
    #[serde(default)]
    #[garde(dive)]
    pub questions: Vec<Question>,
    /// The final review decision with its justification.
    #[garde(dive)]
    pub recommendation: Recommendation,
}

/// Display-only pull request statistics, already formatted for the header.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct PullRequestMetadata {
    /// Changed-file count, e.g. "60 changed".
    #[garde(custom(non_blank))]
    pub files: String,
    /// Added-line count, e.g. "+4,835".
    #[garde(custom(non_blank))]
    pub additions: String,
    /// Deleted-line count, e.g. "-217".
    #[garde(custom(non_blank))]
    pub deletions: String,
    /// Test outcome, e.g. "378 passed".
    #[garde(custom(non_blank))]
    pub tests: String,
    /// Merge state, e.g. "CLEAN".
    #[garde(custom(non_blank))]
    pub merge: String,
}

/// A reviewer statement backed by at least one cited code excerpt.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct SupportedClaim {
    /// The statement itself, one or two sentences.
    #[garde(custom(non_blank))]
    pub text: String,
    /// How the claim was established.
    #[garde(skip)]
    pub basis: ClaimBasis,
    /// Code excerpts that support the claim.
    #[schemars(length(min = 1))]
    #[garde(dive, length(min = 1))]
    pub evidence: Vec<CodeExcerpt>,
}

/// One stop in the suggested reading order.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ReadingStep {
    /// Repository-relative file path, or a list of paths read together.
    #[serde(deserialize_with = "path_or_paths")]
    #[schemars(schema_with = "path_or_paths_schema")]
    #[garde(length(min = 1), inner(custom(non_blank)))]
    pub path: Vec<String>,
    /// Why the reviewer should read this file at this point.
    #[garde(dive)]
    pub reason: SupportedClaim,
}

/// Accepts `"path": "a.rs"` and `"path": ["a.rs", "b.rs"]` interchangeably.
fn path_or_paths<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PathInput {
        One(String),
        Many(Vec<String>),
    }

    Ok(match PathInput::deserialize(deserializer)? {
        PathInput::One(path) => vec![path],
        PathInput::Many(paths) => paths,
    })
}

/// Schema counterpart of [`path_or_paths`]: one path or a non-empty list.
fn path_or_paths_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "anyOf": [
            { "type": "string", "minLength": 1 },
            { "type": "array", "items": { "type": "string", "minLength": 1 }, "minItems": 1 }
        ]
    })
}

/// A claim filed under one review line.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LineMapEntry {
    /// The review line this claim belongs to.
    #[garde(skip)]
    pub line: ReviewLine,
    /// Short label for the claim.
    #[garde(custom(non_blank))]
    pub title: String,
    /// What the review asserts about this line.
    #[garde(dive)]
    pub claim: SupportedClaim,
}

/// A risk identified in the change.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Risk {
    /// Severity of the risk.
    #[garde(skip)]
    pub level: RiskLevel,
    /// Short label for the risk.
    #[garde(custom(non_blank))]
    pub title: String,
    /// What the risk is and why it matters.
    #[garde(dive)]
    pub claim: SupportedClaim,
}

/// One verification item: something the review checked or could not check.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    /// How far the item has been proven.
    #[garde(skip)]
    pub status: VerificationStatus,
    /// Short label for the verification item.
    #[garde(custom(non_blank))]
    pub title: String,
    /// What was verified, or what evidence is missing.
    #[garde(dive)]
    pub claim: SupportedClaim,
}

/// An open question for the pull request author.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Question {
    /// Short label for the question.
    #[garde(custom(non_blank))]
    pub title: String,
    /// The question itself.
    #[garde(custom(non_blank))]
    pub text: String,
    /// Code excerpts that prompted the question.
    #[schemars(length(min = 1))]
    #[garde(dive, length(min = 1))]
    pub evidence: Vec<CodeExcerpt>,
}

/// The final review decision. An `approve` decision cannot include blockers.
#[derive(Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(custom(validate_recommendation))]
pub struct Recommendation {
    /// The verdict on the pull request.
    #[garde(skip)]
    pub decision: ReviewDecision,
    /// Justification for the decision.
    #[garde(dive)]
    pub summary: SupportedClaim,
    /// Issues that must be resolved before merging.
    #[serde(default)]
    #[garde(dive)]
    pub blockers: Vec<SupportedClaim>,
    /// Improvements that can land after this pull request.
    #[serde(default)]
    #[garde(dive)]
    pub follow_ups: Vec<SupportedClaim>,
}

/// A cited code excerpt. `code` must contain exactly
/// `end_line - start_line + 1` lines.
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
#[garde(custom(validate_excerpt))]
pub struct CodeExcerpt {
    /// Short caption naming what the excerpt shows.
    #[garde(custom(non_blank))]
    pub name: String,
    /// Repository-relative path of the excerpted file.
    #[garde(custom(non_blank))]
    pub path: String,
    /// First excerpted line, 1-indexed.
    #[schemars(range(min = 1))]
    #[garde(range(min = 1))]
    pub start_line: u32,
    /// Last excerpted line, inclusive; must be >= start_line.
    #[schemars(range(min = 1))]
    #[garde(range(min = 1))]
    pub end_line: u32,
    /// Syntax-highlighting language tag, e.g. "rust" or "typescript".
    #[garde(custom(non_blank))]
    pub language: String,
    /// The excerpted source, one line per range line.
    #[garde(custom(non_blank))]
    pub code: String,
}

impl Serialize for CodeExcerpt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let lines = self
            .code
            .lines()
            .enumerate()
            .map(|(index, text)| CodeLine {
                number: self.start_line + index as u32,
                text,
            })
            .collect::<Vec<_>>();
        let mut excerpt = serializer.serialize_struct("CodeExcerpt", 7)?;
        excerpt.serialize_field("name", &self.name)?;
        excerpt.serialize_field("path", &self.path)?;
        excerpt.serialize_field("start_line", &self.start_line)?;
        excerpt.serialize_field("end_line", &self.end_line)?;
        excerpt.serialize_field("language", &self.language)?;
        excerpt.serialize_field("code", &self.code)?;
        excerpt.serialize_field("lines", &lines)?;
        excerpt.end()
    }
}

#[derive(Serialize)]
struct CodeLine<'a> {
    number: u32,
    text: &'a str,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
/// How a claim was established: read directly at the cited site (observed)
/// or concluded from several sites together (synthesis).
pub enum ClaimBasis {
    #[serde(rename(serialize = "Observed", deserialize = "observed"))]
    Observed,
    #[serde(rename(serialize = "Synthesis", deserialize = "synthesis"))]
    Synthesis,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
/// The review dimension a line-map claim addresses, from user-visible
/// behavior down to hidden coupling and degradation.
pub enum ReviewLine {
    #[serde(rename(serialize = "Visible line", deserialize = "visible"))]
    Visible,
    #[serde(rename(serialize = "Hidden line", deserialize = "hidden"))]
    Hidden,
    #[serde(rename(serialize = "Data line", deserialize = "data"))]
    Data,
    #[serde(rename(serialize = "State line", deserialize = "state"))]
    State,
    #[serde(rename(serialize = "Permission line", deserialize = "permission"))]
    Permission,
    #[serde(rename(serialize = "Error line", deserialize = "error"))]
    Error,
    #[serde(rename(serialize = "Test line", deserialize = "test"))]
    Test,
    #[serde(rename(serialize = "Complexity line", deserialize = "complexity"))]
    Complexity,
    #[serde(rename(serialize = "Degradation line", deserialize = "degradation"))]
    Degradation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
/// Risk severity: blocker (must fix before merge), should_fix (fix in this
/// pull request), or follow_up (may land later).
pub enum RiskLevel {
    #[serde(rename(serialize = "Blocker", deserialize = "blocker"))]
    Blocker,
    #[serde(rename(serialize = "Should fix", deserialize = "should_fix"))]
    ShouldFix,
    #[serde(rename(serialize = "Follow-up", deserialize = "follow_up"))]
    FollowUp,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
/// How far a verification item has been proven by tests or evidence.
pub enum VerificationStatus {
    #[serde(rename(serialize = "Verified", deserialize = "verified"))]
    Verified,
    #[serde(rename(serialize = "Partial", deserialize = "partial"))]
    Partial,
    #[serde(rename(serialize = "Unproven", deserialize = "unproven"))]
    Unproven,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
/// The overall verdict on the pull request.
pub enum ReviewDecision {
    #[serde(rename(serialize = "Approve", deserialize = "approve"))]
    Approve,
    #[serde(rename(serialize = "Request changes", deserialize = "request_changes"))]
    RequestChanges,
    #[serde(rename(serialize = "Comment only", deserialize = "comment_only"))]
    CommentOnly,
}

/// JSON Schema for the review payload, generated from the types above and
/// rendered into `generate --help`.
pub fn review_schema_help() -> String {
    let schema = schemars::schema_for!(GuidedReview);
    format!(
        "Review JSON schema (generated from the Rust types):\n{}",
        serde_json::to_string_pretty(schema.as_value()).expect("schema serializes to JSON")
    )
}

pub fn parse_review(source: &str) -> Result<GuidedReview, ReviewInputError> {
    let review: GuidedReview = serde_json::from_str(source)?;
    review
        .validate()
        .map_err(|report| ReviewInputError::InvalidReview(report.to_string()))?;
    Ok(review)
}

#[derive(Debug, Error)]
pub enum ReviewInputError {
    #[error("could not parse JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("review failed validation:\n{0}")]
    InvalidReview(String),
}

fn non_blank(value: &str, _context: &()) -> garde::Result {
    if value.trim().is_empty() {
        Err(garde::Error::new("must not be blank"))
    } else {
        Ok(())
    }
}

fn validate_excerpt(excerpt: &CodeExcerpt, _context: &()) -> garde::Result {
    if excerpt.start_line == 0 || excerpt.end_line < excerpt.start_line {
        return Err(garde::Error::new(
            "end_line must be greater than or equal to start_line",
        ));
    }

    let expected_line_count = (excerpt.end_line - excerpt.start_line + 1) as usize;
    let actual_line_count = excerpt.code.lines().count();
    if actual_line_count != expected_line_count {
        return Err(garde::Error::new(format!(
            "line range describes {expected_line_count} lines but code contains {actual_line_count}"
        )));
    }

    Ok(())
}

fn validate_recommendation(recommendation: &Recommendation, _context: &()) -> garde::Result {
    if recommendation.decision == ReviewDecision::Approve && !recommendation.blockers.is_empty() {
        return Err(garde::Error::new(
            "an approve recommendation cannot include blockers",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::parse_review;

    fn supported_claim(text: &str) -> Value {
        json!({
            "text": text,
            "basis": "observed",
            "evidence": [{
                "name": "Relevant code",
                "path": "src/core.rs",
                "start_line": 1,
                "end_line": 1,
                "language": "rust",
                "code": "fn core() {}"
            }]
        })
    }

    fn valid_review() -> Value {
        json!({
            "title": "Review",
            "metadata": {
                "files": "1 changed",
                "additions": "+1",
                "deletions": "-0",
                "tests": "1 passed",
                "merge": "CLEAN"
            },
            "thesis": supported_claim("Thesis"),
            "reading_order": [{
                "path": "src/core.rs",
                "reason": supported_claim("Start here")
            }],
            "line_map": [{
                "line": "visible",
                "title": "Visible behavior",
                "claim": supported_claim("Visible behavior")
            }],
            "risks": [],
            "verification": [{
                "status": "verified",
                "title": "Verification title",
                "claim": supported_claim("Verification")
            }],
            "questions": [],
            "recommendation": {
                "decision": "comment_only",
                "summary": supported_claim("Recommendation"),
                "blockers": [],
                "follow_ups": []
            }
        })
    }

    #[test]
    fn schema_help_documents_input_enum_values() {
        let help = super::review_schema_help();

        // Enums rename differently for input and output; the schema must
        // document what `parse_review` accepts, not what templates receive.
        assert!(help.contains("\"observed\""));
        assert!(!help.contains("\"Observed\""));
    }

    #[test]
    fn accepts_an_otherwise_minimal_valid_review() {
        parse_review(&valid_review().to_string()).expect("valid review");
    }

    #[test]
    fn accepts_a_reading_step_with_multiple_paths() {
        let mut review = valid_review();
        review["reading_order"][0]["path"] = json!(["src/core.rs", "src/render.rs"]);

        parse_review(&review.to_string()).expect("valid multi-path reading step");
    }

    #[test]
    fn rejects_a_reading_step_without_paths() {
        let mut review = valid_review();
        review["reading_order"][0]["path"] = json!([]);

        let error = parse_review(&review.to_string()).expect_err("empty path list");

        assert!(error.to_string().contains("reading_order[0].path"));
    }

    #[test]
    fn rejects_a_claim_without_code_evidence() {
        let mut review = valid_review();
        review["thesis"]["evidence"] = json!([]);

        let error = parse_review(&review.to_string()).expect_err("unsupported claim");

        assert!(error.to_string().contains("thesis.evidence"));
    }

    #[test]
    fn rejects_an_excerpt_whose_range_does_not_match_its_code() {
        let mut review = valid_review();
        review["thesis"]["evidence"][0]["end_line"] = json!(2);

        let error = parse_review(&review.to_string()).expect_err("mismatched line range");

        assert!(
            error
                .to_string()
                .contains("line range describes 2 lines but code contains 1")
        );
    }

    #[test]
    fn rejects_approve_when_blockers_are_present() {
        let mut review = valid_review();
        review["recommendation"]["decision"] = json!("approve");
        review["recommendation"]["blockers"] = json!([supported_claim("Blocker")]);

        let error = parse_review(&review.to_string()).expect_err("contradictory recommendation");

        assert!(
            error
                .to_string()
                .contains("an approve recommendation cannot include blockers")
        );
    }

    #[test]
    fn rejects_unknown_fields_at_the_json_boundary() {
        let mut review = valid_review();
        review
            .as_object_mut()
            .expect("review object")
            .insert("unexpected".to_owned(), json!(true));

        let error = parse_review(&review.to_string()).expect_err("unknown field");

        assert!(error.to_string().contains("unknown field `unexpected`"));
    }
}
