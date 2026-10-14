//! `Badge`: a neutral and a warning count on the corner of icon buttons, and a button that
//! raises one of them.

mod common;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::badge::{Badge, BadgeTone};
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::theme::Theme;

/// Marks the button whose badge "Add one" raises.
#[derive(Component)]
struct Counted;

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
            (
                IconButton::new(icon.clone(), "Trains", IconButtonSize::Large),
                Badge::count(3),
                Counted,
            ),
            (
                IconButton::new(icon.clone(), "Warnings", IconButtonSize::Large),
                Badge::count(12).with_tone(BadgeTone::Warning),
            ),
            IconButton::new(icon, "Stops", IconButtonSize::Large),
            (
                PushButton::from_label("Add one"),
                // A new count is a new `Badge`; the pill follows it.
                Observer::new(
                    |_: On<Activate>, mut badges: Query<&mut Badge, With<Counted>>| {
                        for mut badge in &mut badges {
                            let count = badge.text.parse::<usize>().unwrap_or(0);
                            badge.text = (count + 1).to_string();
                        }
                    }
                ),
            ),
            (
                PushButton::from_label("Change theme"),
                Observer::new(|_: On<Activate>, mut theme: ResMut<Theme>| {
                    theme.light_dark_toggle();
                }),
            ),
        ],
    ));
}
