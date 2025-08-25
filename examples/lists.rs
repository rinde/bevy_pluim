use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use ui::WidgetsPlugin;
use ui::list::{CornerStyle, List, ListLayout};
use ui::push_button::PushButton;
use ui::select::{Item, ItemValues, ItemsRef, Selection};
use ui::theme::Theme;

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
        .spawn((
            Selection::one_or_more(),
            related!(ItemValues[
                Item::new("Netherlands"),
                Item::new("Canada"),
                Item::new("Indonesia"),
                Item::new("Portugal"),
                Item::new("Belgium")
            ]),
        ))
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
            (
                List,
                Selection::reference(selection_ref),
                ItemsRef(selection_ref),
            ),
            (
                List,
                ListLayout::Horizontal,
                Selection::zero_or_more(),
                related!(ItemValues[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                List,
                ListLayout::VerticalGrid { columns: 3 },
                Selection::reference(selection_ref),
                ItemsRef(selection_ref)
            ),
            (
                List,
                CornerStyle::Tabs,
                Selection::zero_or_one(),
                ItemsRef(selection_ref)
            ),
            (
                List,
                ListLayout::Horizontal,
                CornerStyle::Tabs,
                Selection::zero_or_more(),
                ItemsRef(selection_ref)
            ),
            (
                List,
                ListLayout::VerticalGrid { columns: 3 },
                CornerStyle::Tabs,
                Selection::zero_or_more(),
                related!(ItemValues[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
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
