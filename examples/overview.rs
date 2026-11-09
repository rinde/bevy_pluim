//! A widget overview: a vertical `TabBar` on the left with one tab per widget, and for the tab
//! picked a page with that widget's variants over example data.
//!
//! Every page has the same layout: a header, then the variants in a centred, wrapping row, each a
//! group with a secondary label naming what is different about it. The theme is switched with the
//! `LightDarkToggle` above the tabs, which is one of the widgets of the library, too.

mod common;

use bevy::asset::HandleTemplate;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use bevy_pluim::badge::{Badge, BadgeTone};
use bevy_pluim::breadcrumbs::BreadCrumbs;
use bevy_pluim::card::Card;
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::color_palette_combo::ColorPaletteCombo;
use bevy_pluim::combo::{Combo, ComboPlaceholder};
use bevy_pluim::divider::Divider;
use bevy_pluim::filter_chips::{Chip, FilterChips};
use bevy_pluim::filter_combo::FilterCombo;
use bevy_pluim::gallery::{Gallery, GalleryCard};
use bevy_pluim::icon_button::{IconButton, IconButtonSize, ToggleState, Tone};
use bevy_pluim::kbd::Kbd;
use bevy_pluim::label::Label;
use bevy_pluim::light_dark_toggle::LightDarkToggle;
use bevy_pluim::link::Link;
use bevy_pluim::list::{CornerStyle, ItemNote, List, ListLayout};
use bevy_pluim::numeric_stepper::{NumericStepper, Value};
use bevy_pluim::panel::{Panel, PanelRefs};
use bevy_pluim::pill::{Pill, PillTone};
use bevy_pluim::push_button::{ButtonSize, IconPlacement, PushButton};
use bevy_pluim::scroll_area::ScrollArea;
use bevy_pluim::segmented_control::{ItemIcon, SegmentLabels, SegmentedControl};
use bevy_pluim::select::{Item, Selection};
use bevy_pluim::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use bevy_pluim::slider::Slider;
use bevy_pluim::stack::{Stack, StackChildSpawner, StackItem};
use bevy_pluim::tab_bar::TabBar;
use bevy_pluim::tailwind::COLORS;
use bevy_pluim::text_field::TextField;
use bevy_pluim::toast::Toast;
use bevy_pluim::toolbar::{Tool, Toolbar, ToolbarLabels, ToolbarStyle};
use bevy_pluim::tooltip::Tooltip;
use bevy_pluim::virtual_list::{RowHeight, VirtualList};
use bevy_pluim::{UiRoot, WidgetsPlugin};

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
        app.add_systems(Startup, setup).add_systems(
            Update,
            (fill_panels, fill_control_panels, fill_setting_rows),
        );
    }
}

/// The icon every icon button, tool and card of the overview shows.
#[derive(Resource)]
struct Icon(Handle<Image>);

