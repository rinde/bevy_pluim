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
#[derive(Component, Default, Clone)]
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
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::FlexStart,
            column_gap: px(20.0),
            padding: UiRect::all(px(20.0)),
        }
        Children [
            IconButton { icon: {icon.clone()}, name: "Trains", size: IconButtonSize::Large }
            ~{Badge::count(3)}
            Counted
            --
            IconButton { icon: {icon.clone()}, name: "Warnings", size: IconButtonSize::Large }
            ~{Badge::count(12).with_tone(BadgeTone::Warning)}
            --
            IconButton { icon, name: "Stops", size: IconButtonSize::Large }
            --
            ~{PushButton::from_label("Add one")}
            on(add_one)
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| {
                theme.light_dark_toggle();
            })
        ]
    });
}

/// Raises the count of the badge on the `Counted` button: a new count is a new `Badge`, and the
/// pill follows it.
fn add_one(
    _: On<Activate>,
    mut commands: Commands,
    badges: Query<(Entity, &Badge), With<Counted>>,
) {
    for (entity, badge) in &badges {
        let count = badge.text.parse::<usize>().unwrap_or(0);
        commands.entity(entity).insert(Badge {
            text: (count + 1).to_string(),
            ..badge.clone()
        });
    }
}
