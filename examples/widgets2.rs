use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use ui::WidgetsPlugin;
use ui::colors::BG_SECONDARY;
use ui::combo2;
use ui::selectable_items::SelectableItems;
use ui::selectable_items::Selection;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Root container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(BG_SECONDARY),
        children![
            (
                combo2::Combo,
                children![
                //    combo2::ComboButton
                ]
            ),
            SelectableItems::new(["hello", "who", "is", "there"]),
            (
                SelectableItems::new(["Netherlands", "Canada", "Indonesia", "Portugal"]),
                Selection(Some(1))
            ),
        ],
    ));
}
