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
use bevy::ui::Pressed;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::badge::Badge;
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::color_palette_combo::ColorPaletteCombo;
use bevy_pluim::filter_chips::{Chip, FilterChips};
use bevy_pluim::gallery::{Gallery, GalleryCard};
use bevy_pluim::icon_button::{IconButton, IconButtonSize};
use bevy_pluim::list::{ItemNote, List};
use bevy_pluim::numeric_stepper::{NumericStepper, Value, ValueRef};
use bevy_pluim::panel::Panel;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::scroll_area::{ScrollArea, ScrollContents};
use bevy_pluim::segmented_control::{Segment, SegmentIndicator, SegmentedControl};
use bevy_pluim::select::{Item, ItemOf, ItemValues, Selection, SelectionValue};
use bevy_pluim::setting_row::{SettingRow, SettingRows};
use bevy_pluim::slider::Slider;
use bevy_pluim::tab_bar::TabBar;
use bevy_pluim::tailwind::COLORS;
use bevy_pluim::theme_editor::ThemeEditor;
use bevy_pluim::toast::Toast;
use bevy_pluim::toolbar::{Tool, Toolbar, ToolbarStyle};
use bevy_pluim::tooltip::Tooltip;
use bevy_pluim::virtual_list::{RowHeight, VirtualList};
use bevy_stagecraft::HeadlessPlugins;

/// Frames played before timing starts, so the first layout and spawns settle.
const WARMUP_FRAMES: usize = 60;
/// Frames after the spawn that count towards the startup time (the widgets are built by then).
const STARTUP_FRAMES: usize = 3;
/// Frames timed unless `--frames` says otherwise.
const DEFAULT_FRAMES: usize = 300;
/// Values changed per frame by the stepper workloads.
const CHANGED_STEPPERS: usize = 20;

/// How many `List` selections the `lists-selection-changing` workload replaces every frame.
const CHANGED_LISTS: usize = 50;
/// Steppers mirroring each shared value in `steppers-shared-changing`.
const MIRRORS: usize = 10;
/// Wheel messages sent per frame by `wheel-deep`, as a trackpad sends at a high rate.
const WHEEL_EVENTS: usize = 16;
/// Entities of the deep tree the fake pointers hover (the leaf and its nearest ancestors).
const HOVERED_ANCESTORS: usize = 8;

/// Marks an entity of the wheel tree that the fake pointers hover.
#[derive(Component)]
struct WheelHovered;

/// Puts the marked entities in the hover map of two pointers, after picking has
/// rebuilt it (there is no real pointer in a headless run).
fn fake_hover(
    hovered: Query<Entity, With<WheelHovered>>,
    mut hover_map: ResMut<bevy::picking::hover::HoverMap>,
) {
    use bevy::picking::backend::HitData;
    use bevy::picking::pointer::PointerId;
    for pointer in [PointerId::Mouse, PointerId::Touch(0)] {
        let map = hover_map.entry(pointer).or_default();
        for entity in &hovered {
            map.insert(entity, HitData::new(Entity::PLACEHOLDER, 0.0, None, None));
        }
    }
}

/// Frames the fake pointer rests on one tooltip host: longer than the tooltip
/// delay (0.5 s is 30 frames at 60 per second), so each popup spawns, is laid
/// out and placed before the pointer moves on and despawns it.
const HOVER_REST_FRAMES: usize = 45;

/// Moves a fake mouse pointer over the `Tooltip` hosts one after the other,
/// resting on each for `HOVER_REST_FRAMES`, after picking has rebuilt the hover map and before it updates `Hovered`.
fn hover_tooltip_hosts(
    hosts: Query<Entity, With<Tooltip>>,
    mut hover_map: ResMut<bevy::picking::hover::HoverMap>,
    mut order: Local<Vec<Entity>>,
    mut frame: Local<usize>,
) {
    use bevy::picking::backend::HitData;
    use bevy::picking::pointer::PointerId;
    if order.is_empty() {
        order.extend(&hosts);
    }
    let step = *frame / HOVER_REST_FRAMES;
    *frame += 1;
    if order.is_empty() {
        return;
    }
    let index = step % order.len();
    if let Some(&host) = order.get(index) {
        hover_map
            .entry(PointerId::Mouse)
            .or_default()
            .insert(host, HitData::new(Entity::PLACEHOLDER, 0.0, None, None));
    }
}

