//! The HUD of the redesign, built from this crate's widgets over sample data.
//!
//! Shared by the `hud` example and the headless smoke test in `tests/headless.rs`: add
//! [`HudPlugin`] to an app with `WidgetsPlugin` and an `Assets<Image>`. Nothing here knows a
//! game; the trains, stops and warnings are constants and the wiring between the widgets is
//! plain systems and observers.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private module"
)]

mod icons;

use bevy::asset::HandleTemplate;
use bevy::ecs::system::SystemId;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::{DEFAULT_REM_SIZE_PX, RemSize};
use bevy::ui_widgets::Activate;
use bevy::ui_widgets::{TextInput, ValueChange as SliderChange};
use icons::Icons;
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::badge::Badge;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::divider::Divider;
use rin_bevy_ui::filter_chips::{Chip, FilterChips};
use rin_bevy_ui::gallery::{Gallery, GalleryCard};
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::kbd::Kbd;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::list::{CornerStyle, ItemView, ItemViewCreator, ItemViewSpawner, List};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value, ValueChange, ValueFormat};
use rin_bevy_ui::panel::{Panel, PanelClosed, PanelRefs};
use rin_bevy_ui::pill::{Pill, PillTone};
use rin_bevy_ui::push_button::{ButtonSize, PushButton};
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::segmented_control::SegmentedControl;
use rin_bevy_ui::select::{Item, ItemOf, Selection, SelectionParam, SelectionValue};
use rin_bevy_ui::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use rin_bevy_ui::slider::Slider;
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::text_field::{TextField, TextFieldCommit};
use rin_bevy_ui::theme::{Theme, ThemeMode};
use rin_bevy_ui::toast::Toast;
use rin_bevy_ui::toolbar::{Tool, Toolbar, ToolbarLabels, ToolbarStyle};

/// Spawns the HUD at startup and wires its widgets together.
#[derive(Debug, Clone, Copy)]
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_panel_closed)
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    fill_panels,
                    fill_setting_rows,
                    apply_preferences,
                    sync_run_state,
                    sync_warnings_panel,
                    filter_train_rows,
                    hotkeys,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrainStatus {
    Running,
    Waiting,
    Problem,
}

impl TrainStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Waiting",
            Self::Problem => "Problem",
        }
    }

    const fn tone(self) -> PillTone {
        match self {
            Self::Running => PillTone::Info,
            Self::Waiting => PillTone::Warning,
            Self::Problem => PillTone::Problem,
        }
    }
}

#[derive(Clone, Copy)]
struct Train {
    name: &'static str,
    route: &'static str,
    status: TrainStatus,
}

const TRAINS: [Train; 4] = [
    Train {
        name: "Northern Express",
        route: "Harbour - Ridge Junction",
        status: TrainStatus::Running,
    },
    Train {
        name: "Coal 12",
        route: "Mine - Smelter",
        status: TrainStatus::Waiting,
    },
    Train {
        name: "Valley Local",
        route: "Ridge Junction - Marsh End",
        status: TrainStatus::Running,
    },
    Train {
        name: "Freight 7",
        route: "Smelter - Harbour",
        status: TrainStatus::Problem,
    },
];

const STOPS: [(&str, &str); 6] = [
    ("Harbour", "2 platforms"),
    ("Ridge Junction", "3 platforms"),
    ("Mine", "1 platform"),
    ("Smelter", "2 platforms"),
    ("Marsh End", "1 platform"),
    ("Old Quarry", "1 platform"),
];

const WARNINGS: [(&str, &str); 3] = [
    ("Freight 7 is stuck", "No path from Smelter to Harbour"),
    ("Coal 12 is waiting", "Signal at Mine is red"),
    ("Marsh End is empty", "No train serves this stop"),
];

/// What a panel of the panel toolbar shows.
#[derive(Component, Clone, Copy)]
enum PanelKind {
    Trains,
    Stops,
    Schedules,
    Preferences,
    Warnings,
}

