use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use ui::WidgetsPlugin;
use ui::combo2;
use ui::float::Float;
use ui::push_button::PushButton;
use ui::selectable_items::SelectableItems;
use ui::selectable_items::Selection;
use ui::theme::Theme;

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
                Selection(Some(1)),
                Float
            ),
            (
                Text::new("Hello, World!"),
                TextColor(theme.colors.text_primary)
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
