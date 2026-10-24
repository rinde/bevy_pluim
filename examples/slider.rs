//! `Slider`: continuous, stepped, with its value shown, disabled, and one that
//! shares its value with a `NumericStepper`.

use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::{Activate, SliderValue, ValueChange};
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::numeric_stepper::{self, FloatValue, NumericStepper, Value};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::slider::Slider;
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

/// The slider whose value the [`SharedStepper`] shows too.
#[derive(Component, Debug, Clone, Copy)]
struct SharedSlider;

/// The stepper whose value the [`SharedSlider`] shows too.
#[derive(Component, Debug, Clone, Copy)]
struct SharedStepper;

fn setup(mut commands: Commands) -> Result {
    let shared_value = Value::new_f32(5.0, 0.0, 20.0, 0.5)?;
    commands.spawn(Camera2d);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Column,
            row_gap: px(20.0),
            padding: UiRect::all(px(20.0)),
        }
        Children [
            ~{valid_slider(0.5, 0.0, 1.0, None).with_name("Volume")}
            --
            ~{valid_slider(3.0, 0.0, 10.0, Some(1.0)).with_name("Cars")}
            Node { width: px(320.0) }
            --
            ~{valid_slider(60.0, 0.0, 120.0, Some(10.0)).with_format(kilometres_per_hour).with_name("Speed")}
            --
            ~{valid_slider(4.0, 0.0, 10.0, Some(1.0)).with_format(whole).with_name("Locked")}
            InteractionDisabled
            --
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(20.0),
            }
            Children [
                ~{valid_slider(5.0, 0.0, 20.0, Some(0.5)).with_name("Shared")}
                ~{SharedSlider}
                on(slider_moved)
                --
                NumericStepper
                ~{shared_value}
                ~{SharedStepper}
                on(stepper_stepped)
            ]
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
    Ok(())
}

/// The speed with its unit.
fn kilometres_per_hour(value: f32) -> String {
    format!("{value:.0} km/h")
}

/// The value as a whole number.
fn whole(value: f32) -> String {
    format!("{value:.0}")
}

/// Shows the slider's new value in the stepper.
fn slider_moved(
    change: On<ValueChange<f32>>,
    mut stepper: Single<&mut Value, With<SharedStepper>>,
) {
    if let Value::F32(float_value) = **stepper
        && let Ok(moved) = FloatValue::new(
            change.value,
            float_value.min(),
            float_value.max(),
            float_value.step(),
        )
    {
        **stepper = Value::F32(moved);
    }
}

/// Moves the slider to the stepper's new value.
fn stepper_stepped(
    change: On<numeric_stepper::ValueChange>,
    slider: Single<Entity, With<SharedSlider>>,
    mut commands: Commands,
) {
    if let Some(value) = change.value.as_f32() {
        commands.entity(*slider).insert(SliderValue(value));
    }
}

/// A slider with a valid range and step, or an abort: these are fixed values.
fn valid_slider(value: f32, min: f32, max: f32, step: Option<f32>) -> Slider {
    Slider::new(value, min, max)
        .and_then(|slider| match step {
            Some(step) => slider.with_step(step),
            None => Ok(slider),
        })
        .unwrap_or_else(|_| std::process::abort())
}