/// A setting of the Preferences panel.
#[derive(Component, Clone, Copy)]
enum Pref {
    Appearance,
    WarningsButton,
    ToolLabels,
    TextSize,
    CompanyName,
}

#[derive(Component, Default, Clone)]
struct PauseButton;

#[derive(Component, Default, Clone)]
struct RunLabel;

#[derive(Component, Default, Clone)]
struct WarningsButton;

#[derive(Component, Default, Clone)]
struct WarningsPanel;

#[derive(Component, Default, Clone)]
struct Dock;

/// The chips that filter a list of trains.
#[derive(Component)]
struct TrainFilter(Entity);

/// The creator of the rows of the trains list.
#[derive(Resource)]
struct TrainRows(SystemId<In<ItemViewSpawner>>);

/// The selection of the panel toolbar.
#[derive(Resource)]
struct PanelBar(Entity);

/// The selection and the tools of the tool dock.
#[derive(Resource)]
struct DockTools {
    selection: Entity,
    tools: Vec<Entity>,
}

fn spawn_hud(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut theme: ResMut<Theme>) {
    let icons = Icons::draw(&mut images);
    theme.set_mode(ThemeMode::Dark);
    commands.spawn(Camera2d);
    let train_rows = commands.register_system(train_row);
    commands.insert_resource(TrainRows(train_rows));
    commands.insert_resource(SimulationSpeed(1.0));

    let bar = spawn_panel_bar(&mut commands, &icons);
    let tools = spawn_dock_tools(&mut commands, &icons);
    let run_state = run_state(&icons);
    let panels = panels(bar, &icons);
    let dock = dock(tools);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
        }
        Children [
            @{run_state}
            --
            @{panels}
            --
            @{dock}
        ]
    });
    commands.insert_resource(icons);
}

/// Top left: the pause button, what the game is doing and the key that toggles it.
fn run_state(icons: &Icons) -> impl Scene {
    let pause = icons.pause.clone();
    bsn! {
        ~{Card}
        Node {
            position_type: PositionType::Absolute,
            left: px(16.0),
            top: px(16.0),
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            padding: UiRect::axes(px(8.0), px(6.0)),
        }
        Children [
            IconButton { icon: pause, name: "Pause", size: IconButtonSize::Regular }
            ~{ToggleState(false)}
            PauseButton
            --
            ~{Label::regular("Running")}
            RunLabel
            --
            ~{Kbd::new("P")}
            --
            ~{Slider::new(0.0, SPEED_MIN_OCTAVE, SPEED_MAX_OCTAVE)
                .with_step(1.0)
                .with_format(speed_label)
                .with_name("Speed")}
            Node { width: px(120.0) }
            on(on_speed_changed)
        ]
    }
}

/// The slowest speed, in octaves (powers of two) from normal: 0.25x.
const SPEED_MIN_OCTAVE: f32 = -2.0;

/// The fastest speed, in octaves from normal: 8x.
const SPEED_MAX_OCTAVE: f32 = 3.0;

/// The simulation speed the example's slider sets, as a multiple of normal.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
struct SimulationSpeed(f32);

/// The speed a slider position in octaves stands for.
fn speed_of(octaves: f32) -> f32 {
    octaves.exp2()
}

/// The slider's text: "0.25x", "1x", "8x".
fn speed_label(octaves: f32) -> String {
    format!("{}x", speed_of(octaves))
}

/// Keeps the speed the slider was moved to.
fn on_speed_changed(change: On<SliderChange<f32>>, mut speed: ResMut<SimulationSpeed>) {
    speed.0 = speed_of(change.value);
}

