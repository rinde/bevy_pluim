# Working on bevy-pluim

Guidance for coding agents. This file is the index: it holds the rules
that apply to every change and points to the detail, which lives in
`docs/`. If a doc and the code disagree, the code wins and the doc gets
fixed.

## What this is

A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons,
lists, combos, scroll areas, tabs, breadcrumbs, colour palettes, a slider and a
numeric stepper, all themed from one `Theme` resource. It is used by nopjes,
but it is a library in its own right.

**No application or game state.** The crate takes Bevy types only, so its
widgets can be exercised without an application behind them. Anything that
knows what a picked item *means* belongs in the application that uses the
widget, not here.

## Core beliefs

1. **The repo is the source of truth.** If it is not in the code or the docs,
   do not assume it. Read before you act.
2. **Read before write.** Read the module you are changing, its
   `docs/modules/<module>.md` and a sibling widget before adding anything;
   match what is already there.
3. **Small, verifiable changes.** One PR does one thing.
4. **Docs stay in sync.** When behaviour changes, update the doc of the
   module you changed (`docs/modules/<module>.md`, or the `docs/design/`,
   testing or examples doc it concerns) in the same PR. Change this file only
   when a module, doc, convention or command is added, removed or changes; it
   stays an index of one line per module. A stale doc is worse than none.
5. **No gold-plating.** Implement what is asked; no abstractions, config or
   features "just in case".
6. **Explicit over implicit.** Name things clearly and let the type system
   carry the meaning.

## Issues and coordination

Work is tracked in GitHub issues. Issues ready for implementation use the
`ready for agent` label. When working on an issue, follow its title and
description; ask for clarification if its scope is genuinely ambiguous.
Changes should be made through pull requests.

Run `just ci` locally to check the same lint, test and unused-dependency
recipes used by CI.

## Where to find what

Read only what the task needs: the row of the module you touch tells you
which doc to open, and each doc links on to the design notes it relies on.

| Read | When |
| --- | --- |
| `docs/modules/<module>.md` | before changing `src/<module>.rs`: its behaviour, edge cases, tests and bench workloads |
| [`docs/design/pointer-and-buttons.md`](docs/design/pointer-and-buttons.md) | pointer and touch input, `MouseEvent`, `Hovered`, `UiRoot`, how buttons activate |
| [`docs/design/keyboard-focus.md`](docs/design/keyboard-focus.md) | tab stops, the focus ring, arrow keys in composite widgets, directional navigation, hidden nodes |
| [`docs/design/lists-and-selection.md`](docs/design/lists-and-selection.md) | `ListBox` and `Selection`, custom item views, shared selections and items |
| [`docs/design/popups.md`](docs/design/popups.md) | combo popups: focus, opening and closing, placement, why a `Float` stays in its window |
| [`docs/design/scrolling.md`](docs/design/scrolling.md) | scroll areas: wheel, finger pan, clamping, focus-ring room, scroll-into-view |
| [`docs/examples.md`](docs/examples.md) | the examples, building trees with `bsn!`, the scenario gallery |
| [`docs/testing.md`](docs/testing.md) | test targets, headless tests, writing a scenario, determinism, recording |
| [`docs/scenario-files.md`](docs/scenario-files.md) | the `tests/ron/` format and checks, and what each `WidgetsSetup` screen holds |
| [`docs/benchmarks.md`](docs/benchmarks.md) | the `bench` example, its workloads and recorded numbers |

### Modules

**Foundation**

- [`lib.rs`](docs/modules/lib.md): `WidgetsPlugin` (the one plugin an app adds), `UiRoot`, and the crate-private `MouseEvent` every widget reacts to.
- [`theme.rs`](docs/modules/theme.md): `Theme`: the light and dark `ThemeColors`, fonts, rem type scale, radii and shadows, with the WCAG contrast guard.
- [`theme_editor.rs`](docs/modules/theme_editor.md): `ThemeEditor`: a card that edits the `Theme` in use, every colour slot and number, while the widgets follow.
- [`tailwind.rs`](docs/modules/tailwind.md): The 242 Tailwind colours and `color_name`, for colour pickers and accessible names.
- [`focus.rs`](docs/modules/focus.md): Crate-private keyboard focus: tab stops, roving groups, the focus ring, hidden-state tracking and the accessibility sync helpers.
- [`glyph_images.rs`](docs/modules/glyph_images.md): Crate-private per-app cache of the images the library draws (chevron, minus, plus, sun, computer, moon).

