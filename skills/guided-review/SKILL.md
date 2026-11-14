---
name: guided-review
description: Review code and produce Guided Review artifacts. Use when the user asks for review, code review, PR review, architecture review, a guided review, PR walkthrough, line map, suggested reading order, or a structured PR review summary with risk and verification focus; prioritize concrete bugs, risks, overengineering, style drift, and missing focused tests.
---

# Guided Review

A Guided Review turns a pull request diff into a readable map backed by code
evidence. It is not a file-by-file summary and not a replacement for review
judgment. Its job is to reduce comprehension cost so a human reviewer can make
a better approve, request-changes, or comment-only decision.

Start from concrete findings, not a broad summary.

## Review Workflow

1. Inspect the actual diff or files under review before judging.
2. For GitHub PRs, after inspecting enough test and generated evidence, mark
   safe test-only and generated-only files as viewed so the remaining
   changed-files view stays focused.
3. Check for correctness bugs, behavioral regressions, implementation
   degradation, newly introduced concepts, overengineering, style drift,
   missing focused tests, and boundary violations.
4. Lead with findings ordered by severity. Include file and line references.
5. If there are no issues, say so and mention remaining test gaps or residual
   risk.

## Review Standards

- First check formatting, indentation, naming, and local style consistency.
- Then check over-defensive code, unnecessary complexity, speculative
  abstraction, implementation degradation, poor modularity, and missing
  focused tests.
- Treat implementation degradation as a real finding: flag changes that keep
  behavior working while worsening ownership, data flow, boundaries,
  complexity, performance, or testability.
- If a PR introduces a new concept — a domain term, lifecycle state,
  permission boundary, storage model, async contract, public API shape, or
  review category future contributors must understand — require a clear name,
  one entry point, the invariant it owns, and tests or docs that make the
  concept teachable instead of leaving it implicit.

```text
New concept: workspace invitation lifecycle
entry point -> state owner -> permission rule -> persistence -> docs/tests
```

- A PR degrades the implementation when it spreads one responsibility across
  more places, duplicates an existing helper, weakens an invariant, moves
  business logic into the wrong layer, broadens an API for one caller, or
  makes the same behavior harder to test or change.

```text
Finding: implementation degradation
before: request cache owns invalidation
after: components duplicate query keys and refetch timing
impact: behavior works today, but correctness moved out of the cache layer
```

- Do not ask for runtime validation that duplicates static types unless data
  crosses an untrusted boundary.

## Artifact Output

A useful Guided Review has:

1. One-sentence thesis: the core behavior or system change.
2. Suggested reading order: where to start, what to read next, and why.
3. Line map: visible line, hidden line, and cross-cutting lines.
4. Risk focus: paths, boundaries, migrations, rollback points, implementation
   degradation, or assumptions that deserve attention.
5. Concept focus: any new domain term, lifecycle state, permission boundary,
   storage model, async contract, public API shape, or review category the PR
   expects future contributors to understand.
6. Verification focus: tests, screenshots, logs, manual checks, CI signals,
   and unproven claims.
7. Questions for the author: only questions affecting understanding,
   correctness, risk, or review decision.
8. Review recommendation: approve, request changes, or comment only, with
   blockers separated from non-blocking follow-up.

## Line Model

- Visible line: the PR's declared story: user-facing behavior, requirement
  mapping, happy path, removed behavior.
- Hidden line: assumptions the diff does not explain directly: invariants,
  coupling, history, migration risk, operational effect, and review shape.
- Concept line: any new idea the PR adds to the codebase: the name future
  contributors will search for, the entry point where it first appears, the
  module or type that owns its invariant, and the tests or docs that teach it.
- Data line: input, validation, transformation, persistence, output.
- State line: loading, cache, refresh, invalidation, concurrency, retry.
- Permission line: identity, authorization, tenant isolation, auditability.
- Error line: failure classification, propagation, user messaging, retry,
  fallback.
- Test line: what tests prove, what they do not prove, and whether they fail
  for the right reason.
- Complexity line: whether abstraction serves a current need or a speculative
  future need.
- Degradation line: whether the code now has worse ownership, data flow,
  boundaries, performance, or testability even if behavior works.

## Process

1. Establish scope: read the PR title, description, linked issue, design
   notes, incident context, CI status, and review requests. Write down what
   the PR claims to solve before judging the diff.
