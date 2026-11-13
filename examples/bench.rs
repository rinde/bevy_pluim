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
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::color_palette_combo::ColorPaletteCombo;
use bevy_pluim::filter_chips::{Chip, FilterChips};
use bevy_pluim::gallery::{Gallery, GalleryCard};
use bevy_pluim::list::{ItemNote, List};
use bevy_pluim::numeric_stepper::{NumericStepper, Value, ValueRef};
use bevy_pluim::push_button::PushButton;
use bevy_pluim::scroll_area::{ScrollArea, ScrollContents};
use bevy_pluim::segmented_control::{Segment, SegmentIndicator, SegmentedControl};
use bevy_pluim::select::{Item, ItemOf, ItemValues, Selection, SelectionValue};
use bevy_pluim::tab_bar::TabBar;
use bevy_pluim::tailwind::COLORS;
use bevy_pluim::theme_editor::ThemeEditor;
use bevy_pluim::toast::Toast;
use bevy_pluim::tooltip::Tooltip;
use bevy_pluim::virtual_list::{RowHeight, VirtualList};
use bevy_stagecraft::HeadlessPlugins;

/// Frames played before timing starts, so the first layout and spawns settle.
const WARMUP_FRAMES: usize = 60;
/// Frames timed unless `--frames` says otherwise.
const DEFAULT_FRAMES: usize = 300;
/// Values changed per frame by the stepper workloads.
const CHANGED_STEPPERS: usize = 20;
/// Steppers mirroring each shared value in `steppers-shared-changing`.
const MIRRORS: usize = 10;

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
    /// Many independent `List`s of three noted items; one note is marked changed every frame.
    ListNotesChanging,
    /// The same lists, idle: the floor the changing workload is read against.
    ListNotesIdle,
    /// One `List` of many items, replaced by a new one with all its items every frame:
    /// the initial build of the views.
    ListItemsInitial,
    /// One `List` of many items; one item is added every frame: an incremental change.
    ListItemsAdding,
    /// Many closed `ColorPaletteCombo`s over one shared 242-colour source, idle.
    PaletteCombosIdle,
    /// Several `ThemeEditor`s, idle: each has a closed palette combo per colour slot.
    ThemeEditorsIdle,
    /// One source of many items shared by `SHARED_LISTS` hidden `List`s; one item is added every
    /// frame, so one `ItemsChanged` reaches every watcher.
    ItemsSharedAdding,
    /// One `ColorPalette` of many swatches; one item is added every frame (#679).
    PaletteItemsAdding,
    /// One `TabBar` of many tabs; one item is added every frame (#679).
    TabItemsAdding,
    /// One `SegmentedControl` of many segments; one item is added every frame (#679).
    SegmentItemsAdding,
    /// Many independent `NumericStepper`s, idle: the floor the changing workloads are read against.
    SteppersIdle,
    /// The same steppers; `CHANGED_STEPPERS` values are marked changed every frame.
    SteppersChanging,
    /// Groups of ten `NumericStepper`s mirroring one shared `Value`; `CHANGED_STEPPERS` shared
    /// values are changed every frame.
    SteppersSharedChanging,
    /// Many parents, each with a `ToastStack` holding one long toast, idle (#678).
    ToastsIdle,
    /// The same parents; `TOAST_BURST` short toasts are spawned under other parents every frame.
    ToastsBursting,
    /// Many independent `SegmentedControl`s of three segments, idle.
    SegmentedControlsIdle,
    /// Many independent `SegmentedControl`s; one control's selection moves to its next segment
    /// every frame, so its indicator slides (#677).
    IndicatorsAnimating,
    /// The same controls; one control's indicator is despawned every frame and must be made again
    /// (#677).
    IndicatorsRemoved,
    /// Many independent `Gallery`s of three cards, idle.
    GalleriesIdle,
    /// Many independent `TabBar`s of three tabs, idle (#670).
    TabBarsIdle,
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
    const ALL: [Self; 38] = [
        Self::BevyTexts,
        Self::PushButtons,
        Self::BevyNodes,
        Self::Tooltips,
        Self::VirtualList,
        Self::VirtualListScrolling,
        Self::VirtualLists,
        Self::FilterChipsOneChanging,
        Self::FilterChipsIdle,
        Self::ListNotesChanging,
        Self::ListNotesIdle,
        Self::SteppersIdle,
        Self::PaletteCombosIdle,
        Self::ThemeEditorsIdle,
        Self::ListItemsInitial,
        Self::ListItemsAdding,
        Self::ItemsSharedAdding,
        Self::PaletteItemsAdding,
        Self::TabItemsAdding,
        Self::SegmentItemsAdding,
        Self::SteppersChanging,
        Self::SteppersSharedChanging,
        Self::ToastsIdle,
        Self::ToastsBursting,
        Self::SegmentedControlsIdle,
        Self::IndicatorsAnimating,
        Self::IndicatorsRemoved,
        Self::GalleriesIdle,
        Self::TabBarsIdle,
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
            Self::ListNotesChanging => "list-notes-changing",
            Self::ListNotesIdle => "list-notes-idle",
            Self::ListItemsInitial => "list-items-initial",
            Self::ListItemsAdding => "list-items-adding",
            Self::ItemsSharedAdding => "items-shared-adding",
            Self::PaletteItemsAdding => "palette-items-adding",
            Self::TabItemsAdding => "tab-items-adding",
            Self::SegmentItemsAdding => "segment-items-adding",
            Self::SteppersIdle => "steppers-idle",
            Self::PaletteCombosIdle => "palette-combos-idle",
            Self::ThemeEditorsIdle => "theme-editors-idle",
            Self::SteppersChanging => "steppers-changing",
            Self::SteppersSharedChanging => "steppers-shared-changing",
            Self::ToastsIdle => "toasts-idle",
            Self::ToastsBursting => "toasts-bursting",
            Self::SegmentedControlsIdle => "segmented-controls-idle",
            Self::IndicatorsAnimating => "indicators-animating",
            Self::IndicatorsRemoved => "indicators-removed",
            Self::GalleriesIdle => "galleries-idle",
            Self::TabBarsIdle => "tab-bars-idle",
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
            Self::BevyNodes
            | Self::Tooltips
            | Self::ListItemsInitial
            | Self::ListItemsAdding
            | Self::PaletteItemsAdding
            | Self::TabItemsAdding
            | Self::SegmentItemsAdding
            | Self::ItemsSharedAdding => 5_000,
            Self::VirtualLists => 20,
            Self::FilterChipsOneChanging
            | Self::FilterChipsIdle
            | Self::ListNotesChanging
            | Self::ListNotesIdle
            | Self::ToastsIdle
            | Self::ToastsBursting
            | Self::SegmentedControlsIdle
            | Self::GalleriesIdle
            | Self::IndicatorsAnimating
            | Self::IndicatorsRemoved
            | Self::TabBarsIdle
            | Self::RovingGroupsFocus
            | Self::RovingGroupsSelection
            | Self::RovingGroupsIdle => 500,
            Self::SteppersIdle | Self::SteppersChanging | Self::SteppersSharedChanging => 2_000,
            Self::PaletteCombosIdle => 50,
            Self::ThemeEditorsIdle => 1,
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
        frame(&mut app, run.workload, run.size, 0);
    }

    let mut times = Vec::with_capacity(run.frames);
    for index in 0..run.frames {
        let start = Instant::now();
        frame(&mut app, run.workload, run.size, index);
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
fn frame(app: &mut App, workload: Workload, size: usize, index: usize) {
    if workload == Workload::ListItemsInitial {
        // A new list with all its items each frame, so every frame builds `size` views.
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<List>>();
        let old: Vec<Entity> = query.iter(world).collect();
        let parent = old
            .first()
            .and_then(|list| world.get::<ChildOf>(*list))
            .map(ChildOf::parent);
        for list in old {
            world.entity_mut(list).despawn();
        }
        if let Some(parent) = parent {
            world.spawn((List, list_items(size), ChildOf(parent)));
        }
    }
    if workload == Workload::ListItemsAdding {
        // One more item for the list's `ItemValues`, so one `ItemsChanged` per frame.
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<List>>();
        if let Some(list) = query.iter(world).next() {
            world.spawn((Item::new(format!("Added {index}")), ItemOf(list)));
        }
    }
    if workload == Workload::ItemsSharedAdding {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, (With<ItemValues>, Without<List>)>();
        if let Some(source) = query.iter(world).next() {
            world.spawn((Item::new(format!("Added {index}")), ItemOf(source)));
        }
    }
    if workload == Workload::PaletteItemsAdding {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<ColorPalette>>();
        if let Some(palette) = query.iter(world).next() {
            world.spawn((
                Item {
                    data: palette_color(index),
                },
                ItemOf(palette),
            ));
        }
    }
    if workload == Workload::TabItemsAdding {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<TabBar>>();
        if let Some(bar) = query.iter(world).next() {
            world.spawn((Item::new(format!("Added {index}")), ItemOf(bar)));
        }
    }
    if workload == Workload::SegmentItemsAdding {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<SegmentedControl>>();
        if let Some(control) = query.iter(world).next() {
            world.spawn((Item::new(format!("Added {index}")), ItemOf(control)));
        }
    }
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
    if workload == Workload::ListNotesChanging {
        // Marks one note changed without altering it, so the frame shows what finding the
        // affected rows costs and not a relayout.
        let world = app.world_mut();
        let mut query = world.query::<&mut ItemNote>();
        let count = query.iter(world).count();
        if count > 0
            && let Some(mut note) = query.iter_mut(world).nth(index * 7 % count)
        {
            note.set_changed();
        }
    }
    if matches!(
        workload,
        Workload::SteppersChanging | Workload::SteppersSharedChanging
    ) {
        // Marks `CHANGED_STEPPERS` values changed without moving them, so the frame shows what
        // the stepper systems cost and not a relayout of the root for a wider label.
        let world = app.world_mut();
        let mut query = world.query::<&mut Value>();
        let count = query.iter(world).count();
        for offset in 0..CHANGED_STEPPERS.min(count) {
            if let Some(mut value) = query.iter_mut(world).nth((index * 7 + offset * 13) % count) {
                value.set_changed();
            }
        }
    }
    if workload == Workload::ToastsBursting {
        // A burst of short toasts under other parents: duplicates are rare (the text has the
        // frame number), and each toast goes after a few frames, so the stacks stay.
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<ToastParent>>();
        let parents: Vec<Entity> = query.iter(world).collect();
        if !parents.is_empty() {
            for offset in 0..TOAST_BURST {
                if let Some(parent) = parents.get((index * 7 + offset * 13) % parents.len()) {
                    world.spawn((
                        Toast::new(format!("Burst {index}.{offset}")).with_duration(0.05),
                        ChildOf(*parent),
                    ));
                }
            }
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
    if workload == Workload::IndicatorsAnimating {
        // Moves the selection of one control to its next segment, so that control's indicator
        // slides over the following frames while the others stay settled.
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &ItemValues), With<SegmentedControl>>();
        let count = query.iter(world).count();
        let next = query
            .iter(world)
            .nth(index * 7 % count.max(1))
            .and_then(|(entity, items)| items.iter().nth(index % 3).map(|item| (entity, item)));
        if let Some((entity, item)) = next {
            world
                .entity_mut(entity)
                .insert(SelectionValue::One(item).into_component());
        }
    }
    if workload == Workload::IndicatorsRemoved {
        // Despawns the indicator of one control; the next frame makes it again.
        let world = app.world_mut();
        let mut indicators = world.query_filtered::<Entity, With<SegmentIndicator>>();
        let all: Vec<Entity> = indicators.iter(world).collect();
        if let Some(entity) = all.get(index * 7 % all.len().max(1)).copied() {
            world.despawn(entity);
        }
    }
    app.update();
}

/// Marks a parent node of the toast workloads.
#[derive(Component)]
struct ToastParent;

/// Toasts spawned per frame by `toasts-bursting`.
const TOAST_BURST: usize = 20;

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
            Workload::SteppersIdle | Workload::SteppersChanging => {
                for _ in 0..run.size {
                    if let Ok(value) = Value::new_u32(0, 0, u32::MAX, 1) {
                        root.spawn((NumericStepper::default(), value));
                    }
                }
            }
            Workload::SteppersSharedChanging => {
                for _ in 0..run.size.div_ceil(MIRRORS) {
                    let Ok(value) = Value::new_u32(0, 0, u32::MAX, 1) else {
                        continue;
                    };
                    let shared = root.spawn(value).id();
                    for _ in 0..MIRRORS {
                        root.spawn((NumericStepper::default(), ValueRef(shared)));
                    }
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
            Workload::ListNotesChanging | Workload::ListNotesIdle => {
                for row in 0..run.size {
                    root.spawn((
                        List,
                        Selection::builder()
                            .one()
                            .build_with_items((0..3).map(move |item| {
                                (
                                    Item::new(format!("Item {row}.{item}")),
                                    ItemNote::new(format!("Note {row}.{item}")),
                                )
                            })),
                    ));
                }
            }
            Workload::ListItemsInitial | Workload::ListItemsAdding => {
                root.spawn((List, list_items(run.size)));
            }
            Workload::PaletteCombosIdle => {
                let colors = root
                    .spawn(
                        Selection::builder()
                            .zero_or_one()
                            .build_with_items(COLORS.iter().map(|c| Item { data: c.color })),
                    )
                    .id();
                for _ in 0..run.size {
                    root.spawn((
                        ColorPaletteCombo,
                        Selection::builder()
                            .zero_or_one()
                            .build_with_items_ref(colors),
                    ));
                }
            }
            Workload::ThemeEditorsIdle => {
                for _ in 0..run.size {
                    root.spawn(ThemeEditor);
                }
            }
            Workload::ItemsSharedAdding => {
                let source = root.spawn(list_items(run.size)).id();
                // Hidden, so layout does not drown the cost of the item events.
                root.spawn(Node {
                    display: Display::None,
                    ..default()
                })
                .with_children(|hidden| {
                    for _ in 0..SHARED_LISTS {
                        hidden.spawn((
                            List,
                            Selection::builder().one().build_with_items_ref(source),
                        ));
                    }
                });
            }
            Workload::ToastsIdle | Workload::ToastsBursting => {
                for row in 0..run.size {
                    root.spawn((Node::default(), ToastParent))
                        .with_child(Toast::new(format!("Long {row}")).with_duration(86_400.0));
                }
            }
            Workload::PaletteItemsAdding => {
                root.spawn((
                    ColorPalette,
                    Selection::builder()
                        .one()
                        .build_with_items((0..run.size).map(|i| Item {
                            data: palette_color(i),
                        })),
                ));
            }
            Workload::TabItemsAdding => {
                root.spawn((TabBar, list_items(run.size)));
            }
            Workload::SegmentItemsAdding => {
                root.spawn((SegmentedControl, list_items(run.size)));
            }
            Workload::SegmentedControlsIdle
            | Workload::IndicatorsAnimating
            | Workload::IndicatorsRemoved => {
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
            Workload::TabBarsIdle => {
                for row in 0..run.size {
                    root.spawn((
                        TabBar,
                        Selection::builder().one().build_with_items(
                            (0..3).map(move |tab| Item::new(format!("Tab {row}.{tab}"))),
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

/// How many lists share the source of `items-shared-adding`.
const SHARED_LISTS: usize = 8;
/// A colour that differs by index, for the palette workload.
#[expect(clippy::cast_precision_loss, reason = "a bench colour")]
const fn palette_color(index: usize) -> Color {
    Color::hsl((index % 360) as f32, 0.7, 0.5)
}

/// A `One` selection over `size` items, for the list workloads.
fn list_items(size: usize) -> impl Bundle {
    Selection::builder()
        .one()
        .build_with_items((0..size).map(|item| Item::new(format!("Item {item}"))))
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
