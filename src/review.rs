use garde::Validate;
use serde::{
    Deserialize, Deserializer, Serialize,
    ser::{SerializeStruct, Serializer},
};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct GuidedReview {
    #[garde(custom(non_blank))]
    pub title: String,
    #[garde(dive)]
    pub metadata: PullRequestMetadata,
    #[garde(dive)]
    pub thesis: SupportedClaim,
    #[garde(dive, length(min = 1))]
    pub reading_order: Vec<ReadingStep>,
    #[garde(dive, length(min = 1))]
    pub line_map: Vec<LineMapEntry>,
    #[serde(default)]
    #[garde(dive)]
    pub risks: Vec<Risk>,
    #[garde(dive, length(min = 1))]
    pub verification: Vec<Verification>,
    #[serde(default)]
    #[garde(dive)]
    pub questions: Vec<Question>,
    #[garde(dive)]
    pub recommendation: Recommendation,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct PullRequestMetadata {
    #[garde(custom(non_blank))]
    pub files: String,
    #[garde(custom(non_blank))]
    pub additions: String,
    #[garde(custom(non_blank))]
    pub deletions: String,
    #[garde(custom(non_blank))]
    pub tests: String,
    #[garde(custom(non_blank))]
    pub merge: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct SupportedClaim {
    #[garde(custom(non_blank))]
    pub text: String,
    #[garde(skip)]
    pub basis: ClaimBasis,
    #[garde(dive, length(min = 1))]
    pub evidence: Vec<CodeExcerpt>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ReadingStep {
    #[serde(deserialize_with = "path_or_paths")]
    #[garde(length(min = 1), inner(custom(non_blank)))]
    pub path: Vec<String>,
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

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LineMapEntry {
    #[garde(skip)]
    pub line: ReviewLine,
    #[garde(dive)]
    pub claim: SupportedClaim,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Risk {
    #[garde(skip)]
    pub level: RiskLevel,
    #[garde(dive)]
    pub claim: SupportedClaim,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    #[garde(skip)]
    pub status: VerificationStatus,
    #[garde(dive)]
    pub claim: SupportedClaim,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Question {
    #[garde(custom(non_blank))]
    pub text: String,
    #[garde(dive, length(min = 1))]
    pub evidence: Vec<CodeExcerpt>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(custom(validate_recommendation))]
pub struct Recommendation {
    #[garde(skip)]
    pub decision: ReviewDecision,
    #[garde(dive)]
    pub summary: SupportedClaim,
    #[serde(default)]
    #[garde(dive)]
    pub blockers: Vec<SupportedClaim>,
    #[serde(default)]
    #[garde(dive)]
    pub follow_ups: Vec<SupportedClaim>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(custom(validate_excerpt))]
pub struct CodeExcerpt {
    #[garde(custom(non_blank))]
    pub name: String,
    #[garde(custom(non_blank))]
    pub path: String,
    #[garde(range(min = 1))]
    pub start_line: u32,
    #[garde(range(min = 1))]
    pub end_line: u32,
    #[garde(custom(non_blank))]
    pub language: String,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ClaimBasis {
    #[serde(rename(serialize = "Observed", deserialize = "observed"))]
    Observed,
    #[serde(rename(serialize = "Synthesis", deserialize = "synthesis"))]
    Synthesis,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RiskLevel {
    #[serde(rename(serialize = "Blocker", deserialize = "blocker"))]
    Blocker,
    #[serde(rename(serialize = "Should fix", deserialize = "should_fix"))]
    ShouldFix,
    #[serde(rename(serialize = "Follow-up", deserialize = "follow_up"))]
    FollowUp,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum VerificationStatus {
    #[serde(rename(serialize = "Verified", deserialize = "verified"))]
    Verified,
    #[serde(rename(serialize = "Partial", deserialize = "partial"))]
    Partial,
    #[serde(rename(serialize = "Unproven", deserialize = "unproven"))]
    Unproven,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReviewDecision {
    #[serde(rename(serialize = "Approve", deserialize = "approve"))]
    Approve,
    #[serde(rename(serialize = "Request changes", deserialize = "request_changes"))]
    RequestChanges,
    #[serde(rename(serialize = "Comment only", deserialize = "comment_only"))]
    CommentOnly,
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
                "claim": supported_claim("Visible behavior")
            }],
            "risks": [],
            "verification": [{
                "status": "verified",
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
