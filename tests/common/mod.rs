//! Shared by the scenario tests: an app to play scenarios in, and the camera
//! and root node every scenario's setup needs.

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
///
/// `Scenario::headless_app` is not used because it returns an app that
/// refuses `add_plugins`. `ClearColor` is normally inserted by the render
/// plugins, which `HeadlessPlugins` leaves out, but `theme.rs` writes to it
/// whenever the `Theme` changes.
pub fn widgets_app() -> App {
    let mut app = App::new();
    app.add_plugins(HeadlessPlugins::default());
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    app
}

/// Spawns a `Camera2d` and the one root `Node` that fills the window, and
/// returns the root.
///
/// A scenario's setup runs before the first update, so `UiRoot`'s
/// `PostStartup` check finds exactly this root.
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