/// The items of the panel toolbar, each with the creator of its panel, and their selection.
fn spawn_panel_bar(commands: &mut Commands, icons: &Icons) -> Entity {
    let bar = commands.spawn_empty().id();
    let panels = [
        (
            "Trains",
            0,
            PanelKind::Trains,
            &icons.train,
            Some(TRAINS.len()),
        ),
        ("Stops", 0, PanelKind::Stops, &icons.stop, Some(STOPS.len())),
        ("Schedules", 0, PanelKind::Schedules, &icons.schedule, None),
        (
            "Preferences",
            1,
            PanelKind::Preferences,
            &icons.preferences,
            None,
        ),
    ];
    let mut first = None;
    for (name, group, kind, icon, count) in panels {
        let create = register_panel(commands, kind, icons.close.clone());
        let mut item = commands.spawn((
            ItemOf(bar),
            Item {
                data: Tool::in_group(icon.clone(), name, group),
            },
            StackItem::new(create),
        ));
        if let Some(count) = count {
            item.insert(Badge::count(count));
        }
        let id = item.id();
        first.get_or_insert(id);
    }
    // The first panel is open at the start; activating its tool again closes it.
    commands
        .entity(bar)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(first)));
    commands.insert_resource(PanelBar(bar));
    bar
}

/// Top right: the warnings button, the panel toolbar over `bar` and the panel it opens.
fn panels(bar: Entity, icons: &Icons) -> impl Scene {
    let bell = icons.bell.clone();
    let close = icons.close.clone();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(16.0),
            right: px(16.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: px(10.0),
        }
        Children [
            Node {
                align_items: AlignItems::Center,
                column_gap: px(10.0),
            }
            Children [
                IconButton { icon: bell, name: "Warnings", size: IconButtonSize::Large, tone: Tone::Warning }
                ~{ToggleState(false)}
                ~{Badge::count(WARNINGS.len())}
                WarningsButton
                --
                ~{Card}
                Node {
                    padding: UiRect::all(px(4.0)),
                }
                Children [
                    ~{Toolbar::new(ToolbarStyle::Underlined)}
                    ~{Selection::reference(bar)}
                ]
            ]
            --
            Panel { title: "Warnings", subtitle: {Some("3 open".to_string())}, close_icon: {HandleTemplate::Handle(close)} }
            ~{PanelKind::Warnings}
            WarningsPanel
            --
            Stack { selection_ref: bar }
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
            }
        ]
    }
}

/// The tools of the tool dock, each with the creator of its option bar, and their selection.
fn spawn_dock_tools(commands: &mut Commands, icons: &Icons) -> Entity {
    let selection = commands.spawn_empty().id();
    let hint = commands.register_system(
        |In(spawner): In<StackChildSpawner>, mut commands: Commands| {
            spawner.spawn_scene(&mut commands, hint_bar());
        },
    );
    let rail = commands.register_system(
        |In(spawner): In<StackChildSpawner>, mut commands: Commands| {
            spawner.spawn_scene(&mut commands, rail_bar());
        },
    );
    let vehicles = icons.clone();
    let train = commands.register_system(
        move |In(spawner): In<StackChildSpawner>, mut commands: Commands| {
            spawner.spawn_scene(&mut commands, vehicle_bar(&vehicles));
        },
    );
    let tools = [
        (
            Tool::in_group(icons.select.clone(), "Select", 0).with_key_hint("1"),
            StackItem::new_with_reference(0, hint),
        ),
        (
            Tool::in_group(icons.rail.clone(), "Rail", 1).with_key_hint("2"),
            StackItem::new(rail),
        ),
        (
            Tool::in_group(icons.signal.clone(), "Signal", 1).with_key_hint("3"),
            StackItem::new_reference(0),
        ),
        (
            Tool::in_group(icons.train.clone(), "Train", 2).with_key_hint("4"),
            StackItem::new(train),
        ),
        (
            Tool::in_group(icons.bulldoze.clone(), "Bulldoze", 3)
                .with_key_hint("5")
                .with_tone(Tone::Danger),
            StackItem::new_reference(0),
        ),
    ];
    let tools: Vec<Entity> = tools
        .into_iter()
        .map(|(tool, stack_item)| {
            commands
                .spawn((ItemOf(selection), Item { data: tool }, stack_item))
                .id()
        })
        .collect();
    if let Some(first) = tools.first() {
        commands
            .entity(selection)
            .insert(Selection::Value(SelectionValue::One(*first)));
    }
    commands.insert_resource(DockTools { selection, tools });
    selection
}

