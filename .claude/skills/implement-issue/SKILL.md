---
name: implement-issue
description: Implement one GitHub issue of rinde/rin-bevy-ui end to end — branch, code, checks, pull request — or, given its pull request, address the review, CI failures and merge conflicts on it. The implementer agents the coordinator dispatches run this; run it by hand as /implement-issue <issue> [pr].
argument-hint: <issue-number> [pr-number]
---

# Implement an issue

The arguments, or the prompt that started you, name the issue `<issue>` and
perhaps its pull request `<pr>`. You own the issue on `rinde/rin-bevy-ui` until its
pull request is ready to merge. You never merge, approve or review your own pull request, and
never push to `main`.

Talk to GitHub through `gh api` and the REST endpoints below. GraphQL is
blocked in Claude Code cloud sessions, so `gh pr create`, `gh issue view` and
friends may fail where `gh api repos/...` works.

## 1. Read and decide whether you can start

- The issue, its labels and comments: `gh api repos/rinde/rin-bevy-ui/issues/<issue>`
  and `.../issues/<issue>/comments`. A bare-title issue's title is the
  specification.
- An existing pull request: if none was given, look for an open one whose head
  branch starts `RINUI-<issue>/` or whose body says `Closes #<issue>`
  (`gh api "repos/rinde/rin-bevy-ui/pulls?state=open&per_page=100"`). If one exists,
  go to **Follow up** below instead.
- **Dependencies.** Every issue in
  `gh api repos/rinde/rin-bevy-ui/issues/<issue>/dependencies/blocked_by`, every open
  sub-issue in `.../issues/<issue>/sub_issues`, and every issue the body names
  as `Depends on #N`, `Blocked by #N` or `After #N` must be closed as completed.
  If one is not, stop and report which.
- **Ambiguity.** If the issue is genuinely ambiguous — two reasonable readings
  that lead to different code — do not guess. Comment on the issue with the
  specific question and the readings you see, remove `ready-for-agent`, add
  `Needs refinement`, and stop.

## 2. Start

1. Add the `In progress` label:
   `gh api repos/rinde/rin-bevy-ui/issues/<issue>/labels -f "labels[]=In progress"`.
2. Branch from a fresh `main`, named `RINUI-<issue>/<descriptive-slug>`:
   `git fetch origin main && git switch -c RINUI-<issue>/<slug> origin/main`.
   You are usually in a worktree of your own under `.claude/worktrees/`;
   another implementer may be working in the main checkout or a sibling
   worktree, so touch nothing outside yours.
3. Read `AGENTS.md` in full, then the module you are changing and a sibling
   widget; `push_button.rs` is the reference widget to copy.

**Build in your worktree's own `target/`, never in a shared one.** Two
worktrees on one `CARGO_TARGET_DIR` mix up same-named artifacts (a stale
`ThemeColors`, and `tests/ron_scenarios` reads `tests/ron` through a path
compiled into its binary, so it can play another worktree's files). Do not
point `CARGO_TARGET_DIR` at the main checkout's `target/`, and never `cp -r` it
(about 19G; copies filled the disk, #165). Instead:

1. Check space: `df -h .` should show more than 30G free; if not, stop and
   report rather than building.
2. Seed `target/` with a copy-on-write clone, which costs no disk, and only if
   the main checkout has a build:
   `MAIN="$(git rev-parse --path-format=absolute --git-common-dir)/.."`, then on
   macOS `cp -c -R "$MAIN/target" ./target`, on Linux
   `cp -R --reflink=always "$MAIN/target" ./target`. Both fail instead of
   copying when the filesystem cannot clone; then remove any partial `./target`
   and build cold (a long Bevy compile) rather than copying for real.
3. After seeding, `touch tests/*.rs src/lib.rs` so cargo rebuilds this crate's
   own targets from your sources, whatever the seed held.

## 3. Implement

Follow the rules in `AGENTS.md` without exception: no application or game
state, one plugin per widget registered in `WidgetsPlugin`, styling from
`Res<Theme>` with no hard-coded colours, `MouseEvent` rather than raw
`Pointer<…>`, no panics, and every public item documented. Name tests as
sentences, and drive widget behaviour through a headless `App` in `tests/` as
`AGENTS.md` describes. Keep the change to what the issue asks; note anything
else worth doing in the pull request rather than doing it.

## 4. Before you call it done

1. `just fmt`
2. `just lint`
3. `just test`
4. New behaviour has a test, where the headless harness allows it
5. `AGENTS.md` updated if a module, example, convention or command changed

All green, then commit with a message that says what the change does, in the
imperative (`Close the combo on an outside click`), and push with
`git push -u origin <branch>`.

## 5. Open the pull request

```sh
gh api repos/rinde/rin-bevy-ui/pulls -f base=main -f head=<branch> \
  -f title="<imperative summary>" -f body="$(cat /tmp/pr-body-<issue>.md)"
```

The body starts with `Closes #<issue>`, then says what changed and why, how it
was tested (the commands you ran and that they passed), and anything a
headless test could not exercise because it needs a window or renderer. The
coordinator has it reviewed once you report it.

## Follow up

Given a pull request, bring it to a mergeable state:

1. Check out its branch in your worktree:
   `git fetch origin <branch> && git switch <branch>` (or
   `git switch -c <branch> origin/<branch>`), and `git pull` to be current.
2. **Merge conflict** (`mergeable` is false): `git merge origin/main`, resolve,
   re-run the checks, push. Never rebase, amend or force-push a pushed branch.
3. **Red CI**: read the failing job's log
   (`gh api repos/rinde/rin-bevy-ui/commits/<sha>/check-runs`, then the job's
   `details_url` or `gh api repos/rinde/rin-bevy-ui/actions/jobs/<id>/logs`),
   reproduce it locally, fix the cause, push. A job that failed for billing
   reasons is not yours to fix. Never skip, disable or loosen a test.
4. **Review**: read every review (`.../pulls/<pr>/reviews`) and inline comment
   (`.../pulls/<pr>/comments`), from agents and humans. Fix every `Blocking`
   point; take the nits that are plainly right. For a point you do not take,
   reply on its thread with why:
   `gh api repos/rinde/rin-bevy-ui/pulls/<pr>/comments/<id>/replies -f body="..."`.
   Push the fixes, then reply on each thread you fixed naming the commit.
5. Re-run **Before you call it done** before every push.

## Report

End with: the issue, the pull request's number and URL, what you did, what the
checks said, and anything left open or that a human needs to decide.
