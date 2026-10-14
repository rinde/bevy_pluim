//! `Tooltip`: rest the pointer on the button, or on the icon button (which has its name as a
//! tooltip), to see the popup.

mod common;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui::tooltip::Tooltip;

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
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        children![
            (
                PushButton::from_label("Hover me"),
                Tooltip::new("A tip after half a second"),
            ),
            IconButton::new(icon, "Close", IconButtonSize::Large),
            (
                PushButton::from_label("Change theme"),
                Observer::new(|_: On<Activate>, mut theme: ResMut<Theme>| {
                    theme.light_dark_toggle();
                }),
            ),
        ],
    ));
}
