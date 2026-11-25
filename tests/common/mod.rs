//! Shared by the scenario tests: the app and the camera and root node.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private test module"
)]

use bevy::camera::ClearColor;
use bevy::prelude::*;
use bevy_pluim::WidgetsPlugin;
use bevy_stagecraft::HeadlessPlugins;

/// A finished headless app with [`WidgetsPlugin`], ready for
/// `Scenario::run_in`.
// `Scenario::headless_app` refuses `add_plugins`; `theme.rs` writes `ClearColor`.
pub fn widgets_app() -> App {
    let mut app = App::new();
    app.add_plugins(HeadlessPlugins::default());
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    app
}

/// Spawns a `Camera2d` and the one full-window root `Node`, and returns the root.
pub fn spawn_camera_and_root(world: &mut World) -> Entity {
    world.spawn(Camera2d);
    world
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(20),
            padding: UiRect::all(px(20)),
            ..default()
        })
        .id()
}
