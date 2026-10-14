//! `IconButton`: regular, small and large buttons, a toggle, the warning and danger tones, and a
//! button that switches the theme.

mod common;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::push_button::PushButton;
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

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);
    let icon = common::cross_icon(&mut images);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        children![
            IconButton::new(icon.clone(), "Small", IconButtonSize::Small),
            IconButton::new(icon.clone(), "Regular", IconButtonSize::Regular),
            IconButton::new(icon.clone(), "Large", IconButtonSize::Large),
            // A toggle: an `Activate` flips its `ToggleState`.
            (
                IconButton::new(icon.clone(), "Toggle", IconButtonSize::Regular),
                ToggleState(false),
            ),
            (
                IconButton::new(icon.clone(), "Warning", IconButtonSize::Large)
                    .with_tone(Tone::Warning),
            ),
            (IconButton::new(icon, "Danger", IconButtonSize::Regular).with_tone(Tone::Danger)),
            (
                PushButton::from_label("Change theme"),
                Observer::new(|_: On<Activate>, mut theme: ResMut<Theme>| {
                    theme.light_dark_toggle();
                }),
            ),
        ],
    ));
}
