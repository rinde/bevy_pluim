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
| `push-buttons` | 1 000 | idle `PushButton`s |
| `bevy-nodes` | 5 000 | Bevy baseline: plain 20 px nodes, the same count as `tooltips` |
| `tooltips` | 5 000 | idle nodes each with a `Tooltip` (no pointer, so none shows) |
| `virtual-list` | 100 000 items | one `VirtualList` in a scroll area, idle |
| `virtual-list-scrolling` | 100 000 items | the same list, scroll position moved every frame |
| `virtual-lists` | 20 lists of 1 000 | many virtual lists at once, idle |
| `filter-chips-one-changing` | 500 rows of 3 chips | many independent `FilterChips`; one row is marked changed every frame (no relayout), so a rebuild check that visits every row shows |
| `filter-chips-idle` | 500 rows of 3 chips | the same rows with no change: the floor `filter-chips-one-changing` is read against |
| `list-notes-changing` | 500 `List`s of 3 noted items | many independent lists; one `ItemNote` is marked changed every frame (no relayout), so a note sync that visits every row shows (#666) |
| `list-notes-idle` | the same | the same lists with no change: the floor `list-notes-changing` is read against |
| `list-items-initial` | 5 000 | one `List` whose whole `List` entity is respawned every frame, so the first `ItemsChanged` builds every view (#676) |
| `list-items-adding` | 5 000 | one `List`; one item is added every frame, so each `ItemsChanged` reconciles the existing views with one more (#676) |
| `steppers-idle` | 2 000 | independent `NumericStepper`s with no change: the floor the next two are read against (#663) |
| `steppers-changing` | 2 000 | the same steppers; 20 values are marked changed every frame (`set_changed`, the value itself unchanged, so no relayout from a new width) |
| `steppers-shared-changing` | 2 000 | groups of 10 mirror steppers (`ValueRef`) around one shared `Value`; 20 shared values marked changed per frame |
| `toasts-idle` | 500 parents | one long-lived `Toast` under each of many parents, no new toast: the floor `toasts-bursting` is read against (#678) |
| `toasts-bursting` | 500 parents | the same parents; 20 short `Toast`s (0.05 s) are raised under pseudo-random parents every frame, so stacks are looked up, reused and emptied (#678) |
| `segmented-controls-idle` | 500 `SegmentedControl`s of 3 segments | idle frames of independent controls (#661) |
| `indicators-animating` | 500 `SegmentedControl`s | one control's selection re-inserted per frame, so its indicator slides (#677) |
| `indicators-removed` | 500 `SegmentedControl`s | one `SegmentIndicator` despawned per frame, made again by `spawn` (#677) |
| `galleries-idle` | 500 `Gallery`s of 3 cards | idle frames of independent galleries (#661) |
| `tab-bars-idle` | 500 `TabBar`s of 3 tabs | idle frames of independent tab bars (#670) |
| `roving-groups-idle` | 500 `SegmentedControl`s of 3 segments | independent roving groups with no change: the floor the two below are read against (#662) |
| `roving-groups-focus` | the same | the `InputFocus` moves to a segment of another control every frame |
| `roving-groups-selection` | the same | one control's `Selection` is replaced every frame, so `Changed<Selection>` fires for that control alone (the control's own redraw is part of the cost) |
| `ticks-{filter,summary}-{idle,one,all}` | 100 000 | change detection of one table component: a `Changed<T>` filter reader against a `#[component(summary_tick)]` reader (`contiguous_iter`, skipping a table whose summary tick is old); nothing, one entity or every entity mutated per frame (#656) |

**Library overhead is the difference between a widget workload and its Bevy
baseline** (`push-buttons` minus `bevy-texts`, `tooltips` minus `bevy-nodes`).
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
`HiddenChanges` of `focus.rs`). Mean frame time in microseconds of
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
rest of the frame is other systems. Animation and removal are dominated by something else:
`indicators-animating` replaces a `Selection` every frame (about 30 ms at 500 controls), which
costs the selection widgets' own systems, not the indicator (follow-up ticket).

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
