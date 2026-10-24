//! A widget overview: a vertical `TabBar` on the left with one tab per widget, each showing a few
//! variants of that widget over example data.

mod common;

use bevy::asset::HandleTemplate;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::badge::{Badge, BadgeTone};
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::divider::Divider;
use rin_bevy_ui::filter_chips::{Chip, FilterChips};
use rin_bevy_ui::gallery::{Gallery, GalleryCard};
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::kbd::Kbd;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::link::Link;
use rin_bevy_ui::list::{CornerStyle, List, ListLayout};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::panel::{Panel, PanelRefs};
use rin_bevy_ui::pill::{Pill, PillTone};
use rin_bevy_ui::push_button::{ButtonSize, PushButton};
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::segmented_control::SegmentedControl;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use rin_bevy_ui::slider::Slider;
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::tab_bar::TabBar;
use rin_bevy_ui::tailwind::COLORS;
use rin_bevy_ui::text_field::TextField;
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui::toast::Toast;
use rin_bevy_ui::toolbar::{Tool, Toolbar, ToolbarStyle};
use rin_bevy_ui::tooltip::Tooltip;
use rin_bevy_ui::virtual_list::{RowHeight, VirtualList};
use rin_bevy_ui::{UiRoot, WidgetsPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WidgetsPlugin, OverviewPlugin))
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .run();
}

/// The overview's camera, UI and the systems that fill its panels.
#[derive(Debug, Clone, Copy)]
pub struct OverviewPlugin;

impl Plugin for OverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (fill_panels, fill_setting_rows));
    }
}

/// The icon every icon button, tool and card of the overview shows.
#[derive(Resource)]
struct Icon(Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Icon(common::cross_icon(&mut images)));

    let tabs = vec![
        (
            Item::new("Buttons"),
            StackItem::new(commands.register_system(buttons)),
        ),
        (
            Item::new("Icon buttons"),
            StackItem::new(commands.register_system(icon_buttons)),
        ),
        (
            Item::new("Labels"),
            StackItem::new(commands.register_system(labels)),
        ),
        (
            Item::new("Tags"),
            StackItem::new(commands.register_system(tags)),
        ),
        (
            Item::new("Lists"),
            StackItem::new(commands.register_system(lists)),
        ),
        (
            Item::new("Combos"),
            StackItem::new(commands.register_system(combos)),
        ),
        (
            Item::new("Colours"),
            StackItem::new(commands.register_system(colours)),
        ),
        (
            Item::new("Segmented control"),
            StackItem::new(commands.register_system(segmented)),
        ),
        (
            Item::new("Filter chips"),
            StackItem::new(commands.register_system(chips)),
        ),
        (
            Item::new("Tabs and trails"),
            StackItem::new(commands.register_system(navigation)),
        ),
        (
            Item::new("Slider and stepper"),
            StackItem::new(commands.register_system(values)),
        ),
        (
            Item::new("Text field"),
            StackItem::new(commands.register_system(text_fields)),
        ),
        (
            Item::new("Toolbar"),
            StackItem::new(commands.register_system(toolbars)),
        ),
        (
            Item::new("Gallery"),
            StackItem::new(commands.register_system(galleries)),
        ),
        (
            Item::new("Cards and panels"),
            StackItem::new(commands.register_system(cards)),
        ),
        (
            Item::new("Setting rows"),
            StackItem::new(commands.register_system(setting_rows)),
        ),
        (
            Item::new("Scroll area"),
            StackItem::new(commands.register_system(scroll_areas)),
        ),
        (
            Item::new("Tooltip and toast"),
            StackItem::new(commands.register_system(feedback)),
        ),
    ];

    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Row,
        }
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                height: percent(100.0),
                padding: UiRect::all(px(10.0)),
            }
            Children [
                TabBar
                #tabs
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                }
                @{Selection::builder().one().scene_with_items(tabs)}
                --
                ~{PushButton::from_label("Change theme")}
                on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
            ]
            --
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(px(20.0)),
            }
            Stack { selection_ref: #tabs }
        ]
    });
}

fn countries() -> [Item<String>; 5] {
    ["Netherlands", "Canada", "Indonesia", "Portugal", "Belgium"].map(Item::new)
}

fn buttons(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                ~{Label::header("Push buttons")}
                --
                Node { column_gap: px(10.0), align_items: AlignItems::Center }
                Children [
                    ~{PushButton::from_label("Regular")}
                    --
                    ~{PushButton::from_label("Compact")}
                    ~{ButtonSize::Compact}
                    --
                    ~{PushButton::from_label("Disabled")}
                    InteractionDisabled
                ]
                --
                ~{Label::header("Links")}
                --
                Node { column_gap: px(10.0), align_items: AlignItems::Center }
                Children [
                    ~{Link::new("Timetable")}
                    --
                    ~{Link::new("Depot map")}
                ]
            ]
        },
    );
}

