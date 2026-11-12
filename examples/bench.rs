//! A repeatable CPU and memory benchmark of the widgets, in a headless app.
//!
//! Run it optimised: `just bench` (all workloads at their default sizes) or
//! `just bench -- --workload tooltips --size 5000`. See `docs/benchmarks.md`
//! for how to read and compare the numbers.
//!
//! Each workload spawns its widgets under one root, runs warm-up frames, then
//! times `App::update` for a fixed number of frames of fixed 1/60 s time, and
//! prints one markdown table row: frame time (mean, median, 95th percentile,
//! maximum), entities, and the resident memory the workload added. The suite
//! runs every workload in a fresh process so one cannot leave memory behind for
//! the next. The `bevy *` workloads use no widget of this crate and are the
//! Bevy baseline the library's overhead is read against.

use std::io::Write as _;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use bevy::camera::ClearColor;
use bevy::ecs::system::SystemChangeTick;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::filter_chips::{Chip, FilterChips};
use bevy_pluim::gallery::{Gallery, GalleryCard};
use bevy_pluim::push_button::PushButton;
use bevy_pluim::scroll_area::{ScrollArea, ScrollContents};
use bevy_pluim::segmented_control::{Segment, SegmentedControl};
use bevy_pluim::select::{Item, Selection};
use bevy_pluim::tooltip::Tooltip;
use bevy_pluim::virtual_list::{RowHeight, VirtualList};
use bevy_stagecraft::HeadlessPlugins;

/// Frames played before timing starts, so the first layout and spawns settle.
const WARMUP_FRAMES: usize = 60;
/// Frames timed unless `--frames` says otherwise.
const DEFAULT_FRAMES: usize = 300;

/// What is spawned and what happens each frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Workload {
    /// Plain Bevy nodes with a text child: the baseline for the buttons.
    BevyTexts,
    /// Plain Bevy nodes: the baseline for the tooltips.
    BevyNodes,
    /// `PushButton`s, idle.
    PushButtons,
    /// Nodes with a `Tooltip`, idle (no pointer).
    Tooltips,
    /// A `VirtualList` of many items, idle.
    VirtualList,
    /// The same `VirtualList`, its scroll position moved every frame.
    VirtualListScrolling,
    /// Several `VirtualList`s side by side, idle.
    VirtualLists,
    /// Many independent `FilterChips`; one row is marked changed every frame.
    FilterChipsOneChanging,
    /// The same `FilterChips`, idle: the floor the changing workload is read against.
    FilterChipsIdle,
    /// Many independent `SegmentedControl`s of three segments, idle.
    SegmentedControlsIdle,
    /// Many independent `Gallery`s of three cards, idle.
    GalleriesIdle,
    /// Many independent `SegmentedControl`s (roving-focus groups); the focus moves to
    /// another control's segment every frame.
    RovingGroupsFocus,
    /// The same controls; one control's selection is replaced every frame.
    RovingGroupsSelection,
    /// The same controls, idle: the floor the two roving workloads are read against.
    RovingGroupsIdle,
    /// Change-detection microbenchmark (see `ticks`): a `Changed` filter reader
    /// over a component without a summary tick, nothing mutates.
    TicksFilterIdle,
    /// The same, a contiguous reader that skips on the summary tick, nothing mutates.
    TicksSummaryIdle,
    /// `Changed` filter reader, one entity mutated per frame.
    TicksFilterOne,
    /// Summary-tick reader, one entity mutated per frame (the skip never applies).
    TicksSummaryOne,
    /// `Changed` filter reader, every entity mutated per frame.
    TicksFilterAll,
    /// Summary-tick reader, every entity mutated per frame.
    TicksSummaryAll,
}

