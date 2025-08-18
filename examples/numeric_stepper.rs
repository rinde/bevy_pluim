use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use ui::WidgetsPlugin;
use ui::numeric_stepper::{NumericStepper, Value};
use ui::push_button::PushButton;
use ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_systems(Update, update_theme.run_if(resource_changed::<Theme>))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(10.0),
            column_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(theme.colors.background_primary),
        Root,
        children![
            NumericStepper,
            (NumericStepper, Value::new_u16(10, 5, 15, 1)),
            (NumericStepper, Value::new_u32(10, 5, 15, 1)),
            (NumericStepper, Value::new_f32(10.0, 5.0, 15.0, 0.5)),
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

#[derive(Component)]
struct Root;

#[derive(Component)]
struct HelloWorldLabel;

fn update_theme(
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