/// Bottom: the tool dock over `selection`, with the bar of the active tool above it.
fn dock(selection: Entity) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            right: px(0.0),
            bottom: px(16.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(10.0),
        }
        Children [
            Stack { selection_ref: selection }
            --
            ~{Card}
            Node {
                padding: UiRect::all(px(6.0)),
            }
            Children [
                ~{Toolbar::new(ToolbarStyle::Filled)}
                ~{Selection::reference(selection)}
                Dock
            ]
        ]
    }
}

/// A bar of options above the tool dock, holding `contents`.
fn option_bar(contents: impl SceneList) -> impl Scene {
    bsn! {
        ~{Card}
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(14.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
        }
        Children [ {contents} ]
    }
}

/// The bar for the tools that have no options: how to get back to the Select tool.
fn hint_bar() -> impl Scene {
    option_bar(bsn_list! {
        ~{Kbd::new("Esc")}
        --
        ~{Label::secondary("to go back to Select")}
    })
}

/// The Rail tool's options.
fn rail_bar() -> impl Scene {
    option_bar(bsn_list! {
        ~{Label::overline("Tracks")}
        --
        SegmentedControl
        @{Selection::builder().one().scene_with_items(["Single", "Double"].map(Item::new))}
        --
        @{Divider::vertical_scene()}
        --
        ~{Label::overline("Level")}
        --
        NumericStepper { format: {level as ValueFormat} }
        ~{Value::new_u8(0, 0, 56, 7)}
        --
        @{Divider::vertical_scene()}
        --
        ~{Label::overline("Signals")}
        --
        SegmentedControl
        @{Selection::builder().one().scene_with_items(["None", "Block"].map(Item::new))}
        --
        @{Divider::vertical_scene()}
        --
        ~{Kbd::new("R")}
        --
        ~{Label::secondary("rotate")}
    })
}

/// "Ground" at 0 and "+7", "+14", ... above it.
fn level(value: &Value) -> String {
    match value.as_u8() {
        Some(0) | None => "Ground".to_string(),
        Some(n) => format!("+{n}"),
    }
}

/// The Train tool's vehicles.
fn vehicle_bar(icons: &Icons) -> impl Scene {
    let cards = [
        ("Steam", &icons.locomotive, "Locomotives"),
        ("Diesel", &icons.locomotive, "Locomotives"),
        ("Box", &icons.boxcar, "Wagons"),
        ("Tank", &icons.boxcar, "Wagons"),
        ("Flat", &icons.flatcar, "Wagons"),
    ]
    .map(|(label, image, group)| Item {
        data: GalleryCard::new(image.clone(), label, group),
    });
    option_bar(bsn_list! {
        Node {
            width: px(520.0),
        }
        @ScrollArea::scene(
            bsn_list! {
                Gallery
                @{Selection::builder().one().scene_with_items(cards)}
            },
            false,
            true,
        )
    })
}

fn panel_for(kind: PanelKind, close: Handle<Image>) -> Panel {
    match kind {
        PanelKind::Trains => Panel::new("Trains").with_subtitle("4 in service"),
        PanelKind::Stops => Panel::new("Stops").with_subtitle("6 stops"),
        PanelKind::Schedules => Panel::new("Schedules"),
        PanelKind::Preferences => Panel::new("Preferences"),
        PanelKind::Warnings => Panel::new("Warnings").with_subtitle("3 open"),
    }
    .with_close_icon(close)
}

