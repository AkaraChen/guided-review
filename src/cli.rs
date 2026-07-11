use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::PullRequestRef;

#[derive(Debug, Parser)]
#[command(
    name = "guided-review",
    version,
    about = "Generate a code-backed Guided Review as a self-contained HTML page",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate one HTML page for one GitHub pull request.
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// GitHub pull request in OWNER/REPO#NUMBER format.
    #[arg(value_name = "OWNER/REPO#NUMBER")]
    pub pull_request: PullRequestRef,

    /// Review JSON file. Use '-' to read the JSON body from stdin.
    #[arg(short, long, value_name = "FILE", default_value = "-")]
    pub review: PathBuf,

    /// Destination HTML file. Defaults to OWNER-REPO-NUMBER-guided-review.html.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn parses_generate_contract() {
        let cli = Cli::try_parse_from([
            "guided-review",
            "generate",
            "github/desktop#144",
            "--review",
            "review.json",
            "--output",
            "review.html",
        ])
        .expect("valid CLI arguments");

        let Command::Generate(args) = cli.command;
        assert_eq!(args.pull_request.owner, "github");
        assert_eq!(args.pull_request.repository, "desktop");
        assert_eq!(args.pull_request.number, 144);
        assert_eq!(args.review.to_string_lossy(), "review.json");
        assert_eq!(
            args.output.expect("output path").to_string_lossy(),
            "review.html"
        );
    }

    #[test]
    fn defaults_review_body_to_stdin() {
        let cli = Cli::try_parse_from(["guided-review", "generate", "github/desktop#144"])
            .expect("valid CLI arguments");

        let Command::Generate(args) = cli.command;
        assert_eq!(args.review.to_string_lossy(), "-");
        assert!(args.output.is_none());
    }

    #[test]
    fn rejects_malformed_pull_request_reference() {
        let error = Cli::try_parse_from(["guided-review", "generate", "github/desktop/pull/144"])
            .expect_err("malformed pull request should fail");

        assert!(error.to_string().contains("OWNER/REPO#NUMBER"));
    }
}