**Selection and lists**

- [`select.rs`](docs/modules/select.md): The selection model: `Selection`, `SelectionValue`, items (`Item<T>`, `ItemOf`, `ItemsRef`), `ItemsChanged`, and the shared helpers that reconcile views with items.
- [`list.rs`](docs/modules/list.md): `List`: a `ListBox` of item views over a `Selection`, with layouts, corner styles, notes and custom views.
- [`virtual_list.rs`](docs/modules/virtual_list.md): `VirtualList`: rows of one height for many thousands of items, rendering only those in view.
- [`stack.rs`](docs/modules/stack.md): `Stack`: shows the one `StackItem` child that matches a selection, building children lazily.
- [`breadcrumbs.rs`](docs/modules/breadcrumbs.md): `BreadCrumbs`: a trail of `Link`s for a `OneOrMore` selection; a click trims the selection.

**Popups**

- [`combo.rs`](docs/modules/combo.md): `Combo`: a button that opens a `List` in a popup.
- [`filter_combo.rs`](docs/modules/filter_combo.md): `FilterCombo`: a `Combo` whose popup has a search box that filters the options.
- [`popup.rs`](docs/modules/popup.md): Crate-private parts the combo popups share: button look, outside click, open/close side effects, combo-box accessibility.
- [`float.rs`](docs/modules/float.md): `Float`: the viewport-placed popup layer, and the helpers that keep popups inside the window.
- [`tooltip.rs`](docs/modules/tooltip.md): `Tooltip`: a popup shown after the pointer rests on a node, or a finger is held on it (a long press: no `Activate` on lift).
- [`toast.rs`](docs/modules/toast.md): `Toast`: a short-lived message stacked along the bottom of its parent.

**Buttons and inputs**

- [`push_button.rs`](docs/modules/push_button.md): `PushButton`: a label and optional icon; the reference widget to copy.
- [`icon_button.rs`](docs/modules/icon_button.md): `IconButton`: an icon-only button with sizes, tones and an optional toggle state.
- [`link.rs`](docs/modules/link.md): `Link`: clickable text, a button with `Role::Link`.
- [`color_push_button.rs`](docs/modules/color_push_button.md): `ColorPushButton`: a button showing a colour square.
- [`numeric_stepper.rs`](docs/modules/numeric_stepper.md): `NumericStepper` and its validated `Value`: - / + buttons, arrow keys, drag to scrub, click or F2 to type.
- [`slider.rs`](docs/modules/slider.md): `Slider`: a themed horizontal range control over Bevy's headless slider.
- [`text_field.rs`](docs/modules/text_field.md): `TextField`: a single-line text input with change and commit events.

**Composite pickers**

- [`segmented_control.rs`](docs/modules/segmented_control.md): `SegmentedControl`: a track of segments that picks one item, with optional icons.
- [`tab_bar.rs`](docs/modules/tab_bar.md): `TabBar`: Bevy's tab list over a selection of `Item<String>`.
- [`light_dark_toggle.rs`](docs/modules/light_dark_toggle.md): `LightDarkToggle`: a Light / System / Dark switch driving the `ThemePreference` resource.
- [`indicator.rs`](docs/modules/indicator.md): Crate-private sliding highlight behind the selected item of a track (segmented control, tab bar, light-dark toggle).
- [`filter_chips.rs`](docs/modules/filter_chips.md): `FilterChips`: a wrapping row of pill chips that picks one `Item<Chip>`.
- [`gallery.rs`](docs/modules/gallery.md): `Gallery`: grouped picture cards that pick one item.
- [`toolbar.rs`](docs/modules/toolbar.md): `Toolbar`: a row of tool buttons over `Item<Tool>`, filled or underlined.
- [`color_palette.rs`](docs/modules/color_palette.md): `ColorPalette`: a grid of colour swatches over a selection of `Item<Color>`.
- [`color_palette_combo.rs`](docs/modules/color_palette_combo.md): `ColorPaletteCombo`: a `ColorPalette` in a combo-style popup.

**Containers and display**

