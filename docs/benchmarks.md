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
