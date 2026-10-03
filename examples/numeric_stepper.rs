//! `NumericStepper`.

use bevy::prelude::*;
use bevy::ui_render::UiDebugOptions;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::theme::Theme;

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
            ..default()
        })
        .add_observer(on_press)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
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
    trigger: On<Pointer<Release>>,
    button: Query<(), With<ThemeButton>>,
    mut theme: ResMut<Theme>,
) {
    if button.get(trigger.event().event_target()).is_ok() {
        theme.light_dark_toggle();
    }
}