/// A creator for the lazily made panel of `kind` in the stack.
fn register_panel(
    commands: &mut Commands,
    kind: PanelKind,
    close: Handle<Image>,
) -> SystemId<In<StackChildSpawner>> {
    commands.register_system(
        move |In(spawner): In<StackChildSpawner>, mut commands: Commands| {
            spawner.spawn(&mut commands, (panel_for(kind, close.clone()), kind));
        },
    )
}

/// The row of a train: its name and route, and a pill with what it is doing.
fn train_row(spawner: In<ItemViewSpawner>, trains: Query<&Item<Train>>, mut commands: Commands) {
    let ItemViewSpawner { item, view } = *spawner;
    let Ok(train) = trains.get(item) else {
        return;
    };
    let train = train.data;
    commands
        // Parented before the scene adds its `Node`, or it would be taken for a `UiRoot`.
        .spawn(ChildOf(view))
        .apply_scene(bsn! {
            Node {
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(px(10.0), px(8.0)),
            }
            Children [
                Node {
                    flex_direction: FlexDirection::Column,
                }
                Children [
                    ~{Label::regular(train.name)}
                    --
                    ~{Label::caption(train.route)}
                ]
                --
                ~{Pill::new(train.status.label()).with_tone(train.status.tone())}
            ]
        });
}

/// Fills the strip, body and footer of a panel that was just made.
fn fill_panels(
    panels: Query<(&PanelRefs, &PanelKind), Added<PanelRefs>>,
    train_rows: Res<TrainRows>,
    mut commands: Commands,
) {
    for (refs, kind) in &panels {
        match kind {
            PanelKind::Trains => {
                let count = |status: TrainStatus| {
                    TRAINS.iter().filter(|train| train.status == status).count()
                };
                let chips = [
                    Chip::with_count("All", TRAINS.len()),
                    Chip::with_count("Running", count(TrainStatus::Running)),
                    Chip::with_count("Waiting", count(TrainStatus::Waiting)),
                    Chip::with_count("Problem", count(TrainStatus::Problem)),
                ]
                .map(|chip| Item { data: chip });
                let filter = commands
                    .spawn((
                        FilterChips,
                        Selection::builder().one().build_with_items(chips),
                        ChildOf(refs.strip),
                    ))
                    .id();
                commands.spawn((
                    List,
                    CornerStyle::Separated,
                    ItemViewCreator::new(train_rows.0),
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items(TRAINS.map(|train| Item { data: train })),
                    TrainFilter(filter),
                    ChildOf(refs.body),
                ));
                commands.spawn((
                    Label::caption("Select a train to follow it"),
                    ChildOf(refs.footer),
                ));
            }
            PanelKind::Stops => spawn_show_rows(&mut commands, refs.body, &STOPS),
            PanelKind::Warnings => spawn_show_rows(&mut commands, refs.body, &WARNINGS),
            PanelKind::Schedules => {
                commands.spawn((Label::secondary("No schedules yet."), ChildOf(refs.body)));
            }
            PanelKind::Preferences => {
                let rows = [
                    (Pref::Appearance, "Appearance", "Dark or light colours"),
                    (
                        Pref::WarningsButton,
                        "Warnings button",
                        "Show it in the top right corner",
                    ),
                    (
                        Pref::ToolLabels,
                        "Tool labels",
                        "Names under the tool icons",
                    ),
                    (Pref::TextSize, "Text size", "The size of all text"),
                    (Pref::CompanyName, "Company name", "Painted on your trains"),
                ]
                .into_iter()
                .map(|(pref, title, description)| {
                    bsn! {
                        ~{SettingRow::new(title, description)}
                        ~{pref}
                    }
                })
                .collect::<Vec<_>>();
                commands
                    // Parented before the scene adds its `Node`, or it would be taken for a `UiRoot`.
                    .spawn(ChildOf(refs.body))
                    .apply_scene(bsn! {
                        SettingRows
                        Children [ {rows} ]
                    });
            }
        }
    }
}