/// The tabs, each with the system that spawns its page the first time it is shown.
fn tab_pages(commands: &mut Commands) -> Vec<(Item<String>, StackItem)> {
    let mut tabs = Vec::new();
    let mut add = |name: &str, page: StackItem| tabs.push((Item::new(name), page));
    add("Buttons", StackItem::new(commands.register_system(buttons)));
    add(
        "Icon buttons",
        StackItem::new(commands.register_system(icon_buttons)),
    );
    add("Labels", StackItem::new(commands.register_system(labels)));
    add("Tags", StackItem::new(commands.register_system(tags)));
    add("Lists", StackItem::new(commands.register_system(lists)));
    add("Combos", StackItem::new(commands.register_system(combos)));
    add(
        "Filter combos",
        StackItem::new(commands.register_system(filter_combos)),
    );
    add("Colours", StackItem::new(commands.register_system(colours)));
    add(
        "Segmented control",
        StackItem::new(commands.register_system(segmented)),
    );
    add(
        "Light dark toggle",
        StackItem::new(commands.register_system(light_dark)),
    );
    add(
        "Filter chips",
        StackItem::new(commands.register_system(chips)),
    );
    add(
        "Tabs and trails",
        StackItem::new(commands.register_system(navigation)),
    );
    add(
        "Slider and stepper",
        StackItem::new(commands.register_system(values)),
    );
    add(
        "Text field",
        StackItem::new(commands.register_system(text_fields)),
    );
    add(
        "Toolbar",
        StackItem::new(commands.register_system(toolbars)),
    );
    add(
        "Gallery",
        StackItem::new(commands.register_system(galleries)),
    );
    add(
        "Cards and panels",
        StackItem::new(commands.register_system(cards)),
    );
    add(
        "Setting rows",
        StackItem::new(commands.register_system(setting_rows)),
    );
    add(
        "Scroll area",
        StackItem::new(commands.register_system(scroll_areas)),
    );
    add(
        "Tooltip and toast",
        StackItem::new(commands.register_system(feedback)),
    );
    tabs
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Icon(common::cross_icon(&mut images)));

    let tabs = tab_pages(&mut commands);
    let selection = commands
        .spawn(Selection::builder().one().build_with_items(tabs))
        .id();

    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Row,
        }
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                height: percent(100.0),
                padding: UiRect::all(px(10.0)),
                row_gap: px(10.0),
            }
            Children [
                // The theme control; it is the widget of the "Light dark toggle" page, too.
                LightDarkToggle
                --
                TabBar
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                }
                ~{Selection::reference(selection)}
            ]
            --
            // The pages scroll when one is taller than the window; each is as wide as this area
            // and centres its contents.
            Node {
                flex_grow: 1.0,
                min_width: px(0.0),
                height: percent(100.0),
                padding: UiRect::all(px(20.0)),
            }
            Children [
                Node { width: percent(100.0), height: percent(100.0) }
                @pages(selection)
            ]
        ]
    });
}

/// The area the pages are shown in.
fn pages(selection: Entity) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                Stack { selection_ref: selection }
                Node { width: percent(100.0) }
            }
        },
        true,
        false,
    )
}

// ---- The layout every page shares ----

/// A page: a column as wide as the pages area, with everything centred.
fn page() -> Node {
    Node {
        width: percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(24.0),
        ..default()
    }
}

/// The variants of a page, centred and wrapping.
fn groups() -> Node {
    Node {
        max_width: px(960.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        column_gap: px(40.0),
        row_gap: px(28.0),
        ..default()
    }
}

/// Widgets side by side inside a variant.
fn row() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(12.0),
        ..default()
    }
}

/// One variant: a secondary label naming what is different about it, over the widget.
fn variant(caption: &'static str, content: impl Scene) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(8.0),
        }
        Children [
            ~{Label::secondary(caption)}
            --
            @{content}
        ]
    }
}

fn countries() -> [Item<String>; 5] {
    ["Netherlands", "Canada", "Indonesia", "Portugal", "Belgium"].map(Item::new)
}

fn stations() -> [(Item<String>, ItemNote); 3] {
    [
        (Item::new("Amsterdam"), ItemNote::new("")),
        (Item::new("Delft"), ItemNote::warning("< unreachable >")),
        (Item::new("Utrecht"), ItemNote::new("4 km")),
    ]
}

