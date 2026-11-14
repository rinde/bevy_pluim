# Widget benchmarks

`examples/bench.rs` measures what the widgets cost per frame and in memory,
in a headless app (the tester's `HeadlessPlugins`: a virtual 1280 x 720
window, fixed 1/60 s frames, picking and UI layout, no GPU, no rendering). It
is part of #652; it is not run by `just ci` and asserts nothing, because a
timing assertion on shared hardware is brittle.

## Running it

```sh
just bench                                              # every workload, default sizes
just bench -- --workload tooltips --size 20000          # one workload
just bench -- --frames 1000                             # more timed frames
```

Always an optimised build (`just bench` uses `--release`; the header says
`DEBUG` otherwise and the numbers then mean nothing). The suite runs each
workload in a fresh process, so memory from one does not leak into the next,
and prints a markdown table headed by the build, CPU, thread count, warm-up
and time step. Each workload spawns its widgets, plays 60 warm-up frames, then
times `App::update` for 300 frames (`--frames`).

| Column | Meaning |
| --- | --- |
| mean / median / p95 / max us | wall time of one `App::update`, all schedules, layout included |
| entities | live entities after the run |
| memory added MiB | growth of the resident set (Linux `/proc/self/statm`) from the empty app to the end of the run; 0 where `/proc` is missing |

Heap allocation counts are not reported: a counting global allocator needs
`unsafe`, which the crate forbids. Use `heaptrack` or `valgrind --tool=dhat`
on the same binary (`target/release/examples/bench --workload NAME`) for those,
and a Bevy `trace`/Tracy build for per-system time.

## Workloads

