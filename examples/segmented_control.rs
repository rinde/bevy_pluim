//! `SegmentedControl`: one with its own selection, a three-way one, two that share one, and
//! ones with icons, with and without their text.

mod common;

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::segmented_control::{ItemIcon, SegmentLabels, SegmentedControl};
use bevy_pluim::select::{Item, Selection};
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

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = common::cross_icon(&mut images);
    let with_icon = |text| (Item::new(text), ItemIcon(icon.clone()));
    commands.spawn(Camera2d);
    let shared = commands
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["Dark", "Light"].map(Item::new)),
        )
        .id();
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            SegmentedControl
            @{Selection::builder().one().scene_with_items(["Auto", "Manual"].map(Item::new))}
            --
            SegmentedControl ~{Selection::reference(shared)}
            --
            SegmentedControl ~{Selection::reference(shared)}
            --
            SegmentedControl
            @{Selection::builder().one().scene_with_items(["Dark", "System", "Light"].map(Item::new))}
            --
            SegmentedControl
            @{Selection::builder().one().scene_with_items(["Grid", "List"].map(with_icon))}
            --
            SegmentedControl ~{SegmentLabels::Hidden}
            @{Selection::builder().one().scene_with_items(["Map", "Table"].map(with_icon))}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}