fn buttons(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Buttons")}
                --
                ~{groups()}
                Children [
                    @variant("Regular", bsn! { ~{PushButton::from_label("Regular")} })
                    --
                    @variant("Compact", bsn! {
                        ~{PushButton::from_label("Compact")}
                        ~{ButtonSize::Compact}
                    })
                    --
                    @variant("Disabled", bsn! {
                        ~{PushButton::from_label("Disabled")}
                        InteractionDisabled
                    })
                    --
                    @variant("Toggle (off, then on)", bsn! {
                        ~{row()}
                        Children [
                            ~{PushButton::from_label("Off")}
                            ~{ToggleState(false)}
                            --
                            ~{PushButton::from_label("On")}
                            ~{ToggleState(true)}
                        ]
                    })
                    --
                    @variant("Icon placement: left, right, top, bottom", bsn! {
                        ~{row()}
                        Children [
                            ~{PushButton::new("Left", "+")}
                            ~{IconPlacement::Left}
                            --
                            ~{PushButton::new("Right", "+")}
                            ~{IconPlacement::Right}
                            --
                            ~{PushButton::new("Top", "+")}
                            ~{IconPlacement::Top}
                            --
                            ~{PushButton::new("Bottom", "+")}
                            ~{IconPlacement::Bottom}
                        ]
                    })
                    --
                    @variant("Link", bsn! { ~{Link::new("Timetable")} })
                    --
                    @variant("Disabled link", bsn! {
                        ~{Link::new("Depot map")}
                        InteractionDisabled
                    })
                ]
            ]
        },
    );
}

fn icon_buttons(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    let (small, regular, large, toggle) = (icon.clone(), icon.clone(), icon.clone(), icon.clone());
    let (warning, danger, badge, disabled) = (icon.clone(), icon.clone(), icon.clone(), icon);
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Icon buttons")}
                --
                ~{groups()}
                Children [
                    @variant("Small", bsn! {
                        IconButton { icon: {small}, name: "Small", size: IconButtonSize::Small }
                    })
                    --
                    @variant("Regular", bsn! {
                        IconButton { icon: {regular}, name: "Regular", size: IconButtonSize::Regular }
                    })
                    --
                    @variant("Large", bsn! {
                        IconButton { icon: {large}, name: "Large", size: IconButtonSize::Large }
                    })
                    --
                    @variant("Toggle", bsn! {
                        IconButton { icon: {toggle}, name: "Toggle", size: IconButtonSize::Regular }
                        ~{ToggleState(false)}
                    })
                    --
                    @variant("Warning tone", bsn! {
                        IconButton { icon: {warning}, name: "Warning", size: IconButtonSize::Large, tone: Tone::Warning }
                    })
                    --
                    @variant("Danger tone", bsn! {
                        IconButton { icon: {danger}, name: "Danger", size: IconButtonSize::Regular, tone: Tone::Danger }
                    })
                    --
                    @variant("With a badge", bsn! {
                        IconButton { icon: {badge}, name: "Trains", size: IconButtonSize::Large }
                        ~{Badge::count(3)}
                    })
                    --
                    @variant("Disabled", bsn! {
                        IconButton { icon: {disabled}, name: "Disabled", size: IconButtonSize::Regular }
                        InteractionDisabled
                    })
                ]
                --
                ~{Label::caption("Every icon button shows its name as a tooltip after half a second.")}
            ]
        },
    );
}

fn labels(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Labels")}
                --
                ~{groups()}
                Children [
                    @variant("Header", bsn! { ~{Label::header("Header")} })
                    --
                    @variant("Title", bsn! { ~{Label::title("Title")} })
                    --
                    @variant("Regular", bsn! { ~{Label::regular("Regular text")} })
                    --
                    @variant("Secondary", bsn! { ~{Label::secondary("Secondary text")} })
                    --
                    @variant("Caption", bsn! { ~{Label::caption("Caption")} })
                    --
                    @variant("Overline", bsn! { ~{Label::overline("Overline")} })
                    --
                    @variant("Divider", bsn! {
                        @{Divider::horizontal_scene()}
                        Node { width: px(200.0) }
                    })
                    --
                    @variant("Key caps", bsn! {
                        ~{row()}
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
                    })
                ]
            ]
        },
    );
}

