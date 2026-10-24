//! `NumericStepper`.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value, ValueFormat};
use rin_bevy_ui::push_button::PushButton;
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

fn setup(mut commands: Commands) -> Result {
    let u16_value = Value::new_u16(10, 5, 15, 1)?;
    let u32_value = Value::new_u32(10, 5, 15, 1)?;
    let f32_value = Value::new_f32(10.0, 5.0, 15.0, 0.5)?;
    let level_value = Value::new_u8(0, 0, 56, 7)?;
    commands.spawn(Camera2d);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(10.0),
            column_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            NumericStepper
            --
            NumericStepper
            ~{u16_value}
            --
            NumericStepper
            ~{u32_value}
            --
            NumericStepper
            ~{f32_value}
            --
            NumericStepper { format: {level as ValueFormat}, decrease_name: "Lower the level", increase_name: "Raise the level" }
            ~{level_value}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
    Ok(())
}

/// "Ground" at 0 and "+7", "+14", ... above it.
fn level(value: &Value) -> String {
    match value.as_u8() {
        Some(0) | None => "Ground".to_string(),
        Some(n) => format!("+{n}"),
    }
}
