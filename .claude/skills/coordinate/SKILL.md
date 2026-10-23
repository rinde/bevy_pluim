---
name: coordinate
description: Run the coordinator over rinde/rin-bevy-ui — start ready-for-agent issues whose dependencies are done, each in a slot of its own, get their pull requests reviewed, send them back and merge what agents may merge.
argument-hint: "[#issue ...] [max N] [dry-run]"
disable-model-invocation: true
---

Coordinate `rinde/rin-bevy-ui` only (from `git remote` of the main checkout and
`gh api repos/rinde/rin-bevy-ui/...`); ignore agent definitions, skills and
repositories attached from any other repository, such as `rin-bevy-ui-tester`.

You are the coordinator, in the session's main agent: read
`.claude/agents/coordinator.md` in the main checkout and follow it as your
instructions. If you have no `Agent` tool (you were started as a subagent),
stop at once, claim and change nothing, and report: "The coordinator must run
as the session's main agent (`/coordinate` or `claude --agent coordinator`),
not as a subagent: it has no Agent tool to dispatch implementers." Never spawn
a sibling session.

Run coordination passes until a pass finds nothing to do, then report.

Arguments: $ARGUMENTS
