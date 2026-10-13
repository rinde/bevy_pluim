---
name: review-pr
description: Review a pull request on rinde/rin-bevy-ui against its issue and the repository's rules and make sure CI passed, running it locally where GitHub did not, then approve it or request changes with detailed inline comments. The reviewer agents the coordinator dispatches run this; run it by hand as /review-pr <number> [sonnet|opus].
argument-hint: <pr-number> [sonnet|opus]
allowed-tools: Read, Grep, Glob, Write, Bash(gh api:*), Bash(git fetch:*), Bash(git diff:*), Bash(git log:*), Bash(git show:*), Bash(git switch:*), Bash(git status:*), Bash(git rev-parse:*), Bash(just ci:*)
---

# Review a pull request

The arguments, or the prompt that started you, name the pull request `<pr>`
and the reviewer variant, `sonnet` or `opus` (`sonnet` if none is named),
which goes into the review's marker below. Review pull request `<pr>` on
`rinde/rin-bevy-ui`, make sure CI passed on it, and post **one** GitHub review that
either approves it or requests changes, with an inline comment on every point
you raise. You only review and run checks. You never push, merge or edit the
branch.

Talk to GitHub through `gh api` and the REST endpoints below. GraphQL is
blocked in Claude Code cloud sessions, so `gh pr view`, `gh issue view` and
friends may fail where `gh api repos/...` works.

## 1. Gather

- The pull request: `gh api repos/rinde/rin-bevy-ui/pulls/<pr>`. Note `head.sha` (the
  commit you review), `base.ref`, `draft`, `body` and `user.login`.
- The diff: `gh api repos/rinde/rin-bevy-ui/pulls/<pr> -H "Accept: application/vnd.github.diff"`,
  and the file list with `gh api --paginate repos/rinde/rin-bevy-ui/pulls/<pr>/files`.
- The full files, not just the hunks, from a checkout of the head. As a
  reviewer agent you are in a worktree of your own under `.claude/worktrees/`:
  fetch the head and check it out there,
  `git fetch origin pull/<pr>/head:refs/remotes/origin/pr/<pr>` then
  `git switch --detach origin/pr/<pr>`, and confirm with `git status` that the
  worktree is clean and `HEAD` is `head.sha`. Never switch the branch of the
  main checkout or of another agent's worktree.
- The issue it closes: the `Closes #N` / `Fixes #N` in the body, else the
  `RINUI-<N>/` prefix of the branch. Read it with
  `gh api repos/rinde/rin-bevy-ui/issues/<N>`, its comments with
  `.../issues/<N>/comments`, and note its labels. With no issue, review the
  pull request on its own description.
- Earlier reviews and threads: `gh api repos/rinde/rin-bevy-ui/pulls/<pr>/reviews` and
  `gh api --paginate repos/rinde/rin-bevy-ui/pulls/<pr>/comments`. On a re-review,
  check whether each earlier point was dealt with and do not raise again what
  was fixed or answered convincingly.
- CI on the head: `gh api repos/rinde/rin-bevy-ui/commits/<head.sha>/check-runs`.
  See the next step for what to make of it.
- The rules: `AGENTS.md`, all of it, and a sibling of every widget the diff
  touches, so you can tell whether the change matches what is already there.

## 2. Make sure CI passed

An approval means CI passed on exactly `head.sha`, on GitHub or on your
machine. GitHub's CI is the jobs **Format, clippy and docs**, **Tests** and
**Unused dependencies** in the check runs of `head.sha`; together they are
`just ci`.

- **All three succeeded** → CI passed on GitHub: `ci=github` in the marker.
- **One failed** with a step's output → a blocking finding: name the job and
  quote the failing lines from its log
  (`gh api repos/rinde/rin-bevy-ui/actions/jobs/<id>/logs`), on the line of the
  change that causes it where you can tell.
- **One did not run** — missing, cancelled, skipped, or failed for billing
  reasons without a step's output (see **Agents** in `AGENTS.md`) — or is still
  queued or running → run CI yourself, in the checkout of the head:
  `just ci`. It builds Bevy, so give the command a long timeout (up to
  45 minutes). In a worktree, build in the worktree's own `target/`, never a shared
  `CARGO_TARGET_DIR` (it gives stale test binaries) and never a full copy of
  the main checkout's 19G `target/` (it fills the disk). Check `df -h .` shows
  more than 30G free, seed with a copy-on-write clone (macOS
  `cp -c -R "$(git rev-parse --path-format=absolute --git-common-dir)/../target" ./target`,
  Linux `cp -R --reflink=always ...`; if it fails, remove the partial copy and
  build cold), `touch tests/*.rs src/lib.rs`, then `just ci`.
  - It passes → CI passed locally: `ci=local` in the marker, and say in the
    body that GitHub's CI did not run and `just ci` passed on `head.sha`.
  - A format, clippy, doc, test or unused-dependency check fails → a
    blocking finding, as above, with the failing output.
  - It cannot run at all — a missing toolchain, tool or system library,
    nothing to do with the change →
    you cannot approve. Post no review; report what stopped it, so a human
    can fix the machine. You may still post a `request_changes` review if the
    code has blocking findings of its own.

