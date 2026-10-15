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
    let trains = Panel::new("Trains")
        .with_subtitle("3 open")
        .with_close_icon(close.clone());
    let stops = Panel::new("Stops")
        .with_close_icon(close)
        .with_max_body_height(150.0);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            // The app decides what closing means: this one despawns the panel.
            common::inserted(trains)
            Lines(3)
            on(|closed: On<PanelClosed>, mut commands: Commands| {
                commands.entity(closed.entity).despawn();
            })
            --
            // A body past 150 pixels scrolls.
            common::inserted(stops)
            Lines(12)
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
    // The panels name their bodies once spawned; the app fills them.
    commands.queue(|world: &mut World| {
        world.flush();
        let panels: Vec<(Entity, usize)> = world
            .query::<(&PanelRefs, &Lines)>()
            .iter(world)
            .map(|(refs, lines)| (refs.body, lines.0))
            .collect();
        for (body, lines) in panels {
            for line in 1..=lines {
                world.spawn((Label::regular(format!("Line {line}")), ChildOf(body)));
            }
        }
    });
}

/// How many lines of text the body of a panel gets.
#[derive(Component, Default, Clone)]
struct Lines(usize);