/// Rows with a title, a caption and a "Show" button, which would centre the map on it.
fn spawn_show_rows(commands: &mut Commands, parent: Entity, rows: &[(&'static str, &'static str)]) {
    for &(title, caption) in rows {
        let name = title.to_string();
        commands
            // Parented before the scene adds its `Node`, or it would be taken for a `UiRoot`.
            .spawn(ChildOf(parent))
            .apply_scene(bsn! {
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::axes(px(10.0), px(8.0)),
                }
                Children [
                    Node {
                        flex_direction: FlexDirection::Column,
                    }
                    Children [
                        ~{Label::regular(title)}
                        --
                        ~{Label::caption(caption)}
                    ]
                    --
                    ~{PushButton::from_label("Show")}
                    ~{ButtonSize::Compact}
                    // `Activate` does not bubble, so the observer sits on the button itself.
                    on(move |_: On<Activate>, roots: Query<Entity, With<UiRoot>>, mut commands: Commands| {
                        info!("Show {name} on the map");
                        if let Some(root) = roots.iter().next() {
                            commands.spawn((Toast::new(format!("Showing {name} on the map")), ChildOf(root)));
                        }
                    })
                ]
            });
    }
}

/// Puts the control of a setting into the slot of its row.
fn fill_setting_rows(
    rows: Query<(&SettingRowRefs, &Pref), Added<SettingRowRefs>>,
    mut commands: Commands,
) {
    for (refs, pref) in &rows {
        let options = match pref {
            Pref::Appearance => ["Dark", "Light"],
            Pref::WarningsButton | Pref::ToolLabels => ["Shown", "Hidden"],
            Pref::TextSize => {
                commands
                    .spawn((
                        NumericStepper {
                            format: as_percent,
                            name: Some("Text size".to_string()),
                            ..default()
                        },
                        Value::new_u16(100, 75, 150, 25),
                        ChildOf(refs.control_slot),
                    ))
                    .observe(set_text_size);
                continue;
            }
            Pref::CompanyName => {
                commands
                    .spawn((
                        TextField::new("Rail & Co")
                            .with_name("Company name")
                            .with_max_width(px(180.0)),
                        ChildOf(refs.control_slot),
                    ))
                    .observe(|commit: On<TextFieldCommit>| {
                        info!("Company name: {}", commit.value);
                    });
                continue;
            }
        };
        commands.spawn((
            SegmentedControl,
            *pref,
            Selection::builder()
                .one()
                .build_with_items(options.map(Item::new)),
            ChildOf(refs.control_slot),
        ));
    }
}

/// "100%" for a value of 100.
fn as_percent(value: &Value) -> String {
    format!("{}%", value.as_u16().unwrap_or(100))
}

/// Scales all text, the widgets' included, by setting the root font size.
fn set_text_size(change: On<ValueChange>, mut rem_size: ResMut<RemSize>) {
    if let Some(percent) = change.value.as_u16() {
        rem_size.0 = DEFAULT_REM_SIZE_PX * f32::from(percent) / 100.0;
    }
}