fn icon_buttons(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                ~{Label::header("Icon buttons")}
                --
                Node { column_gap: px(16.0), align_items: AlignItems::Center }
                Children [
                    IconButton { icon: {icon.clone()}, name: "Small", size: IconButtonSize::Small }
                    --
                    IconButton { icon: {icon.clone()}, name: "Regular", size: IconButtonSize::Regular }
                    --
                    IconButton { icon: {icon.clone()}, name: "Large", size: IconButtonSize::Large }
                    --
                    IconButton { icon: {icon.clone()}, name: "Toggle", size: IconButtonSize::Regular }
                    ~{ToggleState(false)}
                    --
                    IconButton { icon: {icon.clone()}, name: "Warning", size: IconButtonSize::Large, tone: Tone::Warning }
                    --
                    IconButton { icon: {icon.clone()}, name: "Danger", size: IconButtonSize::Regular, tone: Tone::Danger }
                    --
                    IconButton { icon, name: "Trains", size: IconButtonSize::Large }
                    ~{Badge::count(3)}
                ]
            ]
        },
    );
}

fn labels(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(8.0),
            }
            Children [
                ~{Label::header("Header")}
                --
                ~{Label::title("Title")}
                --
                ~{Label::regular("Regular text")}
                --
                ~{Label::secondary("Secondary text")}
                --
                ~{Label::caption("Caption")}
                --
                ~{Label::overline("Overline")}
                --
                @{Divider::horizontal_scene()}
                Node { width: px(300.0) }
                --
                Node { column_gap: px(10.0), align_items: AlignItems::Center }
                Children [
                    ~{Label::regular("Press")}
                    --
                    ~{Kbd::new("Esc")}
                    --
                    ~{Label::regular("to cancel, or")}
                    --
                    ~{Kbd::new("R")}
                    --
                    ~{Label::regular("to rotate")}
                ]
            ]
        },
    );
}

fn tags(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                ~{Label::header("Pills")}
                --
                Node { column_gap: px(10.0), align_items: AlignItems::Center }
                Children [
                    ~{Pill::new("Idle")}
                    --
                    ~{Pill::new("Running").with_tone(PillTone::Info)}
                    --
                    ~{Pill::new("Waiting").with_tone(PillTone::Warning)}
                    --
                    ~{Pill::new("No path").with_tone(PillTone::Problem)}
                ]
                --
                ~{Label::header("Badges")}
                --
                Node { column_gap: px(20.0), align_items: AlignItems::Center }
                Children [
                    IconButton { icon: {icon.clone()}, name: "Trains", size: IconButtonSize::Large }
                    ~{Badge::count(3)}
                    --
                    IconButton { icon, name: "Warnings", size: IconButtonSize::Large }
                    ~{Badge::count(12).with_tone(BadgeTone::Warning)}
                ]
            ]
        },
    );
}

fn lists(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(20.0),
                row_gap: px(20.0),
            }
            Children [
                List
                @{Selection::builder().one().scene_with_items(countries())}
                --
                List
                ~{CornerStyle::Separated}
                @{Selection::builder().zero_or_more().scene_with_items(countries())}
                --
                List
                ~{ListLayout::VerticalGrid { columns: 2 }}
                ~{CornerStyle::Tabs}
                @{Selection::builder().zero_or_one().scene_with_items(countries())}
                --
                // Ten thousand rows, of which only those in view are spawned.
                Node { width: px(220.0), height: px(260.0) }
                @virtual_list_scroll_area()
            ]
        },
    );
}

/// A scroll area over a `VirtualList` of ten thousand rows.
fn virtual_list_scroll_area() -> impl Scene {
    ScrollArea::scene(
        || {
            bsn_list! {
                ~{VirtualList::new(row_height(28.0))}
                @{Selection::builder().one().scene_with_items((1..=10_000).map(|row| Item::new(format!("Row {row}"))))}
            }
        },
        true,
        false,
    )
}

fn row_height(height: f32) -> RowHeight {
    RowHeight::new(height).unwrap_or_else(|_| std::process::abort())
}

fn combos(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(20.0),
            }
            Children [
                ~{Combo}
                @{Selection::builder().zero_or_one().scene_with_items(countries())}
                --
                ~{Combo}
                @{Selection::builder().one().scene_with_items(countries())}
            ]
        },
    );
}

