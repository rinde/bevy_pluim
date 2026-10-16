//! `Slider`: continuous, stepped, with its value shown, disabled, and one that
//! shares its value with a `NumericStepper`.

use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::{Activate, SliderValue, ValueChange};
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::numeric_stepper::{self, NumericStepper, Value};
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

fn setup(mut commands: Commands) {
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
            ~{Slider::new(0.5, 0.0, 1.0).with_name("Volume")}
            --
            ~{Slider::new(3.0, 0.0, 10.0).with_step(1.0).with_name("Cars")}
            Node { width: px(320.0) }
            --
            ~{Slider::new(60.0, 0.0, 120.0).with_step(10.0).with_format(kilometres_per_hour).with_name("Speed")}
            --
            ~{Slider::new(4.0, 0.0, 10.0).with_step(1.0).with_format(whole).with_name("Locked")}
            InteractionDisabled
            --
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(20.0),
            }
            Children [
                ~{Slider::new(5.0, 0.0, 20.0).with_step(0.5).with_name("Shared")}
                ~{SharedSlider}
                on(slider_moved)
                --
                NumericStepper
                ~{Value::new_f32(5.0, 0.0, 20.0, 0.5)}
                ~{SharedStepper}
                on(stepper_stepped)
            ]
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
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
    if let Value::F32 { value, .. } = stepper.as_mut() {
        *value = change.value;
    }
}

/// Moves the slider to the stepper's new value.
fn stepper_stepped(
    change: On<numeric_stepper::ValueChange>,
    slider: Single<Entity, With<SharedSlider>>,
    mut commands: Commands,
) {
    if let Value::F32 { value, .. } = change.value {
        commands.entity(*slider).insert(SliderValue(value));
    }
}