/// What is spawned and what happens each frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Workload {
    /// Plain Bevy nodes with a text child: the baseline for the buttons.
    BevyTexts,
    /// Plain Bevy nodes: the baseline for the tooltips.
    BevyNodes,
    /// A node with a `Text` child: the baseline for the push buttons that has
    /// their parent and child entities.
    BevyTextChildren,
    /// `PushButton`s, idle.
    PushButtons,
    /// Nodes with a `Tooltip`, idle (no pointer).
    Tooltips,
    /// Nodes with a `Tooltip`, the pointer moving over them one at a time and
    /// resting long enough for each popup to show: hover and popup cost.
    TooltipsHovering,
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
    /// The same `FilterChips`; one row's selection moves to its next chip every frame, so only
    /// that row is restyled (#709).
    FilterChipsSelectionChanging,
    /// Many independent `List`s of three noted items; one note is marked changed every frame.
    ListNotesChanging,
    /// The same lists, idle: the floor the changing workload is read against.
    ListNotesIdle,
    /// Many independent `List`s of three items; `CHANGED_LISTS` selections are replaced every
    /// frame, so the lists' selection sync shows (#671).
    ListsSelectionChanging,
    /// The same lists, idle: the floor the changing workload is read against.
    ListsSelectionIdle,
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
    /// Many `IconButton`s each with a `Badge`, idle (#685).
    BadgesIdle,
    /// The same buttons; `CHANGED_STEPPERS` badges get a new count every frame (#685).
    BadgesChanging,
    /// Many `Panel`s, idle (#685).
    PanelsIdle,
    /// The same panels; `CHANGED_STEPPERS` are inserted again every frame (#685).
    PanelsChanging,
    /// Many `SettingRows` columns of five rows, idle (#689).
    SettingRowsIdle,
    /// The same columns; in `CHANGED_STEPPERS` of them a row is removed at the end and a new one
    /// inserted at the front every frame (#689).
    SettingRowsChanging,
    /// Many independent `SegmentedControl`s of three segments, idle.
    SegmentedControlsIdle,
    /// The same controls; one control's selection moves to its next segment every frame, and
    /// only that control is restyled (#709).
    SegmentedControlsSelectionChanging,
    /// Many independent `SegmentedControl`s; one control's selection moves to its next segment
    /// every frame, so its indicator slides (#677).
    IndicatorsAnimating,
    /// The same controls; one control's indicator is despawned every frame and must be made again
    /// (#677).
    IndicatorsRemoved,
    /// Many independent `Gallery`s of three cards, idle.
    GalleriesIdle,
    /// One `FilterChips` of `size` chips; every frame its first item goes and a new one is added.
    FilterChipsEditing,
    /// One `Toolbar` of `size` tools, edited like `FilterChipsEditing`.
    ToolbarEditing,
    /// One `Gallery` of `size` cards, edited like `FilterChipsEditing`.
    GalleryEditing,
    /// Many independent `TabBar`s of three tabs, idle (#670).
    TabBarsIdle,
    /// Many deep chains of nodes with a button at the bottom, idle (#684).
    DeepTreesIdle,
    /// The same chains; two nested nodes of one chain flip `Display::None` every frame, so
    /// their subtrees overlap (#684).
    DeepTreesDisplay,
    /// The same chains; one chain moves to another parent every frame (#684).
    DeepTreesReparenting,
    /// Many independent `SegmentedControl`s (roving-focus groups); the focus moves to
    /// another control's segment every frame.
    RovingGroupsFocus,
    /// The same controls; one control's selection is replaced every frame.
    RovingGroupsSelection,
    /// The same controls, idle: the floor the two roving workloads are read against.
    RovingGroupsIdle,
    /// Wheel input over a deep, nested scroll tree: `WHEEL_EVENTS` trackpad messages a frame for
    /// two pointers (#682).
    WheelDeep,
    /// The same tree and hover map with no wheel input: the floor `wheel-deep` is read against.
    WheelDeepIdle,
    /// Many `Slider`s, idle.
    SlidersIdle,
    /// The same sliders; one is pressed and the one before it released every frame.
    SlidersPressing,
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
    const ALL: [Self; 60] = [
        Self::BevyTexts,
        Self::BevyTextChildren,
        Self::PushButtons,
        Self::BevyNodes,
        Self::Tooltips,
        Self::TooltipsHovering,
        Self::VirtualList,
        Self::VirtualListScrolling,
        Self::VirtualLists,
        Self::FilterChipsOneChanging,
        Self::FilterChipsIdle,
        Self::FilterChipsSelectionChanging,
        Self::ListNotesChanging,
        Self::ListNotesIdle,
        Self::ListsSelectionChanging,
        Self::ListsSelectionIdle,
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
        Self::BadgesIdle,
        Self::BadgesChanging,
        Self::PanelsIdle,
        Self::PanelsChanging,
        Self::SettingRowsIdle,
        Self::SettingRowsChanging,
        Self::SegmentedControlsIdle,
        Self::SegmentedControlsSelectionChanging,
        Self::IndicatorsAnimating,
        Self::IndicatorsRemoved,
        Self::GalleriesIdle,
        Self::FilterChipsEditing,
        Self::ToolbarEditing,
        Self::GalleryEditing,
        Self::TabBarsIdle,
        Self::DeepTreesIdle,
        Self::DeepTreesDisplay,
        Self::DeepTreesReparenting,
        Self::RovingGroupsFocus,
        Self::RovingGroupsSelection,
        Self::RovingGroupsIdle,
        Self::WheelDeep,
        Self::WheelDeepIdle,
        Self::SlidersIdle,
        Self::SlidersPressing,
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
            Self::BevyTextChildren => "bevy-text-children",
            Self::PushButtons => "push-buttons",
            Self::Tooltips => "tooltips",
            Self::TooltipsHovering => "tooltips-hovering",
            Self::VirtualList => "virtual-list",
            Self::VirtualListScrolling => "virtual-list-scrolling",
            Self::VirtualLists => "virtual-lists",
            Self::FilterChipsOneChanging => "filter-chips-one-changing",
            Self::FilterChipsIdle => "filter-chips-idle",
            Self::FilterChipsSelectionChanging => "filter-chips-selection-changing",
            Self::ListNotesChanging => "list-notes-changing",
            Self::ListNotesIdle => "list-notes-idle",
            Self::ListsSelectionChanging => "lists-selection-changing",
            Self::ListsSelectionIdle => "lists-selection-idle",
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
            Self::BadgesIdle => "badges-idle",
            Self::BadgesChanging => "badges-changing",
            Self::PanelsIdle => "panels-idle",
            Self::PanelsChanging => "panels-changing",
            Self::SettingRowsIdle => "setting-rows-idle",
            Self::SettingRowsChanging => "setting-rows-changing",
            Self::SegmentedControlsIdle => "segmented-controls-idle",
            Self::SegmentedControlsSelectionChanging => "segmented-controls-selection-changing",
            Self::IndicatorsAnimating => "indicators-animating",
            Self::IndicatorsRemoved => "indicators-removed",
            Self::GalleriesIdle => "galleries-idle",
            Self::FilterChipsEditing => "filter-chips-editing",
            Self::ToolbarEditing => "toolbar-editing",
            Self::GalleryEditing => "gallery-editing",
            Self::TabBarsIdle => "tab-bars-idle",
            Self::DeepTreesIdle => "deep-trees-idle",
            Self::DeepTreesDisplay => "deep-trees-display",
            Self::DeepTreesReparenting => "deep-trees-reparenting",
            Self::RovingGroupsFocus => "roving-groups-focus",
            Self::RovingGroupsSelection => "roving-groups-selection",
            Self::RovingGroupsIdle => "roving-groups-idle",
            Self::WheelDeep => "wheel-deep",
            Self::WheelDeepIdle => "wheel-deep-idle",
            Self::SlidersIdle => "sliders-idle",
            Self::SlidersPressing => "sliders-pressing",
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
            Self::BevyTexts | Self::BevyTextChildren | Self::PushButtons => 1_000,
            Self::BevyNodes
            | Self::Tooltips
            | Self::TooltipsHovering
            | Self::ListItemsInitial
            | Self::ListItemsAdding
            | Self::PaletteItemsAdding
            | Self::TabItemsAdding
            | Self::SegmentItemsAdding
            | Self::ItemsSharedAdding => 5_000,
            Self::VirtualLists => 20,
            Self::FilterChipsEditing
            | Self::ToolbarEditing
            | Self::GalleryEditing
            | Self::WheelDeep
            | Self::WheelDeepIdle => 200,
            Self::FilterChipsOneChanging
            | Self::FilterChipsIdle
            | Self::FilterChipsSelectionChanging
            | Self::ListNotesChanging
            | Self::ListNotesIdle
            | Self::ListsSelectionChanging
            | Self::ListsSelectionIdle
            | Self::ToastsIdle
            | Self::ToastsBursting
            | Self::BadgesIdle
            | Self::BadgesChanging
            | Self::PanelsIdle
            | Self::PanelsChanging
            | Self::SettingRowsIdle
            | Self::SettingRowsChanging
            | Self::SegmentedControlsIdle
            | Self::SegmentedControlsSelectionChanging
            | Self::GalleriesIdle
            | Self::IndicatorsAnimating
            | Self::IndicatorsRemoved
            | Self::TabBarsIdle
            | Self::DeepTreesIdle
            | Self::DeepTreesDisplay
            | Self::DeepTreesReparenting
            | Self::RovingGroupsFocus
            | Self::RovingGroupsSelection
            | Self::RovingGroupsIdle => 500,
            Self::SteppersIdle
            | Self::SteppersChanging
            | Self::SteppersSharedChanging
            | Self::SlidersIdle
            | Self::SlidersPressing => 2_000,
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
            ("--workload", Some(name)) => match Workload::parse(&name) {
                Some(parsed) => workload = Some(parsed),
                None => return unknown_workload(&name),
            },
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

/// Reports a `--workload` name that is none of the workloads and lists the valid ones.
fn unknown_workload(name: &str) -> ExitCode {
    let names: Vec<_> = Workload::ALL.iter().map(|w| w.name()).collect();
    let _ = writeln!(
        std::io::stderr().lock(),
        "unknown workload `{name}`; valid workloads: {}",
        names.join(", ")
    );
    ExitCode::from(2)
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
        "| workload | size | startup ms | mean us | median us | p95 us | max us | entities | memory added MiB |"
    );
    let _ = writeln!(
        out,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
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
                let _ = writeln!(out, "| {} | failed | | | | | | | |", workload.name());
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
    if run.workload == Workload::TooltipsHovering {
        app.add_systems(
            PreUpdate,
            hover_tooltip_hosts
                .after(bevy::picking::hover::generate_hovermap)
                .before(bevy::picking::hover::update_is_hovered),
        );
    }
    if matches!(run.workload, Workload::WheelDeep | Workload::WheelDeepIdle) {
        app.add_systems(
            PreUpdate,
            fake_hover.after(bevy::picking::PickingSystems::Hover),
        );
    }
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

    // Startup: spawning the workload and its first frames, which build the widgets.
    let startup = Instant::now();
    spawn(app.world_mut(), run);
    for _ in 0..STARTUP_FRAMES {
        frame(&mut app, run.workload, run.size, 0);
    }
    let startup = startup.elapsed();
    for _ in STARTUP_FRAMES..WARMUP_FRAMES {
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
        "| {} | {} | {:.1} | {:.0} | {:.0} | {:.0} | {:.0} | {} | {:.1} |",
        run.workload.name(),
        run.size,
        startup.as_secs_f64() * 1e3,
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
    if workload == Workload::FilterChipsEditing {
        edit_items::<FilterChips, _>(app, |index| Chip::new(format!("Added {index}")), index);
    }
    if workload == Workload::ToolbarEditing {
        edit_items::<Toolbar, _>(
            app,
            |index| Tool::in_group(Handle::default(), format!("Added {index}"), 0),
            index,
        );
    }
    if workload == Workload::GalleryEditing {
        edit_items::<Gallery, _>(
            app,
            |index| GalleryCard::new(Handle::default(), format!("Added {index}"), "Group"),
            index,
        );
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
    if workload == Workload::WheelDeep {
        for event in 0..WHEEL_EVENTS {
            let sign = if (index + event).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            app.world_mut()
                .write_message(bevy::input::mouse::MouseWheel {
                    unit: bevy::input::mouse::MouseScrollUnit::Pixel,
                    x: 0.0,
                    y: sign,
                    window: Entity::PLACEHOLDER,
                    phase: bevy::input::touch::TouchPhase::Moved,
                });
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
    if workload == Workload::ListsSelectionChanging {
        // Replaces the selection of `CHANGED_LISTS` lists (an immutable component) with a
        // clone, so `Changed<Selection>` fires for those lists alone.
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Selection), With<List>>();
        let count = query.iter(world).count();
        let picked: Vec<_> = (0..CHANGED_LISTS.min(count))
            .filter_map(|offset| {
                query
                    .iter(world)
                    .nth((index * 7 + offset * 13) % count)
                    .map(|(entity, selection)| (entity, selection.clone()))
            })
            .collect();
        for (entity, selection) in picked {
            world.entity_mut(entity).insert(selection);
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
    if workload == Workload::BadgesChanging {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Badge>>();
        let hosts: Vec<Entity> = query.iter(world).collect();
        for offset in 0..CHANGED_STEPPERS.min(hosts.len()) {
            if let Some(host) = hosts.get((index * 7 + offset * 13) % hosts.len()) {
                world.entity_mut(*host).insert(Badge::count(index + offset));
            }
        }
    }
    if workload == Workload::PanelsChanging {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Panel>>();
        let panels: Vec<Entity> = query.iter(world).collect();
        for offset in 0..CHANGED_STEPPERS.min(panels.len()) {
            if let Some(panel) = panels.get((index * 7 + offset * 13) % panels.len()) {
                world
                    .entity_mut(*panel)
                    .insert(Panel::new(format!("Panel {index}.{offset}")));
            }
        }
    }
    if workload == Workload::SettingRowsChanging {
        // Removes the last row of a column and inserts a new one at its front, so the dividers
        // of that column are looked at again and no other column is.
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<SettingRows>>();
        let columns: Vec<Entity> = query.iter(world).collect();
        for offset in 0..CHANGED_STEPPERS.min(columns.len()) {
            let Some(column) = columns.get((index * 7 + offset * 13) % columns.len()) else {
                continue;
            };
            let last = world
                .get::<Children>(*column)
                .and_then(|children| children.last().copied());
            if let Some(last) = last {
                world.entity_mut(last).despawn();
            }
            let row = world
                .spawn(SettingRow::new(
                    format!("Row {index}.{offset}"),
                    "Description",
                ))
                .id();
            world.entity_mut(*column).insert_children(0, &[row]);
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
    if workload == Workload::FilterChipsSelectionChanging {
        // Moves the selection of one row to its next chip: only that row is restyled.
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &ItemValues), With<FilterChips>>();
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
    if matches!(
        workload,
        Workload::IndicatorsAnimating | Workload::SegmentedControlsSelectionChanging
    ) {
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
    if workload == Workload::DeepTreesDisplay {
        // Two nested nodes of one chain flip together, so two dirty roots overlap.
        let world = app.world_mut();
        let mut query = world.query::<(&DeepChain, &mut Node)>();
        let chain = index * 7 % size.max(1);
        let display = if index.is_multiple_of(2) {
            Display::None
        } else {
            Display::Flex
        };
        for (marker, mut node) in query.iter_mut(world) {
            if marker.chain == chain && marker.flips {
                node.display = display;
            }
        }
    }
    if workload == Workload::DeepTreesReparenting {
        // One chain moves to the other of two parents.
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &DeepChain, &ChildOf)>();
        let moved: Vec<(Entity, Entity)> = query
            .iter(world)
            .filter(|(_, marker, _)| marker.chain == index * 7 % size.max(1) && marker.top)
            .map(|(entity, _, parent)| (entity, parent.parent()))
            .collect();
        let mut spares = world.query_filtered::<Entity, With<DeepSpare>>();
        let spares: Vec<Entity> = spares.iter(world).collect();
        for (entity, parent) in moved {
            if let Some(other) = spares.iter().copied().find(|spare| *spare != parent) {
                world.entity_mut(entity).insert(ChildOf(other));
            }
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
    if workload == Workload::SlidersPressing {
        // Presses one slider and releases the one pressed the frame before, so the
        // frame shows what restyling costs next to the idle sliders.
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Slider>>();
        let sliders: Vec<Entity> = query.iter(world).collect();
        if !sliders.is_empty() {
            if let Some(&last) = sliders.get(index.wrapping_sub(1) * 7 % sliders.len()) {
                world.entity_mut(last).remove::<Pressed>();
            }
            if let Some(&next) = sliders.get(index * 7 % sliders.len()) {
                world.entity_mut(next).insert(Pressed);
            }
        }
    }
    app.update();
}

/// Nodes in a chain of the deep-tree workloads, each with a button at the bottom.
const DEEP_DEPTH: usize = 40;

/// Marks the nodes of a chain that a deep-tree workload changes.
#[derive(Component)]
struct DeepChain {
    chain: usize,
    top: bool,
    flips: bool,
}

/// Marks a parent a deep-tree workload moves chains between.
#[derive(Component)]
struct DeepSpare;

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
            Workload::BevyTextChildren => {
                for index in 0..run.size {
                    root.spawn(Node::default())
                        .with_child(Text::new(format!("Button {index}")));
                }
            }
            Workload::PushButtons => {
                for index in 0..run.size {
                    root.spawn(PushButton::from_label(format!("Button {index}")));
                }
            }
            Workload::Tooltips | Workload::TooltipsHovering => {
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
            Workload::SlidersIdle | Workload::SlidersPressing => {
                for _ in 0..run.size {
                    let Ok(slider) = Slider::new(0.5, 0.0, 1.0) else {
                        return;
                    };
                    root.spawn(slider);
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
            Workload::WheelDeep | Workload::WheelDeepIdle => {
                wheel_tree(root, run.size);
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
            Workload::FilterChipsOneChanging
            | Workload::FilterChipsIdle
            | Workload::FilterChipsSelectionChanging => {
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
            Workload::ListsSelectionChanging | Workload::ListsSelectionIdle => {
                for row in 0..run.size {
                    root.spawn((
                        List,
                        Selection::builder().one().build_with_items(
                            (0..3).map(move |item| Item::new(format!("Item {row}.{item}"))),
                        ),
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
            Workload::BadgesIdle | Workload::BadgesChanging => {
                for row in 0..run.size {
                    root.spawn((
                        IconButton::new(
                            Handle::default(),
                            format!("Button {row}"),
                            IconButtonSize::Regular,
                        ),
                        Badge::count(row),
                    ));
                }
            }
            Workload::PanelsIdle | Workload::PanelsChanging => {
                for row in 0..run.size {
                    root.spawn(Panel::new(format!("Panel {row}")));
                }
            }
            Workload::SettingRowsIdle | Workload::SettingRowsChanging => {
                for column in 0..run.size {
                    root.spawn(SettingRows).with_children(|rows| {
                        for row in 0..5 {
                            rows.spawn(SettingRow::new(
                                format!("Row {column}.{row}"),
                                "Description",
                            ));
                        }
                    });
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
            | Workload::SegmentedControlsSelectionChanging
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
            Workload::DeepTreesIdle
            | Workload::DeepTreesDisplay
            | Workload::DeepTreesReparenting => {}
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
            Workload::FilterChipsEditing => {
                root.spawn((
                    FilterChips,
                    Selection::builder()
                        .one()
                        .build_with_items((0..run.size).map(|chip| Item {
                            data: Chip::new(format!("Chip {chip}")),
                        })),
                ));
            }
            Workload::ToolbarEditing => {
                root.spawn((
                    Toolbar::new(ToolbarStyle::Filled),
                    Selection::builder()
                        .one()
                        .build_with_items((0..run.size).map(|tool| Item {
                            data: Tool::in_group(
                                Handle::default(),
                                format!("Tool {tool}"),
                                (tool / 4) as u32,
                            ),
                        })),
                ));
            }
            Workload::GalleryEditing => {
                root.spawn((
                    Gallery,
                    Selection::builder()
                        .one()
                        .build_with_items((0..run.size).map(|card| Item {
                            data: GalleryCard::new(
                                Handle::default(),
                                format!("Card {card}"),
                                format!("Group {}", card / 8),
                            ),
                        })),
                ));
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
    if matches!(
        run.workload,
        Workload::DeepTreesIdle | Workload::DeepTreesDisplay | Workload::DeepTreesReparenting
    ) {
        deep_trees(world, root, run.size);
    }
}

/// `chains` chains of `DEEP_DEPTH` nodes under one of two spare parents, each with a button at
/// the bottom.
fn deep_trees(world: &mut World, root: Entity, chains: usize) {
    let spare = world
        .spawn((Node::default(), DeepSpare, ChildOf(root)))
        .id();
    world.spawn((Node::default(), DeepSpare, ChildOf(root)));
    for chain in 0..chains {
        let mut parent = spare;
        for depth in 0..DEEP_DEPTH {
            let node = world.spawn((Node::default(), ChildOf(parent))).id();
            if depth == 0 || depth == 2 || depth == DEEP_DEPTH / 2 {
                world.entity_mut(node).insert(DeepChain {
                    chain,
                    top: depth == 0,
                    flips: depth != 0,
                });
            }
            parent = node;
        }
        world.spawn((
            PushButton::from_label(format!("Deep {chain}")),
            ChildOf(parent),
        ));
    }
}

/// How many lists share the source of `items-shared-adding`.
const SHARED_LISTS: usize = 8;
/// A colour that differs by index, for the palette workload.
#[expect(clippy::cast_precision_loss, reason = "a bench colour")]
const fn palette_color(index: usize) -> Color {
    Color::hsl((index % 360) as f32, 0.7, 0.5)
}

/// A `One` selection over `size` items, for the list workloads.
/// Despawns the first item of the one widget `W` and adds a new item last, so the item count
/// stays `size` and one small structural change reaches the widget every frame.
fn edit_items<W: Component, T: Send + Sync + 'static>(
    app: &mut App,
    new: impl FnOnce(usize) -> T,
    index: usize,
) {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(Entity, &ItemValues), With<W>>();
    let Some((widget, items)) = query.iter(world).next() else {
        return;
    };
    let first = items.iter().next();
    if let Some(first) = first {
        world.entity_mut(first).despawn();
    }
    world.spawn((Item { data: new(index) }, ItemOf(widget)));
}

fn list_items(size: usize) -> impl Bundle {
    Selection::builder()
        .one()
        .build_with_items((0..size).map(|item| Item::new(format!("Item {item}"))))
}

/// A vertical scroll area whose contents are a chain of `depth` nested nodes; halfway down is
/// a nested horizontal scroll area holding the rest, ending in a leaf larger than both areas.
/// The leaf, its nearest ancestors and one node of the outer area are `WheelHovered`.
fn wheel_tree(parent: &mut ChildSpawner, depth: usize) {
    parent.spawn((
        Node {
            width: px(400),
            height: px(400),
            ..default()
        },
        ScrollArea::bundle(
            move |area: &mut ChildSpawner| wheel_chain(area, depth / 2, depth, true),
            true,
            false,
        ),
    ));
}

fn wheel_chain(parent: &mut ChildSpawner, left: usize, depth: usize, outer: bool) {
    let mut node = parent.spawn(Node {
        flex_direction: FlexDirection::Column,
        ..default()
    });
    if left == 0 && outer {
        // Hover one node of the outer area, so both areas are wheel targets.
        node.insert(WheelHovered);
        node.with_children(|child| {
            child.spawn((
                Node {
                    width: px(300),
                    height: px(300),
                    ..default()
                },
                ScrollArea::bundle(
                    move |area: &mut ChildSpawner| wheel_chain(area, depth / 2, depth, false),
                    false,
                    true,
                ),
            ));
        });
    } else if left == 0 {
        node.insert(WheelHovered);
        node.with_children(|child| {
            child.spawn(Node {
                width: px(2_000),
                height: px(2_000),
                ..default()
            });
        });
    } else {
        if !outer && left <= HOVERED_ANCESTORS {
            node.insert(WheelHovered);
        }
        node.with_children(|child| wheel_chain(child, left - 1, depth, outer));
    }
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