fn tags(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    let (neutral, warning) = (icon.clone(), icon);
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tags")}
                --
                ~{groups()}
                Children [
                    @variant("Pill: neutral", bsn! { ~{Pill::new("Idle")} })
                    --
                    @variant("Pill: info", bsn! { ~{Pill::new("Running").with_tone(PillTone::Info)} })
                    --
                    @variant("Pill: warning (waiting)", bsn! {
                        ~{Pill::new("Waiting").with_tone(PillTone::Warning)}
                    })
                    --
                    @variant("Pill: problem", bsn! {
                        ~{Pill::new("No path").with_tone(PillTone::Problem)}
                    })
                    --
                    @variant("Badge: neutral count", bsn! {
                        IconButton { icon: {neutral}, name: "Trains", size: IconButtonSize::Large }
                        ~{Badge::count(3)}
                    })
                    --
                    @variant("Badge: warning count", bsn! {
                        IconButton { icon: {warning}, name: "Warnings", size: IconButtonSize::Large }
                        ~{Badge::count(12).with_tone(BadgeTone::Warning)}
                    })
                ]
            ]
        },
    );
}

fn lists(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Lists")}
                --
                ~{groups()}
                Children [
                    @variant("One selected", bsn! {
                        List
                        @{Selection::builder().one().scene_with_items(countries())}
                    })
                    --
                    @variant("Separated, many selected", bsn! {
                        List
                        ~{CornerStyle::Separated}
                        @{Selection::builder().zero_or_more().scene_with_items(countries())}
                    })
                    --
                    @variant("Two-column grid, tabs corners, none or one", bsn! {
                        List
                        ~{ListLayout::VerticalGrid { columns: 2 }}
                        ~{CornerStyle::Tabs}
                        @{Selection::builder().zero_or_one().scene_with_items(countries())}
                    })
                    --
                    // Ten thousand rows, of which only those in view are spawned.
                    @variant("Virtual list, 10,000 rows", bsn! {
                        Node { width: px(220.0), height: px(260.0) }
                        @virtual_list_scroll_area(VirtualList::new(row_height(28.0)))
                    })
                    --
                    @variant("Virtual list, separated", bsn! {
                        Node { width: px(220.0), height: px(260.0) }
                        @virtual_list_scroll_area(
                            VirtualList::separated(row_height(44.0)).unwrap_or_else(|_| std::process::abort())
                        )
                    })
                ]
            ]
        },
    );
}

/// A scroll area over a `VirtualList` of ten thousand rows.
fn virtual_list_scroll_area(list: VirtualList) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                ~{list}
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
            #Page
            ~{page()}
            Children [
                ~{Label::header("Combos")}
                --
                ~{groups()}
                Children [
                    @variant("None or one, with a placeholder", bsn! {
                        ~{Combo}
                        ~{ComboPlaceholder("Pick a country".to_string())}
                        @{Selection::builder().zero_or_one().scene_with_items(countries())}
                    })
                    --
                    @variant("Exactly one", bsn! {
                        ~{Combo}
                        @{Selection::builder().one().scene_with_items(countries())}
                    })
                    --
                    @variant("Options with a note", bsn! {
                        ~{Combo}
                        @{Selection::builder().zero_or_one().scene_with_items(stations())}
                    })
                    --
                    @variant("One or more", bsn! {
                        ~{Combo}
                        @{Selection::builder().one_or_more().scene_with_items(countries())}
                    })
                    --
                    @variant("None or more", bsn! {
                        ~{Combo}
                        @{Selection::builder().zero_or_more().scene_with_items(countries())}
                    })
                ]
            ]
        },
    );
}

fn filter_combos(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let shared = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        )
        .id();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Filter combos")}
                --
                ~{Label::caption("Type to filter the options, Arrow Down and Up to move through the matches, Enter to pick.")}
                --
                ~{groups()}
                Children [
                    @variant("Shares its selection with the combo beside it", bsn! {
                        ~{row()}
                        Children [
                            ~{FilterCombo::default().with_name("Country")}
                            ~{Selection::reference(shared)}
                            --
                            ~{Combo}
                            ~{Selection::reference(shared)}
                        ]
                    })
                    --
                    @variant("Exactly one", bsn! {
                        ~{FilterCombo::default().with_name("Country")}
                        @{Selection::builder().one().scene_with_items(countries())}
                    })
                    --
                    @variant("None or more, stays open for the next pick", bsn! {
                        ~{FilterCombo::default().with_name("Stations")}
                        @{Selection::builder().zero_or_more().scene_with_items(countries())}
                    })
                    --
                    @variant("A long list: 80 options, the popup scrolls", bsn! {
                        ~{FilterCombo::default().with_name("Number")}
                        @{Selection::builder().zero_or_one().scene_with_items((1..=80).map(|n| Item::new(format!("Option {n}"))))}
                    })
                ]
            ]
        },
    );
}

