//! A widget overview: a vertical `TabBar` on the left with one tab per widget, and for the tab
//! picked a page with that widget's variants over example data. Every example of the library is a
//! tab here, the theme editor included; only the `hud` and the tester's `gallery` are their own.
//!
//! Every page has the same layout: a header, then the variants in a centred, wrapping row, each a
//! group with a secondary label naming what is different about it. The theme is switched with the
//! `LightDarkToggle` above the tabs, which is one of the widgets of the library, too.

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