fn colours(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let swatches = || {
        COLORS
            .iter()
            .filter(|c| c.variant == rin_bevy_ui::tailwind::Variant::V500)
            .map(|c| Item { data: c.color })
            .collect::<Vec<_>>()
    };
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                ~{Label::header("Colour palette")}
                --
                ~{ColorPalette}
                ~{ListLayout::VerticalGrid { columns: 11 }}
                @{Selection::builder().one().scene_with_items(swatches())}
                --
                ~{Label::header("Colour palette combo")}
                --
                ColorPaletteCombo
                ~{ListLayout::VerticalGrid { columns: 11 }}
                @{Selection::builder().zero_or_one().scene_with_items(swatches())}
            ]
        },
    );
}

fn segmented(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                SegmentedControl
                @{Selection::builder().one().scene_with_items(["Auto", "Manual"].map(Item::new))}
                --
                SegmentedControl
                @{Selection::builder().one().scene_with_items(["Dark", "System", "Light"].map(Item::new))}
            ]
        },
    );
}

fn chips(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let items = || {
        vec![
            Item {
                data: Chip::with_count("All", 24),
            },
            Item {
                data: Chip::with_count("Running", 12),
            },
            Item {
                data: Chip::with_count("Waiting", 7),
            },
            Item {
                data: Chip::with_count("Problems", 3),
            },
            Item {
                data: Chip::new("Depot"),
            },
        ]
    };
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                FilterChips
                @{Selection::builder().one().scene_with_items(items())}
                --
                // Narrow, so that the chips wrap.
                FilterChips
                Node {
                    width: px(220.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6.0),
                    row_gap: px(6.0),
                }
                @{Selection::builder().one().scene_with_items(items())}
            ]
        },
    );
}

fn navigation(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(16.0),
            }
            Children [
                ~{Label::header("Tab bar")}
                --
                TabBar
                @{Selection::builder().one().scene_with_items(["Overview", "Trains", "Stops"].map(Item::new))}
                --
                ~{Label::header("Bread crumbs")}
                --
                ~{BreadCrumbs}
                @{Selection::builder().one_or_more().scene_with_items(["Home", "Lines", "Line 4"].map(Item::new))}
            ]
        },
    );
}

#[expect(clippy::expect_used, reason = "the constant ranges are valid")]
fn values(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let float_value = Value::new_f32(10.0, 5.0, 15.0, 0.5).expect("a valid float range");
    let level_value = Value::new_u8(0, 0, 56, 7).expect("a valid level range");
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(16.0),
            }
            Children [
                ~{valid_slider(0.5, 0.0, 1.0, None).with_name("Volume")}
                --
                ~{valid_slider(60.0, 0.0, 120.0, Some(10.0)).with_format(kilometres_per_hour).with_name("Speed")}
                --
                ~{valid_slider(4.0, 0.0, 10.0, Some(1.0)).with_name("Locked")}
                InteractionDisabled
                --
                Node { column_gap: px(16.0), align_items: AlignItems::Center }
                Children [
                    NumericStepper
                    --
                    NumericStepper
                    ~{float_value}
                    --
                    NumericStepper
                    ~{level_value}
                ]
            ]
        },
    );
}

fn kilometres_per_hour(value: f32) -> String {
    format!("{value:.0} km/h")
}

fn text_fields(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(12.0),
            }
            Children [
                ~{TextField::new("Amsterdam Centraal").with_name("Name")}
                --
                ~{TextField::new("")
                    .with_placeholder("Seed")
                    .with_name("Seed")
                    .with_max_width(Val::Px(200.0))}
            ]
        },
    );
}

fn toolbars(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let tools = |icon: &Handle<Image>| {
        vec![
            Item {
                data: Tool::in_group(icon.clone(), "Select", 0).with_key_hint("1"),
            },
            Item {
                data: Tool::in_group(icon.clone(), "Rail", 1).with_key_hint("2"),
            },
            Item {
                data: Tool::in_group(icon.clone(), "Signal", 1).with_key_hint("3"),
            },
            Item {
                data: Tool::in_group(icon.clone(), "Bulldoze", 2).with_tone(Tone::Danger),
            },
        ]
    };
    let filled = tools(&icon.0);
    let underlined = tools(&icon.0);
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(16.0),
            }
            Children [
                ~{Toolbar::new(ToolbarStyle::Filled)}
                @{Selection::builder().one().scene_with_items(filled)}
                --
                ~{Toolbar::new(ToolbarStyle::Underlined)}
                @{Selection::builder().zero_or_one().scene_with_items(underlined)}
            ]
        },
    );
}

fn galleries(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let cards = [
        ("Steam", "Locomotives"),
        ("Diesel", "Locomotives"),
        ("Electric", "Locomotives"),
        ("Passenger", "Wagons"),
        ("Box wagon", "Wagons"),
        ("Tank wagon", "Wagons"),
    ]
    .map(|(label, group)| Item {
        data: GalleryCard::new(icon.0.clone(), label, group),
    });
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { width: px(600.0) }
            @gallery_scroll_area(cards)
        },
    );
}

