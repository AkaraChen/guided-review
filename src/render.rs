use handlebars::{Handlebars, handlebars_helper};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{GuidedReview, PullRequestRef};

const REVIEW_TEMPLATE: &str = include_str!("../templates/guided-review.html.hbs");
const EVIDENCE_PARTIAL: &str = include_str!("../templates/evidence.html.hbs");
const FILE_LINK_PARTIAL: &str = include_str!("../templates/file-link.html.hbs");
const CLAIM_CARD_PARTIAL: &str = include_str!("../templates/claim-card.html.hbs");
const THEME_SWITCH_PARTIAL: &str = include_str!("../templates/theme-switch.html.hbs");
const PAGE_HEADER_PARTIAL: &str = include_str!("../templates/page-header.html.hbs");

/// Repeated components are web components: each partial renders a custom
/// element whose class, styles, and registry guard live in a sibling .js file,
/// emitted once into the page head via the `component-scripts` partial.
/// Components rendered a single time (for example theme-switch) instead inline
/// their own <style>/<script> tags directly in their template.
const COMPONENT_SCRIPTS: &[&str] = &[
    include_str!("../templates/status-badge.js"),
    include_str!("../templates/file-link.js"),
    include_str!("../templates/claim-card.js"),
    include_str!("../templates/code-card.js"),
];

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

// GitHub anchors each file in a PR "Files changed" view as #diff-<sha256(path)>.
handlebars_helper!(diff_anchor: |path: String| {
    Sha256::digest(path.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
});

handlebars_helper!(short_path: |path: String| shorten_path(&path));
handlebars_helper!(inc: |value: u64| value + 1);

/// Middle-elides a long path (`src/…/ai/runner.ts`), always keeping the file name.
const SHORT_PATH_MAX_CHARS: usize = 40;

fn shorten_path(path: &str) -> String {
    if path.chars().count() <= SHORT_PATH_MAX_CHARS {
        return path.to_owned();
    }

    let segments: Vec<&str> = path.split('/').collect();
    for skip in 1..segments.len() {
        let candidate = format!("{}/…/{}", segments[0], segments[skip..].join("/"));
        if candidate.chars().count() <= SHORT_PATH_MAX_CHARS {
            return candidate;
        }
    }

    format!("…/{}", segments.last().copied().unwrap_or(path))
}

pub fn render_review(
    pull_request: &PullRequestRef,
    review: &GuidedReview,
) -> Result<String, RenderReviewError> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_helper("diff_anchor", Box::new(diff_anchor));
    handlebars.register_helper("short_path", Box::new(short_path));
    handlebars.register_helper("inc", Box::new(inc));
    handlebars.register_template_string("guided-review", REVIEW_TEMPLATE)?;
    handlebars.register_partial("evidence", EVIDENCE_PARTIAL)?;
    handlebars.register_partial("file-link", FILE_LINK_PARTIAL)?;
    handlebars.register_partial("claim-card", CLAIM_CARD_PARTIAL)?;
    handlebars.register_partial("theme-switch", THEME_SWITCH_PARTIAL)?;
    handlebars.register_partial("page-header", PAGE_HEADER_PARTIAL)?;
    handlebars.register_partial(
        "component-scripts",
        format!("<script>\n{}</script>", COMPONENT_SCRIPTS.join("\n")),
    )?;

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

#[cfg(test)]
mod tests {
    use super::{SHORT_PATH_MAX_CHARS, shorten_path};

    #[test]
    fn keeps_short_paths_untouched() {
        assert_eq!(shorten_path("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn elides_middle_segments_but_keeps_the_file_name() {
        let shortened = shorten_path("src/main/backend/ai/workspace-agent-runner.ts");

        assert_eq!(shortened, "src/…/ai/workspace-agent-runner.ts");
        assert!(shortened.chars().count() <= SHORT_PATH_MAX_CHARS);
    }

    #[test]
    fn falls_back_to_the_file_name_when_nothing_else_fits() {
        let file_name = "a-very-long-single-file-name-with-many-words.ts";
        let shortened = shorten_path(&format!("deeply/nested/{file_name}"));

        assert_eq!(shortened, format!("…/{file_name}"));
    }
}

#[derive(Debug, Error)]
pub enum RenderReviewError {
    #[error("invalid built-in Handlebars template: {0}")]
    InvalidTemplate(#[from] handlebars::TemplateError),
    #[error("Handlebars could not render the review: {0}")]
    Render(#[from] handlebars::RenderError),
}