| Workload | Default size | What it isolates |
| --- | --- | --- |
| `bevy-texts` | 1 000 | Bevy baseline: nodes with `Text`, the same count as `push-buttons` |
| `bevy-text-children` | 1 000 | Bevy baseline with the buttons' entity structure: a node with a `Text` child each, two entities per button (`push-buttons` has about four) |
| `push-buttons` | 1 000 | idle `PushButton`s |
| `bevy-nodes` | 5 000 | Bevy baseline: plain 20 px nodes, the same count as `tooltips` |
| `tooltips` | 5 000 | idle nodes each with a `Tooltip` (no pointer, so none shows) |
| `tooltips-hovering` | 5 000 | the same hosts with a fake mouse pointer that rests on one host after another for 45 frames (longer than the 0.5 s delay), so each frame measures hover tracking and each popup's spawn, layout, placement and despawn; read it against `tooltips`; `HoverLock` is covered by `tooltips-scrubbing` |
| `tooltips-scrubbing` | 5 000 | the `tooltips-hovering` hosts and the same fake pointer, plus one `NumericStepper` held in a scrub drag (a primary `PointerPress` once and a `PointerDrag` past the 3 px threshold every frame, with Left held in `ButtonInput<MouseButton>` so `end_scrub` keeps it), so the crate-private `HoverLock` is on: `MouseKind::Over` is dropped and `update_tooltips` shows no popup; read it against `tooltips-hovering`; the run counts `TooltipPopup`s outside the timing and prints an error row if any shows, so a lock that never engaged cannot go unnoticed |
| `virtual-list` | 100 000 items | one `VirtualList` in a scroll area, idle |
| `virtual-list-scrolling` | 100 000 items | the same list, scroll position moved every frame |
| `virtual-lists` | 20 lists of 1 000 | many virtual lists at once, idle |
| `filter-chips-one-changing` | 500 rows of 3 chips | many independent `FilterChips`; one row is marked changed every frame (no relayout), so a rebuild check that visits every row shows |
| `filter-chips-idle` | 500 rows of 3 chips | the same rows with no change: the floor `filter-chips-one-changing` is read against |
| `filter-chips-selection-changing` | 500 rows of 3 chips | one row's selection re-inserted per frame, the other rows unrelated (#709) |
| `list-notes-changing` | 500 `List`s of 3 noted items | many independent lists; one `ItemNote` is marked changed every frame (no relayout), so a note sync that visits every row shows (#666) |
| `list-notes-idle` | the same | the same lists with no change: the floor `list-notes-changing` is read against |
| `lists-selection-changing` | 2 000 `List`s | many independent lists; 50 selections are replaced in one frame (#671) |
| `lists-selection-idle` | the same | the same lists with no change: the floor `lists-selection-changing` is read against |
| `list-items-initial` | 5 000 | one `List` whose whole `List` entity is respawned every frame, so the first `ItemsChanged` builds every view (#676) |
| `list-items-adding` | 5 000 | one `List`; one item is added every frame, so each `ItemsChanged` reconciles the existing views with one more (#676) |
| `list-items-horizontal-adding` | 5 000 | the same with `ListLayout::Horizontal` (#766) |
| `list-items-grid-adding` | 5 000 | the same with `ListLayout::VerticalGrid { columns: 4 }` (#766) |
| `items-shared-adding` | 5 000 | one source of items shared by eight `List`s under a `Display::None` parent; one item is added every frame, so one `ItemsChanged` reaches eight watchers (#681) |
| `palette-items-adding` | 5 000 | one `ColorPalette`; one item is added every frame (#679) |
| `tab-items-adding` | 5 000 | one `TabBar`; one item is added every frame (#679) |
| `segment-items-adding` | 5 000 | one `SegmentedControl`; one item is added every frame (#679) |
| `steppers-idle` | 2 000 | independent `NumericStepper`s with no change: the floor the next two are read against (#663) |
| `steppers-changing` | 2 000 | the same steppers; 20 values are marked changed every frame (`set_changed`, the value itself unchanged, so no relayout from a new width) |
| `steppers-shared-changing` | 2 000 | groups of 10 mirror steppers (`ValueRef`) around one shared `Value`; 20 shared values marked changed per frame |
| `toasts-idle` | 500 parents | one long-lived `Toast` under each of many parents, no new toast: the floor `toasts-bursting` is read against (#678) |
| `toasts-bursting` | 500 parents | the same parents; 20 short `Toast`s (0.05 s) are raised under pseudo-random parents every frame, so stacks are looked up, reused and emptied (#678) |
| `badges-idle` | 500 buttons | `IconButton`s each with a `Badge`, no change: the floor for `sync_badges` and `mirror_item_badges` (#685) |
| `badges-changing` | 500 buttons | the same buttons; 20 get a new `Badge::count` every frame (#685) |
| `panels-idle` | 500 panels | `Panel`s, no change: the floor for `sync_optional_sections` (#685) |
| `panels-changing` | 500 panels | the same panels; 20 are inserted again with a new title every frame (#685) |
| `setting-rows-idle` | 500 columns | `SettingRows` of five rows each, no change: the floor for `sync_dividers` (#689) |
| `setting-rows-changing` | 500 columns | the same columns; in 20 of them the last row is despawned and a new one inserted at the front every frame (#689) |
| `segmented-controls-idle` | 500 `SegmentedControl`s of 3 segments | idle frames of independent controls (#661) |
| `indicators-animating` | 500 `SegmentedControl`s | one control's selection re-inserted per frame, so its indicator slides (#677) |
| `indicators-removed` | 500 `SegmentedControl`s | one `SegmentIndicator` despawned per frame, made again by `spawn` (#677) |
| `segmented-controls-selection-changing` | 500 `SegmentedControl`s | one control's selection re-inserted per frame, the others unrelated (#709) |
| `palette-combos-idle` | 50 | closed `ColorPaletteCombo`s over the 242 Tailwind colours, nothing changing (#680) |
| `theme-editors-idle` | 1 | `ThemeEditor`s with every popup closed, nothing changing (#680) |
| `galleries-idle` | 500 `Gallery`s of 3 cards | idle frames of independent galleries (#661) |
| `filter-chips-editing`, `toolbar-editing`, `gallery-editing` | 200 items | one widget; every frame an item is removed or inserted in the middle (#690) |
| `tab-bars-idle` | 500 `TabBar`s of 3 tabs | idle frames of independent tab bars (#670) |
| `deep-trees-idle`, `deep-trees-display`, `deep-trees-reparenting` | chains of 40 nodes with a button | idle, `Display::None` flips and reparenting of deep trees (#684) |
| `roving-groups-idle` | 500 `SegmentedControl`s of 3 segments | independent roving groups with no change: the floor the two below are read against (#662) |
| `roving-groups-focus` | the same | the `InputFocus` moves to a segment of another control every frame |
| `roving-groups-selection` | the same | one control's `Selection` is replaced every frame, so `Changed<Selection>` fires for that control alone (the control's own redraw is part of the cost) |
| `wheel-deep` | a 200-deep chain in a vertical `ScrollArea`, with a nested horizontal area in the middle | 16 `MouseWheel` messages per frame, over two pointers whose hover map holds the nested area's 8 ancestors and a leaf (the headless app has no picking backend, so the example writes the hover map); the cost of `update_scroll_position` (#682) |
| `wheel-deep-idle` | the same | the same tree and hover map, no wheel messages: the floor `wheel-deep` is read against |
| `sliders-idle` | 2000 sliders | no change: the floor `sliders-pressing` is read against (#698) |
| `sliders-pressing` | 2000 sliders | one slider released and another pressed every frame, so `restyle_sliders` visits only those (#698) |
| `filter-chips-restyling` | 500 rows of 3 chips | the same rows with the `Theme` marked changed every frame, so every chip is restyled with the font it has; the font is written only on a difference, so no relayout (#710): mean 28.3 ms per frame with the unconditional write, 4.0 ms without (release build, 500 rows) |
| `ticks-{filter,summary}-{idle,one,all}` | 100 000 | change detection of one table component: a `Changed<T>` filter reader against a `#[component(summary_tick)]` reader (`contiguous_iter`, skipping a table whose summary tick is old); nothing, one entity or every entity mutated per frame (#656) |

**Library overhead is the difference between a widget workload and its Bevy
baseline** (`push-buttons` minus `bevy-texts`, `tooltips` minus `bevy-nodes`).
`bevy-texts` has one entity per button and `push-buttons` several (a button, its
label and what Bevy requires of them), so that difference includes the cost of
the child entities themselves, not only the library's systems. `bevy-text-children`
is a closer baseline: a node with a `Text` child (about 2 300 entities at the
default size, against 1 300 for `bevy-texts` and 4 300 for `push-buttons`). It still
has fewer entities than `push-buttons`, so compare the `entities` column and read
`push-buttons` minus `bevy-text-children` as an upper bound of the library's own cost.
An unknown `--workload` name is an error (exit code 2) that lists the valid names.
The virtual list has no like-for-like Bevy baseline; read it against an empty
frame (`bevy-nodes --size 1`) and against its scrolling variant. Per-widget
costs include the library's theme, accessibility and focus systems, which run
for any `WidgetsPlugin` app.

## Comparing a change

1. Build and run on the same machine, idle otherwise, with the same arguments:
   `just bench > before.md` on `main`, then `just bench > after.md` on the branch.
2. Run each at least three times and compare medians; a difference inside the
   run-to-run spread (look at p95 and max too) is noise.
3. Treat memory and entity counts as the stable signals (they barely vary) and
   timings as ratios on one machine, never as absolute numbers across machines.
4. Put the two tables, the hardware line and the command in the PR description.

## Baseline

Numbers are recorded in the PR that introduced the benchmark (hardware in the
header line of each table); regenerate them rather than editing a copy here.

## Bevy 0.20 summary ticks (#656)

`#[component(summary_tick)]` keeps one tick per table column, so a reader
using `Query::contiguous_iter` can skip a whole table when nothing in the column
changed. The cost is a store of the summary tick on every mutation, and it only
works for dense, table-stored queries without `Changed<T>`. The `ticks-*`
workloads put 100 000 entities with one `u32` component in the app, mutate
none, the first or all of them per frame, and read them with either reader.
Median `App::update` in us, three runs each (4 vCPU Intel Xeon 2.1 GHz,
`target/release/examples/bench --workload NAME`):

| Mutated per frame | `Changed<T>` filter | summary tick |
| --- | --- | --- |
| none | 5539 / 5528 / 6112 | 5515 / 5886 / 6971 |
| one | 5401 / 5706 / 5994 | 5433 / 5736 / 5846 |
| all | 5694 / 5983 / 6331 | 6443 / 6446 / 6594 |

Idle and one-mutation are indistinguishable (the spread between runs is larger
than any difference, and the frame is dominated by work other than the reader).
Mutating every entity costs about 0.5 ms more with the summary tick, with
nothing saved on the read side. **Decision: not adopted.** The library's
per-frame systems already read `Changed<..>` on sparse sets of entities or
walk only what is on screen, most widget components are immutable and changed by
re-insert (which a summary tick does not help), and a dense unfiltered scan is
exactly what "nothing that runs every frame walks the whole world" rules out.
The read saving is below the noise of this benchmark at 100 000 entities, so no
follow-up ticket is filed.

## Headless widgets of Bevy 0.20 (#656)

Reviewed against the `bevy_ui_widgets` 0.20 source (the release notes site was
not reachable from the build container):

- Already used: `Button`, `ListBox`, `Scrollbar`, `Slider`, `TabList` / `Tab`
  (`tab_bar.rs`), text input, `FixedNode`, rem sizing and shuffled schedules.
- `RadioGroup` / `RadioButton`: not adopted. `RovingGroup` also serves the
  toolbar, gallery, chips and `LightDarkToggle`, supports many-selections and
  a `Selection` that keeps the last word; the headless group would be a second
  model for the same keys with no code or measured overhead saved.
- `ScrollArea`: not adopted, see the wheel handling in `AGENTS.md`.
- Popover, menu and dialog: not adopted; changing `Float` is the separate
  ticket described in `AGENTS.md`.

## Idle virtual lists (#655)

`show_rows` visits every `VirtualList` and its row pool each `PostUpdate`. A
change-gated version (skip a list whose range, pool, item count, `Selection`
and `ItemValues` are unchanged) was prototyped and measured against `main` on
`virtual-list`, `virtual-list-scrolling` and `virtual-lists`, and so was a build
with `show_rows` switched off entirely. All three were inside the run-to-run
spread (about 2.0 to 2.7 ms a frame, of which an empty frame is about 1.95 ms),
so the system costs well under 50 us for 20 lists and the gate was not kept: it
needed extra per-list state and change-detection queries for no measurable gain.
The invalidation paths a gate would have to keep (idle frames, theme, selection,
item text, added items) are covered by tests in `tests/scenarios.rs` so a later
gate has a guard. Re-measure with the workloads above before adding one.

## NumericStepper value sync (#663)

`on_change` used to scan every fill, label, spin button and increase and
decrease button for each `Changed<Value>`. `ValueRef` is now a Bevy
relationship whose target `ValueReferrers` lists the views of a value, so
`on_change` visits only those, and it writes the label text, fill width and
accessibility node only when they differ, so a `Value` that is touched without
a visible change dirties no text, layout or accessibility node.

Measured with `cargo build --release --example bench`, three runs each, mean
frame time in ms (4 vCPU box, size 2 000), `main` against this change:

| workload | before | after |
| --- | --- | --- |
| `steppers-idle` | 11.4 | 11.4 |
| `steppers-changing` | 39 to 41 | 11.7 to 12.5 |
| `steppers-shared-changing` | 40 to 41 | 11.9 to 12.6 |

Memory added is unchanged (about 120 MiB, 24 323 entities). Only the scan
removal alone made no measurable difference (40 to 42 ms): the cost was the
rewritten text and node triggering text and UI layout, which the equality
guards remove. A step that really changes the displayed number still pays for
that layout, as it must. The idle floor (about 11 ms for 2 000 steppers) is
other per-frame systems, not `on_change`.

## Idle toast placement (#678)

`place_toasts` used to build a vector of every `ToastStack` each `Update`, and
`place_stack` visited every stack, even with no new toast and an unchanged
`ToastPlacement`. `place_toasts` now runs only while a toast has not moved into
a stack (`any_fresh_toast`) and looks parents up in a `HashMap`; `place_stack`
runs only when `ToastPlacement` changed or a `ToastStack` was added, so a stack
made late still gets an unchanged placement. Empty stacks stay (one small node
per parent, reused by the next toast there).

Measured with `cargo build --release --example bench`, three runs each, mean
frame time in microseconds (shared 4 vCPU box, noisy), `main` against this change:

| workload | size | before | after |
| --- | --- | --- | --- |
| `toasts-idle` | 500 | 2460 / 2612 / 2736 | 2931 / 2617 / 2400 |
| `toasts-idle` | 2000 | 6879 (one run) | 5916 / 5451 / 5771 |
| `toasts-bursting` | 500 | 5550 / 5196 / 7222 | 5569 / 5501 / 5447 |
| `toasts-bursting` | 2000 | 16824 (one run) | 16390 / 20263 / 19708 |

Entities and memory are unchanged (500: 1 821 / 1 881 entities, about 12.5 / 16 MiB;
2 000: 6 321 / 6 381 entities, about 45 / 49 MiB). The difference is within the noise:
the frame cost of these workloads is the cards' own text and UI layout, not the stack
scan, which is a few microseconds for 500 parents. The change removes the per-frame
allocation and the per-stack visits (guarded by the run conditions and the lifecycle
tests), not a measurable share of the frame, so no speed-up is claimed.

## Idle accessibility synchronization (#661)

`sync_accessibility` in the item-view widgets used to visit every view each `PostUpdate`.
They now look only at views whose `AccessibilityNode` changed (every Bevy reset writes it
through `DerefMut` in `UiSystems::PostLayout`, before `AccessibilitySync`), whose view component
changed, or, for the selection widgets, that `StructureChanges::affected` names; `SegmentedControl`
also follows `Added<Selected>`, removed `Selected` and changed `Item<String>`.
`cargo build --release --example bench`, then `./target/release/examples/bench --workload NAME --size 500`
three times each (mean frame time in microseconds; shared cloud container, noisy):

| workload | before (3 runs) | after (3 runs) |
| --- | --- | --- |
| `filter-chips-idle` | 4316 / 4462 / 4157 | 4159 / 4109 / 4204 |
| `segmented-controls-idle` | 4090 (1 run) | 4277 / 4398 / 4197 |
| `galleries-idle` | 5039 (1 run) | 4942 / 4900 / 4859 |

The gain is within noise to a few percent: the idle frame of these workloads is dominated by other
systems, so the change is mainly a scaling guarantee (no per-view work on a quiet frame).
## Roving-focus invalidation (#662)

`sync_roving_groups` used to look at every group on any focus move, and at every
group that enters at the selected item on any `Selection` change, each look walking
the group's descendants and allocating its item list. It now looks at a group on a
focus move only when the group held the focus or owns the newly focused item, and on a
selection change only when the changed `Selection` is the group's own or the one its
`Selection::Ref` resolves to. Item, children, visibility, disabling and removal
triggers are unchanged. Measured on the workloads above (500 groups, release build,
mean frame time in us, two runs each, same machine):

| workload | before | after |
| --- | --- | --- |
| `roving-groups-idle` | 5169 / 4645 | 4742 / 4727 |
| `roving-groups-focus` | 5280 / 4961 | 4757 / 4544 |
| `roving-groups-selection` | 35386 / 34036 | 32830 / 32567 |

Focus moves went from about 0.5 ms over idle to inside the run-to-run spread. A
selection change costs about 28 ms in both, which is the replaced control and the
widgets that follow its `Selection`, not the roving groups, so the narrowing buys
about 1 to 2 ms there. Tests: `a_selection_or_focus_change_looks_only_at_the_groups_it_concerns`
and `a_shared_selection_change_looks_at_every_group_that_reads_it_and_no_other`.

## Idle TabBar focus-index synchronization (#670)

`tab_bar.rs`'s `sync_tab_index` used to visit every bar and its tabs each `PostUpdate` and walk the
ancestors of every bar. It now looks only at bars that are new, whose `SelectedTab` or `Children`
changed, that have a tab whose `InteractionDisabled` (added or removed) or `TabIndex` changed, that
held or now hold the focus, or whose visibility, `Display::None` or parent changed (the shared
`HiddenDirty` of `focus.rs`). Mean frame time in microseconds of
`bench --workload tab-bars-idle --size 500`, three runs (shared cloud container, noisy):

| before | after |
| --- | --- |
| 3557 / 3783 / 3464 | 3293 / 3536 / 3519 |

The gain is a few percent: the rest of the idle frame is other systems. The point is that a quiet
frame does no per-bar work (`a_quiet_tab_bar_is_not_revisited_and_a_change_to_its_tabs_or_focus_is`).

## Note synchronisation of lists (#666)

`sync_item_notes` used to visit every default `ItemView` of the world when any
`ItemNote` changed or was removed. It now looks at the views of the changed
item only (a private `NoteViewOf` / `NoteViews` relationship), and at all of
them when the `Theme` changes. Workloads `list-notes-changing` (one note
changed per frame) and `list-notes-idle`, 500 independent lists, release build,
mean frame time in microseconds, three runs each:

| Workload | Before | After |
| --- | --- | --- |
| `list-notes-changing` | 4739, 5203, 4644 (mean 4862) | 4883, 4414, 4133 (mean 4477) |
| `list-notes-idle` | 4639, 4569, 4596 (mean 4601) | 4749, 4443, 4436 (mean 4543) |

The saving is about 8 % per changing frame, inside the run-to-run spread of a
frame that is dominated by the rest of the app; the point is that the work no
longer grows with the number of unrelated lists. The frame time of the idle
workload is unchanged, as expected. Test:
`a_note_change_resyncs_only_the_rows_of_its_item_and_a_theme_change_all_of_them`
(fails on the old code: the unrelated list was resynced).

## Selection synchronisation of lists (#671)

`list.rs`'s `selection_change` ran once per changed selection and scanned every
`List` for each, so N simultaneous changes cost N x lists. It now collects the
changed entities into a set and walks the lists once, restyling a list only
when its own or its referenced selection is in the set. Workloads
`lists-selection-changing` (50 selections replaced per frame) and
`lists-selection-idle`, 2000 independent lists, release build, mean frame time
in microseconds:

| Workload | Before | After |
| --- | --- | --- |
| `lists-selection-changing` | 11188 | 10361 |
| `lists-selection-idle` | 10452 | 9920 |

Cost over idle fell from about 0.74 ms to about 0.44 ms per frame (single runs;
the frame is dominated by the rest of the app). The remaining cost is one
pass over the lists, linear in lists but not in changes x lists. Test:
`simultaneous_selection_changes_update_only_the_lists_that_follow_them`.
## Sliding-indicator lifecycle (#677)

`indicator.rs`'s `spawn` ran every `Update` for `SegmentedControl`, `TabBar` and `LightDarkToggle`
and, for each track, scanned all indicators for one with that owner: N tracks cost N squared owner
checks per idle frame. It now checks a track by its own `Children`, and only for candidates: the
tracks whose component changed (`Changed<T>`, which includes a new track) or, after any indicator of
that kind was removed (`RemovedComponents<I>`), every track once. A quiet frame visits no track, a
re-inserted track keeps its one indicator, and a despawned indicator is made again once with the
right owner (`every_track_has_exactly_one_indicator_after_reinsertions`,
`a_removed_indicator_is_made_again_once_and_other_tracks_keep_theirs`,
`despawning_a_track_leaves_the_indicators_of_the_others_alone`,
`a_track_that_gains_its_widget_later_gets_one_indicator`). `animate` is linear in the tracks (one
child lookup each) and is unchanged. Mean frame time in microseconds, three runs (shared cloud
container, noisy), `bench --workload NAME --size N`:

| workload | size | before | after |
| --- | --- | --- | --- |
| `segmented-controls-idle` | 500 | 4238 / 4517 / 4304 | 3965 / 3926 / 4100 |
| `segmented-controls-idle` | 2000 | 14046 / 13166 / 12720 | 11099 / 10991 / 10997 |
| `indicators-animating` | 500 | 30673 / 32289 / 32972 | 30399 / 31099 / 30793 |
| `indicators-animating` | 2000 | 205235 / 206140 / 210757 | 205189 / 204951 / 205033 |
| `indicators-removed` | 500 | 8719 / 8819 / 8295 | 8609 / 8157 / 8247 |
| `indicators-removed` | 2000 | 33661 / 32753 / 32676 | 31694 / 33546 / 32489 |

Entities (7829 and 30329) and memory added (about 40 and 153 MiB) are unchanged. Idle frames gain
about 7 % at 500 controls and 15 % at 2000, growing with the count as the quadratic term goes; the
rest of the frame is other systems. Animation and removal are dominated by something else. A profile
of `indicators-animating` (#726, callgrind, 500 controls) puts about 83 % of the instructions in
`ui_layout_system`: the sliding indicators write `Node::left` and `width` every frame, which dirties
their ancestors up to the root, and taffy lays the whole tree out again. The selection widgets' own
systems are small. The largest was `theme_segmented_controls`, which scanned every control, segment,
indicator, label and icon for each control whose selection changed; it now visits only that control
and its children (`replacing_one_controls_selection_restyles_only_that_control`). Measured on one
machine back to back, mean frame time of `indicators-animating` before and after the change is 9.2 ms
and 8.1 ms at 500 controls and 40.4 ms and 35.3 ms at 2000, so the gain is about 12 %. The 30 ms and
205 ms in the table above predate other changes on `main` and are stale; moving the indicator without
a layout pass was the follow-up #768, done: see "Sliding indicators without layout (#768)" below.

## Selection restyling (#709)

`segmented_control.rs`'s `selection_change` and `filter_chips.rs`'s `restyle_changed_chips` reacted
to any changed `Selection` by looking at every control or chip row, so one selection change cost
work in every unrelated widget. Both now collect the changed selections and touch only the widgets
whose own `Selection`, or the one their `Selection::Ref` resolves to, changed (the scoping #659 gave
`TabBar`). `FilterChips` also restyles only the chips of those rows, the hovered ones and the chips
of a changed `Item<Chip>`, where it ran `theme_chips` over all of them; as a side effect a hover or
an item edit in a frame where a selection changed is no longer dropped. `list.rs`, `combo.rs` and
`breadcrumbs.rs` were already scoped per widget and are unchanged. Tests:
`a_segmented_control_follows_only_the_selection_it_draws_from`,
`filter_chips_restyle_only_the_rows_whose_selection_changed` and
`a_hover_and_an_item_edit_are_restyled_in_a_frame_where_a_selection_changed` (the first two fail on
the old code). Mean frame time in microseconds, three runs (shared cloud container, noisy),
`bench --workload NAME --size N`:

| workload | size | before | after |
| --- | --- | --- | --- |
| `filter-chips-selection-changing` | 500 | 26122 / 26824 / 26229 | 8354 / 8173 / 8409 |
| `filter-chips-selection-changing` | 2000 | 119975 / 111106 / 107197 | 32930 / 30060 / 29007 |
| `segmented-controls-selection-changing` | 500 | 32899 / 32936 / 31752 | 9476 / 9855 / 9259 |
| `segmented-controls-selection-changing` | 2000 | 209373 / 209096 / 215321 | 35371 / 31774 / 32758 |

Entities (6824 and 26324 for chips, 7828 and 30328 for segments) and memory are unchanged. A frame
with one changed selection costs about a third of what it did for chips and a sixth for segmented
controls at 2000 widgets. `theme_chips` still walked all chip views once per chip it restyled; #760
made it look up the one chip it is given (`Query::get_mut`), so the work scales with the chips
restyled. At size 2000 the mean stayed 29749 / 31480 / 32962 us, because the frame was dominated
by `restyle_changed_chips`'s own scan of all chip views.

#779 removed that scan: `restyle_changed_chips` now walks the `FilterChips` rows (one cheap check
each) and, for a row whose selection or items changed, only that row's `Children`; a changed
`Item<Chip>` is matched to rows through the items entity of `sources_of` and to chips by
`ChipView::item`. At size 2000 the mean is now 12980 / 12016 / 12035 us (about 2.6 times faster than
the 29749 / 31480 / 32962 us above). Test:
`an_item_edit_restyles_only_the_chips_of_rows_drawing_from_its_items`.

## Large item changes in a `List` (#676)

`on_items_changed` in `list.rs` cloned the item list and called `retain` once
per existing view, so reconciling N views with N items was quadratic. It now
builds one `HashSet` of the wanted items and one of the items that already have
a view, and is linear. Views are kept, in place, when their item is wanted and
no earlier view shows it; any other view is despawned; new items get views in
item order, custom views and selection as before.

Measured with `--frames 15` (Intel Xeon @ 2.10 GHz, 4 threads, release build),
mean frame time in microseconds and memory added in MiB, before and after:

| Workload | Items | Before | After | Memory added |
| --- | --- | --- | --- | --- |
| `list-items-initial` | 1 000 | 98 000-101 000 | 91 700 | 32 / 32 |
| `list-items-initial` | 4 000 | 797 000-822 000 | 790 000 | 124 / 124 |
| `list-items-initial` | 16 000 | 7 560 000 | not rerun | 490 |
| `list-items-adding` | 1 000 | 56 000-61 000 | 57 000 | 27.5 / 27.5 |
| `list-items-adding` | 4 000 | 603 000-643 000 | 604 000 | 93 / 93 |

The frame time hardly moves because `on_items_changed` was never the bottleneck:
the Taffy grid layout and `ui_layout_system` of the rows take almost all of the
frame (about 600 ms for 4 000 rows, by sampling). Timing the observer alone
(temporary instrumentation, not committed), in microseconds: initial build
1 000 items 216-273 to 160-190, 4 000 items 940-1 580 to 614-900, 16 000 items
3 100-6 700 to 2 500-3 100; adding one item to 4 000, 205-230 to about 150.
Temporary memory is two `HashSet`s of N entities (about 16 bytes an entry),
replacing the cloned `Vec` and its per-row clone, so it is the same order and
does not show in the resident set. For long lists use `VirtualList`.

## Large item changes in palettes, tabs and segments (#679)

`ColorPalette`, `TabBar` and `SegmentedControl` had the same quadratic
reconciliation as `List` had: `on_items_changed` cloned the items and ran
`retain` once per existing view, and `order_tabs` / `order_segments` searched
all children for every item. `on_items_changed` now uses the `List` approach
(one `HashSet` of wanted items, one of those already shown), and the two
ordering systems share `select.rs`'s `items_in_order`, which maps each item to
its first child with one `HashMap` and appends children that show no item
(the app's own) in their original order. Source order, selection state and
children of the app are unchanged.

Measured with `--frames 15` (same machine and build as above), mean frame time
in microseconds and memory added in MiB (before / after both listed as
before -> after, memory the same either way):

| Workload | Items | Mean frame before -> after | Memory added |
| --- | --- | --- | --- |
| `palette-items-adding` | 1 000 | 24 056 -> 26 673 | 20.7 / 20.9 |
| `palette-items-adding` | 4 000 | 185 100 -> 194 620 | 73.1 / 70.8 |
| `tab-items-adding` | 1 000 | 18 446 -> 17 152 | 21.5 / 21.6 |
| `tab-items-adding` | 4 000 | 96 738 -> 58 358 | 70.5 / 71.2 |
| `segment-items-adding` | 1 000 | 19 308 -> 14 093 | 26.4 / 26.6 |
| `segment-items-adding` | 4 000 | 100 015 -> 67 956 | 89.2 / 90.6 |

Tabs and segments get about 40 % faster at 4 000 items. The palette does not
move beyond noise: as for the `List`, its frame is dominated by layout and
swatch styling, not by the reconciliation, so the change there removes the
clones and the quadratic `retain` but is not measurable in frame time. The
temporary memory is two sets (and one map when ordering) of N entries, in
place of the cloned `Vec`s, and does not show in the resident set.

## Wheel-event hierarchy traversal (#682)

`update_scroll_position` built two hash sets per pointer for every wheel message and
walked the ancestors of every hovered entity, then of every scroll target, again for each
message. The hover map and the hierarchy do not change while the system runs, so it now
finds the targets once per frame (one entry per pointer, so an area two pointers are on
still scrolls once per pointer), keeps its sets and the resolved list in a `Local`, returns
at once in a frame with no wheel message, and applies each message to the resolved
targets in turn. Messages are not summed: clamping is not associative, so a message
that scrolls to the limit and one that scrolls back give a different result from their
sum. Delta units, the Ctrl swap, the horizontal fallback, the innermost-area rule and the
`FixedNode` boundary are unchanged.

Measured with `wheel-deep` and `wheel-deep-idle` (release build, size 200, 16 messages
per frame, median frame time in us over three runs, 4 vCPU Intel Xeon @ 2.10 GHz,
`cargo run --release --example bench -- --workload wheel-deep`):

| workload | before | after |
| --- | --- | --- |
| `wheel-deep` | 2829 / 2594 / 2568 (2594) | 1893 / 1816 / 1726 (1816) |
| `wheel-deep-idle` | 2022 / 1799 / 1656 (1799) | 1796 / 1463 / 1548 (1548) |

Over the idle floor the wheel handling went from about 0.8 ms to about 0.3 ms per frame
for 16 messages (the idle runs themselves vary by 0.2 ms, so read the difference loosely).
Allocations are not counted (no counter, see above); by the code, the two sets per pointer
per message became buffers that live as long as the system. Event coalescing and an
ancestor cache were not tried: the traversal now runs once per frame, a cache would
have to be invalidated by every hierarchy and hover change, and no workload showed a
remaining cost worth it. Tests: `many_wheel_messages_in_one_frame_are_applied_and_clamped_one_after_another`,
`a_wheel_line_is_21_pixels_and_control_swaps_the_axes`,
`an_area_that_scrolls_sideways_alone_takes_a_vertical_turn_for_a_horizontal_one`,
`only_the_innermost_hovered_scroll_area_takes_the_wheel_also_at_its_limit`,
`a_fixed_node_stops_the_search_for_a_scroll_area_above_it` and
`an_area_two_pointers_are_on_scrolls_once_for_each_pointer_and_message` in `tests/headless.rs`.

## Shared item snapshot (#681)

`ItemsChanged::items` is an `Arc<[Entity]>`: `items_change` copies the source's
items once and every observer (`watch_items`, `List`, `SegmentedControl`,
`TabBar`, palettes) clones the `Arc`, instead of one copy for the event plus
one per watcher. The snapshot is taken when the change is detected, so
ordering and intermediate state are as before. `items-shared-adding` at 5 000
items, mean `App::update` over 300 frames, optimised build, main against this
change (two runs of main): 113.1 / 109.4 ms before, 115.0 ms after, memory
added 1056 MiB both. The difference is within noise: with real `List`s the
per-frame cost is the views each list reconciles (8 x 5 000), not the copy
of 5 000 `Entity` ids (40 KB per copy). The change removes the copies
(`1 + N` down to 1 per change, and the `Vec` per watcher) but this workload
cannot show a gain; there is no allocation counter (see above).

## Closed palette popups (#680)

A `ColorPaletteCombo` built a swatch per item (242 in the theme editor) even
while its popup was hidden. The palette is now dormant while the popup is
closed (no `ItemView` children, `PaletteDormant`); opening wakes it and builds
the swatches before layout, closing despawns them (deferred to `Last`, so a pick
that closes the popup is still seen by the `ListBox`). The popup is shown after
it has been placed, one frame after the click. Selection stays on the combo.

Release build, one run each, before / after:

| Workload | Mean frame (us) | p95 (us) | Entities | RSS (MB) |
| --- | --- | --- | --- | --- |
| `palette-combos-idle` 50 | 27348 / 1644 | 35424 / 1936 | 37580 / 1278 | 232.9 / 3.9 |
| `theme-editors-idle` 1 | 23171 / 2183 | 29192 / 2832 | 33701 / 1755 | 220.2 / 14.1 |
| `theme-editors-idle` 5 | 124847 / 4147 | not recorded | 167117 / 7395 | 1026.1 / 44.1 |

### Startup time (#735)

The `bench` table has a `startup ms` column: the time to spawn the workload and
run its first three frames, which is when the theme editors are built, measured
before the warm-up. Release build, three runs each (`--frames 30`), `before` is
the parent of the #734 commit with this bench copied in, `after` is `main`:

| Workload | Startup before (ms) | Startup after (ms) |
| --- | --- | --- |
| `theme-editors-idle` 1 | 472 / 462 / 523 | 29.9 / 30.1 / 32.9 |
| `theme-editors-idle` 5 | 2304 / 2341 / 2180 | 93.4 / 97.6 / 93.9 |

Startup went from about 0.5 s to 30 ms for one editor and from about 2.3 s to
95 ms for five, in step with the entity counts (33 701 to 1 755 and 167 117 to
7 395): the swatches are no longer built until a popup opens.

Swatches built when a popup opens take the theme in use at that moment, also when
the `Theme` changed while the popup was closed
(`swatches_built_after_a_theme_change_while_the_popup_was_closed_carry_the_new_theme`).

## Shared hidden-state change set (#684)

`sync_tab_index`, `sync_accessibility_hidden`, `sync_roving_groups` and the tab bar's
`sync_tab_index` each used to build their own `HiddenChanges` (visibility, `Display::None` flips,
reparents and a subtree walk per changed root). `focus.rs`'s `collect_hidden_changes` now does it
once per frame into the `HiddenDirty` resource, with one deduplicated walk, so overlapping roots no
longer revisit descendants. Workloads `deep-trees-idle`, `deep-trees-display` (flips `Display::None`
on three nested nodes of one chain per frame) and `deep-trees-reparenting` (moves one chain per
frame), 200 chains of 40 nodes each with a button at the bottom, release build, size 200, two runs
each, mean / median frame time in microseconds:

| Workload | Before | After |
| --- | --- | --- |
| `deep-trees-idle` | 5227 / 5019, 5920 / 5923 | 5657 / 5488, 5668 / 5494 |
| `deep-trees-display` | 71005 / 69162, 74272 / 70724 | 70477 / 67317, 70991 / 65918 |
| `deep-trees-reparenting` | 102272 / 91229, 97616 / 85178 | 100059 / 89843, 93887 / 80541 |

Entities and memory are the same (the resource is one extra entity, 42 MiB added either way). The
differences are inside the run-to-run spread: the frame is dominated by layout and the UI systems,
not the hidden-state collection, so the consolidation is CPU-neutral. It is kept for one
definition of "what changed" (a consumer can no longer miss a root another one saw) and the
regression test for overlapping flips and a reparent in one frame.

## Idle badge and panel synchronization (#685)

`mirror_item_badges` and `sync_badges` (badge.rs) and `sync_optional_sections`
(panel.rs) visited every badge host, toolbar button and panel each `Update`.
They now return at once on a quiet frame: badges look at the hosts whose
`Badge` changed or was removed, new nodes, nodes whose `AccessibilityNode`
changed (Bevy rewrites it, which drops the description) and the buttons of a
changed or new `Item<Tool>`; panels look at the panels whose `Panel` or
`PanelRefs` changed or whose strip or footer `Children` changed (a private
`PanelSection` marker names the panel). A `Theme` change still visits all.

Measured with `cargo run --release --example bench`, one run each, mean frame
time in microseconds (shared box, noisy), `main` against this change:

| workload | size | before mean / median | after mean / median |
| --- | --- | --- | --- |
| `badges-idle` | 500 | 1981 / 1890 | 2136 / 2077 |
| `badges-changing` | 500 | 2246 / 2149 | 2573 / 2506 |
| `panels-idle` | 500 | 5259 / 4797 | 4739 / 4587 |
| `panels-changing` | 500 | 6336 / 6056 | 7440 / 6891 |

Entities and memory are unchanged (badges 2 821 entities, 14.7 to 15.3 MiB;
panels 9 831 entities, 52.9 to 53.4 MiB). Honest reading: at 500 hosts the
scans were cheap next to layout and accessibility updates, so the differences
are within run-to-run noise (the badge rows even read slightly higher after,
the panel idle row lower); the change removes the per-frame walk, whose cost
grows with the number of hosts, rather than a visible share of this frame.

## Targeted view reconciliation (#690)

`FilterChips::sync_chips`, `Toolbar::sync_buttons` and `Gallery::sync_cards`
despawned and respawned every view on any structural change. They now plan
against the views that exist (`select.rs`: `plan_views`, `order_views`): a view
whose item stays keeps its entity (so focus, hover and app children survive),
only missing views are spawned, stale ones despawned, and the children are
reordered once, only when the order differs. The gallery rebuilds its cheap
group scaffolding and re-parents the kept cards into it.

Measured with `cargo run --release --example bench`, 300 frames, one run each
(shared box, noisy), `main` against this change, mean / p95 frame time in
microseconds. Allocation counts are not measurable here (see above).

| workload | items | before mean / p95 | after mean / p95 | RSS MiB before -> after |
| --- | --- | --- | --- | --- |
| `filter-chips-editing` | 50 | 3364 / 4294 | 2139 / 2776 | 4.5 -> 4.3 |
| `filter-chips-editing` | 200 | 9562 / 12449 | 2539 / 3326 | 10.0 -> 8.3 |
| `toolbar-editing` | 50 | 3265 / 4405 | 1855 / 2316 | 4.9 -> 4.4 |
| `toolbar-editing` | 200 | 8007 / 9871 | 2441 / 3109 | 10.5 -> 8.2 |
| `gallery-editing` | 50 | 3342 / 4027 | 2150 / 3049 | 6.3 -> 5.4 |
| `gallery-editing` | 200 | 8733 / 10763 | 3349 / 4666 | 12.8 -> 10.4 |

Entity counts are unchanged. The cost now grows far more slowly with the item
count, since one edit touches one view.

## Setting row dividers (#689)

`sync_dividers` (setting_row.rs) collected a `Vec` of the rows of every `SettingRows`
column on each `Update`, 500 allocations a frame at 500 columns. It now looks only at columns
whose `Children` changed (or that are new) and at the parents of rows whose `SettingRowRefs`
arrived, and finds the last row with a reverse scan, so nothing is collected at all.

Measured with `cargo run --release --example bench --workload ..`, 500 columns of five rows,
mean / median frame time in microseconds, `main` against this change:

| workload | before mean / median | after mean / median |
| --- | --- | --- |
| `setting-rows-idle` | 11379 / 10779 (second run 10894 / 10327) | 10690 / 10207 |
| `setting-rows-changing` | 108384 / 104276 | 103022 / 100481 |

Entities (18 324) and memory (135 to 149 MiB) are unchanged. Honest reading: the saving is
inside run-to-run noise next to layout and accessibility work; the change removes the per-frame
allocation and the walk, which grow with the number of columns. There is no allocation counter
(it needs `unsafe`), so the allocation count is by construction: one `Vec` per non-empty column
per frame before, none after.

## HoverLock during a scrub drag (#756)

`HoverLock` is crate-private and set only while a `NumericStepper` scrub drag is on, so
the bench cannot set it. `tooltips-scrubbing` drives a real drag through Bevy's public
picking events (`PointerPress`, then `PointerDrag` each frame) on one stepper, so the
stepper's own observer sets the lock; the library gets no bench hook. Left is held in
`ButtonInput<MouseButton>` because `end_scrub` (`Last`) clears the lock otherwise. The
fake pointer of `tooltips-hovering` keeps moving over the 5 000 hosts, so the lock is
what separates the two runs.

Measured (release build, size 5 000, two runs each, 4 vCPU Intel Xeon @ 2.10 GHz,
`cargo run --release --example bench -- --workload NAME`; columns as in the bench
output: mean, median, p95 and max frame time in us):

| workload | mean | median | p95 | max |
| --- | --- | --- | --- | --- |
| `tooltips-hovering` | 2863 / 2715 | 2585 / 2451 | 5705 / 4074 | 8752 / 11193 |
| `tooltips-scrubbing` | 2508 / 2486 | 2453 / 2490 | 2872 / 2731 | 4230 / 3560 |

With the lock on, no popup is spawned, laid out or despawned, so the tail (p95, max) drops
to the floor of hover tracking; the median is about the same because most frames of
`tooltips-hovering` show no popup either. The lock itself costs nothing measurable: a
`bool` read in the `MouseEvent` observer transformer and once per hovered tooltip. Memory
is the stepper's 16 more entities. Run-to-run variation is about 0.2 ms, so read the
difference loosely.

## Sliding indicators without layout (#768)

`indicator.rs`'s `animate` used to write the indicator's `Node::left` and `width` every frame while it
slid, which dirtied layout up to the root. The `Node` now holds the goal (the selected item's laid-out
box) and is written only when that box moves; the slide is the indicator's `UiTransform` (translation
and scale easing to rest), which needs no layout pass.

Measured on one machine back to back (release build, `bench --workload indicators-animating --size N`,
mean frame time in microseconds, parent commit 2798db7 against this change; the #677 table above is
stale, see #709):

| size | before (2798db7) | after (#768) |
| --- | --- | --- |
| 500 | 10923 | 10175 |
| 2000 | 40079 | 36529 |

Honest reading: about 7 to 9 % better, which is close to run-to-run noise, and frame time still grows
roughly linearly with the number of controls. The acceptance of #768 (time stops growing with control
count) is **not met** by this change alone. Entities are unchanged. The sliding frames themselves no
longer write a `Node`, but what dominates the workload is not shown to be the per-frame selection
re-insert of the workload (that attribution is unverified); finding the residual cost is tracked in #782.

### What still costs, found with a profile (#782)

A callgrind profile of `indicators-animating` (500 controls) put almost all the remaining time in
taffy's flexbox layout, so something still dirtied layout every frame. Two causes:

1. **The segmented control restyled with unconditional `Node` writes.** `selection_change` restyles the
   changed control on every selection change, and `style_control`, `style_segment` and
   `style_indicator` assigned `Node` fields through `&mut Node`. Bevy's `upsert_node` calls taffy's
   `set_style`, which always marks the node dirty, up to the root, even for equal values or a
   border-radius-only change. The three now clone the node, edit the clone and write it back with
   `set_if_neq` (`reinserting_a_segmented_controls_selection_does_not_change_any_node`). `TabBar` and
   `LightDarkToggle` have the same pattern in their `theme_*` systems; not in this workload, filed
   separately.
2. **The workload's own legitimate write.** It moves one indicator's goal per frame, a real `Node`
   write. Taffy then dirties that node's ancestors, and the bench root is one wrapping flex row with
   every control as a flat child, whose flex pass is O(children) even with cached children. With
   that write disabled the frame cost equals `segmented-controls-idle`. No way of writing a `Node`
   avoids this (also none that would compensate the corner radius under a scale), so the remaining
   growth is Bevy's cost of one layout write under a wide parent, not the indicator's.

To separate the two, `bevy-nodes-resizing` is the plain-Bevy baseline: the same rows (a control with
three text segments and an absolute child) with no widgets, moving one node's `left` per frame.

Mean frame time in microseconds, release build, one machine. Before (main at d1686d0, which has #781) and after (this change) are two binaries run alternately, three runs each in one session, so drift hits both; run-to-run noise is about 20 %:

| workload | size | before (#781) | after (#782) |
| --- | --- | --- | --- |
| `indicators-animating` | 500 | 7321 / 10534 / 8466 | 8250 / 6397 / 7226 |
| `indicators-animating` | 2000 | 32529 / 39339 / 36896 | 28213 / 22277 / 25038 |
| `segmented-controls-idle` | 500 | 3595 | 3968 to 4036 |
| `segmented-controls-idle` | 2000 | 12572 | 11742 to 11836 |
| `bevy-nodes-resizing` (baseline) | 500 | - | 3817 to 4512 |
| `bevy-nodes-resizing` (baseline) | 2000 | - | 12273 to 13234 |

The `segmented-controls-idle` and `bevy-nodes-resizing` rows come from an earlier session, not the back-to-back runs above, and are indicative only.

Reading: at 2000 controls `indicators-animating` improves by about 25 to 30 % (the ranges do not overlap); at 500 the improvement (means 8774 before, 7291 after) is within the noise; and the plain-Bevy baseline grows with the control count in the same way, about 3x from 500 to 2000 controls. The restyle fix removes the layout dirtying that the widgets caused themselves; what is left
is the baseline's one-write-per-frame cost. The acceptance of #768 therefore holds relative to the
plain-Bevy baseline (the library adds a roughly constant overhead per control, not a growing one),
not as an absolute flat line, which a single layout write per frame under a flat root cannot give.

Stale `UiTransform`: with no selected item the indicator is `Display::None` and the transform was
not drawn, and a re-shown indicator starts from the goal (`placed` is false), so a stale value was
never visible; `animate` now also resets it to rest when it hides. Corner radius: the radius is a
`Node` field, so a scale stretches it for the frames of a slide (a 6 px radius on a box scaled 1.4x
horizontally is drawn 8.4 px by 6 px wide). It cannot be compensated without writing the `Node`,
which is what this change avoids; it is visible only mid-slide and snaps back at rest.

### Decision on the residual growth (#789)

Two options were weighed: (a) change the workload so the controls sit under separate roots or
contained subtrees, so one dirty node relayouts only its own subtree; (b) accept the baseline-relative
result and amend the acceptance of #768. **(b) is taken.** The growth is taffy's flex pass over one wide
flat parent after one legitimate `Node` write, which `bevy-nodes-resizing` shows with no widget code at
all; the library cannot remove it, and the sliding frames themselves already write no `Node`. Option (a)
would make the number flat by measuring a different scene, one an app with a wide flat row does not
have, and would hide the cost it is meant to show, so the workload stays as it is.

The amended acceptance of #768 is: `indicators-animating` grows with the control count like
`bevy-nodes-resizing` (both about 3x from 500 to 2000 controls), at a roughly constant factor above it
(about 2x in the table above, against a run-to-run noise of about 20 %), and a frame in which no goal
moves writes no `Node` (`a_sliding_indicator_moves_by_transform_and_leaves_its_node_alone`). It is not
a flat line in the control count, and the factor above the baseline is accepted, not
within noise. The hide-time `UiTransform` reset is covered by
`an_indicator_hidden_mid_slide_has_its_transform_back_at_rest` in `tests/scenarios.rs`, which slides an
indicator, clears the selection, and asserts `Display::None`, translation `Val2::ZERO` and scale
`Vec2::ONE`.

## Cost of a list row and of the non-vertical layouts (#766)

Two changes, measured with `--frames 15` (release build; columns: mean frame time in
microseconds, entities at the end, memory in MiB).

- **One entity fewer per row.** A default row had a view, a label and an observer entity (from
  `view.observe(handle_mouse_events)`). The hover and press colours are now handled by one
  global observer, which acts only on views carrying the private `DefaultRow` marker, so a
  palette swatch (its own observer) and a custom view (`handle_custom_view_mouse_events`) are
  untouched. A row is a view and a label (a note child exists only with an `ItemNote`). Dropping
  the label child too was considered and not done: the label is where an `ItemNote` and the
  rename sync (`ItemLabel`) hang, and a view with its own `Text` would mix with app children
  of a custom view.
- **`ListLayout::Horizontal` is a flex row**, not a one-row grid, as `Vertical` became a flex
  column in #729 (a grid with `ColumnDense` and no explicit tracks was the expensive part in
  Taffy). `VerticalGrid` needs the grid and stays one. There is no cheaper layout for it
  (columns of equal-height rows share tracks), so it is only measured.

| Workload | Items | Before | After | Entities before / after |
| --- | --- | --- | --- | --- |
| `list-items-initial` | 1 000 | 76 800 | 60 500 | 4 333 / 3 334 |
| `list-items-initial` | 4 000 | 299 600 | 231 700 | 16 333 / 12 334 |
| `list-items-adding` | 1 000 | 37 100 | 31 000 | 4 633 / 3 559 |
| `list-items-adding` | 4 000 | 120 600 | 115 000 | 16 633 / 12 559 |
| `list-items-horizontal-adding` | 1 000 | 63 400 | 21 600 | 4 633 / 3 559 |
| `list-items-horizontal-adding` | 4 000 | 693 300 | 86 000 | 16 633 / 12 559 |
| `list-items-grid-adding` | 1 000 | 35 900 | not changed | 4 633 / 3 559 |
| `list-items-grid-adding` | 4 000 | 315 100 | not changed | 16 633 / 12 559 |

The numbers of the #676 section above were measured before #729 and are not comparable.