impl Workload {
    const ALL: [Self; 20] = [
        Self::BevyTexts,
        Self::PushButtons,
        Self::BevyNodes,
        Self::Tooltips,
        Self::VirtualList,
        Self::VirtualListScrolling,
        Self::VirtualLists,
        Self::FilterChipsOneChanging,
        Self::FilterChipsIdle,
        Self::SegmentedControlsIdle,
        Self::GalleriesIdle,
        Self::RovingGroupsFocus,
        Self::RovingGroupsSelection,
        Self::RovingGroupsIdle,
        Self::TicksFilterIdle,
        Self::TicksSummaryIdle,
        Self::TicksFilterOne,
        Self::TicksSummaryOne,
        Self::TicksFilterAll,
        Self::TicksSummaryAll,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::BevyTexts => "bevy-texts",
            Self::BevyNodes => "bevy-nodes",
            Self::PushButtons => "push-buttons",
            Self::Tooltips => "tooltips",
            Self::VirtualList => "virtual-list",
            Self::VirtualListScrolling => "virtual-list-scrolling",
            Self::VirtualLists => "virtual-lists",
            Self::FilterChipsOneChanging => "filter-chips-one-changing",
            Self::FilterChipsIdle => "filter-chips-idle",
            Self::SegmentedControlsIdle => "segmented-controls-idle",
            Self::GalleriesIdle => "galleries-idle",
            Self::RovingGroupsFocus => "roving-groups-focus",
            Self::RovingGroupsSelection => "roving-groups-selection",
            Self::RovingGroupsIdle => "roving-groups-idle",
            Self::TicksFilterIdle => "ticks-filter-idle",
            Self::TicksSummaryIdle => "ticks-summary-idle",
            Self::TicksFilterOne => "ticks-filter-one",
            Self::TicksSummaryOne => "ticks-summary-one",
            Self::TicksFilterAll => "ticks-filter-all",
            Self::TicksSummaryAll => "ticks-summary-all",
        }
    }

    /// The size used when `--size` is not given: widgets, items or lists.
    const fn default_size(self) -> usize {
        match self {
            Self::BevyTexts | Self::PushButtons => 1_000,
            Self::BevyNodes | Self::Tooltips => 5_000,
            Self::VirtualLists => 20,
            Self::FilterChipsOneChanging
            | Self::FilterChipsIdle
            | Self::SegmentedControlsIdle
            | Self::GalleriesIdle
            | Self::RovingGroupsFocus
            | Self::RovingGroupsSelection
            | Self::RovingGroupsIdle => 500,
            Self::VirtualList
            | Self::VirtualListScrolling
            | Self::TicksFilterIdle
            | Self::TicksSummaryIdle
            | Self::TicksFilterOne
            | Self::TicksSummaryOne
            | Self::TicksFilterAll
            | Self::TicksSummaryAll => 100_000,
        }
    }

    fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|workload| workload.name() == name)
    }
}

/// What one run was asked to do.
#[derive(Clone, Copy, Debug)]
struct Run {
    workload: Workload,
    size: usize,
    frames: usize,
}

fn main() -> ExitCode {
    let mut workload = None;
    let mut size = None;
    let mut frames = DEFAULT_FRAMES;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next();
        match (arg.as_str(), value) {
            ("--workload", Some(name)) => workload = Workload::parse(&name),
            ("--size", Some(value)) => size = value.parse().ok(),
            ("--frames", Some(value)) => frames = value.parse().unwrap_or(DEFAULT_FRAMES),
            _ => return usage(),
        }
    }
    workload.map_or_else(
        || suite(size, frames),
        |workload| {
            let run = Run {
                workload,
                size: size.unwrap_or_else(|| workload.default_size()),
                frames,
            };
            let _ = writeln!(std::io::stdout().lock(), "{}", measure(run));
            ExitCode::SUCCESS
        },
    )
}

fn usage() -> ExitCode {
    let names: Vec<_> = Workload::ALL.iter().map(|w| w.name()).collect();
    let _ = writeln!(
        std::io::stderr().lock(),
        "usage: bench [--workload {}] [--size N] [--frames N]",
        names.join("|")
    );
    ExitCode::from(2)
}

/// Runs every workload in a process of its own and prints the table.
fn suite(size: Option<usize>, frames: usize) -> ExitCode {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{}", environment());
    let _ = writeln!(
        out,
        "| workload | size | mean us | median us | p95 us | max us | entities | memory added MiB |"
    );
    let _ = writeln!(
        out,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    let Ok(exe) = std::env::current_exe() else {
        return usage();
    };
    for workload in Workload::ALL {
        let mut command = Command::new(&exe);
        command
            .args(["--workload", workload.name()])
            .args(["--frames", &frames.to_string()]);
        if let Some(size) = size {
            command.args(["--size", &size.to_string()]);
        }
        match command.output() {
            Ok(output) if output.status.success() => {
                let _ = out.write_all(&output.stdout);
            }
            _ => {
                let _ = writeln!(out, "| {} | failed | | | | | | |", workload.name());
            }
        }
    }
    ExitCode::SUCCESS
}

/// The build, hardware and workload header that makes numbers comparable.
fn environment() -> String {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|info| {
            info.lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|name| name.trim().to_string())
        })
        .unwrap_or_else(|| "unknown cpu".into());
    let threads = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    let profile = if cfg!(debug_assertions) {
        "DEBUG (numbers are not meaningful, use `just bench`)"
    } else {
        "optimised"
    };
    format!(
        "bevy_pluim bench, {profile} build, {cpu}, {threads} threads, {os} {arch}, \
         {WARMUP_FRAMES} warm-up frames, fixed 1/60 s time, headless 1280x720 window, no GPU",
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
    )
}

/// Resident set size in bytes, 0 where `/proc` is not there.
fn resident_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|statm| statm.split_whitespace().nth(1)?.parse::<u64>().ok())
        .map_or(0, |pages| pages * 4096)
}

