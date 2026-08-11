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
    /// Start Claude Code with a scope-specific Guided Review prompt.
    #[command(visible_alias = "cc")]
    Claude(AgentArgs),
    /// Start Codex with a scope-specific Guided Review prompt.
    Codex(AgentArgs),
    /// Generate one HTML page for one GitHub pull request.
    Generate(GenerateArgs),
    /// Serve a directory of generated review pages over local HTTP.
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
#[command(
    group = clap::ArgGroup::new("local_scope").multiple(false),
    after_help = "When no scope is given, egr selects branch if HEAD is ahead of origin/HEAD, then worktree if local changes exist. Otherwise it asks for an explicit scope. Use -- to pass remaining arguments to the agent."
)]
pub struct AgentArgs {
    /// GitHub pull request to review, or the render target for a local scope.
    #[arg(long, value_name = "OWNER/REPO#NUMBER")]
    pub pr: Option<PullRequestRef>,

    /// Review the current branch against BASE; defaults to origin/HEAD.
    #[arg(long, value_name = "BASE", num_args = 0..=1, default_missing_value = "", group = "local_scope")]
    pub branch: Option<String>,

    /// Review one commit or a commit range.
    #[arg(long, value_name = "RANGE", group = "local_scope")]
    pub commits: Option<String>,

    /// Review staged changes.
    #[arg(long, group = "local_scope")]
    pub staged: bool,

    /// Review unstaged and untracked worktree changes.
    #[arg(long, group = "local_scope")]
    pub worktree: bool,

    /// Review the current contents of one or more files or directories.
    #[arg(long, value_name = "PATH", num_args = 1.., group = "local_scope")]
    pub paths: Vec<PathBuf>,

    /// Print the generated prompt without starting the agent.
    #[arg(long, conflicts_with = "dry_run")]
    pub print: bool,

    /// Print the agent argv without starting it.
    #[arg(long)]
    pub dry_run: bool,

    /// Use the agent's non-interactive command form.
    #[arg(long)]
    pub headless: bool,

    /// Override the agent executable.
    #[arg(long, value_name = "PATH")]
    pub agent_bin: Option<PathBuf>,

    /// Arguments after `--` to pass through to the agent.
    #[arg(last = true, allow_hyphen_values = true)]
    pub agent_args: Vec<String>,
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

    #[test]
    fn parses_agent_commands_and_visible_alias() {
        for name in ["claude", "cc", "codex"] {
            let cli = Cli::try_parse_from(["egr", name, "--staged"]).expect("valid agent command");
            assert!(matches!(
                cli.command,
                Command::Claude(_) | Command::Codex(_)
            ));
        }

        let help = Cli::try_parse_from(["egr", "--help"])
            .expect_err("help exits")
            .to_string();
        assert!(help.contains("cc"));
    }

    #[test]
    fn local_scopes_are_mutually_exclusive() {
        let error = Cli::try_parse_from(["egr", "claude", "--staged", "--worktree"])
            .expect_err("conflicting scopes should fail");
        assert!(error.to_string().contains("cannot be used with"));
    }

    #[test]
    fn pull_request_can_supply_a_local_scope_render_target() {
        let cli = Cli::try_parse_from(["egr", "codex", "--pr", "owner/repo#7", "--branch", "main"])
            .expect("PR and local scope may be combined");
        let Command::Codex(args) = cli.command else {
            panic!("expected codex");
        };
        assert_eq!(args.pr.unwrap().to_string(), "owner/repo#7");
        assert_eq!(args.branch.as_deref(), Some("main"));
    }

    #[test]
    fn passes_arguments_after_separator_through() {
        let cli = Cli::try_parse_from(["egr", "codex", "--staged", "--", "--model", "o3"])
            .expect("agent arguments should not be interpreted by egr");
        let Command::Codex(args) = cli.command else {
            panic!("expected codex");
        };
        assert_eq!(args.agent_args, ["--model", "o3"]);
    }

    #[test]
    fn agent_help_documents_scopes_and_default_selection() {
        for flag in ["-h", "--help"] {
            let help = Cli::try_parse_from(["egr", "claude", flag])
                .expect_err("help exits")
                .to_string();
            for expected in [
                "--pr",
                "--branch",
                "--commits",
                "--staged",
                "--worktree",
                "--paths",
                "origin/HEAD",
            ] {
                assert!(help.contains(expected), "missing {expected} in {flag} help");
            }
        }
    }
}
