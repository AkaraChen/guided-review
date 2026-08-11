use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    process::{Command, ExitCode},
};

use anyhow::{Context, Result, anyhow, bail};
use guided_review_core::{AgentKind, ReviewScope, render_agent_prompt_with_target};

use crate::AgentArgs;

pub fn run(args: AgentArgs, kind: AgentKind) -> Result<ExitCode> {
    let resolved = resolve_scope(&args)?;
    let render_target = args.pr.as_ref().map(ToString::to_string);
    let prompt = render_agent_prompt_with_target(kind, &resolved.scope, render_target.as_deref())
        .context("failed to build the Guided Review agent prompt")?;

    if args.print {
        print!("{prompt}");
        return Ok(ExitCode::SUCCESS);
    }

    eprintln!("scope: {}", resolved.summary);
    let binary = agent_binary(&args, kind);
    let argv = agent_argv(kind, args.headless, &prompt, &args.agent_args);
    if args.dry_run {
        println!("{}", display_argv(&binary, &argv));
        return Ok(ExitCode::SUCCESS);
    }

    let status = Command::new(&binary)
        .args(&argv)
        .status()
        .map_err(|error| spawn_error(kind, &binary, error))?;
    let code = status.code().unwrap_or(1);
    let code = u8::try_from(code).unwrap_or(1);
    Ok(ExitCode::from(code))
}

struct ResolvedScope {
    scope: ReviewScope,
    summary: String,
}

fn resolve_scope(args: &AgentArgs) -> Result<ResolvedScope> {
    if let Some(base) = &args.branch {
        let base = if base.is_empty() {
            origin_head()?
        } else {
            base.clone()
        };
        return Ok(ResolvedScope {
            summary: format!("branch (base {base})"),
            scope: ReviewScope::Branch { base },
        });
    }
    if let Some(range) = &args.commits {
        return Ok(ResolvedScope {
            summary: format!("commits ({range})"),
            scope: ReviewScope::Commits {
                range: range.clone(),
            },
        });
    }
    if args.staged {
        return Ok(ResolvedScope {
            summary: "staged".into(),
            scope: ReviewScope::Staged,
        });
    }
    if args.worktree {
        return Ok(ResolvedScope {
            summary: "worktree".into(),
            scope: ReviewScope::Worktree,
        });
    }
    if !args.paths.is_empty() {
        let paths = args
            .paths
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        return Ok(ResolvedScope {
            summary: format!("paths ({} selected)", paths.len()),
            scope: ReviewScope::Paths { paths },
        });
    }
    if let Some(reference) = &args.pr {
        return Ok(ResolvedScope {
            summary: format!("pr ({reference})"),
            scope: ReviewScope::PullRequest {
                reference: reference.clone(),
            },
        });
    }

    require_git_repository()?;
    let base = origin_head()?;
    let ahead = git_stdout(["rev-list", "--count", &format!("{base}..HEAD")])?
        .parse::<u64>()
        .context("git returned an invalid ahead count")?;
    if ahead > 0 {
        return Ok(ResolvedScope {
            summary: format!("branch (base {base}, {ahead} commits)"),
            scope: ReviewScope::Branch { base },
        });
    }

    if !git_stdout(["status", "--porcelain"])?.is_empty() {
        return Ok(ResolvedScope {
            summary: "worktree (local changes)".into(),
            scope: ReviewScope::Worktree,
        });
    }

    bail!(explicit_scope_message())
}

fn require_git_repository() -> Result<()> {
    let status = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("could not run git while selecting a review scope")?;
    if !status.status.success() {
        bail!(explicit_scope_message());
    }
    Ok(())
}

fn origin_head() -> Result<String> {
    git_stdout([
        "symbolic-ref",
        "--quiet",
        "--short",
        "refs/remotes/origin/HEAD",
    ])
    .map_err(|_| {
        anyhow!("origin/HEAD is not configured; pass an explicit base with --branch <BASE>")
    })
}

fn git_stdout<const N: usize>(args: [&str; N]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("could not run git while selecting a review scope")?;
    if !output.status.success() {
        bail!("git could not resolve the requested review scope");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn explicit_scope_message() -> &'static str {
    "could not infer a review scope; explicitly choose --pr, --branch, --commits, --staged, --worktree, or --paths"
}

fn agent_binary(args: &AgentArgs, kind: AgentKind) -> OsString {
    if let Some(path) = &args.agent_bin {
        return path.as_os_str().to_owned();
    }
    let variable = match kind {
        AgentKind::Claude => "EGR_CLAUDE_BIN",
        AgentKind::Codex => "EGR_CODEX_BIN",
    };
    env::var_os(variable).unwrap_or_else(|| match kind {
        AgentKind::Claude => OsString::from("claude"),
        AgentKind::Codex => OsString::from("codex"),
    })
}

fn agent_argv(
    kind: AgentKind,
    headless: bool,
    prompt: &str,
    passthrough: &[String],
) -> Vec<OsString> {
    let mut argv = Vec::new();
    if headless {
        argv.push(match kind {
            AgentKind::Claude => OsString::from("-p"),
            AgentKind::Codex => OsString::from("exec"),
        });
    }
    argv.push(prompt.into());
    argv.extend(passthrough.iter().map(OsString::from));
    argv
}

fn display_argv(binary: &OsStr, argv: &[OsString]) -> String {
    std::iter::once(binary)
        .chain(argv.iter().map(OsString::as_os_str))
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn spawn_error(kind: AgentKind, binary: &OsStr, error: io::Error) -> anyhow::Error {
    if error.kind() == io::ErrorKind::NotFound {
        let install = match kind {
            AgentKind::Claude => "install Claude Code or set EGR_CLAUDE_BIN/--agent-bin",
            AgentKind::Codex => "install Codex or set EGR_CODEX_BIN/--agent-bin",
        };
        anyhow!("agent executable {binary:?} was not found; {install}")
    } else {
        anyhow!("failed to start agent executable {binary:?}: {error}")
    }
}

#[cfg(test)]
mod tests {
    use super::agent_argv;
    use guided_review_core::AgentKind;

    #[test]
    fn headless_forms_match_each_agent() {
        assert_eq!(
            agent_argv(AgentKind::Claude, true, "prompt", &[]),
            ["-p", "prompt"]
        );
        assert_eq!(
            agent_argv(AgentKind::Codex, true, "prompt", &[]),
            ["exec", "prompt"]
        );
        assert_eq!(
            agent_argv(AgentKind::Codex, false, "prompt", &[]),
            ["prompt"]
        );
    }
}