- [`scroll_area.rs`](docs/modules/scroll_area.md): `ScrollArea`: scrolling contents with themed scroll bars, the wheel handler, a finger's pan and scroll-into-view.
- [`card.rs`](docs/modules/card.md): `Card`: themed background, border, radius and shadow; the smallest theming example.
- [`panel.rs`](docs/modules/panel.md): `Panel`: a card with a header, optional close button, strip, scrolling body and footer.
- [`setting_row.rs`](docs/modules/setting_row.md): `SettingRow` and `SettingRows`: a title, description and control slot, divided rows.
- [`label.rs`](docs/modules/label.md): `Label`: themed text in the styles of the type scale.
- [`kbd.rs`](docs/modules/kbd.md): `Kbd`: a display-only key cap.
- [`pill.rs`](docs/modules/pill.md): `Pill`: a small uppercase status tag.
- [`badge.rs`](docs/modules/badge.md): `Badge`: a count or warning pill in the corner of a button.
- [`icon.rs`](docs/modules/icon.md): `Icon`: an app-provided image tinted with a theme role.
- [`divider.rs`](docs/modules/divider.md): `Divider`: a 1 px horizontal or vertical rule.

### Examples and tests

- Examples (`just example NAME`, need a window): `overview` (every widget, one
  page per group; a new widget or variant goes on its page), `hud` (the
  redesign's HUD over sample data), `bench` (headless CPU and memory
  benchmark) and `gallery` (plays the scenario files in a window). See
  [`docs/examples.md`](docs/examples.md).
- Tests: `tests/headless.rs` (behaviour in a headless `App`),
  `tests/reflection.rs` (type registry), `tests/scenarios.rs` and
  `tests/ron_scenarios.rs` (scripted scenarios, `tests/ron/*.ron`), and the
  GPU-only `tests/recorded_scenarios_gpu.rs`. See
  [`docs/testing.md`](docs/testing.md).

## Conventions

- **`sync_accessibility` systems look at what changed.** Bevy resets a node's role and
  label in `UiSystems::PostLayout` through `DerefMut`, so `Changed<AccessibilityNode>` catches
  every reset before `AccessibilitySync` runs. Gallery, filter chips, toolbar, segmented
  control, tab bar, light-dark toggle, push button, icon button and link therefore query only
  `Or<(Changed<AccessibilityNode>, Changed<View>, ..)>` (a `ParamSet` holds the unfiltered
  query for the rest); the selection widgets add the owners `StructureChanges::affected` names,
  a `SegmentedControl` also `Added<Selected>`, removed `Selected` and changed `Item<String>`,
  buttons removed `ToggleState`, the light-dark toggle a changed `ThemePreference`. The `sync_*`
  helpers write only on a difference, so a system never marks its own nodes changed; an idle
  frame writes none.
- **Nothing that runs every frame walks the whole world.** An app may hand a widget ten
  thousand items, so a system that runs each frame touches what is on
  screen, not every item: `VirtualList` looks at its pooled rows only, and finds an item by
  index in `ItemValues`. Work over all items belongs to a change (`ItemsChanged`, a new
  `Selection`) or a key press, not to the frame. A `List` spawns a view per item, so use a
  `VirtualList` for long lists.
- **Deferred construction decides at run time.** A hook that builds parts on first insert
  (`PushButton`, `Link`, `IconButton`, `Panel`, `SettingRow`) never tests the world for its refs
  while it queues the work: two inserts applied in one flush (`World::insert_batch`) both run
  their hooks before either hook's commands, so both would build. The test for existing parts
  sits inside the queued command or system, so one set of parts and observers is built and the
  last insert's configuration wins (`two_inserts_of_a_panel_in_one_command_batch_*` in
  `tests/headless.rs`).
- **One `Plugin` per widget**, registered in `WidgetsPlugin::build` in
  `lib.rs`. Downstream apps add only `WidgetsPlugin`.
- **Widgets observe `MouseEvent`, not raw pointer events (`PointerPress` and
  friends).** The only observers of them in the library are the producer in
  `lib.rs`; do not add more. `MouseKind` says `Over`, `Pressed`, `Released`, `Click`, `Drag`, `Out` and `Cancel`,
  so a widget can tell a click (a press and a release on the same node) from a
  release that ends a press begun elsewhere.
