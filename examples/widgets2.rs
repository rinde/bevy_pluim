use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use ui::WidgetsPlugin;
use ui::combo2;
use ui::combo3;
use ui::float::Float;
use ui::list::List;
use ui::push_button::PushButton;
use ui::select::Select;
use ui::select::Selection;
use ui::theme::Theme;
use ui::theme::ThemeChangedEvent;

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
        .add_systems(Update, on_press)
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
            (
                combo2::Combo,
                children![
                //    combo2::ComboButton
                ]
            ),
            (Select::new(["hello", "who", "is", "there"]), List),
            (
                Node {
                    width: Val::Px(200.0),
                    left: Val::Px(-300.0),
                    ..default()
                },
                children![(
                    Select::new(["Netherlands", "Canada", "Indonesia", "Portugal"]),
                    Selection(Some(1)),
                    List,
                    Float
                )]
            ),
            (
                Text::new("Hello, World!"),
                TextColor(theme.colors.text_primary),
                HelloWorldLabel
            ),
            (combo3::Combo, Select::new(["hello", "who", "is", "there"])),
            (
                combo3::Combo,
                Select::new(["Netherlands", "Canada", "Indonesia", "Portugal"])
            ),
            (PushButton::new("Change theme"), ThemeButton)
        ],
    ));
}

#[derive(Component)]
struct ThemeButton;

type ExactlyOne<'w, D, F = ()> = Option<Single<'w, D, F>>;

fn on_press(
    button: ExactlyOne<&Interaction, (Changed<Interaction>, With<ThemeButton>)>,
    mut theme: ResMut<Theme>,
) {
    if button.is_some_and(|b| **b == Interaction::Pressed) {
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
