# Working on rin-bevy-ui

Guidance for coding agents. This file explains the codebase to whoever is
changing it. If it and the code disagree, the code wins and this file gets
fixed.

## What this is

A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons,
lists, combos, scroll areas, tabs, breadcrumbs, colour palettes and a
numeric stepper, all themed from one `Theme` resource. It is used by nopjes,
but it is a library in its own right.

**No application or game state.** The crate takes Bevy types only, so its
widgets can be exercised without an application behind them. Anything that
knows what a picked item *means* belongs in the application that uses the
widget, not here. This is the same rule rails applies to `rail_ui`.

This is private software: `publish = false` stays, and there is no licence
file or open-source framing.

## Core beliefs

1. **The repo is the source of truth.** If it is not in the code or this
   file, do not assume it. Read before you act.
2. **Read before write.** Read the module you are changing and a sibling
   widget before adding anything; match what is already there.
3. **Small, verifiable changes.** One PR does one thing.
4. **Docs stay in sync.** When behaviour or a convention changes, update this
   file in the same PR. A stale doc is worse than none.
5. **No gold-plating.** Implement what is asked; no abstractions, config or
   features "just in case".
6. **Explicit over implicit.** Name things clearly and let the type system
   carry the meaning.

## Tickets live on GitHub

Work is tracked as issues on `github.com/rinde/rin-bevy-ui`. Bodies may be
short; treat the title as the specification, and if it is genuinely
ambiguous, ask rather than guess.

**Always work via PRs**, even for small changes. Before stopping, make sure
the PR exists and check whether it needs a rebase onto `main`.

Whenever asked to work on a ticket, always make sure that the ticket has the 
label "In progress" assigned to it, if not, assign it before starting any work.

