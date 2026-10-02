//! `ColorPalette` and `ColorPaletteCombo` over `tailwind::COLORS`.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::UiDebugOptions;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_observer(on_press_add_combo_button)
        .add_observer(on_press_remove_combo_button)
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
    commands.spawn((
        Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        children![
            (
                Selection::reference(selection_ref),
                ColorPalette,
                ListLayout::VerticalGrid { columns: 11 },
            ),
            (
                Node {
                    width: Val::Px(430.0),
                    height: Val::Px(300.0),
                    ..default()
                },
                ScrollArea::bundle(
                    move |parent| {
                        parent.spawn((
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                            ColorPalette,
                            ListLayout::VerticalGrid { columns: 11 },
                            Selection::reference(selection_ref),
                        ));
                    },
                    true,
                    true
                )
            ),
            (
                Node {
                    width: Val::Px(230.0),
                    height: Val::Px(600.0),
                    ..default()
                },
                ScrollArea::bundle(
                    move |parent| {
                        parent.spawn((
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                            ColorPalette,
                            ListLayout::VerticalGrid { columns: 11 },
                            Selection::builder()
                                .zero_or_one()
                                .build_with_items_ref(selection_ref),
                        ));
                    },
                    true,
                    true
                )
            ),
            (
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(selection_ref2),
            ),
            (
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(selection_ref2),
            ),
            (
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(selection_ref),
            ),
            (
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(100.0),
                    ..default()
                },
                ScrollArea::bundle(
                    move |parent| {
                        parent.spawn((
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                            ColorPaletteCombo,
                            ListLayout::VerticalGrid { columns: 11 },
                            Selection::reference(selection_ref),
                        ));
                        parent.spawn((
                            Node {
                                min_width: Val::Px(50.0),
                                width: Val::Px(100.0),
                                height: Val::Px(100.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Card,
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                        ));
                    },
                    true,
                    true
                )
            ),
            (
                Node::DEFAULT,
                children![
                    (PushButton::from_label("Change theme"), ThemeButton),
                    (PushButton::from_label("+"), AddComboButton),
                    (PushButton::from_label("-"), RemoveComboButton),
                ]
            ),
            (Node::DEFAULT, ExtraComponentsArea)
        ],
    ));
}

fn color_palette() -> Vec<impl Bundle> {
    COLORS.iter().map(|c| Item { data: c.color }).collect()
}

#[derive(Component)]
struct ThemeButton;

#[derive(Component)]
struct AddComboButton;

#[derive(Component)]
struct RemoveComboButton;

#[derive(Component)]
struct ExtraComponentsArea;

fn on_press(
    trigger: On<Pointer<Release>>,
    button: Query<(), With<ThemeButton>>,
    mut theme: ResMut<Theme>,
) {
    if button.get(trigger.event().event_target()).is_ok() {
        theme.light_dark_toggle();
    }
}

fn on_press_add_combo_button(
    trigger: On<Pointer<Release>>,
    button: Query<(), With<AddComboButton>>,
    area: Single<Entity, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    if button.get(trigger.event().event_target()).is_ok() {
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
}

fn on_press_remove_combo_button(
    trigger: On<Pointer<Release>>,
    button: Query<(), With<RemoveComboButton>>,
    area: Single<&Children, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    if button.get(trigger.event().event_target()).is_ok()
        && let Some(last) = area.iter().last()
    {
        commands.entity(last).despawn();
    }
}