/// Applies a setting when its control changes.
fn apply_preferences(
    controls: Query<(Entity, &Pref), Changed<Selection>>,
    selections: SelectionParam,
    options: Query<&Item<String>>,
    mut theme: ResMut<Theme>,
    docks: Query<Entity, With<Dock>>,
    mut warnings: Query<&mut Node, With<WarningsButton>>,
    mut commands: Commands,
) {
    for (control, pref) in &controls {
        let Some(chosen) = selections
            .get(control)
            .and_then(SelectionValue::single)
            .and_then(|option| options.get(option).ok())
        else {
            continue;
        };
        let shown = chosen.data == "Shown";
        match pref {
            Pref::Appearance => {
                let mode = if chosen.data == "Dark" {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                if theme.mode() != mode {
                    theme.set_mode(mode);
                }
            }
            Pref::WarningsButton => {
                for mut node in &mut warnings {
                    node.display = if shown { Display::Flex } else { Display::None };
                }
            }
            // A stepper and a text field, which have no selection.
            Pref::TextSize | Pref::CompanyName => {}
            Pref::ToolLabels => {
                for dock in &docks {
                    commands.entity(dock).insert(if shown {
                        ToolbarLabels::Shown
                    } else {
                        ToolbarLabels::Hidden
                    });
                }
            }
        }
    }
}

/// Says "Running" or "Paused" next to the pause button.
fn sync_run_state(
    buttons: Query<&ToggleState, (With<PauseButton>, Changed<ToggleState>)>,
    labels: Query<Entity, With<RunLabel>>,
    mut commands: Commands,
) {
    for paused in &buttons {
        for label in &labels {
            commands.entity(label).insert(Label::regular(if paused.0 {
                "Paused"
            } else {
                "Running"
            }));
        }
    }
}

/// Shows the warnings panel while the warnings button is on.
fn sync_warnings_panel(
    buttons: Query<&ToggleState, (With<WarningsButton>, Changed<ToggleState>)>,
    mut panels: Query<&mut Node, With<WarningsPanel>>,
) {
    for open in &buttons {
        for mut node in &mut panels {
            node.display = if open.0 { Display::Flex } else { Display::None };
        }
    }
}

/// Closing a panel clears the panel toolbar, or turns the warnings button off.
fn on_panel_closed(
    closed: On<PanelClosed>,
    warnings_panels: Query<(), With<WarningsPanel>>,
    warnings_buttons: Query<Entity, With<WarningsButton>>,
    bar: Res<PanelBar>,
    mut selections: SelectionParam,
    mut commands: Commands,
) {
    if warnings_panels.contains(closed.event().event_target()) {
        for button in &warnings_buttons {
            commands.entity(button).insert(ToggleState(false));
        }
    } else {
        selections.update_selection(bar.0, SelectionValue::clear);
    }
}

/// Shows only the trains that match the active chip.
fn filter_train_rows(
    mut views: Query<(&ItemView, &mut Node)>,
    trains: Query<&Item<Train>>,
    filters: Query<&TrainFilter>,
    chips: Query<&Item<Chip>>,
    selections: SelectionParam,
) {
    for (view, mut node) in &mut views {
        let (Ok(train), Ok(filter)) = (trains.get(view.item_ref), filters.get(view.list_ref))
        else {
            continue;
        };
        let wanted = selections
            .get(filter.0)
            .and_then(SelectionValue::single)
            .and_then(|chip| chips.get(chip).ok())
            .is_none_or(|chip| {
                chip.data.label == "All" || chip.data.label == train.data.status.label()
            });
        let display = if wanted { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

/// The keys the tool hints and the pause key show: the library draws them, the app binds them.
///
/// They are off while a text field has the keys, and on the frame it hands them back, since
/// the `Enter` or `Escape` that did so is still a pressed key then.
fn hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    text_inputs: Query<(), With<TextInput>>,
    mut was_typing: Local<bool>,
    dock: Res<DockTools>,
    pause: Query<(Entity, &ToggleState), With<PauseButton>>,
    mut selections: SelectionParam,
    mut commands: Commands,
) {
    let typing = focus
        .get()
        .is_some_and(|entity| text_inputs.contains(entity));
    let handed_back = std::mem::replace(&mut *was_typing, typing);
    if typing || handed_back {
        return;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        for (button, paused) in &pause {
            commands.entity(button).insert(ToggleState(!paused.0));
        }
    }
    let mut pick = |index: usize| {
        if let Some(tool) = dock.tools.get(index) {
            selections.update_selection(dock.selection, |s| s.set_selection([*tool]));
        }
    };
    if keys.just_pressed(KeyCode::Escape) {
        pick(0);
    }
    for (index, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
    ]
    .into_iter()
    .enumerate()
    {
        if keys.just_pressed(key) {
            pick(index);
        }
    }
}