Issues labelled `ready-for-agent` can be run by agents end to end with
`/coordinate`; see [Agents](#agents).

## Agents

A **coordinator** picks up issues that are ready, an **implementer** turns
each into a pull request, a **reviewer** approves it or requests changes, and
the coordinator sends it back or merges it. They live in `.claude/agents/` and
`.claude/skills/` and follow the same rules as anyone else: this file. Nothing
runs on GitHub's side; the coordinator starts every agent, in its own session.

| Agent | Model | Does |
| --- | --- | --- |
| `coordinator` | Sonnet | Decides what is ready, dispatches implementers and reviewers, checks on GitHub that each did its job, merges. Writes no code. |
| `implementer-sonnet`, `implementer-opus` | Sonnet, Opus | Implements one issue in a worktree under `.claude/worktrees/` on a branch `issue-<number>/-<slug>`, opens a pull request that `Closes #N`, and follows it up on review, red CI or a merge conflict. Runs the `implement-issue` skill. |
| `reviewer-sonnet`, `reviewer-opus` | Sonnet, Opus | Reviews one pull request in a worktree of its own, makes sure CI passed on its head, and posts one review that approves or requests changes, with inline comments. Runs the `review-pr` skill. |

The variants differ only in their model; the instructions are the skills, which
also run by hand: `/implement-issue 24`, `/review-pr 25 opus`.

| Label | Meaning |
| --- | --- |
| `ready-for-agent` | The coordinator may start the issue. Agents leave every other issue alone. |
| `opus` | The Opus implementer, and an Opus approval. Without it, Sonnet. |
| `require-human-review` | A human approves and merges; no agent approves or merges it. |
| `In progress` | Someone is working on it; set on dispatch. |
| `Needs refinement` | Not ready. An implementer that finds the issue ambiguous asks on it, sets this and removes `ready-for-agent`. |

An issue starts only when everything it depends on is **closed as completed**:
the issues blocking it in its *Relationships*, its open sub-issues, and any
issue its body names as `Depends on #N`, `Blocked by #N` or `After #N`. A
dependency closed as not planned waits for a human.

**The review verdict.** A reviewer approves only when CI passed on the pull
request's head: GitHub's **Format, clippy and docs**, **Tests** and **Unused
dependencies** jobs, which together are `just ci`. Where one did not run —
missing, cancelled, queued, or failed for billing reasons because the
account's Actions minutes ran out, which shows as a billing annotation rather
than a step's output — the reviewer runs `just ci` on the head itself. Every
agent review starts with a marker the coordinator reads:

```
<!-- agent-review verdict=<approve|request_changes> reviewer=<sonnet|opus> sha=<commit> ci=<github|local|none> -->
```

A verdict counts only for the commit it names, so a push makes it stale. An
approved pull request with CI passed and no conflict is squash-merged unless
the issue has `require-human-review`. After three rounds of changes requested,
the coordinator stops, comments and takes `ready-for-agent` off the issue.

| How | What |
| --- | --- |
| `/coordinate` | Passes until nothing is left to do, then a report. |
| `/coordinate #24 #25` | Only those issues and their pull requests. |
| `/coordinate max 5` | Five implementers at once instead of three. |
| `/coordinate dry-run` | Report what it would do and change nothing. |
| `/loop 30m /coordinate` | Keep coordinating every half hour. |
| `claude --agent coordinator` | The coordinator as the session's main agent. |

Agents talk to GitHub through `gh api` and its REST endpoints, because cloud
sessions block the GraphQL API that `gh issue` and `gh pr` use. An unattended
run needs its tools allowed, or it stops at the first permission prompt. Each
worktree starts with an empty `target/`; the skills point cargo at the main
checkout's build so Bevy is not compiled again.

## Where to find what

| Module | Holds |
| --- | --- |
| `lib.rs` | `WidgetsPlugin` (add this one plugin to an app), `UiRoot`, and `MouseEvent` with the observers that produce it |
| `theme.rs` | `Theme` resource, `ThemeColors`, `DEFAULT_LIGHT_THEME_COLORS` / `DEFAULT_DARK_THEME_COLORS`, `light_dark_toggle`, the shadow constants; also keeps `ClearColor` in step with the theme |
| `tailwind.rs` | `COLORS`, all 242 Tailwind colours as `TailwindColor { class, variant, color }`, for colour pickers; the theme itself uses `bevy::color::palettes::tailwind` directly |
| `select.rs` | the selection model: `Selection` (`Value` or `Ref`), `SelectionValue` (`One` / `ZeroOrOne` / `OneOrMore` / `ZeroOrMore`), the `Selection::builder()`, `Item<T>`, `ItemOf` / `ItemValues` / `ItemsRef`, `ItemsChanged`, and the `SelectionParam` / `ItemParam` system params |
| `stack.rs` | `Stack { selection_ref }` shows the one `StackItem` child matching the selection; children are created lazily by a registered system (`StackChildSpawner`) and hidden, not despawned, when deselected |
| `breadcrumbs.rs` | `BreadCrumbs`: a `Link` trail for a `OneOrMore` selection; clicking a crumb trims the selection back to it |
| `combo.rs` | `Combo`: a button that opens a `List` in a `Float`, `ComboState` (open/closed), closes on an outside click |
| `float.rs` | `Float`: absolute-positioned, `GlobalZIndex(1000)`, large shadow; the popup layer for combos |
| `scroll_area.rs` | `ScrollArea::bundle`, `ScrollContents`, `ScrollBar` / `Orientation` and the thumb, which scrolls the contents when dragged (the wheel is handled by `list.rs`'s `update_scroll_position`) |
| `list.rs` | `List` with `ListLayout` and `CornerStyle`: renders the items of its `Selection` as `ItemView`s and keeps them in sync |
| `color_palette.rs` | `ColorPalette`: a grid of colour swatches over a selection of `Item<Color>` |
| `color_palette_combo.rs` | `ColorPaletteCombo`: a `ColorPalette` inside a combo-style popup |
| `color_push_button.rs` | `ColorPushButton`: a button showing a colour square |
| `numeric_stepper.rs` | `NumericStepper`, its `Value`, `UIntValue<T>`, `ValueChange` |
| `push_button.rs` | `PushButton` with label and optional icon; **the reference widget to copy** |
| `label.rs` | `Label::header` / `Label::regular` |
| `link.rs` | `Link` and its immutable `LinkState` |
| `card.rs` | `Card`: a themed background, border, radius and shadow on a node |

### Examples

Each is `just example NAME` (`cargo run --example NAME`) and needs a window.

| Example | Shows |
| --- | --- |
| `widgets` | a showcase: a scroll area, a list and combos sharing one selection via `Selection::reference`, and buttons that add more |
| `breadcrumbs` | `BreadCrumbs` and a `Stack` sharing one selection, with reused `StackItem` creators |
| `tabs` | a `List` as tab bar driving a `Stack`, with combos, colour palettes and cards inside |
| `color_palette` | `ColorPalette` and `ColorPaletteCombo` over `tailwind::COLORS` |
| `combos` | `Combo` variants |
| `lists` | `List` layouts and corner styles |
| `numeric_stepper` | `NumericStepper` |
| `scrollareas` | `ScrollArea`s scrolling vertically, horizontally or both |

Every example has a button that toggles the light and dark theme, and sets
`GlobalUiDebugOptions`, which is why `bevy_ui_debug` is switched on in
`[dev-dependencies]`.

### Tests

| Target | Holds |
| --- | --- |
| `tests/headless.rs` | widget behaviour driven by hand-written triggers in a headless `App` |
| `tests/reflection.rs` | the widget types and `Theme` are in the type registry |
| `tests/scenarios.rs` | Rust scenarios played by `rin_bevy_ui_tester`, a smoke test that every `WidgetsSetup` screen builds, and the tests of `WidgetsCheck` |
| `tests/ron_scenarios.rs` | `harness = false`: one test per `.ron` file in `tests/ron/`, see [Scenario files](#scenario-files) |
| `tests/common/mod.rs` | `widgets_app()` and `spawn_camera_and_root`, shared by the scenario targets |
| `tests/widgets_setup/mod.rs` | `WidgetsSetup`, the screens scenarios start from, one per widget, each modelled on its example |
| `tests/widgets_check/mod.rs` | `WidgetsCheck`, the checks about widget state the tester's built-in ones cannot say (`Selected`) |

## How the pieces fit

- **`UiRoot`.** An observer on `OnAdd, Node` inserts `UiRoot` on a node
  without a parent and without a `UiTargetCamera`, so root nodes aimed at
  another camera (a second window's UI, an overlay) are not the app's and are
  left alone. The camera is read when the `Node` is added, so a
  `UiTargetCamera` inserted later is not seen: spawn it in the same bundle. A
  `PostStartup` system logs an `error!` (it does not panic) unless there is
  exactly one such root; an app should spawn one. Two roots both get `UiRoot`.
- **Mouse events.** `UiRoot` observes Bevy's `Pointer<Over | Pressed |
  Released | Out>` and re-triggers each as a `MouseEvent` on the original
  target. `MouseEvent` is crate-private: it is how widgets inside this crate
  react to the pointer.
- **Selections are immutable components.** Change one by going through
  `SelectionParam::update_selection` / `update_selection_single`, whose
  closures return `Option<SelectionValue>` from methods such as
  `toggle_selected`, `push`, `pop`, `trim_to_length` and `set_selection`
  (`None` means the change is not allowed for that variant). The new value is
  inserted, which fires `OnInsert` observers.
- **Shared selections.** A widget holding `Selection::Ref(entity)` reads and
  writes the selection on `entity`. `BreadCrumbs`, `List` and `Combo` accept
  one; `Stack` takes the selection entity directly in `selection_ref`.
- **Shared items.** `ItemsRef(entity)` lets several selections draw from one
  `ItemValues`.
- **Scroll area contents keep their natural size.** `ScrollContents` uses
  `align_items: FlexStart` instead of stretching its children. Taffy 0.10
  (Bevy 0.19) reports no overflow for a grid whose items overflow it, so a
  `List` or palette stretched to the area's height would leave nothing to
  scroll. For the same reason, don't give scroll contents `height: 100%`.
  The `scroll_area_*.ron` scenarios guard this: put `Stretch` back and the
  wheel and thumb scenarios fail.
- **A scroll area's scroll position is `ComputedNode::scroll_position`.** The
  `ScrollPosition` component is not clamped (the wheel adds to it past either
  end); the layout clamps to what overflows into `ComputedNode`, which is what
  is on screen. Read that in a scenario (`ComputedNode.scroll_position.y`). It
  is floored and only refreshed by the next layout pass, so
  `handle_thumb_drag` starts each drag event from the current `ScrollPosition`
  (clamped to what overflows) instead: a frame can hold several drag events,
  and each must add to the last.

## Conventions

- **One `Plugin` per widget**, registered in `WidgetsPlugin::build` in
  `lib.rs`. Downstream apps add only `WidgetsPlugin`.
- **Widgets observe `MouseEvent`, not raw `Pointer<…>`.** Three places still
  observe a raw `Pointer<…>`: the crumbs in `breadcrumbs.rs` and the buttons in
  `numeric_stepper.rs` (`Pointer<Released>`), and the scroll thumb in
  `scroll_area.rs` (`Pointer<Drag>`); do not add more. The thumb is there
  because `MouseKind` has only `Over`, `Pressed`, `Released` and `Out`, so
  `MouseEvent` cannot carry a drag; a `MouseKind::Drag` would let it move
  over.
- **Styling comes from `Res<Theme>`.** Each widget has a `theme_*` system
  taking `In<Option<Entity>>`: its hook runs it for the one new entity with
  `run_system_cached_with(theme_x, Some(entity))`, and its plugin runs it for
  all of them with `(|| None::<Entity>).pipe(theme_x).run_if(resource_changed::<Theme>)`.
  `itertools::Either` unifies the two iterators. See `card.rs` for the
  smallest example.
- **Corner radius is a `Node` field.** Since Bevy 0.18 `BorderRadius` is
  `Node::border_radius`, not a component; set it on the node.
- **No hard-coded colours outside `theme.rs` and `tailwind.rs`.** Add a slot
  to `ThemeColors` (both default themes) rather than inlining a colour. One
  `Color::BLACK` border in `color_palette_combo.rs` predates this rule.
- **Setup lives in component hooks and `#[require(…)]`.** `on_add` for
  one-time setup (`PushButton`, `Combo`, `Link`, `ColorPushButton`),
  `on_insert` where re-inserting should re-run it (`Card`, `Label`, `List`,
  `ScrollArea`, `Float`, `NumericStepper`, `ComboState`). Default `Node`
  styling and a `Name` go in `#[require(…)]`.
- **Public widget components, their state and `Theme` derive `Reflect`**, with
  `#[reflect(Component)]` (`#[reflect(Resource)]` for `Theme`), so the type
  registry knows them by name (Bevy's `reflect_auto_register`). A field that
  cannot be reflected gets `#[reflect(ignore)]` with a comment saying why.
  `tests/reflection.rs` fails when a derive is dropped; add a new widget there.
  The short type paths `Label` and `ScrollArea` are also Bevy's own, so a lookup
  by short path is ambiguous for those two; use the full path.
- **State is an immutable component**; a transition inserts a new value
  (`ComboState`, `LinkState`, `Selection`, the palette combo's `State`).
- **Composite widgets keep their children in a refs component**
  (`PushButtonRefs`, `ComboSubRefs`, `ScrollAreaRefs`, `ColorSquareRef`,
  `ValueRef`) instead of searching `Children`.
- **Selections can be shared via `Selection::Ref`**; a widget that displays a
  selection must react when either its own entity or the referenced one
  changes (see `selection_change` in `breadcrumbs.rs` or `list.rs`).
- **Lint groups are `deny`.** `clippy::all`, `pedantic` and `nursery` (and
  `missing_const_for_fn`) fail the build, so `just lint` is the bar, not a
  suggestion. The only allows are the Bevy-shaped ones documented in
  `Cargo.toml`: `too_many_arguments`, `needless_pass_by_value` and
  `type_complexity` (systems take many params, `In<_>` / `Res<_>` / `Trigger<_>`
  by value, and nested `Query` types), plus `too_many_lines` and the numeric
  cast lints. `unused_crate_dependencies` stays off because it misfires per
  example / test target; `cargo machete` (in `just ci`) checks dependencies
  instead. Public functions returning `Result` need a `# Errors` section.
- **No panics in library code.** `unwrap_used`, `expect_used`, `panic`,
  `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr` and
  `indexing_slicing` are denied in `Cargo.toml`, and `clippy.toml` bans the
  panicking Bevy accessors (`Query::single(_mut)`, `World::resource(_mut)`).
  A system or observer that cannot find what it needs uses
  `let Ok(x) = query.get(e) else { return; };` (`continue` in a loop) and does
  nothing; a misconfiguration worth reporting is logged with `error!`. The
  `SelectionValue` methods return `None` for a change the variant does not
  allow. Integration tests carry a file-level `#![expect(clippy::unwrap_used,
  …, reason = "tests may abort loudly")]`.
- **Everything public is documented.** `missing_docs` and rustdoc's
  `missing_crate_level_docs` are `deny`: every public item has a `///`
  comment, every module (and each example) a `//!` header, and
  `RUSTDOCFLAGS="-D warnings"` in `just lint` keeps intra-doc links honest. Say
  what a thing is or does, not how it is implemented.
- **`#[expect(…, reason = "…")]`, never `allow`.** `allow_attributes` and
  `allow_attributes_without_reason` are denied in `Cargo.toml`.
- `#[derive(Debug)]` on every public type (`missing_debug_implementations`
  warns), and `const fn` / `#[must_use]` where clippy asks.

### Adding a widget

1. `src/my_widget.rs` with `MyWidgetPlugin` and the `MyWidget` component.
2. `#[require(…)]` for its default `Node` and `Name`; a hook for setup.
3. A `theme_my_widgets` system in the pattern above.
4. `MouseEvent` observers for hover and press.
5. `pub mod my_widget;` and `add_plugins(MyWidgetPlugin)` in `lib.rs`.
6. An example in `examples/`, and a row in the tables above.

## Running things

`rust-toolchain.toml` pins the compiler, and the `justfile` is the one place
commands are defined; every recipe is a plain `cargo` call you can read out
of it. You need `just` and, for `unused-deps`, `cargo-machete`.

| Task | `just` | Plain cargo |
| --- | --- | --- |
| Format | `just fmt` | `cargo fmt --all` |
| Lint | `just lint` | `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` |
| Test | `just test` | `cargo test --all-targets` then `cargo test --doc` |
| Run an example | `just example NAME` | `cargo run --example NAME` |
| All of CI | `just ci` | lint, test, `cargo machete --with-metadata` |

**The first build is slow.** Bevy takes many minutes to compile from cold;
dependencies are built with `opt-level = 3` even in dev (see `Cargo.toml`) so
the examples are usable, which makes that first build slower still. Later
builds only recompile this crate. Prefer `just test` over running examples in
a loop, and note that examples need a display.

`just bevy-lint` exists but is deliberately **not** part of `just ci`: it
needs a pinned nightly and trails Bevy releases.

### The private tester dependency

`rin_bevy_ui_tester` (`github.com/rinde/rin-bevy-ui-tester`, private) is a
`[dev-dependencies]` git dependency pinned to a revision in `Cargo.toml`; it
drives scripted pointer and keyboard scenarios against the widgets
(`tests/scenarios.rs`, see [Writing a scenario](#writing-a-scenario)).

- **Bumping it** means changing `rev` in `Cargo.toml` and `Cargo.lock`
  together (`cargo update -p rin_bevy_ui_tester`); `--locked` in CI fails if
  they disagree.
- **Bevy upgrades land in the tester first.** It pins `bevy = "=0.19.1"`
  exactly, so this crate's `bevy` requirement must resolve to the same version;
  upgrade Bevy in the tester (and rails), bump the `rev` here, then raise
  `bevy` here.
- **Locally**, Cargo needs your git credentials for the private repository:
  set `CARGO_NET_GIT_FETCH_WITH_CLI=true` so it uses the `git` binary (and so
  your credential helper or SSH agent) instead of its built-in client.
- **In CI**, `ci.yml` sets `CARGO_NET_GIT_FETCH_WITH_CLI=true` for every job
  and the local action `.github/actions/private-deps` configures git to send
  the repository secret `TESTER_REPO_TOKEN` (a token with read access to
  `rinde/rin-bevy-ui-tester`, for example a fine-grained personal access token
  with "Contents: read" on that repository only) for that repository. The
  `lint`, `test`, `unused-deps` and `cross-check` jobs all use it: the first,
  second and fourth build the tests, and `unused-deps` runs
  `cargo machete --with-metadata`, whose `cargo metadata` call (without
  `--no-deps`) reads the manifest of every package in the graph, so Cargo
  fetches the private repository there too. Without the secret those jobs fail
  early with a message saying so. Pull requests from forks get no secrets, so
  none of the four jobs can run for them; that is expected for a private
  project.
- **In cloud sessions**, `.claude/hooks/session-start.sh` exports the same
  variable and fetches the dependency with the container's git credentials.

### Claude Code on the web

`.claude/hooks/session-start.sh`, registered in `.claude/settings.json` under
`SessionStart`, prepares a fresh cloud container: it installs the pinned
toolchain with `clippy` and `rustfmt`, `just` and `cargo-machete` at pinned
versions, the Bevy system headers `.github/workflows/ci.yml` also installs,
then fetches the dependencies (the private tester included, see above) and
warms the build cache with `cargo build --all-targets` so the first real
build isn't a cold Bevy compile. It is a no-op unless `CLAUDE_CODE_REMOTE=true`
or run with `--force`, and is idempotent, with progress on stderr.

## Testing

- unit tests go in a `#[cfg(test)] mod tests` next to the code; `select.rs`'s
  `SelectionValue` tests are the reference for pure-logic coverage;
- integration tests go in `tests/`, driving a headless `App`;
- name tests as sentences: `a_stack_shows_only_the_selected_child`.

### Writing a headless integration test

`tests/headless.rs` builds an `App` with `MinimalPlugins` plus the plugins
`WidgetsPlugin`'s systems need to run without panicking on a missing
resource, none of which need a window or GPU:

- `WindowPlugin { primary_window: None, ..default() }` — registers
  `WindowEvent` and friends, which `bevy_picking` reads; no window is
  actually opened.
- `InputPlugin` — provides `ButtonInput<MouseButton>`, needed by
  `combo.rs`'s outside-click observer.
- `DefaultPickingPlugins` — provides `HoverMap`, needed by
  `list.rs`'s scroll handling.
- `ClearColor` (via `init_resource`) — normally inserted by the render
  plugins; `theme.rs`'s `update_theme` writes to it whenever `Theme`
  changes.

These are all needed unconditionally because `WidgetsPlugin` registers every
widget's systems regardless of which widgets a given test actually spawns.

Spawn entities on `app.world_mut()` directly (component hooks and `OnAdd`/
`OnInsert` observers fire synchronously), then call `app.update()` at least
twice to flush the `Commands` those observers queue and let `Update`-schedule
systems (like `breadcrumbs.rs`'s `selection_change`) react. Register
`StackItem` creator systems with `world.register_system`, matching the
`In<StackChildSpawner>` pattern in `examples/tabs.rs`.

If a widget can't be exercised headlessly because it needs a render or text
plugin, note it in the PR rather than pulling in a window.

### Writing a scenario

A scenario drives the widgets with scripted pointer and keyboard input through
`rin_bevy_ui_tester` (see its README for the `Step`, `Target` and `Check`
vocabulary) and asserts on what they did. Prefer it to hand-written
`Pointer<..>` triggers whenever the claim is about what a person would see
after using the UI.

- **Where.** Scenarios go in `tests/scenarios.rs`, a test target of its own, so
  `tests/headless.rs` stays as it is. Shared code lives in `tests/common/mod.rs`
  and is pulled in with `mod common;`. Its items are `pub` under a file-level
  `#![expect(unreachable_pub, ..)]`, because clippy's `redundant_pub_crate`
  rejects `pub(crate)` there. Keep it small: a helper that is not used by every
  target that pulls the module in fails `dead_code` in the ones that do not.
- **The header.** If a scenario file unwraps (a `Result` from `from_ron`, say),
  give it the same file-level `#![expect(clippy::unwrap_used,
  clippy::indexing_slicing, reason = "tests may abort loudly")]` as the other
  integration tests; `expect`, never `allow`. Leave it off a file that needs
  none, because an unfulfilled `expect` is itself an error under `-D warnings`.
- **Build the app with `common::widgets_app()`**: the tester's
  `HeadlessPlugins` (a virtual window, fixed 1/60 s frames, picking, UI, no
  GPU) plus `WidgetsPlugin`, with `ClearColor` initialised (`HeadlessPlugins`
  does not insert it and `theme.rs`'s `update_theme` writes it), finished and
  cleaned up. Do not use `Scenario::headless_app()`: its app is already
  finished and refuses `add_plugins`. Play with
  `scenario.run_in(&mut widgets_app())`, which returns a `ScenarioReport`;
  assert `report.passed()` with `report.failures()` as the message.
- **Setup.** The scenario's `.setup(|world| ..)` closure builds the UI and runs
  before the first update. It must spawn a `Camera2d` and exactly one root
  `Node` (`common::spawn_camera_and_root` does both and returns the root), or
  `UiRoot`'s `PostStartup` check finds no root.
- **Targets.** `PushButton` inserts Bevy's `Button`, so
  `Target::role(WidgetRole::Button, "Add")` finds it by its label.
- **Checks are data.** There are no check closures. Read widget state with
  `Check::component::<C>(target, ".field")` and `Check::resource::<R>(".field")`
  (the type must derive `Reflect`, with `#[reflect(Component)]` /
  `#[reflect(Resource)]`), or write the app's own `#[reflect(AppCheck)]` enum (the tests have one,
  `WidgetsCheck`; see [Scenario files](#scenario-files)).
- **Frames.** Time is fixed at 1/60 s a frame. A pointer step (`move_to`,
  `press`, `release`) takes one frame and a `Step::check` none, so a check right
  after the step sees its effect, `MouseEvent` observers included; add
  `Step::wait(n)` only for something that needs more frames.
- **Start from a screen.** `.app_setup(WidgetsSetup::Buttons)` (or `setup:
  App(Buttons)` in a file) instead of a `.setup(..)` closure, when a screen of
  `tests/widgets_setup/mod.rs` has the widgets the scenario needs. `mod
  widgets_setup;` next to `mod common;` pulls it in; it is its own module,
  not part of `common`, because `common` items an including target does not
  use fail `dead_code`.
- **Prove the assertion is real.** When you write a scenario, break the
  expectation once (a different theme slot, say) and see it fail before
  trusting it.

### Scenario files

A scenario can be data: `tests/ron/NAME.ron` is read by `tests/ron_scenarios.rs`,
a `harness = false` target (`[[test]]` in `Cargo.toml`) that calls the tester's
`run_ron_tests`, so every file is one test named after it and the output reads
like libtest's. `cargo test --test ron_scenarios push_button` runs the files
whose name contains `push_button`; `just test` runs them with everything else.
Each file runs in a fresh `common::widgets_app()` after `Registry::install`, so
a file can check any `Reflect` type of the app by short type path.

- **The format** is the tester's RON script (`format_version: 3`); see its
  README and `tests/ron/*.ron` there. The same file also plays in the tester's
  gallery and recorder. Write or record it there, then drop it in `tests/ron/`.
  A colour in a check is `Srgba((red:..,green:..,blue:..,alpha:..))`; to get
  the numbers of a theme slot, build the scenario in Rust and print
  `to_ron()`.
- **The setup** is `setup: App(Buttons)`: a variant of `WidgetsSetup`, the one
  `#[reflect(AppSetup)]` enum of the tests (`Buttons`, `Combos`, `Lists`,
  `ScrollAreas`, `BreadCrumbs`, `Tabs`, `ColorPalette`, `NumericStepper`),
  which spawns a `Camera2d`, the one root `Node` and a screen of the widget it is
  named after, following the example of that name (the `NumericStepper` one adds
  steppers the example lacks), each with a "Change theme" button. A variant is
  found by Bevy's automatic registration, so adding one needs no registration
  call; add it to the smoke test in `tests/scenarios.rs` as well. `Buttons`
  also keeps the `Clicks` resource, the clicks on "Add". `AppSetup::capture`
  (recording a session from the app as it is) is not implemented: only the
  tester's gallery and recorder call it, which these tests do not run, so it
  would go untested.
- **The check** is `App(Selected(of: .., items: [..]))`: `WidgetsCheck`, the
  one `#[reflect(AppCheck)]` enum of the tests, in `tests/widgets_check/mod.rs`
  (Rust: `Check::app(WidgetsCheck::Selected { .. })`; `mod widgets_check;`
  pulls it in, like `widgets_setup`, and it stays out of `common` for the
  same `dead_code` reason). A `Selection` holds entities, which mean nothing
  in a script, so `Selected` names the selected items by what they show, in
  the order they were selected: an `Item<String>` by its text, an
  `Item<Color>` by its Tailwind colour (`blue-500`) when it is one. `of` is
  any target that resolves to a widget: the selection is the first on it or
  above it, and a `Selection::Ref` is followed, so the `List`, `Combo`,
  `BreadCrumbs` or `ColorPalette` that shares a selection reports it. It goes
  through `resolve`, which needs the widget laid out (a scenario waits a
  frame or two after its setup) and a `Node`; a bare selection entity
  without one cannot be named, but every widget referring to it can. A
  failure says `<target> has ["b"] selected, not ["a"]`. `facts` proposes a
  `Selected` for each list, combo, trail and palette that one target leads
  back to (the type's `Component(..)` when it is alone, else `Nth(i, ..)`),
  so a recording suggests the selections that changed. Add a variant here
  when a claim about widgets cannot be said with `Component(..)` or
  `Resource(..)`.
- **Targets on a screen with several of one widget.** The `NumericStepper`
  screen puts a `Name` on every stepper (`default stepper`, `u16 stepper`,
  `u32 stepper`, `f32 stepper`, `stepper by two`, and `shared stepper` with
  the `first mirror stepper` and `second mirror stepper` that point to it
  through `ValueRef`), so a file reaches one stepper's button with
  `Within(Name("u16 stepper"), Role(role: Button, label: "+"))` and reads its
  value with `TextOf(Name("u16 stepper"), Exact("- 11 +"))` (the "-", value
  and "+" labels). Move the pointer to the button before every click: a wider
  value moves the buttons.
- **Targets on the `ScrollAreas` screen.** Four 200 by 200 areas side by side
  (they and "Change theme" fit the default 1280 by 720 window, which every
  scenario runs in), each with a `Name`: `two-way scroll area`, `vertical
  scroll area`, `horizontal scroll area` and `fitting scroll area` (three
  squares that never overflow). `MoveTo(Name("vertical scroll area"))` puts
  the pointer over the squares, so `Wheel(x: 0.0, y: -100.0)` scrolls it down
  (a positive wheel y scrolls up). Reach contents with `Within(Name(".."),
  Component("ScrollContents"))` and a thumb with `Within(Name(".."),
  Name("ScrollThumb"))` (`Nth(0, ..)` is the vertical bar's, `Nth(1, ..)` the
  horizontal one's in the two-way area). A horizontal-only area has no
  usable horizontal bar (it ends up 0 wide at the top of the area, a layout
  bug this does not fix), so scroll it with the wheel.
- **Observers on a screen's entities** are added with `.observe(..)` on the
  spawned entity. `Observer::new(..)` in a bundle is a global observer.
- **Prove it fails** as with Rust scenarios: change an expected value in the
  file and see the test fail before trusting it.

## Before you call it done

1. `just fmt`
2. `just lint`
3. `just test`
4. New behaviour has a test, where the harness allows it
5. This file is updated if a module, example, convention or command changed
6. The work is on a PR, rebased onto `main` if needed
