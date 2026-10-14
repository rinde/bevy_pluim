//! `Gallery`: image cards in two titled groups, in a horizontal scroll area.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::gallery::{Gallery, GalleryCard};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection};
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    // No image set ships with the crate: a real app passes its own white-on-transparent images.
    let cards = [
        ("Steam", "Locomotives"),
        ("Diesel", "Locomotives"),
        ("Electric", "Locomotives"),
        ("Passenger", "Wagons"),
        ("Box wagon", "Wagons"),
        ("Tank wagon", "Wagons"),
        ("Hopper", "Wagons"),
        ("Flat wagon", "Wagons"),
    ]
    .map(|(label, group)| Item {
        data: GalleryCard::new(Handle::default(), label, group),
    });
    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .id();
    commands.spawn((
        Node {
            width: Val::Px(500.0),
            ..default()
        },
        ScrollArea::bundle(
            move |s| {
                s.spawn((Gallery, Selection::builder().one().build_with_items(cards)));
            },
            false,
            true,
        ),
        ChildOf(root),
    ));
    commands
        .spawn((PushButton::from_label("Change theme"), ChildOf(root)))
        .observe(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle());
}
