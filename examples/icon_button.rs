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
    let small = IconButton::new(icon.clone(), "Small", IconButtonSize::Small);
    let regular = IconButton::new(icon.clone(), "Regular", IconButtonSize::Regular);
    let large = IconButton::new(icon.clone(), "Large", IconButtonSize::Large);
    let toggle = IconButton::new(icon.clone(), "Toggle", IconButtonSize::Regular);
    let warning =
        IconButton::new(icon.clone(), "Warning", IconButtonSize::Large).with_tone(Tone::Warning);
    let danger = IconButton::new(icon, "Danger", IconButtonSize::Regular).with_tone(Tone::Danger);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::FlexStart,
            column_gap: px(20.0),
            padding: UiRect::all(px(20.0)),
        }
        Children [
            common::inserted(small)
            --
            common::inserted(regular)
            --
            common::inserted(large)
            --
            // A toggle: an `Activate` flips its `ToggleState`.
            common::inserted(toggle)
            ~{ToggleState(false)}
            --
            common::inserted(warning)
            --
            common::inserted(danger)
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| {
                theme.light_dark_toggle();
            })
        ]
    });
}
