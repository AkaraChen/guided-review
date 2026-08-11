#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct Sandbox(PathBuf);

impl Sandbox {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("egr-agent-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stub(sandbox: &Sandbox) -> PathBuf {
    let path = sandbox.path("agent-stub");
    fs::write(
        &path,
        "#!/bin/sh\nprintf '%s\\0' \"$@\" > \"$EGR_STUB_ARGS\"\nexit \"${EGR_STUB_EXIT:-0}\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn run(args: &[&str], envs: &[(&str, &Path)]) -> Output {
    run_in(args, envs, None)
}

fn run_in(args: &[&str], envs: &[(&str, &Path)], cwd: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_egr"));
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    for (name, value) in envs {
        command.env(name, value);
    }
    command.output().unwrap()
}

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn initialized_repository(sandbox: &Sandbox) -> PathBuf {
    let repository = sandbox.path("repo");
    fs::create_dir(&repository).unwrap();
    git(&repository, &["init", "-q", "-b", "main"]);
    git(&repository, &["config", "user.name", "EGR Test"]);
    git(
        &repository,
        &["config", "user.email", "egr@example.invalid"],
    );
    fs::write(repository.join("tracked.txt"), "base\n").unwrap();
    git(&repository, &["add", "tracked.txt"]);
    git(&repository, &["commit", "-q", "-m", "base"]);
    git(
        &repository,
        &["update-ref", "refs/remotes/origin/main", "HEAD"],
    );
    git(
        &repository,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    repository
}

fn recorded_args(path: &Path) -> Vec<String> {
    fs::read(path)
        .unwrap()
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| String::from_utf8(value.to_vec()).unwrap())
        .collect()
}

#[test]
fn print_emits_only_the_prompt_and_does_not_spawn() {
    let sandbox = Sandbox::new();
    let marker = sandbox.path("should-not-exist");
    let missing = sandbox.path("missing-agent");
    let output = run(
        &["codex", "--commits", "abc..def", "--print"],
        &[("EGR_CODEX_BIN", &missing), ("EGR_STUB_ARGS", &marker)],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("abc..def"));
    assert!(output.stderr.is_empty());
    assert!(!marker.exists());
}

#[test]
fn dry_run_keeps_a_multiline_prompt_in_one_argument() {
    let sandbox = Sandbox::new();
    let marker = sandbox.path("should-not-exist");
    let binary = stub(&sandbox);
    let output = run(
        &[
            "codex",
            "--paths",
            "src/quoted file.rs",
            "--dry-run",
            "--",
            "--model",
            "o3",
        ],
        &[("EGR_CODEX_BIN", &binary), ("EGR_STUB_ARGS", &marker)],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("src/quoted file.rs"));
    assert!(stdout.contains("\\n"));
    assert!(stdout.contains("\"--model\" \"o3\""));
    assert!(!marker.exists());
}

#[test]
fn headless_forms_and_passthrough_reach_the_stub_verbatim() {
    for (agent, variable, prefix) in [
        ("claude", "EGR_CLAUDE_BIN", "-p"),
        ("codex", "EGR_CODEX_BIN", "exec"),
    ] {
        let sandbox = Sandbox::new();
        let args_file = sandbox.path("args");
        let binary = stub(&sandbox);
        let output = run(
            &[agent, "--staged", "--headless", "--", "--model", "reviewer"],
            &[(variable, &binary), ("EGR_STUB_ARGS", &args_file)],
        );

        assert!(output.status.success());
        let args = recorded_args(&args_file);
        assert_eq!(args[0], prefix);
        assert!(args[1].contains("Review `git diff --cached`"));
        assert_eq!(&args[2..], ["--model", "reviewer"]);
    }
}

#[test]
fn interactive_form_uses_the_prompt_as_the_first_position_argument() {
    let sandbox = Sandbox::new();
    let args_file = sandbox.path("args");
    let binary = stub(&sandbox);
    let output = run(
        &["codex", "--worktree"],
        &[("EGR_CODEX_BIN", &binary), ("EGR_STUB_ARGS", &args_file)],
    );

    assert!(output.status.success());
    let args = recorded_args(&args_file);
    assert_eq!(args.len(), 1);
    assert!(args[0].contains("Review `git diff`"));
}

#[test]
fn agent_exit_code_is_propagated() {
    let sandbox = Sandbox::new();
    let args_file = sandbox.path("args");
    let exit_code = sandbox.path("3");
    let binary = stub(&sandbox);
    let mut command = Command::new(env!("CARGO_BIN_EXE_egr"));
    let output = command
        .args(["claude", "--staged"])
        .env("EGR_CLAUDE_BIN", &binary)
        .env("EGR_STUB_ARGS", &args_file)
        .env("EGR_STUB_EXIT", exit_code.file_name().unwrap())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn missing_binary_error_includes_path_and_install_hint() {
    let sandbox = Sandbox::new();
    let missing = sandbox.path("not-installed");
    let output = run(&["claude", "--staged"], &[("EGR_CLAUDE_BIN", &missing)]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(missing.to_string_lossy().as_ref()));
    assert!(stderr.contains("install Claude Code"));
}

#[test]
fn default_scope_prefers_an_ahead_branch() {
    let sandbox = Sandbox::new();
    let repository = initialized_repository(&sandbox);
    git(&repository, &["switch", "-q", "-c", "feature"]);
    fs::write(repository.join("tracked.txt"), "feature\n").unwrap();
    git(&repository, &["commit", "-q", "-am", "feature"]);

    let output = run_in(&["codex", "--print"], &[], Some(&repository));

    assert!(output.status.success());
    let prompt = String::from_utf8(output.stdout).unwrap();
    assert!(prompt.contains("current branch against base `origin/main`"));
}

#[test]
fn default_scope_uses_worktree_when_the_branch_is_not_ahead() {
    let sandbox = Sandbox::new();
    let repository = initialized_repository(&sandbox);
    fs::write(repository.join("tracked.txt"), "changed\n").unwrap();

    let output = run_in(&["claude", "--print"], &[], Some(&repository));

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("Review `git diff`")
    );
}

#[test]
fn default_scope_errors_for_clean_or_non_git_directories() {
    let sandbox = Sandbox::new();
    let repository = initialized_repository(&sandbox);

    for cwd in [&repository, &sandbox.0] {
        let output = run_in(&["codex", "--print"], &[], Some(cwd));
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("explicitly choose --pr, --branch"));
    }
}

#[test]
fn branch_without_a_value_requires_origin_head() {
    let sandbox = Sandbox::new();
    git(&sandbox.0, &["init", "-q", "-b", "main"]);

    let output = run_in(&["codex", "--branch", "--print"], &[], Some(&sandbox.0));

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("origin/HEAD is not configured"));
    assert!(stderr.contains("--branch <BASE>"));
}
