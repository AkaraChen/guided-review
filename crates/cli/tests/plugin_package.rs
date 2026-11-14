//! Packaging checks for the Agent Plugins 1.0 layout at the repository root.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const SCHEMA_ID: &str = "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json";
const MANIFEST_FIELDS: &[&str] = &[
    "$schema",
    "name",
    "version",
    "description",
    "author",
    "homepage",
    "repository",
    "license",
    "keywords",
    "extensions",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("readable manifest");
    serde_json::from_str(&text).expect("valid JSON manifest")
}

#[test]
fn root_manifest_uses_closed_agent_plugins_schema() {
    let manifest = read_json(&repo_root().join("plugin.json"));
    let object = manifest.as_object().expect("manifest is an object");

    assert_eq!(
        object.get("$schema").and_then(Value::as_str),
        Some(SCHEMA_ID)
    );
    assert!(object.get("name").and_then(Value::as_str).is_some());
    for key in object.keys() {
        assert!(
            MANIFEST_FIELDS.contains(&key.as_str()),
            "unexpected field {key}"
        );
    }
}

fn frontmatter_name(skill_md: &str) -> Option<&str> {
    let body = skill_md.strip_prefix("---\n")?;
    let (frontmatter, _) = body.split_once("\n---")?;
    frontmatter
        .lines()
        .find_map(|line| line.strip_prefix("name:"))
        .map(str::trim)
}

fn is_valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[test]
fn every_skill_name_matches_its_directory() {
    let entries = fs::read_dir(repo_root().join("skills")).expect("skills directory");
    let mut found = 0;
    for entry in entries {
        let dir = entry.expect("directory entry").path();
        let skill_md = dir.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let text = fs::read_to_string(&skill_md).expect("readable SKILL.md");
        let name = frontmatter_name(&text).expect("SKILL.md frontmatter name");
        let dir_name = dir.file_name().and_then(|n| n.to_str()).expect("utf-8 dir");
        assert_eq!(name, dir_name);
        assert!(is_valid_skill_name(name), "invalid skill name {name}");
        found += 1;
    }
    assert!(found > 0, "no skills discovered under skills/");
}

#[test]
fn vendor_manifests_point_at_existing_skill_paths() {
    for manifest in [".claude-plugin/plugin.json", ".codex-plugin/plugin.json"] {
        let value = read_json(&repo_root().join(manifest));
        let skills = value["skills"].as_str().expect("skills path");
        assert!(skills.starts_with("./"), "{manifest}: {skills}");
        assert!(repo_root().join(skills).is_dir(), "{manifest}: {skills}");
    }
}