/// A horizontal scroll area over the gallery of `cards`.
fn gallery_scroll_area(cards: [Item<GalleryCard>; 6]) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                Gallery
                @{Selection::builder().one().scene_with_items(cards.clone())}
            }
        },
        false,
        true,
    )
}

/// How many lines of text the body of a panel gets.
#[derive(Component, Default, Clone)]
struct Lines(usize);

/// Marks a panel or setting row whose contents have been spawned.
#[derive(Component)]
struct Filled;

fn cards(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let close = icon.0.clone();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(20.0),
            }
            Children [
                ~{Card}
                Node {
                    padding: UiRect::all(px(12.0)),
                    row_gap: px(8.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                }
                Children [
                    ~{Label::title("Card")}
                    --
                    ~{Label::secondary("A themed surface")}
                    --
                    ~{PushButton::from_label("Button on a card")}
                ]
                --
                Panel { title: "Trains", subtitle: {Some("3 open".to_string())}, close_icon: {HandleTemplate::Handle(close.clone())} }
                Lines(3)
                --
                // A body past 150 pixels scrolls.
                Panel { title: "Stops", close_icon: {HandleTemplate::Handle(close)}, max_body_height: 150.0 }
                Lines(12)
            ]
        },
    );
}

/// Fills the bodies of the panels once they are spawned.
fn fill_panels(
    mut commands: Commands,
    panels: Query<(Entity, &PanelRefs, &Lines), Without<Filled>>,
) {
    for (panel, refs, lines) in &panels {
        commands.entity(panel).insert(Filled);
        for line in 1..=lines.0 {
            commands.spawn((Label::regular(format!("Line {line}")), ChildOf(refs.body())));
        }
    }
}

/// The control a setting row gets in its slot.
#[derive(Component, Default, Clone, Copy)]
enum Control {
    #[default]
    Appearance,
    TextSize,
    Reset,
}

fn setting_rows(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            SettingRows
            Node { width: px(420.0) }
            Children [
                ~{SettingRow::new("Appearance", "Light or dark")} Control::Appearance
                --
                ~{SettingRow::new("Text size", "In points")} Control::TextSize
                --
                ~{SettingRow::new("Reset", "Back to the defaults")} Control::Reset
            ]
        },
    );
}

/// Spawns the control of each setting row into its slot once the row is spawned.
fn fill_setting_rows(
    mut commands: Commands,
    rows: Query<(Entity, &SettingRowRefs, &Control), Without<Filled>>,
) -> Result {
    for (row, refs, control) in &rows {
        commands.entity(row).insert(Filled);
        let slot = refs.control_slot;
        match control {
            Control::Appearance => {
                commands.spawn((
                    SegmentedControl,
                    Selection::builder()
                        .one()
                        .build_with_items(["Light", "Dark"].map(Item::new)),
                    ChildOf(slot),
                ));
            }
            Control::TextSize => {
                commands.spawn((
                    NumericStepper::default(),
                    Value::new_u8(12, 8, 20, 1)?,
                    ChildOf(slot),
                ));
            }
            Control::Reset => {
                commands.spawn((PushButton::from_label("Reset"), ChildOf(slot)));
            }
        }
    }
    Ok(())
}

fn scroll_areas(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(20.0),
            }
            Children [
                Node { width: px(200.0), height: px(200.0) }
                @list_scroll_area(true, false)
                --
                Node { width: px(200.0), height: px(200.0) }
                @list_scroll_area(true, true)
            ]
        },
    );
}

/// A scroll area over a list of many countries.
fn list_scroll_area(vertical: bool, horizontal: bool) -> impl Scene {
    ScrollArea::scene(
        || {
            let items = (1..=20).map(|i| Item::new(format!("A rather long country name {i}")));
            bsn_list! {
                List
                @{Selection::builder().zero_or_one().scene_with_items(items)}
            }
        },
        vertical,
        horizontal,
    )
}

fn feedback(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(20.0),
            }
            Children [
                ~{PushButton::from_label("Hover me")}
                ~{Tooltip::new("A tooltip appears after half a second")}
                --
                ~{PushButton::from_label("Raise a toast")}
                on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
                    commands.spawn((Toast::new("Saved to rails.ron"), ChildOf(*root)));
                })
            ]
        },
    );
}

/// A slider with a valid range and step, or an abort: these are fixed values.
fn valid_slider(value: f32, min: f32, max: f32, step: Option<f32>) -> Slider {
    Slider::new(value, min, max)
        .and_then(|slider| match step {
            Some(step) => slider.with_step(step),
            None => Ok(slider),
        })
        .unwrap_or_else(|_| std::process::abort())
}
