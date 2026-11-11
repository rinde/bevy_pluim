//! The overview: the tabs, their pages and the plugin that builds them.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private module"
)]

#[path = "../../common/mod.rs"]
mod common;

mod buttons;
mod cards;
mod colours;
mod combos;
mod feedback;
mod lists;
mod navigation;
mod scroll_area;
mod selection;
mod text;
pub mod theme_editor;
mod toolbars;
mod values;

use bevy::asset::HandleTemplate;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;
use bevy_pluim::UiRoot;
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

#[derive(Debug, Clone, Copy)]
pub struct OverviewPlugin;

impl Plugin for OverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                cards::fill_panels,
                cards::fill_control_panels,
                cards::fill_setting_rows,
            ),
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
    add(
        "Buttons",
        StackItem::new(commands.register_system(buttons::buttons)),
    );
    add(
        "Icon buttons",
        StackItem::new(commands.register_system(buttons::icon_buttons)),
    );
    add(
        "Labels",
        StackItem::new(commands.register_system(text::labels)),
    );
    add("Tags", StackItem::new(commands.register_system(text::tags)));
    add(
        "Lists",
        StackItem::new(commands.register_system(lists::lists)),
    );
    add(
        "Combos",
        StackItem::new(commands.register_system(combos::combos)),
    );
    add(
        "Filter combos",
        StackItem::new(commands.register_system(combos::filter_combos)),
    );
    add(
        "Colours",
        StackItem::new(commands.register_system(colours::colours)),
    );
    add(
        "Segmented control",
        StackItem::new(commands.register_system(selection::segmented)),
    );
    add(
        "Light dark toggle",
        StackItem::new(commands.register_system(selection::light_dark)),
    );
    add(
        "Filter chips",
        StackItem::new(commands.register_system(selection::chips)),
    );
    add(
        "Tabs and trails",
        StackItem::new(commands.register_system(navigation::navigation)),
    );
    add(
        "Slider and stepper",
        StackItem::new(commands.register_system(values::values)),
    );
    add(
        "Text field",
        StackItem::new(commands.register_system(values::text_fields)),
    );
    add(
        "Toolbar",
        StackItem::new(commands.register_system(toolbars::toolbars)),
    );
    add(
        "Gallery",
        StackItem::new(commands.register_system(toolbars::galleries)),
    );
    add(
        "Cards and panels",
        StackItem::new(commands.register_system(cards::cards)),
    );
    add(
        "Setting rows",
        StackItem::new(commands.register_system(cards::setting_rows)),
    );
    add(
        "Scroll area",
        StackItem::new(commands.register_system(scroll_area::scroll_areas)),
    );
    add(
        "Tooltip and toast",
        StackItem::new(commands.register_system(feedback::feedback)),
    );
    add(
        "Theme editor",
        StackItem::new(commands.register_system(theme_editor::theme_editor)),
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

/// A slider with a valid range and step: these are fixed values.
#[expect(clippy::expect_used, reason = "the sliders use fixed valid values")]
fn valid_slider(value: f32, min: f32, max: f32, step: Option<f32>) -> Slider {
    Slider::new(value, min, max)
        .and_then(|slider| match step {
            Some(step) => slider.with_step(step),
            None => Ok(slider),
        })
        .expect("a valid slider")
}
