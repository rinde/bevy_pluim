use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use smallvec::smallvec;
use ui::WidgetsPlugin;
use ui::combo::Combo;
use ui::float::Float;
use ui::list::List;
use ui::push_button::PushButton;
use ui::select::Item;
use ui::select::Items;
use ui::select::Selection;
use ui::theme::Theme;
use ui::theme::ThemeChangedEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_systems(Startup, setup)
        .add_observer(update_theme)
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
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
        BackgroundColor(theme.colors.background_primary),
        Root,
        children![
            // (Select::new(["hello", "who", "is", "there"]), List),
            (
                Node {
                    width: Val::Px(200.0),
                    left: Val::Px(-300.0),
                    ..default()
                },
                children![(
                    // Selection(Some(1)),
                    List,
                    related!(Items[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                    Float
                )]
            ),
            (
                List,
                Selection {
                    items: smallvec![],
                    multi: true,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            // (
            //     Text::new("Hello, World!"),
            //     TextColor(theme.colors.text_primary),
            //     HelloWorldLabel
            // ),
            // (Combo, Select::new(["hello", "who", "is", "there"])),
            (
                Combo,
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                Combo,
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (PushButton::from_label("Change theme"), ThemeButton),
            // PushButton::from_label("This does nothing")
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

#[derive(Component)]
struct Root;

#[derive(Component)]
struct HelloWorldLabel;

fn update_theme(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut background: Query<&mut BackgroundColor, With<Root>>,
    mut hello_world: Query<&mut TextColor, With<HelloWorldLabel>>,
) {
    for mut background_color in &mut background {
        background_color.0 = theme.colors.background_primary;
    }
    for mut text_color in &mut hello_world {
        text_color.0 = theme.colors.text_primary;
    }
}
