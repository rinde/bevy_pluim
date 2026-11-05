//! `Combo` variants: shared and own selections, options with a note, and the `FilterCombo`s that
//! filter their options as a person types.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::combo::Combo;
use bevy_pluim::filter_combo::FilterCombo;
use bevy_pluim::list::ItemNote;
use bevy_pluim::push_button::PushButton;
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
            // Type to filter the options, Arrow Down / Up to move through the matches and Enter
            // to pick one; it shares its selection and items with the combos above.
            ~{FilterCombo::default().with_name("Country")} ~{Selection::reference(selection_ref)}
            --
            // A selection of many stays open after Enter, with the query, for the next pick.
            ~{FilterCombo::default().with_name("Stations")}
            @{Selection::builder().zero_or_more().scene_with_items([
                Item::new("Amsterdam"),
                Item::new("Delft"),
                Item::new("Den Haag"),
                Item::new("Utrecht"),
            ])}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}
