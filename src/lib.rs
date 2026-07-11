mod cli;
mod pull_request;
mod render;
mod review;

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub use cli::{Cli, Command, GenerateArgs};
pub use pull_request::PullRequestRef;
pub use render::{RenderReviewError, render_review};
pub use review::{GuidedReview, ReviewInputError, parse_review};

pub fn execute(cli: Cli) -> Result<PathBuf> {
    match cli.command {
        Command::Generate(args) => generate(args),
    }
}

fn generate(args: GenerateArgs) -> Result<PathBuf> {
    let source = read_review_source(&args.review).with_context(|| {
        format!(
            "failed to read review JSON from {}",
            display_input(&args.review)
        )
    })?;
    let review = parse_review(&source).context("invalid review JSON")?;
    let html =
        render_review(&args.pull_request, &review).context("failed to render review HTML")?;
    let output = args
        .output
        .unwrap_or_else(|| default_output_path(&args.pull_request));

    fs::write(&output, html)
        .with_context(|| format!("failed to write HTML to {}", output.display()))?;

    Ok(output)
}

fn read_review_source(path: &Path) -> io::Result<String> {
    if path == Path::new("-") {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        Ok(source)
    } else {
        fs::read_to_string(path)
    }
}

fn display_input(path: &Path) -> String {
    if path == Path::new("-") {
        "stdin".to_owned()
    } else {
        path.display().to_string()
    }
}

fn default_output_path(pull_request: &PullRequestRef) -> PathBuf {
    format!(
        "{}-{}-{}-guided-review.html",
        pull_request.owner, pull_request.repository, pull_request.number
    )
    .into()
}
