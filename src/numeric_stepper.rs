use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::label::Label;
use crate::push_button::PushButton;
use crate::theme::Theme;

#[derive(Copy, Clone, Debug)]
pub struct NumericStepperPlugin;

impl Plugin for NumericStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_press).add_systems(
            Update,
            (
                on_change,
                (|| None::<Entity>)
                    .pipe(theme_numeric_stepper)
                    .run_if(resource_changed::<Theme>),
            ),
        );
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[component(on_insert=Self::on_insert)]
#[require(Node::default())]
pub struct NumericStepper;

impl NumericStepper {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let (value, value_ref) = if let Some(value) = world.entity(context.entity).get::<Value>() {
            (*value, ValueRef(context.entity))
        } else if let Some(value_ref) = world.entity(context.entity).get::<ValueRef>().copied() {
            let value = *world.entity(value_ref.0).get::<Value>().unwrap();
            world.commands().entity(context.entity).insert(value_ref);
            (value, value_ref)
        } else {
            let default_value = Value::new_u32(1, 0, 10, 1);
            let value_ref = ValueRef(context.entity);
            world
                .commands()
                .entity(context.entity)
                .insert((default_value, value_ref));
            (default_value, value_ref)
        };

        world.commands().entity(context.entity).insert(children![
            (PushButton::from_label("-"), Decrease, value_ref),
            (
                Label::regular(value.to_string()),
                NumericStepperLabel,
                value_ref
            ),
            (PushButton::from_label("+"), Increase, value_ref),
        ]);

        world
            .commands()
            .run_system_cached_with(theme_numeric_stepper, Some(context.entity));
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ValueRef(pub Entity);

#[derive(Component, Clone, Copy, Debug)]
pub enum Value {
    F32 {
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    U32 {
        value: u32,
        min: u32,
        max: u32,
        step: u32,
    },
}

impl Value {
    pub fn new_u32(initial_value: u32, min: u32, max: u32, step: u32) -> Self {
        Value::U32 {
            value: initial_value,
            min,
            max,
            step,
        }
    }

    pub fn new_f32(initial_value: f32, min: f32, max: f32, step: f32) -> Self {
        Value::F32 {
            value: initial_value,
            min,
            max,
            step,
        }
    }

    pub fn increment(&mut self) -> bool {
        match self {
            Value::F32 {
                value, max, step, ..
            } => {
                let new_value = *value + *step;
                if new_value <= *max {
                    *value = new_value;
                    return true;
                }
                false
            }
            Value::U32 {
                value, max, step, ..
            } => {
                let new_value = value.checked_add(*step);
                if let Some(new_value) = new_value
                    && new_value <= *max
                {
                    *value = new_value;
                    return true;
                }
                false
            }
        }
    }

    pub fn decrement(&mut self) -> bool {
        match self {
            Value::F32 {
                value, min, step, ..
            } => {
                let new_value = *value - *step;
                if new_value >= *min {
                    *value = new_value;
                    return true;
                }
                false
            }
            Value::U32 {
                value, min, step, ..
            } => {
                let new_value = value.checked_sub(*step);
                if let Some(new_value) = new_value
                    && new_value >= *min
                {
                    *value = new_value;
                    return true;
                }
                false
            }
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::F32 { value, .. } => write!(f, "{value:.1}"),
            Value::U32 { value, .. } => write!(f, "{value}"),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct Increase;
#[derive(Component, Clone, Copy, Debug)]
struct Decrease;
#[derive(Component, Clone, Copy, Debug)]
struct NumericStepperLabel;

fn theme_numeric_stepper(
    numeric_stepper_ref: In<Option<Entity>>,
    mut numeric_steppers: Query<
        (&mut Node, &mut BorderColor, &mut BorderRadius),
        With<NumericStepper>,
    >,
    theme: Res<Theme>,
) {
    let it = if let Some(numeric_stepper_ref) = numeric_stepper_ref.as_ref() {
        Either::Left([numeric_steppers.get_mut(*numeric_stepper_ref).unwrap()].into_iter())
    } else {
        Either::Right(numeric_steppers.iter_mut())
    };

    for (mut node, mut border_color, mut border_radius) in it {
        node.column_gap = Val::Px(10.0);
        node.border = UiRect::all(theme.border_thickness);
        *border_radius = BorderRadius::all(theme.corner_radius);
        border_color.0 = theme.colors.border;
    }
}
fn on_press(
    trigger: Trigger<Pointer<Released>>,
    decrease: Query<&ValueRef, With<Decrease>>,
    increase: Query<&ValueRef, With<Increase>>,
    mut values: Query<&mut Value>,
) {
    if let Ok(vr) = decrease.get(trigger.target()) {
        let mut value = values.get_mut(vr.0).unwrap();
        value.decrement();
    } else if let Ok(vr) = increase.get(trigger.target()) {
        let mut value = values.get_mut(vr.0).unwrap();
        value.increment();
    }
}

fn on_change(
    changed_values: Query<(Entity, &Value), Changed<Value>>,
    mut numeric_stepper_labels: Query<(&ValueRef, &mut Text), With<NumericStepperLabel>>,
) {
    for (value_ref, value) in &changed_values {
        for (text_value_ref, mut text) in &mut numeric_stepper_labels {
            if text_value_ref.0 == value_ref {
                text.0 = value.to_string();
            }
        }
    }
}