The macOS and Windows compile checks have no local equivalent; their failure is blocking like any other, and their not running is
not.

## 3. Judge

Work through all of it, and say where in the code each finding lives:

1. **Does it do what the issue asks**, all of it and nothing unrelated? A
   bare-title issue's title is the specification.
2. **Correctness.** Trace the changed behaviour from a real caller. Edge cases,
   ordering, off-by-one, state left behind; a widget over a shared
   `Selection::Ref` that misses a change on the referenced entity; children
   left behind or duplicated when a selection or its items change.
3. **The rules that always apply**: no application or game state in the crate,
   which takes Bevy types only; one `Plugin` per widget, registered in
   `WidgetsPlugin`; styling from `Res<Theme>` through a `theme_*` system in the
   `AGENTS.md` pattern, with no hard-coded colours outside `theme.rs` and
   `tailwind.rs`; widgets observe `MouseEvent`, not raw `Pointer<…>`; state is
   an immutable component, setup lives in hooks and `#[require(…)]`; no
   `unwrap`, `expect`, `panic!`, indexing or `allow` in library code, and none
   of the Bevy accessors `clippy.toml` bans; every public item has a doc
   comment and a `Debug` derive; `publish = false` stays and no open-source
   framing.
4. **Tests.** New behaviour is tested where the headless harness allows it,
   and the pull request says why where it does not; tests are named as
   sentences; integration tests drive a headless `App` in `tests/`; tests stay
   small and fast.
5. **Documentation.** `AGENTS.md` updated for a new or changed module,
   example, convention or command, including its tables; a new widget has an
   example.
6. **Readability.** Code reads like the code around it; comments say why, not
   what.

Sort every finding into one of two kinds:

- **Blocking**: wrong behaviour, a broken rule above, a missing test for new
  behaviour, missing docs, red CI caused by
  the change or failing locally.
- **Nit**: style, naming, a clearer way to write it. Optional by definition.

## 4. Decide

- Any blocking finding → the verdict is `request_changes`.
- None, and CI passed on `head.sha` on GitHub or locally → the verdict is
  `approve`, nits and all.

If the issue carries the **`require-human-review`** label, a human must approve
this work. Still review it fully and still record your verdict, but never
submit an `APPROVE` review: submit `COMMENT` for an approve verdict, and say at
the top of the body that a human must approve and merge it.

## 5. Post one review

Write the review as JSON to a file outside the repository, such as
`/tmp/review-<pr>.json`:

```json
{
  "commit_id": "<head.sha>",
  "event": "APPROVE",
  "body": "<!-- agent-review verdict=approve reviewer=sonnet sha=<head.sha> ci=github -->\n## Approved\n...",
  "comments": [
    { "path": "src/list.rs", "line": 42, "side": "RIGHT",
      "body": "**Blocking:** ..." }
  ]
}
```

then post it with
`gh api repos/rinde/rin-bevy-ui/pulls/<pr>/reviews --input /tmp/review-<pr>.json`.

- `event` is `APPROVE` or `REQUEST_CHANGES` to match the verdict, or `COMMENT`
  as above.
- **The body's first line is the marker** the coordinator reads, exactly:
  `<!-- agent-review verdict=<approve|request_changes> reviewer=<sonnet|opus> sha=<head.sha> ci=<github|local|none> -->`,
  where `ci` says where CI passed on `head.sha`: `github`, `local` when you
  ran `just ci` yourself, or `none` when it did not pass anywhere (only with
  `request_changes`).
- Then a heading with the verdict, a short summary of what the change does and
  how well, the blocking findings as a numbered list, the nits, and a line on
  what you checked: CI on `head.sha`, on GitHub or by `just ci` locally, and
  its result; tests; docs.
- One inline comment per finding, on the line it concerns, starting
  `**Blocking:**` or `**Nit:**`. Explain the problem, why it matters here, and
  what to do instead, with a suggested fix where it is short (a ```suggestion
  block replaces the commented line). A comment's `line` must be a line of the
  diff on the `RIGHT` side; a finding that has no such line goes in the body.
- If GitHub refuses the review because a comment's line is not in the diff
  (HTTP 422), move that comment into the body and post again. If it refuses
  `APPROVE` or `REQUEST_CHANGES` because you are the pull request's author,
  post the same review as `COMMENT`; the marker still carries the verdict.

Finish by saying, in a line or two, the verdict, the number of blocking
findings and nits, and the review's URL.
