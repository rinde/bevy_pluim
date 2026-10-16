//! `ColorPalette` and `ColorPaletteCombo` over `tailwind::COLORS`.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::list::ListLayout;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::tailwind::COLORS;
use rin_bevy_ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let selection_ref = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(color_palette()),
        )
        .id();

    let selection_ref2 = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        )
        .id();

    // Root container
    commands.spawn_scene(bsn! {
        Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
        }
        Children [
            ~{Selection::reference(selection_ref)}
            ~{ColorPalette}
            ~{ListLayout::VerticalGrid { columns: 11 }}
            --
            Node {
                width: px(430.0),
                height: px(300.0),
            }
            @palette_scroll_area(selection_ref)
            --
            Node {
                width: px(230.0),
                height: px(600.0),
            }
            @zero_or_one_palette_scroll_area(selection_ref)
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref2)}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref2)}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            @{Selection::builder().zero_or_one().scene_with_items_ref(selection_ref)}
            --
            Node {
                width: px(100.0),
                height: px(100.0),
            }
            @combo_scroll_area(selection_ref)
            --
            Node
            Children [
                ~{PushButton::from_label("Change theme")}
                on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
                --
                ~{PushButton::from_label("+")}
                on(add_combo)
                --
                ~{PushButton::from_label("-")}
                on(remove_combo)
            ]
            --
            Node
            ExtraComponentsArea
        ]
    });
}

/// A scroll area over a palette that shares the selection `selection_ref`.
fn palette_scroll_area(selection_ref: Entity) -> impl Scene {
    ScrollArea::scene(
        bsn_list! {
            Pickable {
                should_block_lower: false,
            }
            ~{ColorPalette}
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref)}
        },
        true,
        true,
    )
}

/// A scroll area over a palette with a selection of its own over the items of `selection_ref`.
fn zero_or_one_palette_scroll_area(selection_ref: Entity) -> impl Scene {
    ScrollArea::scene(
        bsn_list! {
            Pickable {
                should_block_lower: false,
            }
            ~{ColorPalette}
            ~{ListLayout::VerticalGrid { columns: 11 }}
            @{Selection::builder().zero_or_one().scene_with_items_ref(selection_ref)}
        },
        true,
        true,
    )
}

/// A scroll area over a palette combo and a card, to see the popup is not clipped.
fn combo_scroll_area(selection_ref: Entity) -> impl Scene {
    ScrollArea::scene(
        bsn_list! {
            Pickable {
                should_block_lower: false,
            }
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref)}
            --
            Node {
                min_width: px(50.0),
                width: px(100.0),
                height: px(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            ~{Card}
            Pickable {
                should_block_lower: false,
            }
        },
        true,
        true,
    )
}

fn color_palette() -> Vec<Item<Color>> {
    COLORS.iter().map(|c| Item { data: c.color }).collect()
}

#[derive(Component, Default, Clone)]
struct ExtraComponentsArea;

fn add_combo(
    _: On<Activate>,
    area: Single<Entity, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    commands.entity(*area).with_children(|parent| {
        parent.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::builder()
                .zero_or_one()
                .build_with_items(color_palette()),
        ));
    });
}

fn remove_combo(
    _: On<Activate>,
    area: Single<&Children, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    if let Some(last) = area.iter().last() {
        commands.entity(last).despawn();
    }
}
