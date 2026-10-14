//! The redesign's HUD built from this crate's widgets over sample data: run state, a warnings
//! button, the panel toolbar with its panels, the tool dock and the bars above it. A plain
//! background stands in for the map.

mod ui;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use rin_bevy_ui::WidgetsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(ui::HudPlugin)
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .run();
}
