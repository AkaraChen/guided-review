# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 CLI. `src/main.rs` is the thin process entry point and
`src/lib.rs` coordinates input, rendering, and output. Keep command parsing in
`src/cli.rs`, GitHub PR reference parsing in `src/pull_request.rs`, review JSON
types and validation in `src/review.rs`, and Handlebars setup in
`src/render.rs`. Templates live in `templates/`: page markup in `.hbs` files and
web components in sibling `.js` files. Repeated components (status-badge,
file-link, claim-card, code-card) pair a slim SSR partial with a `.js` custom
element that owns its styles and guards `customElements.define`; single-use
components (theme-switch, page-header) are one `.hbs` file with inline
`<style>`/`<script>`. `examples/review.json` is the fixed development payload.
Integration tests belong in `tests/`. Generated files and build artifacts stay
under `target/`.

## Build, Test, and Development Commands

- `cargo check`: compile quickly without producing a release binary.
- `cargo run -- generate owner/repo#123 --review review.json --output review.html`:
  run the CLI against a review payload.
- `cargo test`: run unit and integration tests.
- `cargo fmt --all --check`: verify Rust formatting.
- `cargo clippy --all-targets --all-features -- -D warnings`: enforce lint-clean
  code.
- `just dev`: render the fixed example, watch `src/`, `templates/`, and the
  example JSON, then serve a live-reloading preview. It requires `cargo-watch`
  and `pnpm`.

## Coding Style & Naming Conventions

Use `rustfmt` defaults and four-space indentation. Name modules, functions, and
fields with `snake_case`; use `PascalCase` for structs and enums. Keep side
effects at the CLI edge, reusable failures typed with `thiserror`, and
application context in `anyhow`. Validate untrusted JSON with `garde` and
`serde(deny_unknown_fields)`. Keep Handlebars changes local and preserve default
HTML escaping.

Template gotchas: Handlebars partial hash parameters shadow same-named context
fields inside nested partials — never name a partial parameter after a data
field (a `text` parameter once blanked every code excerpt). For slotted
light-DOM content, document-level styles beat shadow `::slotted` rules; style
such content from the component's injected document-level block, not from its
shadow CSS.

## Testing Guidelines

Use Rust's built-in test harness. Unit-test parsers and validation invariants;
use `tests/generate.rs` for parser-to-renderer behavior. Each test should protect
one product rule, use an otherwise-valid synthetic payload, and fail for one
reason. Prefer behavior checks such as HTML escaping, required sections, or
contiguous code rows. Do not assert incidental fixture wording, language, CSS
comments, or other implementation tokens.

## Commit & Pull Request Guidelines

Use concise Conventional Commit subjects such as
`fix: prevent blank code rows`. Pull requests should explain the affected CLI,
schema, or template behavior; link relevant issues; list verification commands;
and include before/after screenshots for visual or scrolling changes.

## Security & Generated Output

Treat review JSON and code excerpts as untrusted input. Do not weaken escaping,
line-range validation, or strict field handling. Never commit secrets or files
generated under `target/`.
