//! Every widget and its variants, one tab per widget.

mod ui;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy_pluim::WidgetsPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WidgetsPlugin, ui::OverviewPlugin))
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .run();
}
