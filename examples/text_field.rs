//! `TextField`: a name field and a seed field with a placeholder, and what they report.
//!
//! Click a field to type in it; `Enter`, `Escape` (which puts the text back) or a click
//! anywhere else hands the keys back.

use bevy::prelude::*;
use bevy::ui::Val;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::label::Label;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::text_field::{TextField, TextFieldChange, TextFieldCommit};
use bevy_pluim::theme::Theme;

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

/// Marks the label that says what the fields last reported.
#[derive(Component, Default, Clone)]
struct Report;

fn show_change(
    change: On<TextFieldChange>,
    mut commands: Commands,
    report: Query<Entity, With<Report>>,
) {
    for entity in &report {
        commands
            .entity(entity)
            .insert(Label::secondary(format!("Typing: {}", change.value)));
    }
}

fn show_commit(
    commit: On<TextFieldCommit>,
    mut commands: Commands,
    report: Query<Entity, With<Report>>,
) {
    for entity in &report {
        commands
            .entity(entity)
            .insert(Label::regular(format!("Committed: {}", commit.value)));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            ~{TextField::new("Amsterdam Centraal").with_name("Name")}
            on(show_change)
            on(show_commit)
            --
            ~{TextField::new("")
                .with_placeholder("Seed")
                .with_name("Seed")
                .with_max_width(Val::Px(200.0))}
            on(show_change)
            on(show_commit)
            --
            Report ~{Label::secondary("Click a field to type in it")}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}
