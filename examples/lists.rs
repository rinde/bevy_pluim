//! `List` layouts and corner styles.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::list::{CornerStyle, List, ListLayout};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(EguiPlugin::default())
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let selection_ref = commands
        .spawn(Selection::builder().one_or_more().build_with_items([
            Item::new("Netherlands"),
            Item::new("Canada"),
            Item::new("Indonesia"),
            Item::new("Portugal"),
            Item::new("Belgium"),
        ]))
        .id();

    // Root container
    commands.spawn((
        Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        children![
            (List, Selection::reference(selection_ref),),
            (
                List,
                ListLayout::Horizontal,
                Selection::builder().one().build_with_items([
                    Item::new("NL"),
                    Item::new("CA"),
                    Item::new("ID"),
                    Item::new("PT"),
                    Item::new("BE")
                ]),
            ),
            (
                List,
                ListLayout::VerticalGrid { columns: 3 },
                Selection::reference(selection_ref),
            ),
            (
                List,
                CornerStyle::Tabs,
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(selection_ref),
            ),
            (
                List,
                ListLayout::Horizontal,
                CornerStyle::Tabs,
                Selection::builder()
                    .zero_or_more()
                    .build_with_items_ref(selection_ref),
            ),
            (
                List,
                ListLayout::VerticalGrid { columns: 3 },
                CornerStyle::Tabs,
                Selection::builder().zero_or_more().build_with_items([
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ])
            ),
            (PushButton::from_label("Change theme"), ThemeButton),
        ],
    ));
}

#[derive(Component)]
struct ThemeButton;

fn on_press(
    trigger: Trigger<Pointer<Released>>,
    button: Query<(), With<ThemeButton>>,
    mut theme: ResMut<Theme>,
) {
    if button.get(trigger.target()).is_ok() {
        theme.light_dark_toggle();
    }
}