fn colours(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let swatches = || {
        COLORS
            .iter()
            .filter(|c| c.variant == bevy_pluim::tailwind::Variant::V500)
            .map(|c| Item { data: c.color })
            .collect::<Vec<_>>()
    };
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Colours")}
                --
                ~{groups()}
                Children [
                    @variant("Palette, exactly one", bsn! {
                        ~{ColorPalette}
                        ~{ListLayout::VerticalGrid { columns: 11 }}
                        @{Selection::builder().one().scene_with_items(swatches())}
                    })
                    --
                    @variant("Palette in a scroll area", bsn! {
                        Node { width: px(250.0), height: px(120.0) }
                        @palette_scroll_area(swatches())
                    })
                    --
                    @variant("Palette combo, none or one", bsn! {
                        ColorPaletteCombo
                        ~{ListLayout::VerticalGrid { columns: 11 }}
                        @{Selection::builder().zero_or_one().scene_with_items(swatches())}
                    })
                    --
                    @variant("Palette combo, exactly one", bsn! {
                        ColorPaletteCombo
                        ~{ListLayout::VerticalGrid { columns: 11 }}
                        @{Selection::builder().one().scene_with_items(swatches())}
                    })
                ]
            ]
        },
    );
}

/// A scroll area over a palette of `swatches`.
fn palette_scroll_area(swatches: Vec<Item<Color>>) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                ~{ColorPalette}
                ~{ListLayout::VerticalGrid { columns: 6 }}
                @{Selection::builder().one().scene_with_items(swatches.clone())}
            }
        },
        true,
        false,
    )
}

fn segmented(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    let with_icon = move |text| (Item::new(text), ItemIcon(icon.clone()));
    let (text_and_icon, icon_only) = (
        ["Grid", "List"].map(with_icon.clone()),
        ["Map", "Table"].map(with_icon),
    );
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Segmented control")}
                --
                ~{groups()}
                Children [
                    @variant("Two segments", bsn! {
                        SegmentedControl
                        @{Selection::builder().one().scene_with_items(["Auto", "Manual"].map(Item::new))}
                    })
                    --
                    @variant("Three segments", bsn! {
                        SegmentedControl
                        @{Selection::builder().one().scene_with_items(["Dark", "System", "Light"].map(Item::new))}
                    })
                    --
                    @variant("Text and icon", bsn! {
                        SegmentedControl
                        @{Selection::builder().one().scene_with_items(text_and_icon)}
                    })
                    --
                    @variant("Icon only, the text is the tooltip", bsn! {
                        SegmentedControl
                        ~{SegmentLabels::Hidden}
                        @{Selection::builder().one().scene_with_items(icon_only)}
                    })
                ]
            ]
        },
    );
}

fn light_dark(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Light dark toggle")}
                --
                ~{groups()}
                Children [
                    @variant("Light, system or dark; the system follows the operating system", bsn! {
                        LightDarkToggle
                    })
                ]
                --
                ~{Label::caption("The toggle above the tabs is this widget, and it switches the theme of this overview.")}
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
    let shared = commands
        .spawn(Selection::builder().one().build_with_items(items()))
        .id();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Filter chips")}
                --
                ~{groups()}
                Children [
                    @variant("With counts", bsn! {
                        FilterChips
                        ~{Selection::reference(shared)}
                    })
                    --
                    @variant("Narrow, so that the chips wrap", bsn! {
                        FilterChips
                        Node {
                            width: px(220.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(6.0),
                            row_gap: px(6.0),
                        }
                        @{Selection::builder().one().scene_with_items(items())}
                    })
                    --
                    @variant("A second control sharing the selection of the first", bsn! {
                        FilterChips
                        ~{Selection::reference(shared)}
                    })
                ]
            ]
        },
    );
}

