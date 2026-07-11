# guided-review

Generate one self-contained Guided Review HTML page for one GitHub pull request.

## Usage

```console
cargo run -- generate github/desktop#144 \
  --review review.json \
  --output github-desktop-144.html
```

`--review -` reads the JSON body from stdin and is the default:

```console
cargo run -- generate github/desktop#144 < review.json
```

When `--output` is omitted, the command writes
`OWNER-REPO-NUMBER-guided-review.html` in the current directory. On success it
prints the output path.

## Live template preview

```console
just dev
```

This uses the fixed `progressus-hk/agent-workspace#56` reference and
[`examples/review.json`](examples/review.json), opens a local preview, and
regenerates plus reloads the page whenever `templates/`, `src/`, or the example
JSON changes. The generated preview lives at
`target/guided-review-preview/index.html`.

## Review JSON

See [`examples/review.json`](examples/review.json) for a complete schema example.
Its claims and excerpts are illustrative and must be replaced with real lines
from the pull request diff.

The top-level body contains:

- `title`: the pull request title.
- `metadata`: changed-file count, additions, deletions, test result, and merge state.
- `thesis`: one code-backed claim describing the core change.
- `reading_order`: one or more paths with a code-backed reason to read each.
- `line_map`: one or more visible, hidden, or cross-cutting review lines.
- `risks`: blocker, should-fix, or follow-up concerns; this may be empty.
- `verification`: one or more verified, partial, or unproven checks.
- `questions`: decision-relevant author questions; this may be empty.
- `recommendation`: approve, request changes, or comment only, with blockers and
  non-blocking follow-ups kept separate.

Every claim has this shape:

```json
{
  "text": "What the reviewer observed or concluded.",
  "basis": "observed",
  "evidence": [
    {
      "name": "Boundary validation",
      "path": "src/main.rs",
      "start_line": 12,
      "end_line": 14,
      "language": "rust",
      "code": "three\nreal\nlines"
    }
  ]
}
```

The code excerpt must contain exactly the number of lines declared by its
inclusive `start_line` and `end_line` range. Unknown JSON fields, blank values,
unsupported claims, and contradictory approve recommendations with blockers are
rejected before an HTML file is written.

Accepted enum values are:

- `basis`: `observed`, `synthesis`
- `line`: `visible`, `hidden`, `data`, `state`, `permission`, `error`, `test`,
  `complexity`, `degradation`
- `level`: `blocker`, `should_fix`, `follow_up`
- `status`: `verified`, `partial`, `unproven`
- `decision`: `approve`, `request_changes`, `comment_only`
