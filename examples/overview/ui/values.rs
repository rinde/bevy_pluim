//! The values page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;
use bevy::ui_widgets::{SliderValue, ValueChange};
use bevy_pluim::numeric_stepper::{self, FloatValue, ValueFormat};
use bevy_pluim::text_field::{TextFieldChange, TextFieldCommit};

/// The text the Report text field's events are written to.
#[derive(Component, Debug, Clone, Copy, Default)]
struct ReportLabel;

/// The slider whose value the [`SharedStepper`] shows too.
#[derive(Component, Debug, Clone, Copy)]
struct SharedSlider;

/// The stepper whose value the [`SharedSlider`] shows too.
#[derive(Component, Debug, Clone, Copy)]
struct SharedStepper;

#[expect(clippy::expect_used, reason = "the constant ranges are valid")]
pub(super) fn values(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let float_value = Value::new_f32(10.0, 5.0, 15.0, 0.5).expect("a valid float range");
    let level_value = Value::new_u8(0, 0, 56, 7).expect("a valid level range");
    let u16_value = Value::new_u16(10, 5, 15, 1).expect("a valid u16 range");
    let u32_value = Value::new_u32(10, 5, 15, 1).expect("a valid u32 range");
    let shared_value = Value::new_f32(5.0, 0.0, 20.0, 0.5).expect("a valid shared range");
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Slider and stepper")}
                --
                ~{groups()}
                Children [
                    @variant("Slider: continuous", bsn! {
                        ~{valid_slider(0.5, 0.0, 1.0, None).with_name("Volume")}
                    })
                    --
                    @variant("Slider: stepped and formatted", bsn! {
                        ~{valid_slider(60.0, 0.0, 120.0, Some(10.0)).with_format(kilometres_per_hour).with_name("Speed")}
                    })
                    --
                    @variant("Slider: disabled", bsn! {
                        ~{valid_slider(4.0, 0.0, 10.0, Some(1.0)).with_name("Locked")}
                        InteractionDisabled
                    })
                    --
                    @variant("Stepper: default", bsn! { NumericStepper })
                    --
                    @variant("Stepper: float", bsn! {
                        NumericStepper
                        ~{float_value}
                    })
                    --
                    @variant("Stepper: u16", bsn! {
                        NumericStepper
                        ~{u16_value}
                    })
                    --
                    @variant("Stepper: u32", bsn! {
                        NumericStepper
                        ~{u32_value}
                    })
                    --
                    @variant("Stepper: u8 in steps of 7, own format and names", bsn! {
                        NumericStepper { format: {level as ValueFormat}, decrease_name: "Lower the level", increase_name: "Raise the level" }
                        ~{level_value}
                    })
                    --
                    @variant("Slider: stepped, narrow", bsn! {
                        ~{valid_slider(3.0, 0.0, 10.0, Some(1.0)).with_name("Cars")}
                        Node { width: px(160.0) }
                    })
                    --
                    @variant("Slider and stepper sharing a value", bsn! {
                        ~{row()}
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
                    })
                ]
            ]
        },
    );
}

pub(super) fn kilometres_per_hour(value: f32) -> String {
    format!("{value:.0} km/h")
}

pub(super) fn text_fields(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Text field")}
                --
                ~{groups()}
                Children [
                    @variant("With text", bsn! {
                        ~{TextField::new("Amsterdam Centraal").with_name("Name")}
                    })
                    --
                    @variant("Placeholder, narrow", bsn! {
                        ~{TextField::new("")
                            .with_placeholder("Seed")
                            .with_name("Seed")
                            .with_max_width(Val::Px(200.0))}
                    })
                    --
                    @variant("Change and commit events", bsn! {
                        ~{row()}
                        Children [
                            ~{TextField::new("").with_placeholder("Type, then Enter").with_name("Report")}
                            on(|change: On<TextFieldChange>, report: Single<Entity, With<ReportLabel>>, mut commands: Commands| {
                                commands.entity(*report).insert(Label::secondary(format!("Typing: {}", change.value)));
                            })
                            on(|commit: On<TextFieldCommit>, report: Single<Entity, With<ReportLabel>>, mut commands: Commands| {
                                commands.entity(*report).insert(Label::regular(format!("Committed: {}", commit.value)));
                            })
                            --
                            ~{Label::secondary("Nothing yet")}
                            ReportLabel
                        ]
                    })
                ]
            ]
        },
    );
}

/// "Ground" at 0 and "+7", "+14", ... above it.
fn level(value: &Value) -> String {
    match value.as_u8() {
        Some(0) | None => "Ground".to_string(),
        Some(n) => format!("+{n}"),
    }
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