fn navigation(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tabs and trails")}
                --
                ~{groups()}
                Children [
                    @variant("Tab bar", bsn! {
                        TabBar
                        @{Selection::builder().one().scene_with_items(["Overview", "Trains", "Stops"].map(Item::new))}
                    })
                    --
                    @variant("Bread crumbs", bsn! {
                        ~{BreadCrumbs}
                        @{Selection::builder().one_or_more().scene_with_items(["Home", "Lines", "Line 4"].map(Item::new))}
                    })
                ]
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
            #Page
            ~{page()}
            Children [
                ~{Label::header("Slider and stepper")}
                --
                ~{groups()}
                Children [
                    @variant("Slider: continuous", bsn! {
                        ~{valid_slider(0.5, 0.0, 1.0, None).with_name("Volume")}
                    })
                    --
                    @variant("Slider: stepped and formatted", bsn! {
                        ~{valid_slider(60.0, 0.0, 120.0, Some(10.0)).with_format(kilometres_per_hour).with_name("Speed")}
                    })
                    --
                    @variant("Slider: disabled", bsn! {
                        ~{valid_slider(4.0, 0.0, 10.0, Some(1.0)).with_name("Locked")}
                        InteractionDisabled
                    })
                    --
                    @variant("Stepper: default", bsn! { NumericStepper })
                    --
                    @variant("Stepper: float", bsn! {
                        NumericStepper
                        ~{float_value}
                    })
                    --
                    @variant("Stepper: u8, in steps of 7", bsn! {
                        NumericStepper
                        ~{level_value}
                    })
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
            #Page
            ~{page()}
            Children [
                ~{Label::header("Text field")}
                --
                ~{groups()}
                Children [
                    @variant("With text", bsn! {
                        ~{TextField::new("Amsterdam Centraal").with_name("Name")}
                    })
                    --
                    @variant("Placeholder, narrow", bsn! {
                        ~{TextField::new("")
                            .with_placeholder("Seed")
                            .with_name("Seed")
                            .with_max_width(Val::Px(200.0))}
                    })
                ]
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
    let toned = vec![
        Item {
            data: Tool::in_group(icon.0.clone(), "Neutral", 0),
        },
        Item {
            data: Tool::in_group(icon.0.clone(), "Warning", 1).with_tone(Tone::Warning),
        },
        Item {
            data: Tool::in_group(icon.0.clone(), "Danger", 2).with_tone(Tone::Danger),
        },
    ];
    let (filled, hidden, underlined) = (tools(&icon.0), tools(&icon.0), tools(&icon.0));
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Toolbar")}
                --
                ~{groups()}
                Children [
                    @variant("Filled, with key hints and a danger tool", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Filled)}
                        @{Selection::builder().one().scene_with_items(filled)}
                    })
                    --
                    @variant("Filled, labels hidden", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Filled)}
                        ~{ToolbarLabels::Hidden}
                        @{Selection::builder().one().scene_with_items(hidden)}
                    })
                    --
                    @variant("Underlined, none or one, the label is the tooltip", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Underlined)}
                        @{Selection::builder().zero_or_one().scene_with_items(underlined)}
                    })
                    --
                    @variant("Tool tones: neutral, warning, danger", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Filled)}
                        @{Selection::builder().one().scene_with_items(toned)}
                    })
                ]
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
            #Page
            ~{page()}
            Children [
                ~{Label::header("Gallery")}
                --
                ~{groups()}
                Children [
                    @variant("Two groups of cards, in a horizontal scroll area", bsn! {
                        Node { width: px(600.0) }
                        @gallery_scroll_area(cards)
                    })
                ]
            ]
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

