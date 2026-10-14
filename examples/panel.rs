//! `Panel`: one that closes when its close button is activated, and one whose body scrolls.

mod common;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::panel::{Panel, PanelClosed, PanelRefs};
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
    let close = common::cross_icon(&mut images);
    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .id();
    // The app decides what closing means: this one despawns the panel.
    let closable = commands
        .spawn((
            Panel::new("Trains")
                .with_subtitle("3 open")
                .with_close_icon(close.clone()),
            ChildOf(root),
        ))
        .observe(|closed: On<PanelClosed>, mut commands: Commands| {
            commands.entity(closed.entity).despawn();
        })
        .id();
    // A body past 150 pixels scrolls.
    let tall = commands
        .spawn((
            Panel::new("Stops")
                .with_close_icon(close)
                .with_max_body_height(150.0),
            ChildOf(root),
        ))
        .id();
    commands
        .spawn((PushButton::from_label("Change theme"), ChildOf(root)))
        .observe(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle());
    // The panels name their bodies once spawned; the app fills them.
    commands.queue(move |world: &mut World| {
        world.flush();
        for (panel, lines) in [(closable, 3), (tall, 12)] {
            let Some(body) = world.get::<PanelRefs>(panel).map(|refs| refs.body) else {
                continue;
            };
            for line in 1..=lines {
                world.spawn((Label::regular(format!("Line {line}")), ChildOf(body)));
            }
        }
    });
}
