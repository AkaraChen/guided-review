use handlebars::Handlebars;
use serde::Serialize;
use thiserror::Error;

const BASE_TEMPLATE: &str = include_str!("../templates/prompts/base.md.hbs");
const PR_TEMPLATE: &str = include_str!("../templates/prompts/pr.md.hbs");
const BRANCH_TEMPLATE: &str = include_str!("../templates/prompts/branch.md.hbs");
const COMMITS_TEMPLATE: &str = include_str!("../templates/prompts/commits.md.hbs");
const STAGED_TEMPLATE: &str = include_str!("../templates/prompts/staged.md.hbs");
const WORKTREE_TEMPLATE: &str = include_str!("../templates/prompts/worktree.md.hbs");
const PATHS_TEMPLATE: &str = include_str!("../templates/prompts/paths.md.hbs");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    Claude,
    Codex,
}

impl AgentKind {
    fn invocation(self) -> &'static str {
        match self {
            Self::Claude => "Use the guided-review skill",
            Self::Codex => "Use $guided-review",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewScope {
    PullRequest { reference: String },
    Branch { base: String },
    Commits { range: String },
    Staged,
    Worktree,
    Paths { paths: Vec<String> },
}

impl ReviewScope {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PullRequest { .. } => "pr",
            Self::Branch { .. } => "branch",
            Self::Commits { .. } => "commits",
            Self::Staged => "staged",
            Self::Worktree => "worktree",
            Self::Paths { .. } => "paths",
        }
    }

    fn template(&self) -> &'static str {
        match self {
            Self::PullRequest { .. } => PR_TEMPLATE,
            Self::Branch { .. } => BRANCH_TEMPLATE,
            Self::Commits { .. } => COMMITS_TEMPLATE,
            Self::Staged => STAGED_TEMPLATE,
            Self::Worktree => WORKTREE_TEMPLATE,
            Self::Paths { .. } => PATHS_TEMPLATE,
        }
    }

    fn context(&self, agent: AgentKind, render_target: Option<&str>) -> PromptContext {
        let (reference, base, range, paths) = match self {
            Self::PullRequest { reference } => (Some(reference.clone()), None, None, vec![]),
            Self::Branch { base } => (None, Some(base.clone()), None, vec![]),
            Self::Commits { range } => (None, None, Some(range.clone()), vec![]),
            Self::Paths { paths } => (None, None, None, paths.clone()),
            Self::Staged | Self::Worktree => (None, None, None, vec![]),
        };

        PromptContext {
            invocation: agent.invocation(),
            scope_name: self.name(),
            reference,
            base,
            range,
            paths,
            render_target: render_target.map(str::to_owned),
        }
    }
}

#[derive(Serialize)]
struct PromptContext {
    invocation: &'static str,
    scope_name: &'static str,
    reference: Option<String>,
    base: Option<String>,
    range: Option<String>,
    paths: Vec<String>,
    render_target: Option<String>,
}

pub fn render_agent_prompt(
    agent: AgentKind,
    scope: &ReviewScope,
) -> Result<String, RenderAgentPromptError> {
    let render_target = match scope {
        ReviewScope::PullRequest { reference } => Some(reference.as_str()),
        _ => None,
    };
    render_agent_prompt_with_target(agent, scope, render_target)
}

pub fn render_agent_prompt_with_target(
    agent: AgentKind,
    scope: &ReviewScope,
    render_target: Option<&str>,
) -> Result<String, RenderAgentPromptError> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_template_string("agent-prompt", BASE_TEMPLATE)?;
    handlebars.register_partial("scope", scope.template())?;
    Ok(handlebars.render("agent-prompt", &scope.context(agent, render_target))?)
}

#[derive(Debug, Error)]
pub enum RenderAgentPromptError {
    #[error("invalid built-in agent prompt template: {0}")]
    InvalidTemplate(#[from] handlebars::TemplateError),
    #[error("Handlebars could not render the agent prompt: {0}")]
    Render(#[from] handlebars::RenderError),
}

#[cfg(test)]
mod tests {
    use super::{AgentKind, ReviewScope, render_agent_prompt};

    fn scopes() -> Vec<ReviewScope> {
        vec![
            ReviewScope::PullRequest {
                reference: "owner/repo#42".into(),
            },
            ReviewScope::Branch {
                base: "origin/main".into(),
            },
            ReviewScope::Commits {
                range: "main...HEAD".into(),
            },
            ReviewScope::Staged,
            ReviewScope::Worktree,
            ReviewScope::Paths {
                paths: vec!["src/lib.rs".into(), "tests".into()],
            },
        ]
    }

    #[test]
    fn every_scope_has_shared_rules_and_distinct_instructions() {
        let prompts = scopes()
            .iter()
            .map(|scope| render_agent_prompt(AgentKind::Codex, scope).unwrap())
            .collect::<Vec<_>>();

        for prompt in &prompts {
            assert!(prompt.contains("Evidence rules"));
            assert!(prompt.contains("repository-local temporary directory"));
        }
        for (index, prompt) in prompts.iter().enumerate() {
            assert!(!prompts[index + 1..].contains(prompt));
        }
    }

    #[test]
    fn only_pull_requests_request_html_generation() {
        for scope in scopes() {
            let prompt = render_agent_prompt(AgentKind::Codex, &scope).unwrap();
            assert_eq!(prompt.contains("`egr generate"), scope.name() == "pr");
        }
    }

    #[test]
    fn paths_disable_diff_only_review_language() {
        let prompt = render_agent_prompt(
            AgentKind::Codex,
            &ReviewScope::Paths {
                paths: vec!["src/lib.rs".into()],
            },
        )
        .unwrap();

        assert!(prompt.contains("Do not invent a before/after change"));
        assert!(!prompt.contains("deleted code"));
    }

    #[test]
    fn work_in_progress_scopes_are_concise() {
        for scope in [ReviewScope::Staged, ReviewScope::Worktree] {
            let prompt = render_agent_prompt(AgentKind::Codex, &scope).unwrap();
            assert!(prompt.contains("blockers and should-fix findings only"));
            assert!(prompt.contains("Do not produce an HTML artifact"));
        }
    }

    #[test]
    fn agents_differ_only_in_the_invocation() {
        let scope = ReviewScope::Branch {
            base: "origin/main".into(),
        };
        let claude = render_agent_prompt(AgentKind::Claude, &scope)
            .unwrap()
            .replacen("Use the guided-review skill", "<skill>", 1);
        let codex = render_agent_prompt(AgentKind::Codex, &scope)
            .unwrap()
            .replacen("Use $guided-review", "<skill>", 1);

        assert_eq!(claude, codex);
    }

    #[test]
    fn concrete_scope_parameters_are_rendered() {
        for (scope, expected) in [
            (
                ReviewScope::Branch {
                    base: "upstream/trunk".into(),
                },
                "upstream/trunk",
            ),
            (
                ReviewScope::Commits {
                    range: "abc..def".into(),
                },
                "abc..def",
            ),
            (
                ReviewScope::Paths {
                    paths: vec!["src/quoted file.rs".into()],
                },
                "src/quoted file.rs",
            ),
        ] {
            assert!(
                render_agent_prompt(AgentKind::Codex, &scope)
                    .unwrap()
                    .contains(expected)
            );
        }
    }
}
