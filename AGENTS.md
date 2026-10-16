# Working on rin-bevy-ui

Guidance for coding agents. This file explains the codebase to whoever is
changing it. If it and the code disagree, the code wins and this file gets
fixed.

## What this is

A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons,
lists, combos, scroll areas, tabs, breadcrumbs, colour palettes, a slider and a
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
| `coordinator` | Sonnet | Decides what is ready, keeps the slots, dispatches implementers and reviewers into them, checks on GitHub that each did its job, merges, and turns what is left undone (review nits, questions on an issue, anything a pull request left open) into issues it refines and labels `ready-for-agent`, or `require-human-review` where a human must decide first. Writes no code. |
| `implementer-sonnet`, `implementer-opus` | Sonnet, Opus | Implements one issue in the issue's slot on a branch `RINUI-<number>/<slug>`, opens a pull request that `Closes #N`, and follows it up on review, red CI or a merge conflict. Runs the `implement-issue` skill. |
| `reviewer-sonnet`, `reviewer-opus` | Sonnet, Opus | Reviews one pull request in the issue's slot, makes sure CI passed on its head, and posts one review that approves or requests changes, with inline comments. Runs the `review-pr` skill. |

The variants differ only in their model; the instructions are the skills, which
also run by hand: `/implement-issue 24`, `/review-pr 25 opus`.

| Label | Meaning |
| --- | --- |
| `ready-for-agent` | The coordinator may start the issue. Agents leave every other issue alone. On a pull request, a coordinator run that wound down left it there, and a later run may pick the pull request up; without it, an open pull request may be someone else's. |
| `opus` | The Opus implementer, and an Opus approval. Without it, Sonnet. |
| `require-human-review` | A human approves and merges; no agent approves or merges it. On a follow-up the coordinator filed without `ready-for-agent`, a human must decide something before an agent can take it. |
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
A question an implementer puts on an issue is not left for a human: the
coordinator decides it, comments the decision and puts `ready-for-agent` back.
Everything else agents leave behind (a pull request's "anything left open",
untaken nits, flakes) becomes an issue once the pull request merges, labelled
`ready-for-agent`, or `require-human-review` alone when a human must decide
something first. When a run winds down with a pull request of its own still
waiting for CI or a slot, it labels that pull request `ready-for-agent` so
the next run may pick it up; that is the only time a pull request gets the
label.

| How | What |
| --- | --- |
| `/coordinate` | Passes until nothing is left to do, then a report. One slot, so one issue in flight at a time, in a cloud session (`CLAUDE_CODE_REMOTE=true`), two on a local machine. |
| `/coordinate #24 #25` | Only those issues and their pull requests. |
| `/coordinate max 5` | Five slots, so five issues in flight at once, instead of the default. |
| `/coordinate dry-run` | Report what it would do and change nothing. |
| `/loop 30m /coordinate` | Keep coordinating every half hour. |
| `claude --agent coordinator` | The coordinator as the session's main agent. |

Agents talk to GitHub through `gh api` and its REST endpoints, because cloud
sessions block the GraphQL API that `gh issue` and `gh pr` use. An unattended
run needs its tools allowed, or it stops at the first permission prompt.

**Slots.** Every agent works in a slot, a worktree at
`.claude/worktrees/slot-<n>` that the coordinator makes and keeps, numbered
from `slot-1` up to the limit, which is the number of issues in flight at
once. A slot is bound to one issue from its first implementer until the
issue's work is done, usually the merge: the implementer, the reviewer, the
follow-up and the reviewer again all run in that one slot, one at a time,
each building on the `target/` the last one left, and no other issue uses
it meanwhile. The binding is `.claude/worktrees/slot-<n>.issue`, next to
the worktree, so it survives between runs. When the issue is done the
coordinator resets the slot to `origin/main` and frees it; the `target/`
stays, so the next issue starts warm. Agents enter their slot with Claude
Code's `EnterWorktree` tool, given the slot's path. Slots are never removed;
the coordinator checks the free disk space at the start of each run and,
below 30 GiB, sweeps stale files out of free slots' `target/` folders,
oldest first, never a bound slot's.

