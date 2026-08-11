# Agent prompt templates

`base.md.hbs` contains the shared evidence, scratch-directory, and delivery
rules. Each scope partial supplies only its acquisition, claim-source, and
review-emphasis instructions. Keep Claude/Codex differences in the invocation
value rendered by `agent_prompt.rs`; do not fork complete templates by agent.
