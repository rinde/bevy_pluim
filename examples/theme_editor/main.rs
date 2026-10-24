//! A theme editor: pick a colour for each slot of the `ThemeColors` of the light or dark theme,
//! watch the widgets change, and export the pair as Rust text for `Theme::with_colors`.

mod ui;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use rin_bevy_ui::WidgetsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(ui::ThemeEditorPlugin)
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .run();
}
