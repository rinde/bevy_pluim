//! `Toast`: press the buttons to raise messages that stack at the bottom and go after three
//! seconds; the same text restarts the one showing.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui::toast::Toast;
use rin_bevy_ui::{UiRoot, WidgetsPlugin};

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

/// Raises a toast with `text` under the UI root.
fn raise(commands: &mut Commands, root: Entity, text: &'static str) {
    commands.spawn((Toast::new(text), ChildOf(root)));
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(bsn! {
           Node {
               width: percent(100.0),
               height: percent(100.0),
               align_items: AlignItems::FlexStart,
               column_gap: px(20.0),
               padding: UiRect::all(px(20.0)),
           }
           Children [
               ~{PushButton::from_label("Saved")}
               on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
    raise(&mut commands, *root, "Saved to rails.ron");
    })
               --
               ~{PushButton::from_label("Deleted")}
               on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
    raise(&mut commands, *root, "Deleted 3 pieces");
    })
               --
               ~{PushButton::from_label("Refused")}
               on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
    raise(&mut commands, *root, "The platform cannot grow that way");
    })
               --
               ~{PushButton::from_label("Change theme")}
               on(|_: On<Activate>, mut theme: ResMut<Theme>| {
                   theme.light_dark_toggle();
               })
           ]
       });
}
