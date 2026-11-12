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