- **Styling comes from `Res<Theme>`.** Each widget has a `theme_*` system
  taking `In<Option<Entity>>`: its hook runs it for the one new entity with
  `run_system_cached_with(theme_x, Some(entity))`, and its plugin runs it for
  all of them with `(|| None::<Entity>).pipe(theme_x).run_if(resource_changed::<Theme>)`.
  `itertools::Either` unifies the two iterators. See `card.rs` for the
  smallest example.
  A restyle writes `TextFont` and `TextColor` only when the value differs
  (`set_if_neq`, or a compare before assigning a field): every write marks
  the text changed and Bevy lays the whole tree out again, so a colour-only
  theme change writes no `TextFont` and a font-only one no `TextColor`
  (`restyling_text_widgets_*` in `tests/headless.rs`). A widget that owns only
  some `TextFont` fields compares those before assigning. A new widget with
  text follows the same rule.
  A widget with an interaction state (hovered, pressed, selected) takes its
  background from one function of that state (`background_for` in
  `push_button.rs`, `item_background` in `list.rs`) that both its pointer
  handler and its `theme_*` system call, so a theme change under a resting
  pointer keeps the hover colour. The state is read from `Hovered` and `Pressed`,
  not tracked by hand; a `PushButton` with `NoDefaultHover` rests at
  `background_primary` for its owner to colour.
- **A long press on a `Tooltip` node cancels its press.** When the touch popup shows,
  `tooltip.rs` removes `Pressed` from the pressed node up to the tooltip node and triggers a
  `MouseKind::Cancel`, so the look rests and Bevy's `Button` fires no `Activate` on the lift.
  Every `Activate` widget (`IconButton`, `Toolbar`, `SegmentedControl`, an app's button) is
  covered without code of its own; a widget that picks on `MouseKind::Click` or `Released` would
  not be. The drag threshold is the shared `crate::DRAG_THRESHOLD`.
- **Corner radius is a `Node` field.** Since Bevy 0.18 `BorderRadius` is
  `Node::border_radius`, not a component; set it on the node. Since Bevy 0.20
  its corners are `CornerRadius`es and its constructors (`BorderRadius::all`,
  `top`, ..) are not `const`, so a function that builds one cannot be either.
- **No hard-coded colours outside `theme.rs` and `tailwind.rs`.** Add a slot
  to `ThemeColors` (both default themes) rather than inlining a colour.
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
  The short type paths `Label`, `ScrollArea` and `Slider` are also Bevy's own, so a
  lookup by short path is ambiguous for those three; use the full path
  (`bevy_pluim::slider::Slider`).
- **Insert-time configuration is immutable; refs are read-only.** `PushButton`, `IconButton` and
  `Link` are `#[component(immutable)]`: a live button is changed by building or cloning a value,
  editing it outside the world and inserting it again, which updates the children in place (no
  `Changed<..>` synchronisation). `PushButtonRefs`, `IconButtonRefs` and `ColorSquareRef` are
  immutable, queryable records with private fields and no public constructor; read them through
  `PushButtonRefs::label()` / `icon()`, `IconButtonRefs::icon()` and `ColorSquareRef::entity()`.
  `compile_fail` doctests on the types guard both.
- **Input is owned by the app, derived state by the widget.** `Item<T>::data` is edited in place; `ItemOf` is reparented through Bevy's relationship API; `ItemsRef` and `Stack` are immutable, so changing them means inserting a new value, which refreshes their consumers. Nothing hands out `&mut ItemValues`.
- **Configuration is immutable; live state is not.** `Slider`, `NumericStepper` and
  `TextField` are `#[component(immutable)]`: their fields are read by the insert hook, and
  changing a setting (range, format, names, icons, widths, placeholder, initial text) means
  inserting a new component, which rebuilds the parts and syncs the accessibility node (the
  stepper keeps its `ValueRef`, a `TextField` sets its text without a `TextFieldChange`). The
  live data stays mutable where it already is: the stepper's `Value` / `UIntValue` (through
  its validated operations, observed by `on_change`), Bevy's `SliderValue`, and the
  `EditableText` of `TextFieldRefs::input`. A system never takes the three as `&mut` (a
  `compile_fail` doctest on each guards it) and none syncs them with a per-frame `Changed`
  query. A new configuration component follows the same split.
