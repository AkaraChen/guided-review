# Industry Practices

Use these as inputs, not cargo-cult rules; public guidance differs by company
and tool, but the overlap is strong.

- Google frames review around long-term code health: favor approval once a
  change definitely improves the system, even if imperfect. Navigate by first
  deciding whether the change makes sense, then inspecting the main part, then
  reading the rest in a logical sequence — if the main design is wrong, send
  that feedback early.
- Microsoft separates automated checks from human judgment: linters handle
  low-value checks; humans focus on business-logic correctness, changed tests,
  design readability, and maintainability, reading every changed line in a
  logical order with full-file context when needed.
- GitHub emphasizes small focused PRs, clear descriptions, reviewer guidance
  about file order, and author self-review. Copilot code review always leaves
  a comment review, not an approve or request-changes review, so it never
  satisfies required human approvals by itself.
- GitLab splits responsibility by role: reviewers evaluate the chosen
  solution's specifics; maintainers own overall codebase health, with domain
  experts pulled in where specialized knowledge matters.
- Meta's stacked-changes tooling (Sapling / ReviewStack) shows smaller commits
  are easier to reason about, and stack-aware review preserves discussion per
  commit instead of one large blob.
- AI-review practice points the same way: AI handles routine comments,
  summaries, and Q&A at scale while authors stay in control, and AI coding
  shifts the bottleneck from writing code to reviewing and integrating it.
