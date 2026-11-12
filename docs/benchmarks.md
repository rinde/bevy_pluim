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
| `lists-selection-changing` | 2 000 `List`s | many independent lists; 50 selections are replaced in one frame (#671) |
| `lists-selection-idle` | the same | the same lists with no change: the floor `lists-selection-changing` is read against |
| `steppers-idle` | 2 000 | independent `NumericStepper`s with no change: the floor the next two are read against (#663) |
| `steppers-changing` | 2 000 | the same steppers; 20 values are marked changed every frame (`set_changed`, the value itself unchanged, so no relayout from a new width) |
| `steppers-shared-changing` | 2 000 | groups of 10 mirror steppers (`ValueRef`) around one shared `Value`; 20 shared values marked changed per frame |
| `segmented-controls-idle` | 500 `SegmentedControl`s of 3 segments | idle frames of independent controls (#661) |
| `galleries-idle` | 500 `Gallery`s of 3 cards | idle frames of independent galleries (#661) |
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