/// Table component without a summary tick, for the change-detection workloads.
#[derive(Component)]
struct Plain(u32);

/// The same with `#[component(summary_tick)]`.
#[derive(Component)]
#[component(summary_tick)]
struct Summarised(u32);

/// Keeps the readers' work observable so it is not optimised away.
#[derive(Resource, Default)]
struct Sink(u64);

/// Reader of the status quo: a `Changed` filter visits every row's ticks.
fn read_with_filter(query: Query<&Plain, Changed<Plain>>, mut sink: ResMut<Sink>) {
    let mut sum = 0_u64;
    for plain in &query {
        sum += u64::from(plain.0);
    }
    sink.0 = sum;
}

/// Contiguous reader: a table whose summary tick is old is skipped whole, else
/// its rows' change ticks are compared one by one.
fn read_with_summary(
    query: Query<Ref<Summarised>>,
    ticks: SystemChangeTick,
    mut sink: ResMut<Sink>,
) {
    let mut sum = 0_u64;
    let Ok(iter) = query.contiguous_iter() else {
        return;
    };
    for tables in iter {
        let (values, table_ticks) = ContiguousRef::split(tables);
        if table_ticks.summary_tick_is_changed() == Some(false) {
            continue;
        }
        for (value, tick) in values.iter().zip(table_ticks.changed()) {
            if tick.is_newer_than(ticks.last_run(), ticks.this_run()) {
                sum += u64::from(value.0);
            }
        }
    }
    sink.0 = sum;
}

/// Mutates the first entity (`one`) or all of them through change detection.
fn mutate_plain<const ALL: bool>(mut query: Query<&mut Plain>) {
    for mut plain in &mut query {
        plain.0 = plain.0.wrapping_add(1);
        if !ALL {
            break;
        }
    }
}

fn mutate_summarised<const ALL: bool>(mut query: Query<&mut Summarised>) {
    for mut value in &mut query {
        value.0 = value.0.wrapping_add(1);
        if !ALL {
            break;
        }
    }
}

fn measure(run: Run) -> String {
    let mut app = App::new();
    app.add_plugins(HeadlessPlugins::default());
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app.init_resource::<Sink>();
    match run.workload {
        Workload::TicksFilterIdle => {
            app.add_systems(Update, read_with_filter);
        }
        Workload::TicksFilterOne => {
            app.add_systems(Update, (mutate_plain::<false>, read_with_filter).chain());
        }
        Workload::TicksFilterAll => {
            app.add_systems(Update, (mutate_plain::<true>, read_with_filter).chain());
        }
        Workload::TicksSummaryIdle => {
            app.add_systems(Update, read_with_summary);
        }
        Workload::TicksSummaryOne => {
            app.add_systems(
                Update,
                (mutate_summarised::<false>, read_with_summary).chain(),
            );
        }
        Workload::TicksSummaryAll => {
            app.add_systems(
                Update,
                (mutate_summarised::<true>, read_with_summary).chain(),
            );
        }
        _ => {}
    }
    app.finish();
    app.cleanup();
    // One frame for startup, so the empty app's memory is the reference.
    app.update();
    let before = resident_bytes();

    spawn(app.world_mut(), run);
    for _ in 0..WARMUP_FRAMES {
        frame(&mut app, run.workload, 0);
    }

    let mut times = Vec::with_capacity(run.frames);
    for index in 0..run.frames {
        let start = Instant::now();
        frame(&mut app, run.workload, index);
        times.push(start.elapsed());
    }
    let after = resident_bytes();
    let entities = app.world().entities().count_spawned();

    times.sort_unstable();
    let micros = |time: Duration| time.as_secs_f64() * 1e6;
    let mean = times.iter().map(|time| micros(*time)).sum::<f64>() / times.len().max(1) as f64;
    let at = |percent: usize| {
        let index = (times.len() * percent / 100).min(times.len().saturating_sub(1));
        times.get(index).map_or(0.0, |time| micros(*time))
    };
    format!(
        "| {} | {} | {:.0} | {:.0} | {:.0} | {:.0} | {} | {:.1} |",
        run.workload.name(),
        run.size,
        mean,
        at(50),
        at(95),
        at(100),
        entities,
        after.saturating_sub(before) as f64 / (1024.0 * 1024.0),
    )
}

