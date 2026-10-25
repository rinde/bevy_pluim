//! A theme editor: the library's `ThemeEditor` edits every colour and number of the `Theme`
//! of the light or dark half, while the widgets change; "Export" writes the result as Rust text.

mod ui;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use rin_bevy_ui::WidgetsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(ui::ThemeEditorExamplePlugin)
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .run();
}
