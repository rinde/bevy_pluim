# Examples and the gallery

What each example shows, how examples build their trees with `bsn!`, and the scenario gallery.

When the behaviour described here changes, update this file in the same PR.

## Examples

Each is `just example NAME` (`cargo run --example NAME`) and needs a window.

| Example | Shows |
| --- | --- |
| `overview` | the one example of every widget that stands alone, in `examples/overview/` (`main.rs`, and `ui/` with `mod.rs`: the plugin, the tab list, the layout helpers `page` / `groups` / `row` / `variant`, plus a module per group of pages: buttons, text, lists, combos, colours, selection, navigation, values, toolbars, cards, scroll_area, feedback, theme_editor). 21 pages, among them `FilterCombo`, `LightDarkToggle` and the `ThemeEditor` (last, with "Change theme" and "Export", which logs the edited theme as Rust text through the page's `colors_source` / `pair_source` / `numbers_source` / `theme_source`; the library only edits), behind a vertical `TabBar` driving a `Stack` in a vertically scrolling `ScrollArea`; the `LightDarkToggle` above the tabs switches the theme. Every page has the same layout: a `Label::header`, then the variants in a centred, wrapping row, each a `variant(caption, content)` group with a `Label::secondary` naming what differs; the page node is named `Page` and as wide as the area, so pages do not shift when switching tabs. A new widget or variant goes on its page here, not in an example of its own. `Stack` is shown on the "Tabs and trails" page twice: `BreadCrumbs` driving panels with push, pop and replace-the-last-crumb buttons (`StackItem::new`, `new_with_reference` and `new_reference`), and a `TabBar` driving panels of palette combos, a card, a list beside a 100-item scroll area and a 300-card two-way grid; the "Combos" page has a list and combos sharing one `ZeroOrMore` selection with "Add List" / "Add Combo" / "Add PushButton" buttons. Left out on purpose: `Float`, `Icon`, `tailwind`, the crate-private modules and the `hud`. Icons are drawn at startup with `examples/common/mod.rs` (`mod common;`; a directory without a `main.rs` is not an example). `tests/headless.rs` includes `examples/overview/ui/mod.rs` with `#[path]` and shows every page (`the_overview_example_builds_and_every_page_runs`); `tests/scenarios.rs` does too to lay it out in the default 1280 x 720 window (`every_page_of_the_overview_is_centred_in_the_same_area_and_fits_the_window`) |
| `hud` | the redesign's HUD composed from the widgets over sample data, no game: pause button, `Kbd` and a `Slider` for the simulation speed (0.25x to 8x in octaves, written to the example's own `SimulationSpeed`), a panel `Toolbar` with badges driving a `Stack` of `Panel`s (trains with `FilterChips` and `Pill` rows, stops, warnings, preferences with `SettingRow`s, among them a "Text size" `NumericStepper` that sets `RemSize` and a "Company name" `TextField`; the hotkeys are off while a text field has the keys and on the frame it hands them back), and a tool dock whose active tool brings an option bar (`SegmentedControl`s, `NumericStepper`, vehicle `Gallery`). The "Show" buttons of the stops and warnings panels raise a `Toast`. Its icons are drawn at startup. The UI lives in `examples/hud/ui/mod.rs`, which `tests/headless.rs` includes with `#[path]` for a smoke test |
| `bench` | the headless CPU and memory benchmark of the widgets (`just bench`, an optimised build): per-frame time (mean, median, p95, max), startup time (spawn plus the first three frames, #735), entities and RSS for `push-buttons`, `tooltips`, `virtual-list` and the like, each beside a plain-Bevy baseline so the library's overhead is the difference. No allocation counter (it needs `unsafe`, which is forbidden). The `ticks-*` workloads compare `Changed<T>` with Bevy 0.20 `summary_tick` readers; summary ticks were measured and not adopted, and the headless widgets of 0.20 were reviewed (#656, recorded in `docs/benchmarks.md`). The `steppers-*` workloads cover `NumericStepper` value sync (#663). The `deep-trees-*` workloads cover hidden-state change collection (#684). The `list-items-horizontal-adding` and `list-items-grid-adding` workloads cover the `Horizontal` and `VerticalGrid` layouts at thousands of items (#766). The `indicators-*` workloads cover the sliding-indicator lifecycle (#677). The `palette-combos-idle` and `theme-editors-idle` workloads cover closed palette popups (#680). `wheel-deep` and `wheel-deep-idle` cover the wheel handler over a deep tree (#682). `tooltips-hovering` covers hover and popup cost with a fake pointer, `bevy-text-children` is a closer `push-buttons` baseline (a parent and a child per button), and an unknown `--workload` is an error listing the valid names (#704). `badges-idle`, `badges-changing`, `panels-idle` and `panels-changing` cover badge and panel synchronization (#685). `filter-chips-editing`, `toolbar-editing` and `gallery-editing` cover small structural edits (#690). `filter-chips-restyling` covers restyling chips with an unchanged font, which must not relayout (#710). `setting-rows-idle` and `setting-rows-changing` cover the `SettingRows` divider sync (#689). `sliders-idle` and `sliders-pressing` cover slider restyling (#698). `filter-chips-selection-changing` and `segmented-controls-selection-changing` cover a selection change restyling only its own widgets (#709). `tooltips-scrubbing` holds a `NumericStepper` scrub drag through public picking events so the crate-private `HoverLock` is on while the fake pointer hovers tooltip hosts (#756). Not part of `just ci`; see `docs/benchmarks.md` |
| `gallery` | the tester's gallery: the widgets in one window, and a **Scenarios** window listing every file of `tests/ron/` (one group per widget) to watch play, step, take over or record; see [The gallery](examples.md#the-gallery) |

**The examples build their trees with `bsn!`.** Every example but `gallery` (whose few nodes are plain spawns)
spawns its nodes as scenes (`bsn!` / `bsn_list!` and `spawn_scene`), with
`on(..)` for observers; a single entity may still be a plain `spawn`, as may
the `tests/widgets_setup` screens. **Never change a library component for
`bsn!`**: no `Default`, `Clone` or other derive added only so a scene can
patch it. Insert a prepared value instead: `~{PushButton::from_label("Add")}`
or `~{value}` from a variable. A template's output must be a component, which
the scene inserts together with the entity's `ChildOf` and its other
components. For the same reason a scene spawned under a parent outside a
parent scene is `commands.spawn(ChildOf(parent)).apply_scene(..)`, never
`spawn_scene(..)` followed by an insert of `ChildOf`. What does not fit `~{..}`
has a way of its own:

- `~{..}` takes only an `Unpin` value, and a `Handle` is not one, so a widget
  holding one has a `FromTemplate` and is written as a struct in the scene, a
  handle as `icon: {icon.clone()}` (a field value that is a method call or a cast
  goes in braces):
  `IconButton { icon: {icon.clone()}, name: "Pause", size: IconButtonSize::Regular }`
  (derived), `Panel { title: "Trains", close_icon: {HandleTemplate::Handle(close.clone())} }`
  and `NumericStepper { decrease_name: "Lower" }`, written by hand as
  `PanelTemplate` and `NumericStepperTemplate`: their defaults are those of
  `Panel::new("")` and `NumericStepper::default()`, which a derive would not give
  (a `format` function is `{level as ValueFormat}`). A new widget holding a handle
  gets one too.
- `Selection::builder()` and `Divider` have scene constructors, written with
  `@{..}`: `@{Selection::builder().one().scene_with_items(items)}` (the items are
  cloned for each spawn), `scene_with_items_ref(entity)`,
  `@{Divider::vertical_scene()}` and `horizontal_scene()`. The bundle forms stay
  for code that spawns with `Commands`.
- `Stack` derives `FromTemplate`, so a scene writes `Stack { selection_ref: #tabs }`
  with a `#name` of an entity in the same scene (the `tabs` and `breadcrumbs`
  examples put the selection on the `TabBar` / `BreadCrumbs` through `@{Selection::builder()..scene_with_items(..)}`);
  a selection the application also keeps (the `hud`'s resources) is spawned
  first and passed as `Stack { selection_ref: bar }`.

`the_hud_example_builds_and_runs` in `tests/headless.rs` adds `AssetPlugin` and
Bevy's `ScenePlugin`, which spawning a scene needs.

Every example has a way to toggle the light and dark theme (the gallery's
button is on its screens, the `hud`'s is the Appearance setting of its Preferences
panel, the `overview`'s a `LightDarkToggle` above its tabs), and all but `gallery` set `GlobalUiDebugOptions`, which is
why `bevy_ui_debug` is switched on in `[dev-dependencies]`.

## The gallery

`examples/gallery.rs` uses the tester's `gallery` feature (switched on for the
`[dev-dependencies]` entry in `Cargo.toml`, so it also builds with the tests).
It needs a display, and `just gallery-test` (`--run-all`) plays every scenario
once in a real window and exits non-zero naming the failures; it is not part of
`just ci`. Words after `--run-all` keep only the files whose name contains one
of them (`just example gallery -- --run-all stack link`).

- **The tests' enums are shared, not copied.** The example pulls
  `tests/common/mod.rs`, `tests/widgets_setup/mod.rs` and
  `tests/widgets_check/mod.rs` in with `#[path = "../tests/.."] mod ..;`, so the
  scenario files run against the same `WidgetsSetup` and `WidgetsCheck`, and the
  library depends on nothing of the tester.
- **The catalogue is `tests/ron/`**, read when the example starts, grouped by
  the start of the file name (`GROUPS` in the example; a new widget's files need
  a row there or land in "Selection"). A file that does not load is skipped
  with a warning.
- **The reset hook** (`GalleryPlugin::with_reset`) despawns every `UiRoot`, every
  `Camera2d` and every `Selection` and `ItemOf` entity that is not
  `NotUnderTest` (setups spawn selections beside the UI, not under it), and puts
  `Theme` back; the scenario's `WidgetsSetup` builds the screen again. Leave
  the selections out and stale ones make later scenarios fail.
- **The window has the display's scale factor.** The files state logical
  values (see [Scenario files](scenario-files.md)), so they pass at any scale factor.
- **`AppSetup::capture`** returns the `WidgetsSetup` the app was last built from
  (kept in the `CurrentScreen` resource), so **Record** and **New: current
  state** write `setup: App(Lists)` and the file loads back. What was done to
  the screen before recording started is not captured. There is no `AppView`:
  the widgets have no view to name.
