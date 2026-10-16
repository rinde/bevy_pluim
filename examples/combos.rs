//! `Combo` variants.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::list::ItemNote;
use rin_bevy_ui::push_button::PushButton;
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

    let selection_ref = commands
        .spawn(Selection::builder().zero_or_one().build_with_items([
            Item::new("Netherlands"),
            Item::new("Canada"),
            Item::new("Indonesia"),
            Item::new("Portugal"),
            Item::new("Belgium"),
        ]))
        .id();
    // Options with a note, as a station picker marks the ones a train cannot reach.
    let noted_ref = commands
        .spawn(Selection::builder().zero_or_one().build_with_items([
            (Item::new("Amsterdam"), ItemNote::new("")),
            (Item::new("Delft"), ItemNote::warning("< unreachable >")),
            (Item::new("Utrecht"), ItemNote::new("4 km")),
        ]))
        .id();
    // Root container
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
        }
        Children [
            ~{Combo} ~{Selection::reference(selection_ref)}
            --
            ~{Combo}
            @{Selection::builder().zero_or_one().scene_with_items_ref(selection_ref)}
            --
            ~{Combo} ~{Selection::reference(selection_ref)}
            --
            ~{Combo} ~{Selection::reference(noted_ref)}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}