/// One frame; the scrolling workload first moves the scroll position.
fn frame(app: &mut App, workload: Workload, index: usize) {
    if workload == Workload::VirtualListScrolling {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&mut ScrollPosition, With<ScrollContents>>();
        for mut position in query.iter_mut(world) {
            position.0.y = (index * 7 % 100_000) as f32;
        }
    }
    if workload == Workload::FilterChipsOneChanging {
        // Marks one row changed without touching what layout sees, so the frame shows
        // what the widgets' structure systems cost and not a relayout.
        let world = app.world_mut();
        let mut query = world.query::<&mut FilterChips>();
        let count = query.iter(world).count();
        if count > 0
            && let Some(mut chips) = query.iter_mut(world).nth(index * 7 % count)
        {
            chips.set_changed();
        }
    }
    if workload == Workload::RovingGroupsFocus {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Segment>>();
        let count = query.iter(world).count();
        let target = query.iter(world).nth(index * 7 % count.max(1));
        if let (Some(target), Some(mut focus)) = (target, world.get_resource_mut::<InputFocus>()) {
            focus.set(target, FocusCause::Navigated);
        }
    }
    if workload == Workload::RovingGroupsSelection {
        // Replaces the selection of one control (an immutable component) with a clone, so
        // `Changed<Selection>` fires for that control alone.
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Selection), With<SegmentedControl>>();
        let count = query.iter(world).count();
        let picked = query
            .iter(world)
            .nth(index * 7 % count.max(1))
            .map(|(entity, selection)| (entity, selection.clone()));
        if let Some((entity, selection)) = picked {
            world.entity_mut(entity).insert(selection);
        }
    }
    app.update();
}

fn spawn(world: &mut World, run: Run) {
    world.spawn(Camera2d);
    let root = world
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            ..default()
        })
        .id();
    world
        .entity_mut(root)
        .with_children(|root| match run.workload {
            Workload::BevyTexts => {
                for index in 0..run.size {
                    root.spawn((Node::default(), Text::new(format!("Button {index}"))));
                }
            }
            Workload::BevyNodes => {
                for _ in 0..run.size {
                    root.spawn(Node {
                        width: px(20),
                        height: px(20),
                        ..default()
                    });
                }
            }
            Workload::PushButtons => {
                for index in 0..run.size {
                    root.spawn(PushButton::from_label(format!("Button {index}")));
                }
            }
            Workload::Tooltips => {
                for index in 0..run.size {
                    root.spawn((
                        Node {
                            width: px(20),
                            height: px(20),
                            ..default()
                        },
                        Tooltip::new(format!("Tip {index}")),
                    ));
                }
            }
            Workload::TicksFilterIdle | Workload::TicksFilterOne | Workload::TicksFilterAll => {
                for index in 0..run.size {
                    root.spawn(Plain(index as u32));
                }
            }
            Workload::TicksSummaryIdle | Workload::TicksSummaryOne | Workload::TicksSummaryAll => {
                for index in 0..run.size {
                    root.spawn(Summarised(index as u32));
                }
            }
            Workload::VirtualList | Workload::VirtualListScrolling => {
                virtual_list(root, run.size);
            }
            Workload::VirtualLists => {
                for _ in 0..run.size {
                    virtual_list(root, 1_000);
                }
            }
            Workload::RovingGroupsFocus
            | Workload::RovingGroupsSelection
            | Workload::RovingGroupsIdle => {
                for row in 0..run.size {
                    root.spawn((
                        SegmentedControl,
                        Selection::builder().one().build_with_items(
                            ["A", "B", "C"].map(move |label| Item::new(format!("{label} {row}"))),
                        ),
                    ));
                }
            }
            Workload::FilterChipsOneChanging | Workload::FilterChipsIdle => {
                for row in 0..run.size {
                    root.spawn((
                        FilterChips,
                        Selection::builder()
                            .one()
                            .build_with_items((0..3).map(move |chip| Item {
                                data: Chip::new(format!("Chip {row}.{chip}")),
                            })),
                    ));
                }
            }
            Workload::SegmentedControlsIdle => {
                for row in 0..run.size {
                    root.spawn((
                        SegmentedControl,
                        Selection::builder().one().build_with_items(
                            (0..3)
                                .map(move |segment| Item::new(format!("Segment {row}.{segment}"))),
                        ),
                    ));
                }
            }
            Workload::GalleriesIdle => {
                for row in 0..run.size {
                    root.spawn((
                        Gallery,
                        Selection::builder()
                            .one()
                            .build_with_items((0..3).map(move |card| Item {
                                data: GalleryCard::new(
                                    Handle::default(),
                                    format!("Card {row}.{card}"),
                                    "Group",
                                ),
                            })),
                    ));
                }
            }
        });
}

fn virtual_list(parent: &mut ChildSpawner, items: usize) {
    let Ok(row_height) = RowHeight::new(28.0) else {
        return;
    };
    parent.spawn((
        Node {
            width: px(240),
            height: px(200),
            ..default()
        },
        ScrollArea::bundle(
            move |area: &mut ChildSpawner| {
                area.spawn((
                    VirtualList::new(row_height),
                    Selection::builder()
                        .one()
                        .build_with_items((1..=items).map(|row| Item::new(format!("Row {row}")))),
                ));
            },
            true,
            false,
        ),
    ));
}
