//! `IconButton`: regular, small and large buttons, a toggle, the warning and danger tones, and a
//! button that switches the theme.

mod common;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::theme::Theme;

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
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::FlexStart,
            column_gap: px(20.0),
            padding: UiRect::all(px(20.0)),
        }
        Children [
            IconButton { icon: {icon.clone()}, name: "Small", size: IconButtonSize::Small }
            --
            IconButton { icon: {icon.clone()}, name: "Regular", size: IconButtonSize::Regular }
            --
            IconButton { icon: {icon.clone()}, name: "Large", size: IconButtonSize::Large }
            --
            // A toggle: an `Activate` flips its `ToggleState`.
            IconButton { icon: {icon.clone()}, name: "Toggle", size: IconButtonSize::Regular }
            ~{ToggleState(false)}
            --
            IconButton { icon: {icon.clone()}, name: "Warning", size: IconButtonSize::Large, tone: Tone::Warning }
            --
            IconButton { icon, name: "Danger", size: IconButtonSize::Regular, tone: Tone::Danger }
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| {
                theme.light_dark_toggle();
            })
        ]
    });
}
