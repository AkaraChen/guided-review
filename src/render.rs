use handlebars::Handlebars;
use serde::Serialize;
use thiserror::Error;

use crate::{GuidedReview, PullRequestRef};

const REVIEW_TEMPLATE: &str = include_str!("../templates/guided-review.html.hbs");
const EVIDENCE_PARTIAL: &str = include_str!("../templates/evidence.html.hbs");

#[derive(Serialize)]
struct PageContext<'a> {
    pull_request: PullRequestContext<'a>,
    review: &'a GuidedReview,
    generator_version: &'static str,
}

#[derive(Serialize)]
struct PullRequestContext<'a> {
    owner: &'a str,
    repository: &'a str,
    number: u64,
    display: String,
    url: String,
}

pub fn render_review(
    pull_request: &PullRequestRef,
    review: &GuidedReview,
) -> Result<String, RenderReviewError> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("guided-review", REVIEW_TEMPLATE)?;
    handlebars.register_partial("evidence", EVIDENCE_PARTIAL)?;

    let context = PageContext {
        pull_request: PullRequestContext {
            owner: &pull_request.owner,
            repository: &pull_request.repository,
            number: pull_request.number,
            display: pull_request.to_string(),
            url: pull_request.github_url(),
        },
        review,
        generator_version: env!("CARGO_PKG_VERSION"),
    };

    Ok(handlebars.render("guided-review", &context)?)
}

#[derive(Debug, Error)]
pub enum RenderReviewError {
    #[error("invalid built-in Handlebars template: {0}")]
    InvalidTemplate(#[from] handlebars::TemplateError),
    #[error("Handlebars could not render the review: {0}")]
    Render(#[from] handlebars::RenderError),
}
