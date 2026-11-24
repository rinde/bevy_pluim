# Testing

Test targets, headless tests, Rust scenarios, schedule shuffling and the GPU recorder. Scenario files and the screens they start from are in [scenario-files.md](scenario-files.md).

When the behaviour described here changes, update this file in the same PR.

## Test targets

| Target | Holds |
| --- | --- |
| `tests/headless.rs` | widget behaviour driven by hand-written triggers in a headless `App` |
| `tests/reflection.rs` | the widget types and `Theme` are in the type registry |
| `tests/scenarios.rs` | Rust scenarios played by `bevy_stagecraft`, a smoke test that every `WidgetsSetup` screen builds, the tests of `WidgetsCheck`, and generated per-file tests that play each `tests/ron/` file with the UI drawn to an image (`image_target_app`) and under shuffled schedules, see [Determinism](testing.md#determinism) |
| `tests/ron_scenarios.rs` | Generated libtest cases for every `.ron` file and each of its default, scale 2 and text 1.5x variants, see [Scenario files](scenario-files.md) |
| `tests/recorded_scenarios_gpu.rs` | needs `--features record` and a GPU, and is `#[ignore]`d: records every `tests/ron/` file offscreen, see [Recording the scenarios](testing.md#recording-the-scenarios) |
| `tests/common/mod.rs` | `widgets_app()` and `spawn_camera_and_root`, shared by the scenario targets |
| `tests/widgets_setup/mod.rs` | `WidgetsSetup`, the screens scenarios start from, one per widget, each started from its example but a fixture of its own |
| `tests/widgets_check/mod.rs` | `WidgetsCheck`, the checks about widget state the tester's built-in ones cannot say (`Selected`) |

## Unit and integration tests

- unit tests go in a `#[cfg(test)] mod tests` next to the code; `select.rs`'s
  `SelectionValue` tests are the reference for pure-logic coverage;
- integration tests go in `tests/`, driving a headless `App`;
- name tests as sentences: `a_stack_shows_only_the_selected_child`.

## Writing a headless integration test

`tests/headless.rs` builds an `App` with `MinimalPlugins` plus the plugins
`WidgetsPlugin`'s systems need to run without panicking on a missing
resource, none of which need a window or GPU:

- `WindowPlugin { primary_window: None, ..default() }` — registers
  `WindowEvent` and friends, which `bevy_picking` reads; no window is
  actually opened.
- `InputPlugin` — provides `ButtonInput<MouseButton>`, needed by
  `combo.rs`'s outside-click observer.
- `DefaultPickingPlugins` — provides `HoverMap`, needed by
  `scroll_area.rs`'s scroll handling.
- `VisibilityPlugin` — propagates `InheritedVisibility`, which `focus.rs`'s
  `sync_tab_index` reads; without it every tab stop counts as hidden.
- `ClearColor` (via `init_resource`) — normally inserted by the render
  plugins; `theme.rs`'s `update_theme` writes to it whenever `Theme`
  changes.
- `UiScale` (via `init_resource`) — normally inserted by `UiPlugin`; the
  `bevy_ui_widgets` `Scrollbar` of a scroll area reads it when it is pressed,
  so without it any press panics.

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

## Writing a scenario

A scenario drives the widgets with scripted pointer and keyboard input through
`bevy_stagecraft` (see its README for the `Step`, `Target` and `Check`
vocabulary) and asserts on what they did. Prefer it to hand-written pointer
event triggers (`PointerPress` and the like) whenever the claim is about what a
person would see after using the UI.

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
  `WidgetsCheck`; see [Scenario files](scenario-files.md)).
- **Frames.** Time is fixed at 1/60 s a frame. A pointer step (`move_to`,
  `press`, `release`) takes one frame and a `Step::check` none, so a check right
  after the step sees its effect, `MouseEvent` observers included; add
  `Step::wait(n)` only for something that needs more frames.
- **Touch.** The tester's touch steps (`TouchDown(at: T, finger: n)`,
  `TouchMove`, `TouchUp`, `TouchCancel`, `TouchTap(T)`, `TouchDrag(from, to,
  frames)`) write Bevy `TouchInput` messages and a `PointerInput` for
  `PointerId::Touch(finger)`, so the widgets see a touch pointer from the
  finger going down until it lifts. `TouchTap` takes two frames (down, then up
  on the next), because Bevy clicks only an entity the pointer hovered the
  frame before; a check right after it sees the result. `push_button_touch_tap.ron`
  (a `TouchTap` on "Add" counts one `Activate` in `Clicks`) is the smoke
  scenario that the steps reach the widgets; the touch epic (#793) adds the
  per-widget touch files. See the tester's README ("Touch").
  `every_interactive_gallery_group_has_a_touch_scenario` (`tests/scenarios.rs`) reads
  the gallery's `GROUPS` and fails when an interactive group has no `*_touch*.ron`
  file; a display-only group is listed in `GROUPS_WITHOUT_TOUCH` there.
- **Start from a screen.** `.app_setup(WidgetsSetup::Buttons)` (or `setup:
  App(Buttons)` in a file) instead of a `.setup(..)` closure, when a screen of
  `tests/widgets_setup/mod.rs` has the widgets the scenario needs. `mod
  widgets_setup;` next to `mod common;` pulls it in; it is its own module,
  not part of `common`, because `common` items an including target does not
  use fail `dead_code`.
- **Prove the assertion is real.** When you write a scenario, break the
  expectation once (a different theme slot, say) and see it fail before
  trusting it.

## Determinism

Since Bevy 0.20 many system sets (the UI's in `PostUpdate`) are ordered only
where systems' data access conflicts, so a system that runs at the right
moment only by luck would pass on one order and fail on another.
Generated tests in `tests/scenarios.rs` play every `tests/ron/` file once for
each of `PreUpdate`, `Update`, `PostUpdate` and `Last` and each of the fixed
seeds `[1, 2, 3, 0x5eed_cafe]` (never the clock), with that schedule built with
`ScheduleBuildSettings::shuffle_seed` and the single-threaded executor, so the
order is the seed's alone. The `debug` Bevy feature that provides the seed is
in `[dev-dependencies]` only.

- **Reproduce a failure** with the generated test filter, for example
  `cargo test --test scenarios scenario_combo_opens_list_postupdate_seed_5eedcafe
  -- --exact`. The test runs that file with `shuffle(&mut app, schedule,
  seed)` after `Registry::install`, so its name is enough to replay the failure.
- **Fix it by ordering** the system through the data it touches (`.after(..)`
  or a set). Never widen the seed list or skip the file.

## Recording the scenarios

`just record` plays every file of `tests/ron/` with the tester's offscreen
recorder and checks the results without saving recordings. `just record --save`
writes to `target/recordings/` (`RECORD_DIR` overrides it): per scenario PNGs
at the step boundaries and every tenth frame, `filmstrip.png`, `scenario.gif`,
`trace.html`, and `index.html` under `target/recordings/<scenario>/`. Open the
index and look; no picture is compared with a golden one. Each scenario is an
ignored, independently runnable test named `record_<scenario>` in
`recorded_scenarios_gpu`, with separate output directories for parallel runs.
It needs a GPU, or lavapipe (`mesa-vulkan-drivers libvulkan1` and
`WGPU_BACKEND=vulkan`), so `just test` and `just ci` never build it; lint it
with `cargo clippy --test recorded_scenarios_gpu --features record`.

- **A drawn recording is not the headless one.** The recorder's cameras render
  to an `Image`, and Bevy's UI focus (`Interaction`, `RelativeCursorPosition`)
  only follows cameras that render to a window, so a widget must not read it:
  the widgets read hover from picking, which works for any target (see ["Is the
  pointer on it?"](design/pointer-and-buttons.md#is-the-pointer-on-it-is-pickings-hovered)). Combos, palette combos and the hover colours of lists
  and palettes used to read `RelativeCursorPosition` and misbehaved in a
  recording (#100). Generated per-file tests in `tests/scenarios.rs` play each
  file headless with the camera aimed at an image, as the recorder does but
  without a GPU, so this is caught by `just test`. A failure names its file;
  there is no allowlist, so a
  scenario that only fails with the UI drawn to an image is a bug to fix, not a
  name to list.
- **CI.** The `record` job of `ci.yml` sets `RECORD_CHECK_ONLY=1` and runs
  the same scenario tests as `just record` on lavapipe. This uses
  `Recorder::check` to run the rendered checks without generating recordings.
  It is not part of `just ci`.