A slot's `target/` is seeded once, when the slot is new, from the main
checkout's with a copy-on-write clone (`cp -c -R` on macOS, which falls back
to a real copy where it cannot clone, so the skills compare `df` before and
after and remove `./target` if it grew; `cp -R --reflink=always` on Linux,
which refuses), then `touch tests/*.rs src/lib.rs`. Never share one
`CARGO_TARGET_DIR` between worktrees (stale artifacts, and `ron_scenarios`
reads `tests/ron` through a path compiled into its binary, so it can play
another worktree's files) and never `cp -r` the 19G `target/` (it fills the
disk). The skills check `df -h .` first. What slots share is sccache's cache,
not a `target/`: the `justfile` compiles through sccache when it is installed
(see [Running things](#running-things)).

## Where to find what

| Module | Holds |
| --- | --- |
| `lib.rs` | `WidgetsPlugin` (add this one plugin to an app), `UiRoot`, and `MouseEvent` with the observers that produce it |
| `theme.rs` | `Theme` resource (colours, `body_font` and `display_font` handles defaulting to Bevy's font with the application loading its own, the `font_size_*` type scale, in logical pixels at Bevy's default rem size of 20 px, which every widget turns into `FontSize::Rem` with `Theme::font_size(px)` (its own constant sizes too), so the text follows Bevy's `RemSize`: an app that raises `RemSize` scales the widgets' text, at the default nothing moves; sizes tied to text stay `px` unless a `[text 1.5x]` scenario shows them clipping (only the badge is in rem), `control_radius` for buttons, list items, steppers and scroll bars and `panel_radius` for `Card` and `Float`), `ThemeColors` (also the text, divider, accent, danger, warning, info, pill, segmented, card and chip slots of the redesign, and `slider_track` / `slider_fill` for a `Slider`; both defaults use a dark `large_shadow` and `small_shadow`, black, so neither theme glows; both meet WCAG AA contrast (4.5:1 for their text and ink pairs, 3:1 for `focus_ring` and `selection_ring`), guarded by `the_default_themes_meet_wcag_aa_contrast`, which exempts `text_disabled`, `text_tertiary` and the decorative borders; the shadow constants set colour only, offsets stay with the widgets; `Pill` and `Badge` share `pill_background` / `pill_text`), `DEFAULT_LIGHT_THEME_COLORS` / `DEFAULT_DARK_THEME_COLORS`, `ThemeMode`, `Theme::with_colors(light, dark)` for an app's own pair, `set_mode` / `mode` / `light_dark_toggle` (switch `colors`, the set in use; do not assign `colors` by hand), the shadow constants; also keeps `ClearColor` in step with the theme |
| `tailwind.rs` | `COLORS`, all 242 Tailwind colours as `TailwindColor { class, variant, color }`, and `color_name(color)` (Tailwind name, else hex: the accessible name of swatches and colour buttons, and what `WidgetsCheck::Selected` uses), for colour pickers; the theme itself uses `bevy::color::palettes::tailwind` directly |
| `select.rs` | the selection model: `Selection` (`Value` or `Ref`), `SelectionValue` (`One` / `ZeroOrOne` / `OneOrMore` / `ZeroOrMore`), the `Selection::builder()`, `Item<T>`, `ItemOf` / `ItemValues` / `ItemsRef`, `ItemsChanged`, and the `SelectionParam` / `ItemParam` system params |
| `stack.rs` | `Stack { selection_ref }` shows the one `StackItem` child matching the selection; children are created lazily by a registered system (`StackChildSpawner`, whose `spawn` takes a bundle and `spawn_scene` a scene, the stack child being the scene's root) and hidden, not despawned, when deselected |
| `breadcrumbs.rs` | `BreadCrumbs`: a `Link` trail for a `OneOrMore` selection; clicking a crumb trims the selection back to it |
| `combo.rs` | `Combo`: a button that opens a `List` in a `Float`, `ComboState` (open/closed), closes on an outside click. While open, `place_open_lists` (`PostUpdate`, before layout) puts the list right under the button, left edges aligned, so it follows the button when a scroll area around it scrolls; the list is kept 10 px inside the window's left and right edges and goes above the button when it would run off the bottom, like a palette combo's popup (`combo_keeps_list_inside_window.ron`) To a screen reader the `ComboButton` is a `Role::ComboBox` without a label, its value the text of the selected items joined with ", " (empty for none; not the placeholder or the chevron), `expanded` while open and `controls` its `ComboList` (`combo_accessible_combo_box.ron`) |
| `float.rs` | `Float`: requires Bevy's `FixedNode`, so it is a child of its widget but placed by its `left` / `top` in the viewport (logical pixels) and takes no layout, clipping or transform from its ancestors: a `ScrollArea` around the widget does not clip it. `GlobalZIndex(1000)`, large shadow; the popup layer of `Combo`, `ColorPaletteCombo` and `Tooltip`, each of which places its own, `Combo` and `ColorPaletteCombo` through the crate-private `keep_in_window_x` and `below_or_above` of `float.rs` (10 px `WINDOW_MARGIN` from the window's side edges, above the button when it would run off the bottom) (`combo_in_scroll_area_popup_is_not_clipped.ron`, `color_palette_combo_in_scroll_area_popup_is_not_clipped.ron`) |
| `scroll_area.rs` | `ScrollArea::bundle` (for code that spawns through `Commands`) and `ScrollArea::scene(contents, vertical, horizontal)`, the same area as a scene whose contents are a `SceneList` (`@ScrollArea::scene(bsn_list! { .. }, true, false)` in a `bsn!`; it inserts the bundle after the scene is applied), `ScrollContents`, `ScrollBar` / `Orientation` and the thumb, themed here; each bar is also a `bevy_ui_widgets` `Scrollbar` aimed at the contents and its thumb a `ScrollbarThumb` (no `Node`), so the widget sizes and places the thumb, scrolls the contents when it is dragged and pages them on a click on the track, while `show_scroll_bars` hides a bar whose axis fits; and the crate-private `update_scroll_position`, which scrolls any `ScrollPosition` under the pointer with the wheel. Put a `List` or palette in one to scroll it. `ScrollContents` has 3 px of padding on every side (`FOCUS_RING_REACH`) so a focus ring at the edge is not clipped, and `scroll_into_view` counts it as part of the entity. The area is a `Role::ScrollView`; its bars and thumb have no accessibility node (`scroll_area_accessible_scroll_view.ron`) |
| `list.rs` | `List` with `ListLayout` and `CornerStyle` (`Separated` is the panel style: 6 px padding, 2 px gaps, rows rounded on all corners, the selected row with a 1 px `selection_ring` border), a plain grid that does not scroll: a `ListBox` that renders the items of its `Selection` as `ItemView`s (`ListItem`s) and keeps them in sync, `Selected` included; while focused it keeps its arrow, Home and End keys from bubbling on (`keep_navigation_keys`), and lets those of a focused descendant (a button in a row) past (`pass_first_presses_from_descendants`, see "Directional navigation is the app's") `ItemNote { text, tone }` (`NoteTone::{Secondary, Warning}`; `ItemNote::new` / `::warning`) on an `Item<String>` entity adds a note after the label of its default row, in the caption size and `text_secondary` / `warning_text`: a `Combo`'s popup shows it, its button shows the label only. The row's accessible name is the label followed by the note (`Delft < unreachable >`, kept by `sync_item_labels`); changing, inserting or removing a note, or giving it empty text, updates the row in place (`combo_option_notes.ron`). A custom view (`ItemViewCreator`) does not get it. |
| `virtual_list.rs` | `VirtualList { row_height }`: a list of rows of one height for many thousands of items (rails' 10,000 trains) that renders only those in view. Its node is as tall as all rows, so the `ScrollArea` around it (the nearest ancestor whose `overflow.y` scrolls, else the window) scrolls the whole list; under it is a pool of absolutely placed `VirtualListRow { list, index, item }`s (immutable, `ListItem`s): the rows in view, one for a part row and `OVERSCAN_ROWS` (2) beyond each edge. When `ScrollPosition` moves, `show_rows` (`PostUpdate`, before layout) hands rows on to the items now in view (item `i` goes to row `i % pool`, so a scroll by one row re-points one row); a row is never spawned or despawned for a scroll, and the pool only grows, when the view does. A row shows the text of an `Item<String>` in an `ItemLabel`, or with `List`'s `ItemViewCreator` the app fills it: the creator runs again, on the row emptied of its children, when the row is handed to another item (or the app observes the `VirtualListRow` insert). Selection is as for a `List` (`Selected` on the rows of selected items, a click toggles as far as the selection allows) but it is not a `ListBox`, whose observers would only see the pooled rows: it is one tab stop with its own keyboard observer (arrows move the active row, wrapping, Page Up / Page Down by the rows that fit in the view, stopping at the ends, a first Page Down with no active row landing on the first item as Arrow Down does, Home / End, Enter / Space select; each scrolls the active row into view by setting the area's `ScrollPosition`), and `ActiveDescendant` names the row of the active item while it is in the pool; `show_rows` runs before `focus.rs`'s crate-private `DrawFocusRing` set, so the focus ring is on the row the frame it moves, and `activate_on_focus` runs before it as a system that reads `InputFocus` (set in `PreUpdate`), not as an observer of `FocusGained`, which Bevy sends after the layout, so a list focused by Tab or a click names and rings its active row the same frame (`virtual_list_keyboard_pages.ron`). A `Role::ListBox` whose `size_of_set` is the number of items, each shown row a list item with `position_in_set` (its item's index plus one) and the same `size_of_set`, kept by `sync_accessibility` in `AccessibilitySync`, so a pooled row reads as item 5,001 of 10,000. `List` is untouched |
| `color_palette.rs` | `ColorPalette`: a grid of colour swatches (`ListItem`s, `Selected` when in the selection) over a selection of `Item<Color>`; a swatch's accessible name (label of its `AccessibilityNode`, kept up to date) is the name of its colour, `tailwind::color_name`: the Tailwind name (`blue-500`) when it equals one of `COLORS`, else its lowercase sRGB hex (`#3b82f6`, alpha appended only when not 1) |
| `color_palette_combo.rs` | `ColorPaletteCombo`: a `ColorPalette` inside a combo-style popup; the popup, a `Float` child of the combo, is centred under the button when it opens, 10 px inside the window, above the button when it would run off the bottom; its palette gets `Selection::Ref` to the combo's selection when it is spawned, and `SubRefs` keeps its entity. Its button is a `Role::ComboBox` like a `Combo`'s, without a label, its value the colour name of the selection (empty for none), `controls` the popup's palette |
| `color_push_button.rs` | `ColorPushButton`: a button showing a colour square; its accessible name is the name of that colour (same rule as a swatch), kept by a system because Bevy clears a text-less button's label on add; a `ColorPaletteCombo`'s button is not, its colour is its value instead |
| `numeric_stepper.rs` | `NumericStepper`, its `Value`, `UIntValue<T>`, `ValueChange`; its "-" and "+" are small `IconButton`s (accessible names `decrease_name` / `increase_name`, "Decrease" / "Increase" by default; icons `decrease_icon` / `increase_icon`, a drawn minus and plus when left at the default handle) that step on `Activate`; `format` (`fn(&Value) -> String`, default `Display`) says how the value is shown, in a label of at least `label_min_width` (54 px) so the buttons do not move; fields are read when the component is inserted, and one that cannot step (at the minimum or maximum, or a step short of it) gets `InteractionDisabled`; the arrow keys step it too (up / right increase, down / left decrease) through an observer of `FocusedInput<KeyboardInput>` on the stepper, which a focused button's keys bubble up to and which stops them there (pressed, repeated or released, also at a limit), so only the buttons are tab stops and an app's directional navigation never sees them. The stepper's own node is one spin button: an `AccessibilityNode` with `Role::SpinButton`, `numeric_value` / `min_numeric_value` / `max_numeric_value` / `numeric_value_step` from the `Value` (as `f64`), `value` the text `format` shows (kept in step by `on_change`, also for mirrors through `ValueRef`) and `name: Option<String>` as its label (none by default); the value text is a plain `Text`, not a `Label`, so it has no node of its own |
| `slider.rs` | `Slider::new(value, min, max)` with `with_step`, `with_format` (`SliderFormat`, `fn(f32) -> String`), `with_name` and `with_label_min_width`: a horizontal range control over Bevy's headless `bevy::ui_widgets::Slider` (`SliderValue`, `SliderRange`, `SliderStep`, `TrackClick::Snap`), 200 px wide unless the app gives its `Node` a width, 20 px high: a 4 px track in `slider_track`, a fill up to the value in `slider_fill` and a 16 px thumb of control radius (`card_surface` with a 2 px `slider_fill` border and the small shadow; `accent_subtle` hovered, `accent` pressed or dragged) that stays inside the slider at both ends (it moves in a `SliderTravel` node one thumb narrower, as the headless pointer maths assume). Its colours come from one function of disabled, pressed or dragged, and hovered (`look`); `InteractionDisabled` draws it in `text_disabled`, and the headless slider then ignores the pointer and keys. With a `format` its value is shown in a `SliderLabel` of at least `label_min_width` (54 px) in a right margin the slider reserves (a slider with a `format` owns its `margin.right`, set when the `Slider` is inserted; one without leaves it to the app), absolutely placed so that the track does not move as the text changes, secondary text in rem. A press on the track jumps the thumb there and a drag moves it, both through the headless slider's `ValueChange<f32>`, which a global observer snaps to the `step` (from `min`, clamped) before the app's observers on the slider see it, then inserts as the `SliderValue`. One tab stop; while focused left / right move it by `step` (a hundredth of the range without one), Home / End to its ends (Bevy's), and `keep_navigation_keys` stops those keys from bubbling on (pressed, repeated or released); up and down go through. The focus ring is the `TabStop`'s. To a screen reader it is a `Role::Slider` (Bevy's numbers: value, bounds, step; `a_slider_is_a_slider_node_with_its_value_bounds_and_step`) whose `value` is the text `format` shows (none without one) and whose label is `name` (none by default). Fields are read when the component is inserted; an app sets the value by inserting a `SliderValue`. `SliderRefs` names the parts |
| `panel.rs` | `Panel`: a `Card` 380 px wide with a header (title `Label`, optional subtitle beside it, optional close `IconButton` that triggers the `PanelClosed` entity event on the panel), a strip, a `Divider`, a body in a `ScrollArea` that grows with its content up to `max_body_height` (620) and then scrolls, and a footer above a divider. The app spawns contents as children of the entities in `PanelRefs` (`strip`, `body`, `footer`); an empty strip or footer is `Display::None`. Insert a `Panel` again to change title, subtitle, close icon or maximum height. The body is `should_block_lower: false`, or it would cover the scroll contents and the wheel would never reach them. Grow-then-scroll needs only `max_height` on the `ScrollArea` node (`a_panel_body_grows_with_its_content_up_to_its_maximum_and_then_scrolls`). It is a `Role::Group` (from `Card`) with no label of its own that is `labelled_by` its title `Label` (so a new `Panel` needs no relabelling); the close button keeps its own name (`panel_accessible_group.ron`) |
| `push_button.rs` | `PushButton` with label and optional icon; **the reference widget to copy**. `ButtonSize::Compact` (insert it with the button or later) makes it 28 px high with 10 px side padding, a 6 px radius, a semibold 12 px label and no shadow (`Theme::compact_button_*`, `font_size_small`); `Regular` is the default  Its accessible label is its text, not the icon (a combo's button is a combo box and has none). `ButtonNames` is the scenario screen for button labels. |
| `label.rs` | `Label` with `Title` (`Label::title`, also `Label::header`), `Regular`, `Secondary`, `Caption` and `Overline` (shown in upper case, no letter spacing), sized and coloured from the theme's type scale and text slots; it requires Bevy's `ui::widget::Label`, so it is a `Role::Label` accessibility node with its text as value, and the hook inserts that marker again so a new `Label` updates the value |
| `focus.rs` | a crate-private module (its `FocusPlugin` is added by `WidgetsPlugin`): the crate-private `TabStop` marker (requires `TabIndex`), `sync_tab_index` and the focus ring (`draw_focus_ring`, in the crate-private `DrawFocusRing` set in `PostUpdate` before the layout), see "Keyboard focus" below. Also the crate-private `AccessibilitySync` set (`PostUpdate`, after `UiSystems::PostLayout`, where Bevy's `button_changed` sets `Role::Button` when a `Button` changes, and before `AccessibilitySystems::Update`) with its `sync_role` / `sync_toggled` / `sync_label` / `sync_value` / `clear_label` / `sync_controls` / `sync_labelled_by` helpers (`node_id` turns an entity into the `accesskit::NodeId` Bevy gives it), which write a node only when it differs: widgets set their roles, toggled states and labels there, so Bevy does not undo them. A container group without a title of its own (`Card`, a toolbar or radio group) has no label; `SettingRow` is labelled with its title and `Panel` is `labelled_by` its title `Label`; an app can insert Bevy's `AccessibleLabel` on it |
| `link.rs` | `Link` and its immutable `LinkState`; a `Role::Link` to a screen reader (it keeps `Button`, so `Role(role: Button, ..)` still finds it; `link_accessible_role.ron`). Its accessible label is its text. |
| `card.rs` | `Card`: a themed background, border, radius and shadow on a node; a `Role::Group` without a label (`themed_card_accessible_group.ron`) |
| `segmented_control.rs` | `SegmentedControl`: a track of text `Segment`s (`Button`s and tab stops) over a `Selection` of `Item<String>`, own or `Selection::Ref`, normally a `One`; a segment's `Activate` sets the selection to its item (`set_selection`), `Selected` and the `segmented_*` theme colours follow the selection and hover. It is its own widget, not a `List` style, so `List` is untouched. To a screen reader the control is a `Role::RadioGroup` and each segment a `Role::RadioButton`, `toggled` following `Selected` (`segmented_control_accessible_radio_group.ron`); the segments keep `Button`, so `Role(role: Button, ..)` still finds them |
| `tab_bar.rs` | `TabBar`: Bevy's `bevy_ui_widgets` `TabList` of text `TabBarTab`s (Bevy `Tab`s) over a `Selection` of `Item<String>`, own or `Selection::Ref`, normally a `One`, styled from the `segmented_*` slots like a `SegmentedControl`. `SelectedTab` follows the `Selection`, which keeps the last word: Bevy's `tablist_self_update` is not registered, and the `ValueChange<Option<Entity>>` the tab list sends for a click, Enter or Space goes through `SelectionParam::update_selection` (`set_selection`). The tabs are not `Button`s (a `Button` stops the click before the tab list sees it) nor `TabStop`s: Bevy's `TabPlugin` (added by `WidgetsPlugin` unless the app has it) gives the selected or focused tab `TabIndex` 0 and the others -1, so the bar is one tab stop and the arrows, Home and End move the focus between its tabs. A hidden bar (itself or an ancestor `Display::None`, as in a hidden `Stack` panel, or hidden visibility) gives all its tabs -1, so Tab skips it, and the roving 0 comes back when it is shown: `sync_tab_index` in `tab_bar.rs` runs after Bevy's and after visibility propagation and keeps both, with `focus.rs`'s `is_hidden` (`a_hidden_tab_bar_is_not_reached_by_tab_and_is_again_once_shown`, `tab_skips_a_tab_bar_in_a_hidden_stack_panel`). To a screen reader the bar is a `Role::TabList` and each tab a `Role::Tab` labelled with its item's text, `selected` while it is the `SelectedTab` (`tab_bar_accessible_tab_list.ron`) |
| `setting_row.rs` | `SettingRow::new(title, description)`: a title (semibold, `font_size_body`) over a secondary description on the left and an empty control slot on the right (12 px gap and vertical padding), then a `Divider`; `SettingRowRefs { title_id, description_id, control_slot, divider_id }` names them, and the app spawns its control (a `SegmentedControl`, say) as a child of `control_slot`. Put the rows in a `SettingRows` column: a system hides the divider of the last row and shows the others'. A row is a `Role::Group` labelled with its title, its description the node's `description` (`setting_row_accessible_group.ron`) |
| `divider.rs` | `Divider::horizontal()` / `Divider::vertical()`: a 1 px rule in `ThemeColors::divider`, stretching along the other axis unless the caller sets a length |
| `icon.rs` | `Icon::bundle(handle, size)` / `Icon::bundle_with_color`: an `ImageNode` from an app-provided white-on-transparent `Handle<Image>`, tinted with the `IconColor` theme role of its `Icon` (primary text by default); a widget changes the role by inserting `Icon` again |
| `icon_button.rs` | `IconButton::new(handle, name, size)` with `IconButtonSize::{Regular, Small, Large}` (44 px / 22 px icon, 30 px / 16 px icon, 54 px / 24 px icon): a borderless `Button` of control radius with an `Icon` child (`IconButtonRefs`); `name` is the label of its `AccessibilityNode`, kept there by a system because Bevy clears a text-less button's label on add; inserting `IconButton` again swaps icon, name or size; small icons are secondary text until hovered; an app observes `Activate`; an optional immutable `ToggleState(bool)` makes it an on/off toggle: `Activate` inserts the flipped value (an app can insert one too), and an on button shows `accent_subtle`, an `AccentSubtleText` icon and a 2 px `accent_subtle_bar` child along the bottom edge, inset by the control radius to stay on the straight part (spawned with the first `ToggleState`). `IconButton::with_tone(Tone)` (`Neutral` default, `Warning`, `Danger`; also the tone of a `Tool`): `Warning` has `warning_background`, a 1 px `warning_border`, a `WarningText` icon and the large shadow, mixes 25 % of the border colour into the background when hovered or on, and when on shows a 3 px `warning_border` bar (a 54 px warning button with a `Badge` is the redesign's warnings button); `Danger` turns the icon `danger_hover_text` when hovered and is filled with `danger` and `DangerText` when on, without a bar. With a `ToggleState` its accessibility node is `toggled` with it; without one it has no `toggled` at all (`icon_button_accessible_toggle.ron`) |
| `kbd.rs` | `Kbd::new("Esc")`: a display-only key cap, a bordered box (`ThemeColors::border`, `text_secondary` text in the caption size; it reuses those slots and has none of its own); a `Role::Label` accessibility node with the key as value |
| `badge.rs` | `Badge { text, tone }` (`BadgeTone::Neutral` or `Warning`; `Badge::count(n)`): a 16 px high pill (min width 16, 4 px side padding, 10 px semibold text; height, min width and radius are `Val::Rem`, so the pill grows with its text when `RemSize` grows, which in pixels clipped the count) spawned as an absolute, pointer-ignoring child in the top-right corner of its host, on an `IconButton` or on the `Item<Tool>` of a `Toolbar` (mirrored onto that item's button). Neutral uses `pill_background` / `pill_text`, warning `warning_text` as fill with `warning_background` text. Insert a new `Badge` to change it; remove it, or give it empty text, to hide it; the badge node is not in the accessibility tree, its text is the `description` of the host's `AccessibilityNode` (cleared when the badge is removed or empty) |
| `pill.rs` | `Pill::new(text)` with a `PillTone` (`Neutral`, `Info`, `Warning` = waiting, `Problem`): a small uppercase status tag; the app picks the tone; re-insert a `Pill` to change tone or text; a `Role::Status` accessibility node labelled with the text as given |
| `filter_chips.rs` | `FilterChips` over a `One` `Selection` (own or `Selection::Ref`) of `Item<Chip>` (`Chip { label, count: Option<usize> }`): a wrapping row (6 px gap) of 30 px pill `ChipView`s, each a `Button` and tab stop that picks its item on `Activate`; the active chip is filled with `chip_on` / `chip_on_ink`, the others outlined in `chip_border` with `text_secondary` text (`text_primary` on hover) and the count at 0.6 alpha. Changing the `Chip` in an `Item` updates its chip's text in place; adding, removing or reordering items respawns the chips. Text size is `Theme::font_size_chip`. Styled by `theme_chips` (new chip, `Theme` change, hover, selection or item change), not every frame; no arrow-key navigation between chips, each is a tab stop. To a screen reader the row is a `Role::RadioGroup` and each chip a `Role::RadioButton` labelled with its label and count (`Waiting 3`), toggled while active (`filter_chips_accessible_radio_group.ron`) |
| `gallery.rs` | `Gallery` over a `One` `Selection` (own or `Selection::Ref`) of `Item<GalleryCard>` (`GalleryCard { image, label, group }`): neighbouring items with the same `group` form a group, an overline `Label` over a row of 104 x 92 px `GalleryCardView`s (6 px gap; each a `Button` and tab stop that picks its item on `Activate` with `toggle_selected`), groups side by side with a vertical `Divider` between them. A card is a `card_surface` with a 1 px `border` and control radius, a 72 x 30 `Icon` image above a 12 px semibold label that wraps; hover is `hovered_item`; the picked card has a 2 px `selection_ring` border and the `hovered_selected_item` background. The gallery does not shrink, so put it in a horizontal `ScrollArea` to scroll it. Changing the items or their groups respawns the groups; changing a `GalleryCard` updates in place. To a screen reader the gallery is a `Role::RadioGroup` and each card a `Role::RadioButton` labelled with its item's label, toggled while picked |
| `toolbar.rs` | `Toolbar { style }` over a `Selection` (own or `Selection::Ref`) of `Item<Tool>` (`Tool { icon, label, group, key_hint, tone }`): a row of `ToolbarButton`s (`Button`s and tab stops) that pick their item on `Activate` with `toggle_selected`, so a `One` toolbar keeps its tool and a `ZeroOrOne` one clears when the open one is activated; a vertical `Divider` goes between neighbouring tools of different `group`. `ToolbarStyle::Filled` is 66 x 60 px with a label under a 22 px icon, the active button filled with `accent` and `AccentText` ink; `Underlined` is 44 px icon-only, the active one `accent_subtle` with an `AccentSubtleText` icon and an `accent_subtle_bar` bar. A tool's `tone` (`Tool::with_tone`) recolours it: `Danger` turns icon and label `danger_hover_text` when hovered and fills the active button with `danger` and `DangerText` (a bulldoze tool); `Warning` shows `WarningText` on `warning_background` when hovered or active. The tool's label is the `AccessibilityNode` label in both. `ToolbarLabels` (`Shown` default, `Hidden`; immutable, required by `Toolbar`) hides the labels of a `Filled` toolbar: the label gets `Display::None` and the buttons shrink to 56 x 52 px, without respawning; `Underlined` is unaffected. Changing the items, their groups or the style respawns the buttons; changing a `Tool` updates in place. A tool with a `key_hint` (`Tool::with_key_hint("1")`) gets an absolutely placed hint text (10 px semibold, 5 px from the top, 7 px from the right) in the button's ink at 60 % opacity, so it follows the active state; the library only shows it, the app binds the key. To a screen reader the toolbar is a `Role::Toolbar`; its buttons stay `Role::Button`, `toggled` true while their tool is active and false otherwise |
| `tooltip.rs` | `Tooltip { text }`: after the pointer (`Hovered`) rests on the node for `TOOLTIP_DELAY_SECS` (0.5 s) a `TooltipPopup` `Float` (`card_surface`, `border`, small `text_primary`) is spawned as a child of the node, hidden until laid out, then centred under the node, kept 6 px inside the primary window and put above the node when it would run off the bottom; it never takes the pointer. Leaving the node or a `Pressed` hides it, and a press keeps it hidden until the pointer has left and returned. `IconButton` inserts one with its name; the buttons of an underlined `Toolbar` (labels hidden) get one with their tool's label |
| `toast.rs` | `Toast::new(text)` (`with_duration`, 3 s default): a `Card` with the text, spawned as a child of the UI root (`ChildOf(root)`), which moves into a `ToastStack` (an absolute, pointer-ignoring column along the bottom, centred, `GlobalZIndex(1100)`, created by the first toast as a child of that toast's parent) where several stack upwards, newest at the bottom; a new toast with the text of one showing is despawned and restarts that one's timer, and re-inserting a `Toast` on a live entity restarts its own. `ToastPlacement { bottom }` (resource, 24 px) is the stack's distance from the bottom edge, for an app to lift above its dock. A system ticks a private timer with `Time` and despawns the toast when it runs out. `Pickable::IGNORE`, no tab stop; a `Role::Status` node (polite live region) labelled with the text |
| `text_field.rs` | `TextField::new(value)` with `with_placeholder`, `with_name`, `with_min_width` / `with_max_width` (160 / 320 px by default): a single-line text input over Bevy's `TextInput` / `EditableText`. The field is a bordered box (1 px `border`, `card_surface`, control radius, 10 / 7 px padding) holding a `TextInput` child (`TextFieldRefs::input`, a tab stop, body text in `text_primary`, no wrap, so the text scrolls inside the box instead of growing it) and an absolutely placed placeholder (`TextFieldRefs::placeholder`, `text_tertiary`, shown while the text is empty). Bevy's `PointerFocusPlugin` gives the input the focus on a click and takes it on a press elsewhere, and an observer of `AcquireFocus` on the box gives it to the input for a click on the box's border too (`text_field_border_click_focuses.ron`); `Enter` gives it up, `Escape` puts back the text from when it took the focus and gives it up (both by key code, so the tester's `Tap` works; Bevy's own `Escape` reads the logical key). Entity events on the field: `TextFieldChange { value }` on every edit (and an `Escape` restore), `TextFieldCommit { value }` when the focus leaves other than by `Escape` (Enter, Tab, a press elsewhere), changed or not. The focus ring is on the box, after a click too (`RingOn` / `RingTarget` in `focus.rs`). Insert a new `TextField` to set the text (no change event), placeholder, name or widths. To a screen reader the input is a `Role::TextInput` with its text as value and `name` as label (`text_field_accessible_text_input.ron`). It needs `TextInputPlugin` and `PointerFocusPlugin` from `UiWidgetsPlugins` (in `DefaultPlugins` and the tester's `HeadlessPlugins`); `WidgetsPlugin` does not add them |

### Examples

Each is `just example NAME` (`cargo run --example NAME`) and needs a window.

| Example | Shows |
| --- | --- |
| `widgets` | a showcase: a scroll area, a list and combos sharing one selection via `Selection::reference`, and buttons that add more |
| `breadcrumbs` | `BreadCrumbs` and a `Stack` sharing one selection, with reused `StackItem` creators |
| `tabs` | a `TabBar` driving a `Stack`, with combos, colour palettes and cards inside |
| `color_palette` | `ColorPalette` and `ColorPaletteCombo` over `tailwind::COLORS` |
| `combos` | `Combo` variants, one with noted options (`ItemNote`) |
| `lists` | `List` layouts and corner styles, and a `VirtualList` of 10,000 rows in a scroll area |
| `numeric_stepper` | `NumericStepper` |
| `slider` | `Slider`s: continuous, stepped, formatted, disabled, and one sharing its value with a `NumericStepper` |
| `scrollareas` | `ScrollArea`s scrolling vertically, horizontally or both |
| `segmented_control` | `SegmentedControl`s, one with its own selection and two sharing one |
| `filter_chips` | `FilterChips` with counts, one row narrow so that it wraps and a second sharing the selection of the first |
| `panel` | a closable `Panel` (the app despawns it on `PanelClosed`) and one whose body scrolls |
| `setting_row` | a `SettingRows` column of `SettingRow`s with a segmented control, a stepper and a button in their slots |
| `badge` | neutral and warning `Badge`s on icon buttons, and a button that raises a count |
| `tooltip` | a `Tooltip` on a push button and the one an `IconButton` has |
| `text_field` | two `TextField`s, one with a placeholder and a narrower maximum width, and a label reporting their change and commit events |
| `toast` | buttons that raise `Toast`s, which stack at the bottom and go after three seconds |
| `image_gallery` | a `Gallery` of locomotives and wagons in a horizontal `ScrollArea` |
| `hud` | the redesign's HUD composed from the widgets over sample data, no game: pause button, `Kbd` and a `Slider` for the simulation speed (0.25x to 8x in octaves, written to the example's own `SimulationSpeed`), a panel `Toolbar` with badges driving a `Stack` of `Panel`s (trains with `FilterChips` and `Pill` rows, stops, warnings, preferences with `SettingRow`s, among them a "Text size" `NumericStepper` that sets `RemSize` and a "Company name" `TextField`; the hotkeys are off while a text field has the keys and on the frame it hands them back), and a tool dock whose active tool brings an option bar (`SegmentedControl`s, `NumericStepper`, vehicle `Gallery`). The "Show" buttons of the stops and warnings panels raise a `Toast`. Its icons are drawn at startup. The UI lives in `examples/hud/ui/mod.rs`, which `tests/headless.rs` includes with `#[path]` for a smoke test |
| `icon_button` | `IconButton`: small, regular and large buttons, a toggle, warning and danger tones, and the theme toggle |
| `gallery` | the tester's gallery: the widgets in one window, and a **Scenarios** window listing every file of `tests/ron/` (one group per widget) to watch play, step, take over or record; see [The gallery](#the-gallery) |

`icon_button`, `panel`, `badge` and `tooltip` draw their icon with
`examples/common/mod.rs` (`mod common;`; a directory without a `main.rs` is not
an example).

**The examples build their trees with `bsn!`.** Every example but `gallery`
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

Every example has a button that toggles the light and dark theme (the gallery's
is on its screens, the `hud`'s is the Appearance setting of its Preferences
panel), and all but `gallery` set `GlobalUiDebugOptions`, which is
why `bevy_ui_debug` is switched on in `[dev-dependencies]`.

### The gallery

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
  values (see "Scenario files"), so they pass at any scale factor.
- **`AppSetup::capture`** returns the `WidgetsSetup` the app was last built from
  (kept in the `CurrentScreen` resource), so **Record** and **New: current
  state** write `setup: App(Lists)` and the file loads back. What was done to
  the screen before recording started is not captured. There is no `AppView`:
  the widgets have no view to name.

### Tests

| Target | Holds |
| --- | --- |
| `tests/headless.rs` | widget behaviour driven by hand-written triggers in a headless `App` |
| `tests/reflection.rs` | the widget types and `Theme` are in the type registry |
| `tests/scenarios.rs` | Rust scenarios played by `rin_bevy_ui_tester`, a smoke test that every `WidgetsSetup` screen builds, the tests of `WidgetsCheck`, and every `tests/ron/` file played again with the UI drawn to an image (`image_target_app`), and every `tests/ron/` file played under shuffled schedules, see [Determinism](#determinism) |
| `tests/ron_scenarios.rs` | `harness = false`: one test per `.ron` file in `tests/ron/`, see [Scenario files](#scenario-files) |
| `tests/recorded_scenarios_gpu.rs` | needs `--features record` and a GPU, and is `#[ignore]`d: records every `tests/ron/` file offscreen, see [Recording the scenarios](#recording-the-scenarios) |
| `tests/common/mod.rs` | `widgets_app()` and `spawn_camera_and_root`, shared by the scenario targets |
| `tests/widgets_setup/mod.rs` | `WidgetsSetup`, the screens scenarios start from, one per widget, each started from its example but a fixture of its own |
| `tests/widgets_check/mod.rs` | `WidgetsCheck`, the checks about widget state the tester's built-in ones cannot say (`Selected`) |

## How the pieces fit

- **`UiRoot`.** An observer on `OnAdd, Node` inserts `UiRoot` on a node
  without a parent and without a `UiTargetCamera`, so root nodes aimed at
  another camera (a second window's UI, an overlay) are not the app's and are
  left alone. The camera is read when the `Node` is added, so a
  `UiTargetCamera` inserted later is not seen: spawn it in the same bundle. An
  observer of `Insert, ChildOf` removes `UiRoot` again from a node parented
  after its `Node` was added (a scene writes `ChildOf` last), so the marker
  follows the hierarchy. A `PostStartup` system logs an `error!` (it does not
  panic) unless there is exactly one `UiRoot`; an app should spawn one. Two
  roots both get `UiRoot`, so a widget must not take `Single<.., With<UiRoot>>`;
  a popup is a `Float` child of its widget instead, which needs no root at all.
  The tester's cursor overlay spawns its roots with a `UiTargetCamera`, so they
  get no `UiRoot` (`the_cursor_overlays_roots_get_no_ui_root` in
  `tests/scenarios.rs`).
- **Mouse events.** Global observers (registered by `WidgetsPlugin`, not on
  `UiRoot`) watch Bevy's flat pointer events `PointerOver`, `PointerPress`,
  `PointerRelease`, `PointerClick`, `PointerDrag` and `PointerOut` (one generic
  observer over the `PointerEvent` trait) and re-trigger each as a `MouseEvent`
  on the original target, on the first hop of the event only. `MouseEvent` is
  crate-private: it is how widgets inside this crate react to the pointer. They
  are global because Bevy's `Button` calls `propagate(false)` on `PointerPress`,
  `PointerRelease`, `PointerClick`, `PointerDragEnd` and `PointerCancel`: an
  observer on `UiRoot` would never see those for a button. `PointerOver`,
  `PointerDrag` and `PointerOut` are not stopped. `MouseKind` says what the pointer did:
  `Over`, `Pressed`, `Released`, `Click(button)` and `Drag`, plus `Out`.
  `Click` is a press and a release on the same node, which `Released` cannot
  say (it also ends a press that began elsewhere), and `Drag` is a move with a
  button held, which widgets ignore (the scroll thumb's drag is the
  `bevy_ui_widgets` `Scrollbar`'s). The combo's outside-click handler
  reads `ButtonInput<MouseButton>`, not a pointer event, so a `Button` cannot
  starve it.
- **"Is the pointer on it?" is picking's `Hovered`.** A system that asks where
  the pointer is now, outside a `MouseEvent`, reads `bevy::picking::hover::Hovered`
  (required by the component it asks about): true while the mouse pointer is on
  the entity or on any of its descendants, as picking's `HoverMap` says. The
  outside-click systems of `Combo` and `ColorPaletteCombo` read it on their
  button (`ComboButton`, the palette combo's `ColorPaletteComboButton`) and popup
  (`ComboList`, `ColorPaletteComboFloat`), and the `selection_change` of
  `list.rs` and `color_palette.rs` on each `ItemView`, to keep the hover colour of
  the item under the pointer. It is on the button rather than the combo because a
  `Combo`'s popup list is its descendant, so the combo's own `Hovered` would be
  true over the list as well. Bevy's `RelativeCursorPosition` is not used: it is
  only computed for cameras that render to a window, so UI drawn to an image (the
  recorder, any render-to-texture UI) never had the pointer on it (#100).
  `Hovered` follows the hover map, not the geometry: a node is only hovered when
  nothing above it that blocks the pointer (`should_block_lower`) is in the way,
  so an item under an open popup is not. A node with `should_block_lower: false`
  (a combo's button, an `ItemView`) is hovered and lets what is below be hovered
  too; a `Pickable::IGNORE` node (an item's label) is never in the hover map, so
  only the node it lies on counts. It follows `PointerId::Mouse` only, not touch;
  the outside click reads the mouse button anyway. It is updated in `PreUpdate`,
  so an `Update` system sees this frame's pointer.
- **Buttons are `bevy::ui_widgets::Button`.** Every button
  (`PushButton`, `ColorPushButton`, `Link`, `IconButton` and the rest) is a
  headless `bevy_ui_widgets` button, and nothing uses the deprecated `bevy::ui`
  `Button` or `Interaction`. The scroll thumb is no button: a `bevy_ui_widgets`
  `ScrollbarThumb` without a `Node` of ours, sized and placed by its `Scrollbar`,
  that requires `Pickable::default()` so it blocks the pointer and gets the
  `MouseEvent`s of its hover colour.
  A button: `Pressed` while held, `Activate`
  on release over the button and on Enter / Space for a focused one, never while
  `InteractionDisabled`. `WidgetsPlugin` adds `ButtonPlugin` (and `ListBoxPlugin`, `ScrollbarPlugin`, `SliderPlugin` and `TabPlugin`) in
  its `Plugin::finish`, unless the app has it by then (`DefaultPlugins` and the
  tester's `HeadlessPlugins` add all of `UiWidgetsPlugins`; adding twice panics),
  so the order of `WidgetsPlugin` and `UiWidgetsPlugins` does not matter; an app
  driven by hand with `App::update` must call `App::finish` first. A
  `ColorPushButton` with `InteractionDisabled` has its border in `text_disabled`
  (it has no hover colours to ignore). **`Activate` and `MouseEvent`:**
  "the button was used" is `Activate`, so an action observes `Activate` on the
  button itself: it does not bubble (`bevy_ui_widgets`' `Activate` has no
  `propagate`, `an_activate_does_not_bubble_to_an_observer_on_an_ancestor`), so an
  observer on an ancestor never hears it; how the button *looks* under the
  pointer (hover and press colours) is `MouseEvent`. Never do both for one
  thing. A disabled `PushButton` (insert
  `InteractionDisabled`) shows its text in `ThemeColors::text_disabled` and
  ignores hover. A `Link` is disabled through its `LinkState`, not by inserting
  `InteractionDisabled` yourself: `LinkState::Disabled` keeps the `Button` and
  adds `InteractionDisabled` (`Enabled` removes it again), and the link ignores
  hover.
  The crumbs of `BreadCrumbs` observe `Activate`; the current crumb is a
  disabled `Link`.
- **A list can render its own item views.** `ItemViewCreator` (next to the
  `List`, a registered system, like `Stack`'s `StackChildSpawner`) is run once
  per item with an `ItemViewSpawner { item, view }` and spawns the contents
  under the `ItemView`, which the list still owns, spawns and despawns with
  `ItemsChanged` and themes. The items need not be `Item<String>`. Such a view
  is `CustomView`: its children take the pointer, so hover colours come from
  `Hovered` (`custom_view_hover`) and press and release from a global
  `MouseEvent` observer that walks up from the target to the view, ignoring a
  target inside a nested `Button` (which handles its own press; `Button` also
  stops the click from reaching the `ListBox`). Selection and `Selected` are as
  for default views.
- **Lists are `ListBox`es, and `Selection` keeps the last word.** A `List` is a
  `bevy::ui_widgets::ListBox` and its `ItemView`s are `ListItem`s (`Selectable`,
  `Role::ListItem`); `WidgetsPlugin` adds `ListBoxPlugin` unless the app has it.
  `Selected` is Bevy's marker for an item in the selection, and `selection_change`
  in `list.rs` keeps it in step with the `Selection` (own or `Selection::Ref`), so
  it is on exactly the items of the selection, and `ListBoxMultiSelect` is on the
  list while the selection allows more than one item (`OneOrMore`, `ZeroOrMore`).
  **The choice:** the `ListBox` observers send `ValueChange<Entity>` for a click
  on a row (except the first selected row, see below), and for Enter / Space on
  the active row, and insert and remove `Selected` only if the app registers `listbox_update_selection`.
  We do not register it: `on_value_change` turns the `ValueChange` into a
  `SelectionParam::update_selection` instead, so `One`, `ZeroOrOne`, `OneOrMore`
  and `ZeroOrMore` (and `None` for a disallowed change) keep deciding, and
  `Selected` can never disagree with the `Selection`. The `ListBox` ignores a click
  only on the first selected row (`listbox_on_row_click` compares the row with the
  current one); on any other selected row of a multiple selection it still sends the
  `ValueChange`, and `on_value_change` drops it because the row is `Selected`. So
  deselecting is the item's `MouseEvent::Released`. Both paths go by the `Selected`
  the `ListBox` saw, not by the `Selection`: a release is handled after the
  `ValueChange` of the same click, and one that read the `Selection` would undo it. Hover and selected colours stay on `MouseEvent`. A `ListBox` also sets an
  `ActiveDescendant` on a click and, once the list has focus (a `List` is one tab stop),
  handles the arrow keys (all four move, wrapping round), Home and End by moving it, and
  Enter / Space by sending the `ValueChange` of the active row, which goes through
  `on_value_change` like a click, so the `Selection` decides, also a `Selection::Ref` and a
  palette's. Moving the active row selects nothing. Enter on a row that is already
  `Selected` does nothing (the `ListBox` does not send it), so keys cannot deselect.
  The list's focus ring is drawn on the active row, not on the box (see "Keyboard
  focus"). `ItemView`s keep `should_block_lower: false` so the wheel
  reaches the scroll area, which also lets a click through to what is below: a popup
  that floats over other widgets blocks the pointer itself (`Combo`'s list has
  `Pickable::default()`).
  A `ColorPalette` is a `List` with `NoDefaultItems`, so it is a `ListBox` of
  `ListItem`s as well and follows the same pattern: `color_palette.rs` puts
  `Selected` on its swatches (and `ListBoxMultiSelect` on the palette) from its
  `on_selection_change`, using `list.rs`'s `sync_selected` / `sync_multi_select`,
  `list.rs`'s `on_value_change` turns the `ListBox`'s pick into a selection update
  for palettes too, and the swatch's release deselects a swatch that is `Selected`.
  A swatch has no text, so its accessible name is the name of its colour (see
  `color_palette.rs` in the module table); a `Role(role: ListItem)` locator without a label
  still reaches all of them, so the scenarios use `Nth(..)` or
  `App(Accessible(..))`.
- **Keyboard focus: tab stops and the focus ring.** `WidgetsPlugin` adds Bevy's
  `InputFocusPlugin`, `InputDispatchPlugin` and `TabNavigationPlugin` in
  `Plugin::finish` unless the app has them by then (the tester's `HeadlessPlugins` has the first two, not the third), and
  `UiRoot` requires `TabGroup::new(0)`, so the widget tree is one non-modal tab group.
  `PushButton`, `ColorPushButton`, `Link`, `Slider` and `List` (so a `ColorPalette` as well)
  require the crate-private `TabStop`, which requires `TabIndex`. `sync_tab_index`
  owns the value: 0 for a stop that is not `InteractionDisabled` (a `LinkState::Disabled`
  link has it), -1 otherwise or while it has the crate-private `Unreachable` marker (the
  list of a closed combo, put on and taken off with `ComboState` / `State`) or while it is
  hidden (its `InheritedVisibility` is hidden, as Bevy's UI renderer reads it, or itself or an
  ancestor is `Display::None`, as in a hidden `Stack` panel; it is Tab-reachable again once
  shown). `sync_tab_index` runs in `PostUpdate` after `VisibilitySystems::VisibilityPropagate`,
  so an app needs Bevy's `VisibilityPlugin` (in `DefaultPlugins`, and the tester's
  `HeadlessPlugins` through `CameraPlugin`), or every stop counts as hidden; **no widget writes `TabIndex` by hand**. Within the group
  the order is `TabIndex`, then tree order, which is spawn order. The focused `TabStop`
  gets an `Outline` (2 px wide, 1 px offset, `ThemeColors::focus_ring`) while
  `InputFocusVisible` is true (an `Outline` the app or a widget set on it is kept aside and put back when the ring leaves, so the ring owns only its own), except that a list's ring is on its `ActiveDescendant` row
  (a `ListItem`) and the box has none, and a focused `Tab` of a `TabBar` gets the ring too
  (its `TabIndex` is Bevy's `TabPlugin`'s, kept at -1 while the bar is hidden by `tab_bar.rs`, not `focus.rs`'s `sync_tab_index`); the `ListBox` clears the active row when the list
  loses focus, and the ring goes with it, and `ActiveDescendant` is registered in the type
  registry for checks (`App(Active(of: .., is: ..))`, `WidgetsCheck::Active`); a click focuses without showing it (Bevy's
  `TabNavigationPlugin`), Tab shows it. Scenarios check the focus with
  `App(Focused(of: ..))` (`WidgetsCheck::Focused`) and press Tab with `Tap(key: Tab)`
  (`tests/ron/keyboard_tab_order.ron`). A focused entity with the crate-private `RingOn(target)` (a `TextField`'s input) has the ring on its `RingTarget` (the field's box) whether the focus is visible or not, so a clicked field shows where the keys go. `keyboard_focus_ring_in_both_themes.ron` Tabs to "Change theme" and toggles the theme, to record the ring on both. The `*_keyboard_*.ron` files drive lists, palettes and the tab bar (`tabs_keyboard_switches_panel.ron`) with the arrow keys. The lists of closed combos are skipped by Tab, so the palette scenario Tabs 8 times.
- **Directional navigation is the app's.** The library adds none: an app that wants the
  arrow keys (or a gamepad stick) to move the focus between widgets puts Bevy's
  `AutoDirectionalNavigation` on the widgets that take part, adds
  `DirectionalNavigationPlugin`, and calls `AutoDirectionalNavigator::navigate` from an
  observer of `FocusedInput<KeyboardInput>` on the primary window, which gets only the keys
  no focused widget kept (a system reading `ButtonInput<KeyCode>` would see them all); it
  sets `InputFocusVisible` itself, as Bevy's `directional_navigation` example does. What the
  widgets guarantee: a widget that uses the arrows stops them from bubbling on, so they
  never reach that observer. A focused `List` (so a palette and an open combo's
  list too) keeps the arrows, Home and End (the `ListBox` stops a first press,
  `keep_navigation_keys` in `list.rs` the repeats and releases, only while the list itself
  is `InputFocus`), a focused tab of a `TabBar` keeps left, right, Home and End (Bevy's tab
  list stops a first press, `keep_navigation_keys` in `tab_bar.rs` the repeats and
  releases) and lets up and down through, a focused `VirtualList` keeps the arrows, Page Up,
  Page Down, Home, End, Enter and Space (pressed, repeated or released; `navigate` in `virtual_list.rs`,
  only while the list itself is `InputFocus`, so the keys of a focused descendant, a button
  in a creator's row, go on past it), a focused `Slider` keeps left, right, Home and End
  (`keep_navigation_keys` in `slider.rs`, also at a limit) and lets up and down through, and a
  `NumericStepper`
  keeps the arrows that reach it from its buttons, also at a limit. The focus ring follows
  `InputFocus` whatever moved it, while `InputFocusVisible` is true. Every other widget lets
  the arrows through. A focusable child of a list row (a button in a `CustomView`) is not
  the list: its arrows, Home and End all go on past the list. Bevy's `ListBox` would take a
  first press of one as its own, because the `focused_entity` it checks is the hop the event
  has reached, not the focus, so `pass_first_presses_from_descendants` in `list.rs` stops
  that press on the list's child it comes through and triggers it again from the list's
  parent (the window, for a list without one); the active row stays. `tests/scenarios.rs`
  turns it on for two buttons, a list and a stepper (`directional_navigation_screen`), for
  buttons in list rows (`button_in_a_list_row_app`), and for a tab bar
  (`a_held_arrow_key_does_not_move_the_focus_out_of_a_tab_bar`).
- **Hidden widgets are hidden from the accessibility tree.** `sync_accessibility_hidden`
  in `focus.rs` runs next to `sync_tab_index` and marks the `AccessibilityNode` of
  anything hidden by the tab stop rule (`is_hidden`) as hidden, clearing it on show.
- **A combo's popup takes the keyboard focus while it is open.** Opening a `Combo` or a
  `ColorPaletteCombo` (the hook of `ComboState`, the observer of `State`) sets
  `InputFocus` to the popup's list (the palette, for the palette combo) with
  `FocusCause::Navigated`, so the `ListBox` arrows and Enter / Space work on it; closing
  it, however it was closed, moves the focus back to the button if it was still on the
  list. The observers sit on that list, not on the combo, because a palette combo's popup
  is a child of its root: `FocusedInput<KeyboardInput>` closes the combo on Escape, and
  on Enter / Space for a selection of one item at most (the `ListBox` picks the active
  row, or leaves a selected one, and the popup is done; a selection of many stays open),
  and `FocusLost` closes it when the focus leaves the list (Tab away, a click on
  something else). A click on the button or in the popup also takes the focus off the
  list (nothing in them is focusable), but that is not leaving: the pointer being on
  either (their `Hovered`) makes the observer ignore it (and put the focus back on the
  list for a click in the popup). On a close by losing the focus the focus goes back to the
  button only if nothing took it. `ComboState` / `State` stay the one source of truth. The
  `combo_keyboard_*` and `color_palette_combo_keyboard_*` files cover it.
- **Combo buttons open on `Activate` and say whether the popup is open.** The
  `ComboButton` and the `ColorPushButton` of a `ColorPaletteCombo` are `Button`s;
  the combo toggles its `ComboState` (the palette combo its `State`) on their
  `Activate`, so on release and on Enter / Space for a focused button, and a
  press alone opens nothing. Hover and press colours stay `MouseEvent`, observed
  on the combo, which gets the pointer because the button has `should_block_lower:
  false`. The outside-click systems read `ButtonInput<MouseButton>` and the
  `Hovered` of the button and the popup, not `Activate`, and a click on the
  button is inside the combo, so they never close what the same click opens. A
  release on a palette combo's popup is inside as well, whatever the selection:
  its scroll bar, or a drag of its thumb, leaves it open. The popup closes on a
  pick by itself, not through the outside click: a global observer of
  `MouseEvent`s acts on a `MouseKind::Click` (a press and a release on the same
  swatch) on a swatch that belongs to the popup's `ColorPalette` and, for a
  selection of one item at most (`One`, `ZeroOrOne`), inserts `State::Closed`,
  whether or not the click changed the selection (the selected colour of a `One`
  combo, a deselecting click of a `ZeroOrOne` one). A selection of many keeps
  the popup open (a palette combo over one is not supported yet: its
  `on_selection_change` asserts). A `Combo` has not been moved over (#96 left it): a release on
  its list still closes it, unless the selection may hold several items. The
  state is also `expanded` on the button's `AccessibilityNode` (`set_expanded`), set
  where `ComboState` / `State` is inserted (the hook, the observer); the button is a
  `Role::ComboBox` whose value is the selection and which `controls` the popup's list
  (palette), set by each combo's `sync_accessibility` in `AccessibilitySync`. Scenarios
  check `ComboState` / `State`, and `App(Accessible(..))` reads the node's role, value
  and `Expanded` / `Collapsed`; `controls` is checked headless. The popup is still the
  home-made `Float`, not `bevy_ui_widgets`' `MenuButton` / popover: that would
  change positioning, z-index and outside-click behaviour, so it is a ticket of
  its own if wanted.
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
- **A scroll area leaves room for a focus ring.** `ScrollContents` has 3 px of
  padding on every side (`FOCUS_RING_REACH`: the ring's 2 px width and 1 px
  offset), because the area clips at its padding box and the ring of a row at
  the edge would be cut off (#293). `scroll_into_view` counts the same 3 px as
  part of the entity, so a row scrolled into view shows its ring. The maximum
  scroll of every area is 6 px more than the contents' overflow alone.
- **Scroll area contents keep their natural size.** `ScrollContents` uses
  `align_items: FlexStart` instead of stretching its children. Taffy 0.10
  (Bevy 0.19) reported no overflow for a grid whose items overflow it, so a
  `List` or palette stretched to the area's height left nothing to scroll;
  taffy 0.14 (Bevy 0.20) was tried (#265): with `Stretch` the content still
  scrolls, but `list_wheel_scrolls` and
  `scroll_area_keyboard_scrolls_item_into_view` (both scale factors) fail, the
  maximum scroll 2 px short (296, not 298), so the workaround stays. For the
  same reason, don't give scroll contents `height: 100%`. The
  `scroll_area_*.ron` scenarios guard this on taffy 0.10: put `Stretch` back
  and the wheel and thumb scenarios failed there.
- **A scroll area's scroll position is kept in range.** The wheel handler
  (`update_scroll_position` in `scroll_area.rs`) moves only an axis whose
  `Node::overflow` is `Scroll` and leaves `ScrollPosition` inside `[0, max]`
  per axis, `max` being how far the contents overflow in logical pixels (0 when
  they fit); a node that does not scroll along an axis is not moved along it.
  A scenario reads `ScrollPosition.0.y`, the value that was set (logical
  pixels). `ComputedNode.scroll_position.y` is what the layout shows, floored,
  in physical pixels and one layout pass behind, so it doubles at a scale
  factor of 2 and scenario files do not read it. A thumb drag and a click on
  the track are the `bevy_ui_widgets` `Scrollbar`'s, which clamps to the same
  range; a click on the track beside the thumb pages by one visible length
  towards the click (`scroll_area_track_click_pages.ron`). The wheel stays
  ours rather than `bevy_ui_widgets`' `ScrollArea`:
  picking sends a `PointerScroll` to every hovered entity and each bubbles up to
  the area, and an `ItemView`, its `List` and the `ScrollContents` are all
  hovered (`should_block_lower: false`), so one turn would scroll two or three
  times; it also scrolls by `MouseScrollPixelsPerLine` (100 px a line, not our
  21) and has no Ctrl axis swap.

## Conventions

- **A scroll area scrolls a focused row into view.** `scroll_into_view` in
  `scroll_area.rs` observes `bevy_ui_widgets::ScrollIntoView`, which a `ListBox`
  triggers when the arrow keys, Home or End move its active descendant. It finds
  the `ScrollContents` above the entity and moves its `ScrollPosition` by the
  least that shows the entity, per axis only where the contents scroll and within
  `[0, max]`; an entity outside any scroll area is left alone, and so is one in a
  `FixedNode` (a combo's popup) inside one, since scrolling the area would not
  show it. It reads the
  entity's laid-out position, so it is one layout pass behind like the wheel's
  `ComputedNode`. The wheel is untouched, and so is the `Scrollbar`, which
  follows the `ScrollPosition` it sets
  (`scroll_area_keyboard_scrolls_item_into_view.ron`).
- **Nothing that runs every frame walks the whole world.** An app may hand a widget ten
  thousand items (rails' trains), so a system that runs each frame touches what is on
  screen, not every item: `VirtualList` looks at its pooled rows only, and finds an item by
  index in `ItemValues`. Work over all items belongs to a change (`ItemsChanged`, a new
  `Selection`) or a key press, not to the frame. A `List` spawns a view per item, so use a
  `VirtualList` for long lists.
- **One `Plugin` per widget**, registered in `WidgetsPlugin::build` in
  `lib.rs`. Downstream apps add only `WidgetsPlugin`.
- **Widgets observe `MouseEvent`, not raw pointer events (`PointerPress` and
  friends).** The only observers of them in the library are the producer in
  `lib.rs`; do not add more. `MouseKind` says `Over`, `Pressed`, `Released`, `Click` and `Drag`,
  so a widget can tell a click (a press and a release on the same node) from a
  release that ends a press begun elsewhere.
- **Styling comes from `Res<Theme>`.** Each widget has a `theme_*` system
  taking `In<Option<Entity>>`: its hook runs it for the one new entity with
  `run_system_cached_with(theme_x, Some(entity))`, and its plugin runs it for
  all of them with `(|| None::<Entity>).pipe(theme_x).run_if(resource_changed::<Theme>)`.
  `itertools::Either` unifies the two iterators. See `card.rs` for the
  smallest example.
  A widget with an interaction state (hovered, pressed, selected) takes its
  background from one function of that state (`background_for` in
  `push_button.rs`, `item_background` in `list.rs`) that both its pointer
  handler and its `theme_*` system call, so a theme change under a resting
  pointer keeps the hover colour. The state is read from `Hovered` and `Pressed`,
  not tracked by hand; a `PushButton` with `NoDefaultHover` rests at
  `background_primary` for its owner to colour.
- **Corner radius is a `Node` field.** Since Bevy 0.18 `BorderRadius` is
  `Node::border_radius`, not a component; set it on the node. Since Bevy 0.20
  its corners are `CornerRadius`es and its constructors (`BorderRadius::all`,
  `top`, ..) are not `const`, so a function that builds one cannot be either.
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
  The short type paths `Label`, `ScrollArea` and `Slider` are also Bevy's own, so a
  lookup by short path is ambiguous for those three; use the full path
  (`rin_bevy_ui::slider::Slider`).
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
7. A `WidgetsSetup` screen for the scenarios: a variant in
   `tests/widgets_setup/mod.rs`, added to the smoke test in
   `tests/scenarios.rs`, and its notes under [Scenario files](#scenario-files).

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
| Record the scenarios | `just record` | `cargo test --test recorded_scenarios_gpu --features record -- --include-ignored`; needs a GPU, see [Recording the scenarios](#recording-the-scenarios) |
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
- **Bevy upgrades land in the tester first.** It pins `bevy = "=0.20.0"`
  exactly, so this crate's `bevy` requirement must resolve to the same version;
  upgrade Bevy in the tester (and rails), bump the `rev` here, then raise
  `bevy` here. The move to 0.20 went that way: the tester upgraded on its own
  (rinde/rin-bevy-ui-tester#125) and this crate followed.
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
`.claude/settings.json` also sets the auto-compact window to 200k tokens for
Sonnet 5.5 and Opus 5.5 (`modelSettings.<model>.autoCompactWindow`).

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

### Writing a scenario

A scenario drives the widgets with scripted pointer and keyboard input through
`rin_bevy_ui_tester` (see its README for the `Step`, `Target` and `Check`
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
  `ScrollAreas`, `BreadCrumbs`, `Tabs`, `ColorPalette`, `NumericStepper`, `Slider`, `SegmentedControls`, `FilterChips`,
  `Panel`, `SettingRows`, `Badge`, `Gallery`, `Toolbar`, `Tooltip`, `Themed`, `IconButtons`, `TextFields`, `VirtualList`), which spawns a `Camera2d`, the one root `Node` and a screen, each
  with a "Change theme" button.
  - **A screen is a fixture of its own.** It starts from the example of the
    widget it is named after, then adds names, widgets and sizes for testing
    (a name to target, a layout that fits the 1280 by 720 window) and does not
    follow later changes to the example. A change to an example needs no
    matching change to its screen, and vice versa. They are not shared because
    the example spawns through `Commands` in a startup system while a setup is
    handed the `World`, and shared code would put test-only names and widgets
    into every example, or take them out of the screens.
  - **`Buttons` and `Themed` have no example of their own.** `Themed` puts one
    of each widget that restyles with the theme on one screen.
  - **A variant** is found by Bevy's automatic registration, so adding one
    needs no registration call; add it to the smoke test in
    `tests/scenarios.rs` as well.
  - **`Buttons`** also keeps the `Clicks` resource, the `Activate`s of "Add",
    and has a "Disabled" button (`InteractionDisabled`), which Tab skips.
  - **`AppSetup::capture`** (recording a session from the app as it is) is not
    implemented: only the tester's gallery and recorder call it, which these
    tests do not run, so it would go untested.
- **The check** is `App(Selected(of: .., items: [..]))`: `WidgetsCheck`, the
  one `#[reflect(AppCheck)]` enum of the tests, in `tests/widgets_check/mod.rs`
  (Rust: `Check::app(WidgetsCheck::Selected { .. })`; `mod widgets_check;`
  pulls it in, like `widgets_setup`, and it stays out of `common` for the
  same `dead_code` reason). A `Selection` holds entities, which mean nothing
  in a script, so `Selected` names the selected items by what they show, in
  the order they were selected: an `Item<String>` by its text, an
  `Item<Color>` by its Tailwind colour (`blue-500`) when it is one, else its hex (`color_name`). `of` is
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
- **The accessibility check** is `App(Accessible(of: .., role: Some("Button"),
  label: Some("Add"), value: None, states: [Disabled]))`
  (`WidgetsCheck::Accessible`): `AccessibilityNode` wraps an `accesskit::Node`,
  which reflection cannot read, so `Component(..)` cannot say what a widget
  exposes. `of` resolves like `Selected`'s, but the node is read on that entity,
  not above it, and a failure says `<target> has no AccessibilityNode` when it
  has none. `role` is the `accesskit::Role` by its `Debug` name (`"Button"`,
  `"ListItem"`); `label` and `value` are compared exactly; `None` is not
  checked. `states` are `AccessibleState`s that must all hold, unlisted ones
  are not checked: `Selected` / `NotSelected`, `Toggled` / `NotToggled` and
  `Expanded` / `Collapsed` need the state set (a node that cannot be expanded
  is neither), `Disabled` / `Enabled` and `Hidden` / `Shown` read a flag. A
  failure names every difference: `<target> has role Button, not ComboBox, is
  enabled, not Disabled`. **A `PushButton` has no label yet**: Bevy labels its
  `bevy::ui::Button`s with their text, not the `bevy::ui_widgets::Button`s ours
  are, so `accessible_names_today.ron` checks only the role and the disabled
  state. The tester's `Role(role: Button, label: ..)` still finds the button,
  because it reads the text.
- **Targets on a screen with several of one widget.** The `NumericStepper`
  screen puts a `Name` on every stepper (`default stepper`, `u16 stepper`,
  `u32 stepper`, `f32 stepper`, `stepper by two`, and `shared stepper` with
  the `first mirror stepper` and `second mirror stepper` that point to it
  through `ValueRef`), so a file reaches one stepper's button with
  `Within(Name("u16 stepper"), Name("Increase"))` (the icon buttons have no text for a
  `Role` label to match, so they are named after their accessible name) and reads its
  value with `TextOf(Name("u16 stepper"), Exact("11"))`. Move the pointer to the button before every click: a wider
  value moves the buttons. A button at its limit is `InteractionDisabled`, so
  the tester's click on it does nothing (`numeric_stepper_bounds.ron`).
- **The `Panel`, `SettingRows`, `Badge` and `Tooltip` screens.** `Panel`
  has `closable panel` (title "Trains", subtitle "3 open", three lines; its
  `PanelClosed` observer despawns it) and `tall panel` (100 px maximum body,
  twelve lines, so the wheel scrolls it: read `ScrollPosition.0.y` of its
  `ScrollContents`); the close button is `Component("IconButton")` within the
  panel, as an icon button has no text for a `Role` label to match. Panel
  and `SettingRow` do not share a screen: `SettingRows` is the `setting rows`
  column of `appearance row` (a `SegmentedControl`), `text size row` (a
  stepper) and `last row` (a button), whose dividers are `Nth(i, Within(Name
  ("setting rows"), Name("Divider")))` and read `Node.display`. `Badge` has
  `counted button` (3), `warning button` (12), `plain button` and "Add badge";
  the pill is a `Name("Badge")` child with the count as its text. `Tooltip` has
  `tooltip anchor` (tip "Anchor tip") and `help button` (an icon button, tip
  "Help"); the popup's text exists only while it shows, so `Exists(Text(..))`
  says whether it does, after `Wait(40)` frames (0.5 s at 60 per second).
- **The `Gallery` and `Toolbar` screens.** `gallery` (Steam, Diesel and Passenger cards in
  two groups, a `One` selection) and `filled toolbar` (Select, Build, Bulldoze, a `One`
  selection), each with "Change theme". Their `*_text_follows_rem_size.ron` files check that a
  label is `Rem(0.6)` text and fits inside its fixed px card (104 x 92) or button (66 x 60) at
  `[text 1.5x]`; both pass, so those sizes stay px.
- **The `Slider` screen.** A column of `continuous slider` (0 to 1, 320 px wide, name
  "Continuous"), `stepped slider` (0 to 10 in steps of 1, 200 px, name "Stepped"; the
  `SliderChanges` resource keeps the `last` value of its `ValueChange<f32>`s and counts the
  `finals`), `formatted slider` (0 to 100 in steps of 5 from 25, shown as `25 %`, name
  "Speed") and `disabled slider` (`InteractionDisabled`, 3, shown as `3`, no name), then
  "Change theme". The value of a slider is `SliderValue.0`, its parts `Within(Name(..),
  Name("SliderThumb"))` (and `SliderTrack`, `SliderLabel`); a press on the track is a
  `Click(Left)` after `MoveTo(Name(..))` and a drag `Drag(from: <thumb>, to: Screen(..))`.
  The label and the track let the pointer through, so a file cannot `MoveTo` the label.
  A press on the thumb alone does not make the slider `Pressed`, only one on the track or a
  drag does, so `slider_thumb_colours.ron` presses the track for the `accent` thumb.
- **The `TextFields` screen.** `name field` ("Alpha", named "Name"), `seed field`
  (empty, placeholder "Seed", named "Seed"), an "Other" button and "Change theme". The
  `FieldEvents` resource counts the fields' `TextFieldChange`s and `TextFieldCommit`s and keeps
  the last value of each (`changes`, `changed`, `commits`, `committed`). Type into a field
  with `Type(text: "42", into: Within(Name("seed field"), Component("TextInput")))` (the
  tester clicks it first), read its text with `TextOf(..)` of the same target and whether
  the placeholder shows with `Visible(Within(Name("seed field"), Text("Seed")))`. The
  `text_field_*.ron` files cover typing and `Enter`, `Escape` restoring, a click elsewhere,
  a click on the border (at `Screen((20.5, 32.0))`, the left border of `name field`),
  Tab and the screen reader node, and the text following `RemSize`.
- **The `Toast` screen.** "Save" and "Save again" raise a toast "Saved", "Delete" one "Deleted 3 pieces", each shown for 1 s (60 frames), and "Change theme". `Count(with: ["Toast"], ..)` counts the toasts showing (`toast_shows_stacks_and_goes.ron`).
- **The `IconButtons` screen.** Six icon buttons with the default (blank) icon image,
  each with a `Name`: `small button`, `regular button`, `large button`,
  `toggle button` (a `ToggleState` that starts off), `warning button` and
  `danger button`, then "Change theme". `Clicks` counts the `Activate`s of `regular
  button`. `icon_button_hover_colour.ron` reads the background of the regular
  button and the icon tint (`Within(Name("small button"), Component("Icon"))`,
  `ImageNode.color`) of the small one; `icon_button_activates.ron` the `Clicks` and the
  `ToggleState.0` of the toggle.
- **The `SegmentedControls` screen.** A row of `own segmented control` (Auto,
  Manual, its own selection, Auto first) and `first shared segmented control` and
  `second shared segmented control` (Dark, Light, one shared selection through
  `Selection::reference`) and "Change theme". A file reaches a segment with
  `Within(Name("own segmented control"), Role(role: Button, label: "Manual"))`,
  says what a control shows with `App(Selected(of: Name(..), items: [..]))` and
  whether a segment is `Selected(..)`; each segment is a tab stop, so Tab walks
  Auto, Manual, Dark, Light, Dark, Light and Enter / Space selects the focused one.
- **The `FilterChips` screen.** A row of `status chips` (All 4, Running 2, Waiting 1,
  Problems 1; its own selection, All first), `mirror chips` (the same selection through
  `Selection::reference`), `narrow chips` (Alpha, Beta, Gamma, Delta in 150 pixels, so they
  wrap, with a selection of its own) and "Change theme". `App(Selected(of: Name(..),
  items: [..]))` names an `Item<Chip>` by its label. Each chip is a tab stop, so Tab walks
  the four status chips, the four mirror chips and the four narrow ones; Enter / Space
  picks the focused one.
- **The `Lists` screen** names every list after its selection so a file can
  reach it: `one-or-more list` and `one-or-more grid list` (one shared
  `Selection::reference`), `one list` (own items), `zero-or-one list` and
  `zero-or-more list` (own selections over the shared items, `ItemsRef`),
  `zero-or-more grid list` (own items), `separated list` (`CornerStyle::Separated`, `One`, own items NL to BE; `list_separated_ring_follows_selection`), and `scrolling list` (twelve rows
  inside the `list scroll area`, 100 pixels high). "Add country" spawns the
  next of Suriname, Chile and Kenya into the shared items. A file reaches an
  item with `Within(Name("one list"), Role(role: ListItem, label: "CA"))`
  (an `ItemView` is a `ListItem`; target it, not its text, which the pointer
  passes through; `Selected(..)` and `Not(Selected(..))` say whether it is in the
  selection, and a `Tree(..)` shows `list-item "CA" [selected]`). `lists_screen_fits_window` keeps all of it inside the
  window. A list does not scroll itself: the wheel moves the `ScrollContents`
  of the area around it; read where it scrolled to as the scroll area's
  position above describes.
- **The `VirtualList` screen.** A `virtual list` of ten thousand rows ("Row 1" to
  "Row 10000", 28 px high, a `One` selection of its own, "Row 1" first) in the 240 by 200
  `virtual list scroll area`, and "Change theme". Its pool is thirteen rows (eight in view,
  one for a part row, two beyond each edge), counted with `Count(with:
  ["VirtualListRow"], ..)`; a row is reached by the item it shows, `Within(Name("virtual
  list"), Role(role: ListItem, label: "Row 9999"))`, and an item out of view has no row
  (`Not(Exists(..))`). The list is the screen's first tab stop
  (`virtual_list_scrolls_to_the_bottom`, `virtual_list_keyboard_moves_and_selects`,
  `virtual_list_keyboard_pages`, where Page Down moves seven rows, the 200 px view of 28 px
  rows); `a_virtual_list_of_ten_thousand_rows_keeps_its_entities_bounded_at_any_scroll_position`
  in `tests/scenarios.rs` checks that rows and entities stay the same at any scroll
  position and when an item is added, and
  `a_virtual_list_tells_a_screen_reader_where_each_row_is_among_all_its_items` the rows'
  `position_in_set` and `size_of_set`, which
  `a_virtual_list_row_that_shows_no_item_has_no_position_among_the_items` checks are
  cleared on a row that shows no item;
  `a_first_page_down_on_a_virtual_list_without_an_active_row_lands_on_the_first_item`
  that Page Down starts where Arrow Down does.
- **Targets on the `ScrollAreas` screen.** Four 200 by 200 areas side by side
  (they and "Change theme" fit the default 1280 by 720 window, which every
  scenario runs in), each with a `Name`: `two-way scroll area`, `vertical
  scroll area`, `horizontal scroll area` and `fitting scroll area` (three
  squares that never overflow). `MoveTo(Name("vertical scroll area"))` puts
  the pointer over the squares, so `Wheel(x: 0.0, y: -100.0)` scrolls it down
  (a positive wheel y scrolls up). Reach contents with `Within(Name(".."),
  Component("ScrollContents"))` and a thumb with `Within(Name(".."),
  Name("ScrollThumb"))` (`Nth(0, ..)` is the vertical bar's, `Nth(1, ..)` the
  horizontal one's in the two-way area; a horizontal-only area has just the
  one, `Nth(0, ..)`). The grid places the contents and the bars explicitly
  (contents row 1 column 1, vertical bar row 1 column 2, horizontal bar row 2
  column 1), so a bar sits in its own cell whichever bars the area has.
- **Observers on a screen's entities** are added with `.observe(..)` on the
  spawned entity. `Observer::new(..)` in a bundle is a global observer.
- **Every file runs at scale factors 1 and 2.** `ron_scenarios` plays each file
  as written (`name`) and on a window with the same logical size and a scale
  factor of 2 (`name [scale 2]`), so a file states logical values. Read
  `ScrollPosition` (logical) rather than `ComputedNode.scroll_position`
  (physical). `ComputedNode.size` and `UiGlobalTransform` are physical: check
  them with `unit: Logical`, which divides by the scale factor, or compare with
  a `Captured`. `Screen(..)` is in logical pixels and runs in both variants; a
  file that really needs a screen position and cannot run at scale 2 says
  `variants: ["default"]`, with a comment saying why (none does today).
- **Every file runs with larger text too.** A third variant, `name [text
  1.5x]`, plays each file with Bevy's `RemSize` (the root font size the
  widgets' text is in rem of, see `theme.rs`) at 1.5 times its default of 20
  px; the window keeps its size. A `TextFont.font_size` is the same rem value
  in every variant (`Eq(Rem(0.7))` for 14 px), so a file that the larger text
  should change checks sizes against a `Captured` one rather than a number.
  The `*_text_follows_rem_size.ron` files (`Themed`, `NumericStepper`,
  `FilterChips` and `Badge` screens) check a label's font size, that the text fits its widget and that the screen
  fits the window. A file whose numbers hold only for the default text (the
  scroll positions of `list_wheel_scrolls` and
  `scroll_area_keyboard_scrolls_item_into_view`, whose rows grow with the
  text) says `variants: ["default", "scale 2"]` with a comment saying why.
- **Prove it fails** as with Rust scenarios: change an expected value in the
  file and see the test fail before trusting it.
- **Every file is played more than once.**
  - `ron_scenarios` plays it as three variants: `name`, `[scale 2]` and `[text 1.5x]`.
  - `every_scenario_file_passes_with_the_ui_drawn_to_an_image` in `tests/scenarios.rs`
    plays it again headless with the camera rendering to an image as in a recording, so
    a file must pass both ways (see [Recording the scenarios](#recording-the-scenarios)).
  - `every_scenario_file_passes_with_each_schedule_shuffled_by_each_seed` plays it under
    shuffled schedules (see [Determinism](#determinism)).
- **Targets on the `Combos` screen.** A row of `first combo`, `own combo` and
  `third combo` (`first` and `third` share one selection through
  `Selection::reference`, `own` has a selection of its own over the same
  items) and "Change theme", then below it an "Underneath" button and the
  `country list`, which shares the first selection. An open combo's list
  floats over "Underneath", the guard that the popup blocks the pointer for what
  lies beneath it: `Not(Hovered(..))` of it with the pointer on `Canada`
  (`combo_opens_list`), and `Not(Pressed(..))` of it while a click picks
  `Canada` (`combo_popup_click_does_not_reach_underneath`). A `List` and its
  `ItemView`s do not block (`should_block_lower: false`, so the wheel reaches
  the scroll area around them); the combo's list does (`Pickable::default()`
  in `combo.rs`), as does the palette combo's popup, which is why a click on
  either does not reach the `country list` or the `palette` under it. The
  button also keeps the Float's `GlobalZIndex` testable: the tester's
  actionability walk skips nodes with `should_block_lower: false`, so only a
  blocking node can cover a list item.
  Reach the button with `Within(Name("first combo"), Component("ComboButton"))`,
  its list with `Component("ComboList")` and an item with
  `Within(Name("first combo"), Role(role: ListItem, label: "Canada"))` (the
  hidden lists hold the same labels, so always narrow by combo). `ComboState`
  is read with `Component(target: Name(..), path: "ComboState", is: Eq(Open))`;
  `Visible` of the list (or one item) says whether it shows. A click on empty
  space is `MoveTo(Screen((900.0, 500.0)))` then `Click(Left)`. The chevron is
  the icon of the combo's `PushButton`, `Within(Name("first combo"),
  Text("\\/"))`; a `PushButton` icon is `text_primary` like its label (and
  `text_disabled` when disabled).
  `noted combo` (Amsterdam, Delft `< unreachable >`, Utrecht `4 km`; absolute at x 700, y 200, spawned last) shows `ItemNote`s (`combo_option_notes`). `scrolled combo` (its own selection) sits in the 100 pixel high `combo
  scroll area`, absolutely placed at the right (x 1000, y 300) and spawned last
  so nothing moves and the tab order stays; its list reaches below the area and
  "Belgium" there is picked (`combo_in_scroll_area_popup_is_not_clipped`). A 1 by 400
  pixel spacer after the combo makes the area's contents overflow, so the arrow keys
  in the open list would scroll it were `scroll_into_view` not to stop at a `FixedNode`
  (`combo_in_scroll_area_keyboard_does_not_scroll_the_area`). `right edge combo` (x 1200,
  y 100, 100 wide) and `bottom edge combo` (x 500, y 660) are absolutely placed and spawned
  last too; their lists stay inside the window (`combo_keeps_list_inside_window`).
- **Targets on the `BreadCrumbs` and `Tabs` screens.** The trail is
  `Name("BreadCrumbs")` and the stack `Name("Stack")`. A crumb before the last
  is a `Button` (`Within(Name("BreadCrumbs"), Role(role: Button, label:
  "Home"))`); the last is a disabled button (`InteractionDisabled`; `[disabled]`
  in a `Tree(..)` snapshot, `Disabled(..)` as a check) with the same role, so
  `Role(role: Button, label: "Panel A")` reaches it too, as does
  `Nth(i, Within(Name("BreadCrumbs"), Name("Link")))`. The tester will not click
  it (`Force(Click(Left))` does, to show that nothing happens), and the text
  inside is `Pickable::IGNORE`, so a `MoveTo` to the text is "covered by
  `Link`". Read its `BackgroundColor.0`; the colour of its label is
  `TextColor.0` of `Text("..")`. A `Stack` panel hidden by `Display::None` still exists
  but is left out of a `Tree(..)` snapshot, which also accepts extra
  siblings, so say that a panel is hidden with `Not(Visible(Within(Name("Stack"),
  Text("Panel A"))))` and that one was never created with `Not(Exists(..))`.
  The panels that were created are counted by `Count` of `PushButton`s
  (`stack_shows_only_the_selected_item`); the `Tabs` screen's number list
  keeps its selection while its panel is hidden
  (`stack_keeps_state_of_hidden_item`). The combos in Panel A open popups that are descendants of the panel, so hiding it hides them and takes the list out of the tab order (`stack_hidden_panel_hides_combo_popups`). The tabs are the `Tab`s of a
  `TabBar`, `Role(role: Tab, label: "Panel A")`, `Panel B` and `Card demo`;
  the bar is one tab stop, the arrows move between its tabs and Enter or
  Space picks the focused one.
- **Targets on the `ColorPalette` screen.** A row of `first palette combo`,
  `second palette combo` (the two share one selection) and `own palette combo`
  (its own, over the same colours) and "Change theme"; `one palette combo`
  is out of the row (absolute, 300 pixels from the left) with a `One` selection
  over the same colours, slate-50 until another is picked, so that clicking
  the selected colour changes nothing; below the row `palette` (all
  242 Tailwind colours, 22 columns so that it, 836 by 418, fits the window; the
  example's 11 columns are taller than it) and `palette scroll area` (300 by
  300, 11 columns, sharing `palette`'s selection through its inner
  `Component("ColorPalette")`). Swatches have no text: they are `ItemView`s,
  reached by position as `Nth(60, Within(Name("palette"),
  Component("ItemView")))` (`Within` first narrows the `ItemView`s to those
  below `palette`, in document order, and `Nth` then counts in that list; 60
  is red-500, the order of `tailwind::COLORS`). A selected
  swatch is `Selected` (`Check(Selected(Nth(60, ..)))`) and has a `BorderColor.top`
  other than `background_primary`. A combo's
  button is its `ColorPushButton` and the colour it shows the
  `BackgroundColor.0` of its `ColorSquare`. The popup is a `Float` child of the
  combo, so `Within(Name("first palette combo"), ..)` reaches into it as well,
  and is named after it when it is spawned (`first palette combo popup`, `second palette combo
  popup`, `own palette combo popup`; a combo with only its default name has
  `ColorPaletteCombo popup`), whatever order the combos are in. Its swatches
  are `Nth(60, Within(Name("first palette combo popup"),
  Component("ItemView")))`; the combo's state is `Component(target:
  Name("first palette combo"), path: "State", is: Eq(Open))` and the popup's
  position the `UiGlobalTransform.0.translation` (the centre) of the float.
  The open popup lies over the first row of `palette`: swatch 0 of the popup
  (centre 314, 79) is over swatch 7 of `palette`, which is not hovered, pressed
  or selected by a click on it
  (`color_palette_combo_popup_click_does_not_reach_palette`).
  The popup is centred under its button and pushed in to 10 pixels from the
  window's left and right edges; the screen has `left edge palette combo` and
  `right edge palette combo`, absolutely placed in the top corners so nothing
  else moves, which open into those margins
  (`color_palette_combo_keeps_margin_at_window_edge`). The layout rounds the
  popup to whole pixels, so beside the 39 pixel wide button (centre on a half
  pixel) its centre is the button's plus or minus 0.5. The popup's palette refers to
  the combo's selection from the moment it is spawned, so
  `color_palette_combo_pick_as_first_action` opens `own palette combo` and picks a
  swatch as its first steps, with no wait. `scrolled palette combo` (its own
  selection) sits in the 100 pixel high `palette combo scroll area`, absolutely
  placed under `palette scroll area` and spawned last so the tab order stays;
  its popup reaches below the area and a swatch there is picked
  (`color_palette_combo_in_scroll_area_popup_is_not_clipped`).
- **The `Themed` screen and the theme scenarios.** One of each widget that
  follows the theme in a row: `themed button` (a `PushButton` "Other"),
  `themed card`, `themed list` (`One`, `Two`, `Three`, the first selected) and
  "Change theme". The theme scenarios start here because it is the only screen
  with a `Card`. `Theme.colors` is checked whole with
  `Resource(path: "Theme.colors", is: Eq((background_primary: Srgba(..), ..)))`
  (build the scenario in Rust and print `to_ron()` for the numbers),
  `ClearColor` with `Resource(path: "ClearColor.0", ..)`, a shadow with
  `Component(target: .., path: "BoxShadow.0[0].color", ..)` and the colour of
  a label with `TextColor.0` of `Text("..")`. Read a widget the pointer is not
  on: "Change theme" stays hovered after its click, and a hovered item has
  hover colours. The one that reads
  the hovered "Change theme" is `theme_keeps_hover_colour_under_pointer.ron`: a
  button the pointer rests on shows the new theme's `hovered_item`. A button
  let go with the pointer still on it is `hovered_item` too, not
  `selected_item`.

### Determinism

Since Bevy 0.20 many system sets (the UI's in `PostUpdate`) are ordered only
where systems' data access conflicts, so a system that runs at the right
moment only by luck would pass on one order and fail on another.
`every_scenario_file_passes_with_each_schedule_shuffled_by_each_seed` in
`tests/scenarios.rs` plays every `tests/ron/` file once for each of
`PreUpdate`, `Update`, `PostUpdate` and `Last` and each of `SHUFFLE_SEEDS`
(fixed, never the clock), with that schedule built with
`ScheduleBuildSettings::shuffle_seed` and the single-threaded executor, so the
order is the seed's alone. The `debug` Bevy feature that provides the seed is
in `[dev-dependencies]` only.

- **Reproduce a failure** from the file, schedule and seed it names: call
  `shuffle(&mut app, schedule, seed)` on a `widgets_app()` after
  `Registry::install` and play that file with `scenario.run_in(&mut app)`, or
  narrow the test's loops to that file, schedule and seed.
- **Fix it by ordering** the system through the data it touches (`.after(..)`
  or a set). Never widen the seed list or skip the file.

### Recording the scenarios

`just record` plays every file of `tests/ron/` in the tester's offscreen
recorder (`Recorder`, feature `record`, which this crate's `record` feature
turns on) and writes to `target/recordings/` (`RECORD_DIR` overrides it): per
scenario PNGs at the step boundaries and every tenth frame, `filmstrip.png`,
`scenario.gif` and `trace.html`, and one `index.html` with the failures first.
Open the index and look; no picture is compared with a golden one. It needs a
GPU, or lavapipe (`mesa-vulkan-drivers libvulkan1` and `WGPU_BACKEND=vulkan`),
so the test is `#[ignore]`d, its target needs the `record` feature, and
`just test` and `just ci` never build it; lint it with `cargo clippy --test
recorded_scenarios_gpu --features record`.

- **A drawn recording is not the headless one.** The recorder's cameras render
  to an `Image`, and Bevy's UI focus (`Interaction`, `RelativeCursorPosition`)
  only follows cameras that render to a window, so a widget must not read it:
  the widgets read hover from picking, which works for any target (see "Is the
  pointer on it?" above). Combos, palette combos and the hover colours of lists
  and palettes used to read `RelativeCursorPosition` and misbehaved in a
  recording (#100). `every_scenario_file_passes_with_the_ui_drawn_to_an_image`
  in `tests/scenarios.rs` plays every file headless with the camera aimed at an
  image, as the recorder does but without a GPU, so this is caught by `just
  test`. It fails on any file that does not pass; there is no allowlist, so a
  scenario that only fails with the UI drawn to an image is a bug to fix, not a
  name to list.
- **CI.** The `record` job of `ci.yml` runs `just record` on lavapipe and uploads
  `target/recordings` as the `recordings` workflow artifact. It is not part of
  `just ci`, and it was written without being run on GitHub.

## Before you call it done

1. `just fmt`
2. `just lint`
3. `just test`
4. New behaviour has a test, where the harness allows it
5. This file is updated if a module, example, convention or command changed
6. The work is on a PR, rebased onto `main` if needed