#[derive(Component, Default, Clone)]
struct ControlLines;

fn cards(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let (close, close_scrolling) = (icon.0.clone(), icon.0.clone());
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Cards and panels")}
                --
                ~{groups()}
                Children [
                    @variant("Card", bsn! {
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
                    })
                    --
                    @variant("Panel: closable, with a subtitle", bsn! {
                        Panel { title: "Trains", subtitle: {Some("3 open".to_string())}, close_icon: {HandleTemplate::Handle(close)} }
                        Lines(3)
                    })
                    --
                    // A body past 150 pixels scrolls.
                    @variant("Panel: the body scrolls past 150 px", bsn! {
                        Panel { title: "Stops", close_icon: {HandleTemplate::Handle(close_scrolling.clone())}, max_body_height: 150.0 }
                        Lines(12)
                    })
                    --
                    @variant("Panel: scrolling controls", bsn! {
                        Panel { title: "Controls", close_icon: {HandleTemplate::Handle(close_scrolling)}, max_body_height: 180.0 }
                        ControlLines
                    })
                ]
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

#[expect(
    clippy::expect_used,
    reason = "the panel uses a fixed valid value range"
)]
fn fill_control_panels(
    mut commands: Commands,
    panels: Query<(Entity, &PanelRefs), (With<ControlLines>, Without<Filled>)>,
) {
    for (panel, refs) in &panels {
        commands.entity(panel).insert(Filled);
        for index in 0..12 {
            let step_value = Value::new_u8(index, 0, 20, 1).expect("a valid 0 to 20 range");
            let row = commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ChildOf(refs.body()),
                ))
                .id();
            commands.spawn((
                PushButton::from_label(format!("Action {}", index + 1)),
                ChildOf(row),
            ));
            commands.spawn((
                Combo,
                Selection::builder()
                    .zero_or_one()
                    .build_with_items(["Red", "White", "Blue", "Yellow"].map(Item::new)),
                ChildOf(row),
            ));
            commands.spawn((valid_slider(0.5, 0.0, 1.0, None), ChildOf(refs.body())));
            commands.spawn((NumericStepper::default(), step_value, ChildOf(refs.body())));
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
            #Page
            ~{page()}
            Children [
                ~{Label::header("Setting rows")}
                --
                ~{groups()}
                Children [
                    @variant("Rows with a segmented control, a stepper and a button", bsn! {
                        SettingRows
                        Node { width: px(420.0) }
                        Children [
                            ~{SettingRow::new("Appearance", "Light or dark")} Control::Appearance
                            --
                            ~{SettingRow::new("Text size", "In points")} Control::TextSize
                            --
                            ~{SettingRow::new("Reset", "Back to the defaults")} Control::Reset
                        ]
                    })
                ]
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
            #Page
            ~{page()}
            Children [
                ~{Label::header("Scroll area")}
                --
                ~{groups()}
                Children [
                    @variant("Vertical", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @list_scroll_area(true, false)
                    })
                    --
                    @variant("Vertical and horizontal", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @list_scroll_area(true, true)
                    })
                ]
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

fn feedback(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tooltip and toast")}
                --
                ~{groups()}
                Children [
                    @variant("Tooltip on a push button", bsn! {
                        ~{PushButton::from_label("Hover me")}
                        ~{Tooltip::new("A tooltip appears after half a second")}
                    })
                    --
                    @variant("The tooltip an icon button carries", bsn! {
                        IconButton { icon: {icon}, name: "Hover me too", size: IconButtonSize::Regular }
                    })
                    --
                    @variant("Toast", bsn! {
                        ~{PushButton::from_label("Raise a toast")}
                        on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
                            commands.spawn((Toast::new("Saved to rails.ron"), ChildOf(*root)));
                        })
                    })
                ]
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