2. Inventory the diff: list added, modified, deleted, renamed, generated,
   config, test, and documentation files. Group by responsibility. Identify
   entry points, core logic, schemas, migrations, state management, public
   APIs, or UI surfaces.
3. Reconstruct the change graph:

```text
entry point -> validation -> core decision -> state/storage -> side effect -> output -> proof
```

   For each flow, answer: where does the change enter the system, which fact
   or contract does it change, who depends on that new fact later, and does it
   introduce a new concept that should be named, owned, documented, or tested.
4. Extract lines: visible first (product language: who can now do what),
   hidden second (engineering language: what must be true for this to be
   safe), cross-cutting third (reviewer language: security, concurrency,
   performance, compatibility, observability, tests, rollback). Write the
   concept line when the PR adds a new term, state, boundary, storage model,
   async contract, public API shape, or review category. Do not invent lines
   to make the artifact look complete; if evidence is missing, mark the line
   as unproven or ask the author.
5. Re-read by line instead of by file: follow the visible line through entry
   point, core logic, output, and tests; the hidden line through edge cases,
   old data, failure paths, permissions, and concurrency; the concept line
   through naming, ownership, invariants, docs, and tests; the degradation
   line through old vs new ownership, duplicated helpers, broadened APIs, and
   weakened invariants; the test line through names, fixtures, assertions,
   and failure mode; and the deletion line through callers, configuration,
   docs, migrations, and user habits.
6. Classify risk:
   - Blocker: correctness, security, data integrity, compatibility,
     deployability, rollback, or the main requirement is broken.
   - Should fix: likely maintenance cost, test gap, or misuse risk.
   - Follow-up: optional polish or separate work.

## Rendering And Serving With egr

This repository's CLI renders the artifact as a self-contained HTML page and
serves it. Do not hand-write the HTML.

1. Use `egr` if installed; otherwise install it once with
   `cargo install --git https://github.com/AkaraChen/guided-review guided-review`.
2. Create all review-related files — review JSON, generated HTML, and the
   served `out/` directory — under a fresh temp directory, never inside the
   repository under review. Use `mktemp -d` on macOS/Linux; on Windows use a
   new subdirectory of `$env:TEMP` (PowerShell), for example
   `New-Item -ItemType Directory "$env:TEMP\guided-review-$PID"`.
3. Read the payload contract from `egr generate -h`: it prints the JSON
   Schema, generated from the Rust types. Every claim needs `evidence` code
   excerpts whose `code` contains exactly `end_line - start_line + 1` real
   lines from the diff. `examples/review.json` is a complete sample payload.
4. Write the review as JSON, then render one page per PR. Name the generated
   HTML `index.html` by default so the served directory root opens the review
   directly:

```sh
egr generate OWNER/REPO#NUMBER --review review.json --output out/index.html
```

5. Publish the result. In a Multica workspace, just post the HTML file and do
   not serve an HTTP server. Otherwise, prefer an artifact or page-publishing
   tool when the environment has one; failing that, run `egr serve out` in
   the background — it binds 127.0.0.1 on a free port, prints the URL, and
   serves the directory — and share that URL. When neither is possible, share
   the file paths.

## Accuracy Rules

- Separate observed facts from synthesis; the payload's `basis` field records
  this per claim.
- Every claim must cite the code it describes. A statement with no code
  reference is not verifiable — either add the excerpt or drop the statement.
- Prefer precise wording over broad wording: say "GitHub Copilot code review
  does not count toward required approvals" instead of "AI cannot approve
  PRs", because other tools may differ. Do not claim a company-wide practice
  from a single blog post unless the post itself says it is company-wide.
- Do not treat AI output as authoritative. Use AI to draft line maps,
  summarize routine context, and propose questions; keep humans responsible
  for semantic correctness, risk judgment, and merge decisions.
- If the PR is small, keep the Guided Review small. A one-screen review is
  better than a ceremonial template.

## Anti-Patterns

- Summarizing each file without reconstructing system behavior.
- Commenting on naming or formatting before understanding the main design.
- Spending human review time on checks automation should handle.
- Forcing unrelated changes into one fake thesis.
- Reviewing added code while ignoring deleted code, configuration, tests, and
  docs.
- Producing a walkthrough with no judgment.
- Producing judgment with no reading path, so the next reviewer still starts
  from zero.
- Turning a review into a rewrite unless the user asks for fixes.
- Spending findings on harmless preferences when there are real bugs.

For the industry practices behind this process, read
[`references/industry-practices.md`](references/industry-practices.md).
