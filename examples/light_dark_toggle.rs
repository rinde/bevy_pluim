//! `LightDarkToggle`: a light / system / dark switch that follows the system theme by default.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::label::Label;
use bevy_pluim::light_dark_toggle::LightDarkToggle;

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
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            ~{Label::title("Appearance")}
            --
            LightDarkToggle
        ]
    });
}
