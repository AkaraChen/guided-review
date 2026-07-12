use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use guided_review_core::{PullRequestRef, review_schema_help};

#[derive(Debug, Parser)]
#[command(
    name = "egr",
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
    /// Serve a directory of generated review pages over local HTTP.
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
#[command(after_help = review_schema_help())]
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

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Directory to serve.
    #[arg(value_name = "DIR", default_value = ".")]
    pub dir: PathBuf,

    /// Port to bind on 127.0.0.1; 0 picks a free port and prints it.
    #[arg(short, long, value_name = "PORT", default_value_t = 0)]
    pub port: u16,
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

        let Command::Generate(args) = cli.command else {
            panic!("expected the generate subcommand");
        };
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

        let Command::Generate(args) = cli.command else {
            panic!("expected the generate subcommand");
        };
        assert_eq!(args.review.to_string_lossy(), "-");
        assert!(args.output.is_none());
    }

    #[test]
    fn long_help_documents_the_review_schema() {
        for flag in ["-h", "--help"] {
            let error = Cli::try_parse_from(["guided-review", "generate", flag])
                .expect_err("help flag exits with a help message");

            assert!(error.to_string().contains("Review JSON schema"));
        }
    }

    #[test]
    fn rejects_malformed_pull_request_reference() {
        let error = Cli::try_parse_from(["guided-review", "generate", "github/desktop/pull/144"])
            .expect_err("malformed pull request should fail");

        assert!(error.to_string().contains("OWNER/REPO#NUMBER"));
    }
}
