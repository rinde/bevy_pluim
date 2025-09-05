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
    U16(UIntValue<u16>),
    U32(UIntValue<u32>),
}

impl Value {
    #[must_use]
    pub const fn new_u16(initial_value: u16, min: u16, max: u16, step: u16) -> Self {
        Value::U16(UIntValue {
            value: initial_value,
            min,
            max,
            step,
        })
    }

    #[must_use]
    pub const fn new_u32(initial_value: u32, min: u32, max: u32, step: u32) -> Self {
        Value::U32(UIntValue {
            value: initial_value,
            min,
            max,
            step,
        })
    }

    #[must_use]
    pub const fn new_f32(initial_value: f32, min: f32, max: f32, step: f32) -> Self {
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
            Value::U16(uint_value) => uint_value.increment(),
            Value::U32(uint_value) => uint_value.increment(),
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
            Value::U16(uint_value) => uint_value.decrement(),
            Value::U32(uint_value) => uint_value.decrement(),
        }
    }

    #[must_use]
    pub fn can_increment(&self) -> bool {
        match self {
            Value::F32 { value, max, .. } => value < max,
            Value::U16(uint_value) => uint_value.value < uint_value.max,
            Value::U32(uint_value) => uint_value.value < uint_value.max,
        }
    }

    #[must_use]
    pub fn can_decrement(&self) -> bool {
        match self {
            Value::F32 { value, min, .. } => value > min,
            Value::U16(uint_value) => uint_value.value > uint_value.min,
            Value::U32(uint_value) => uint_value.value > uint_value.min,
        }
    }

    #[must_use]
    pub const fn as_f32(&self) -> Option<f32> {
        match self {
            Value::F32 { value, .. } => Some(*value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_u16(&self) -> Option<u16> {
        match self {
            Value::U16(uint_value) => Some(uint_value.value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_u32(&self) -> Option<u32> {
        match self {
            Value::U32(uint_value) => Some(uint_value.value),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::F32 { value, .. } => write!(f, "{value:.1}"),
            Value::U32(uint_value) => write!(f, "{}", uint_value.value),
            Value::U16(uint_value) => write!(f, "{}", uint_value.value),
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
        if value.can_decrement() {
            value.decrement();
        }
    } else if let Ok(vr) = increase.get(trigger.target()) {
        let mut value = values.get_mut(vr.0).unwrap();
        if value.can_increment() {
            value.increment();
        }
    }
}

fn on_change(
    changed_values: Query<(Entity, &Value), Changed<Value>>,
    mut numeric_stepper_labels: Query<(&ValueRef, &mut Text), With<NumericStepperLabel>>,
    mut commands: Commands,
) {
    for (value_ref, value) in &changed_values {
        for (text_value_ref, mut text) in &mut numeric_stepper_labels {
            if text_value_ref.0 == value_ref {
                text.0 = value.to_string();
            }
        }
        commands
            .entity(value_ref)
            .trigger(ValueChange { value: *value });
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ValueChange {
    pub value: Value,
}

#[derive(Debug, Clone, Copy)]
pub struct UIntValue<T> {
    pub value: T,
    pub min: T,
    pub max: T,
    pub step: T,
}

impl<T: UInt> UIntValue<T> {
    fn increment(&mut self) -> bool {
        let new_value = self.value.checked_add(self.step);
        if let Some(new_value) = new_value
            && new_value <= self.max
        {
            self.value = new_value;
            return true;
        }
        false
    }

    fn decrement(&mut self) -> bool {
        let new_value = self.value.checked_sub(self.step);
        if let Some(new_value) = new_value
            && new_value >= self.min
        {
            self.value = new_value;
            return true;
        }
        false
    }
}

pub trait UInt: Sized + Ord + Copy {
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
}

macro_rules! impl_uint {
    ($($i:ty),+ $(,)?) => {
        $(
            impl UInt for $i {
                fn checked_add(self, rhs: Self) -> Option<Self> {
                    self.checked_add(rhs)
                }
                fn checked_sub(self, rhs: Self) -> Option<Self> {
                    self.checked_sub(rhs)
                }
            }

        )+
    };
}
impl_uint!(u8, u16, u32, u64);