- **Display widgets are configured by replacement.** `Label`, `Kbd`, `Pill`, `Toast` and `Badge` are immutable components with public fields: construct or clone a value, edit it outside the world and insert it. A mutable query (`Query<&mut Pill>`) does not compile, so migrate such code to `commands.entity(e).insert(pill.clone().with_tone(..))` (read with `&Pill`). Live data such as `Item<T>::data` and `NumericStepper`'s `Value` is not configuration and stays outside this rule.
- **State is an immutable component**; a transition inserts a new value
  (`ComboState`, `Selection`, the palette combo's `ColorPaletteComboState`).
- **A view's relationships are private and read-only.** `Segment`, `TabBarTab`, `ChipView`, `GalleryCardView` and `ToolbarButton` (and `SegmentIndicator`, which has only the owner) are immutable components with private `item` and `owner` fields and public `item()` / `owner()` getters, built only by their widget (a `pub(crate)` `new`), so typed code can neither construct one nor change a live one (`compile_fail` doc tests on each). They stay `Reflect`, so scenarios and `WidgetsCheck` read the fields by the reflected paths `item` and `owner` (`the_item_and_owner_of_a_selection_view_are_read_by_getters_and_by_reflection`). The application-authored item data (`Chip`, `GalleryCard`, `Tool`, `Item<String>`) stays public and the app supplies meaningful accessible labels; the library has no non-empty-label type or validation. The child-refs records (`ChipRefs`, `GalleryCardRefs`, `ToolbarButtonRefs`) are a different thing and unchanged.
- **Composite widgets keep their children in a refs component**
  (`PushButtonRefs`, `ComboRefs`, `ScrollAreaRefs`, `ColorSquareRef`,
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
  cast lints, and one style allow for tests that is not Bevy-shaped,
  `assert_is_empty` (`assert!(x.is_empty())` reads better in tests than
  `assert_eq!(x, [] as [T; 0])`). `unused_crate_dependencies` stays off because it misfires per
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
3. A `theme_my_widgets` system in the pattern of "Styling comes from `Res<Theme>`".
4. `MouseEvent` observers for hover and press.
5. `pub mod my_widget;` and `add_plugins(MyWidgetPlugin)` in `lib.rs`.
6. Its variants on a page of the `overview` example, a one-line row under
   [Modules](#modules) and a `docs/modules/my_widget.md` in the shape of the
   others.
7. A `WidgetsSetup` screen for the scenarios: a variant in
   `tests/widgets_setup/mod.rs`, added to the smoke test in
   `tests/scenarios.rs`, and its notes under the screens of
   [`docs/scenario-files.md`](docs/scenario-files.md).

## Running things

`rust-toolchain.toml` pins the compiler, and the `justfile` is the one place
commands are defined; every recipe is a plain `cargo` call you can read out
of it. You need `just` and, for `unused-deps`, `cargo-machete`.
`sccache` is optional: when it is on the `PATH` the `justfile` exports it as
`RUSTC_WRAPPER`, so a fresh worktree gets its dependencies from the cache
instead of compiling Bevy again. A `RUSTC_WRAPPER` you set yourself wins, and
`RUSTC_WRAPPER= just test` builds without it. Plain `cargo` calls do not go
through the `justfile` and so do not use it.

| Task | `just` | Plain cargo |
| --- | --- | --- |
| Format | `just fmt` | `cargo fmt --all` |
| Lint | `just lint` | `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` |
| Test | `just test` | `cargo test --all-targets` then `cargo test --doc` |
| Run an example | `just example NAME` | `cargo run --example NAME` |
| Play every scenario in a real window | `just gallery-test` | `cargo run --example gallery -- --run-all` |
| Benchmark CPU and memory | `just bench -- --workload tooltips --size 5000` | `cargo run --release --example bench -- ...`, see `docs/benchmarks.md` |
| Check rendered scenarios | `just record` (`just record --save` to save recordings) | Needs a GPU, see [Recording the scenarios](docs/testing.md#recording-the-scenarios) |
| All of CI | `just ci` | lint, test, `cargo machete --with-metadata` |

**The first build is slow.** Bevy takes many minutes to compile from cold;
dependencies are built with `opt-level = 3` even in dev (see `Cargo.toml`) so
the examples are usable, which makes that first build slower still. Later
builds only recompile this crate. Prefer `just test` over running examples in
a loop, and note that examples need a display.

**Dependencies have no debuginfo** (`debug = false` in
`[profile.dev.package."*"]`), so a test or example binary is far smaller and a
slot's `target/debug` fits a cloud container's disk; this crate keeps its
own. The first build after that setting landed rebuilds every dependency, once
per `target/`. To debug into a dependency, turn it back on for one invocation:
`cargo --config 'profile.dev.package."*".debug=true' test ...` (that rebuilds
the dependencies again, and again when you drop the flag).

**Keep output short.** Build and test logs fill an agent's context fast, so
ask for the part you need:

- Run one test by name, not the whole suite: `just test NAME` (the name is a
  filter passed to `cargo nextest`), or `cargo test -q --test headless NAME`.
  Run the full `just test` once, before you call it done.
- Use quiet output (`cargo test -q`, `cargo build -q`) and cut what comes back:
  `... 2>&1 | tail -n 50`, or `... 2>&1 | grep -E 'FAILED|error|panicked'` to
  see only what broke.
- Read large files in parts (an offset and a line limit, or `sed -n '120,200p'`)
  after finding the place with `grep -n`; `tests/headless.rs` (19k lines) and
  `tests/scenarios.rs` are too long to read whole.

`just bevy-lint` exists but is deliberately **not** part of `just ci`: it
needs a pinned nightly and trails Bevy releases.

### Bevy features

The library depends on `bevy` with `default-features = false` and the features
`ui_api`, `scene`, `picking`, `bevy_ui_widgets` and `bevy_render` (the icons it
draws use `bevy::render::render_resource` image types), so it forces neither
audio, 3D / PBR, glTF nor a windowing backend on a downstream app (#696). The
application chooses the rest: the platform and renderer features (`bevy_winit`,
`bevy_ui_render`, `default_font`, ..), most simply Bevy's `ui` profile or
`default`. The examples and tests get Bevy's default features, `bevy_ui_debug`
and `debug` from `[dev-dependencies]` only. Adding a `use bevy::..` of a module
behind another feature means adding that feature to `[dependencies]`; check
with `cargo check --lib --locked` (dev dependencies are not unified into it) and
with a downstream fixture that sets `default-features = false`:
`cargo tree -e normal` must show no `bevy_audio`, `bevy_pbr`, `bevy_gltf` or
`bevy_winit` (541 packages with Bevy's defaults before, 427 after).

### The scenario tester dependency

`bevy_stagecraft` (`github.com/rinde/bevy_stagecraft`) is a public
`[dev-dependencies]` git dependency pinned to a revision in `Cargo.toml`; it
drives scripted pointer and keyboard scenarios against the widgets
(`tests/scenarios.rs`, see [Writing a scenario](docs/testing.md#writing-a-scenario)).

- **Bumping it** means changing `rev` in `Cargo.toml` and `Cargo.lock`
  together (`cargo update -p bevy_stagecraft`); `--locked` in CI fails if
  they disagree. The repository is public, so no credentials are needed.

### Claude Code on the web

`.claude/hooks/session-start.sh`, registered in `.claude/settings.json` under
`SessionStart`, prepares a fresh cloud container: it installs the pinned
toolchain with `clippy` and `rustfmt`, `just` and
`cargo-machete` at pinned versions, the Bevy system headers
`.github/workflows/ci.yml` also installs,
then fetches the dependencies (including the public scenario tester above) and
warms the build cache with `cargo build --all-targets` so the first real
build isn't a cold Bevy compile. It is a no-op unless `CLAUDE_CODE_REMOTE=true`
or run with `--force`, and is idempotent, with progress on stderr.
Repository hooks run in single-repository cloud sessions; Claude's project
threads and sessions with multiple repositories do not load them (see
[cloud hook limitations](https://code.claude.com/docs/en/cloud-environments#limitations-in-cloud-sessions)).
`.claude/settings.json` also sets the auto-compact window to 200k tokens for
Sonnet 5.5 and Opus 5.5 (`modelSettings.<model>.autoCompactWindow`).

## Testing

- unit tests go in a `#[cfg(test)] mod tests` next to the code; `select.rs`'s
  `SelectionValue` tests are the reference for pure-logic coverage;
- integration tests go in `tests/`, driving a headless `App`;
- name tests as sentences: `a_stack_shows_only_the_selected_child`.

How to write a headless test or a scenario, and how scenarios are replayed
and recorded: [`docs/testing.md`](docs/testing.md) and
[`docs/scenario-files.md`](docs/scenario-files.md).

## Before you call it done

1. `just fmt`
2. `just lint`
3. `just test`
4. New behaviour has a test, where the harness allows it
5. The `docs/` file of each module or design area you changed is updated;
   this file only if a module, doc, example, convention or command was added,
   removed or renamed
6. The work is on a PR, rebased onto `main` if needed
