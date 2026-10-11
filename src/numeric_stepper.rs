//! `NumericStepper`: a value with a decrease and an increase button.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;
use itertools::Either;

use crate::label::Label;
use crate::push_button::PushButton;
use crate::theme::Theme;

/// Registers the systems and observers of [`NumericStepper`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct NumericStepperPlugin;

impl Plugin for NumericStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_activate).add_systems(
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

/// A [`Value`] with a decrease and an increase button.
///
/// The value is either on the same entity or, through [`ValueRef`], on another one. A [`ValueChange`] is triggered on the value's entity when a button changes it.
///
/// The buttons are [`PushButton`]s that change the value when they are
/// [`Activate`]d. A button that cannot step any further, at the minimum or maximum, is
/// [`InteractionDisabled`] until the value moves away from that bound again.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[component(on_insert=Self::on_insert)]
#[require(Node::default())]
pub struct NumericStepper;

impl NumericStepper {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let (value, value_ref) = if let Some(value) = world.entity(context.entity).get::<Value>() {
            (*value, ValueRef(context.entity))
        } else if let Some(value_ref) = world.entity(context.entity).get::<ValueRef>().copied() {
            let Some(value) = world.entity(value_ref.0).get::<Value>().copied() else {
                return;
            };
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

        world
            .commands()
            .entity(context.entity)
            .insert(Children::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<ChildOf>| {
                    let mut decrease =
                        spawner.spawn((PushButton::from_label("-"), Decrease, value_ref));
                    if !value.can_decrement() {
                        decrease.insert(InteractionDisabled);
                    }
                    spawner.spawn((
                        Label::regular(value.to_string()),
                        NumericStepperLabel,
                        value_ref,
                    ));
                    let mut increase =
                        spawner.spawn((PushButton::from_label("+"), Increase, value_ref));
                    if !value.can_increment() {
                        increase.insert(InteractionDisabled);
                    }
                },
            )));

        world
            .commands()
            .run_system_cached_with(theme_numeric_stepper, Some(context.entity));
    }
}

/// Points a [`NumericStepper`] to the entity holding its [`Value`].
#[derive(Component, Clone, Copy, Debug)]
pub struct ValueRef(pub Entity);

/// A number with bounds and a step size.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub enum Value {
    /// A floating point value.
    F32 {
        /// The current value.
        value: f32,
        /// The lowest value.
        min: f32,
        /// The highest value.
        max: f32,
        /// The amount one press adds or subtracts.
        step: f32,
    },
    /// An unsigned 8-bit value.
    U8(UIntValue<u8>),
    /// An unsigned 16-bit value.
    U16(UIntValue<u16>),
    /// An unsigned 32-bit value.
    U32(UIntValue<u32>),
}

impl Value {
    /// A `u8` value starting at `initial_value`, between `min` and `max`, moving by `step`.
    #[must_use]
    pub const fn new_u8(initial_value: u8, min: u8, max: u8, step: u8) -> Self {
        Self::U8(UIntValue {
            value: initial_value,
            min,
            max,
            step,
        })
    }
    /// A `u16` value starting at `initial_value`, between `min` and `max`, moving by `step`.
    #[must_use]
    pub const fn new_u16(initial_value: u16, min: u16, max: u16, step: u16) -> Self {
        Self::U16(UIntValue {
            value: initial_value,
            min,
            max,
            step,
        })
    }

    /// A `u32` value starting at `initial_value`, between `min` and `max`, moving by `step`.
    #[must_use]
    pub const fn new_u32(initial_value: u32, min: u32, max: u32, step: u32) -> Self {
        Self::U32(UIntValue {
            value: initial_value,
            min,
            max,
            step,
        })
    }

    /// An `f32` value starting at `initial_value`, between `min` and `max`, moving by `step`.
    #[must_use]
    pub const fn new_f32(initial_value: f32, min: f32, max: f32, step: f32) -> Self {
        Self::F32 {
            value: initial_value,
            min,
            max,
            step,
        }
    }

    /// Adds one step. Returns `false`, leaving the value as it is, if that would pass the maximum.
    pub fn increment(&mut self) -> bool {
        match self {
            Self::F32 {
                value, max, step, ..
            } => {
                let new_value = *value + *step;
                if new_value <= *max {
                    *value = new_value;
                    return true;
                }
                false
            }
            Self::U8(uint_value) => uint_value.increment(),
            Self::U16(uint_value) => uint_value.increment(),
            Self::U32(uint_value) => uint_value.increment(),
        }
    }

    /// Subtracts one step. Returns `false`, leaving the value as it is, if that would pass the minimum.
    pub fn decrement(&mut self) -> bool {
        match self {
            Self::F32 {
                value, min, step, ..
            } => {
                let new_value = *value - *step;
                if new_value >= *min {
                    *value = new_value;
                    return true;
                }
                false
            }
            Self::U8(uint_value) => uint_value.decrement(),
            Self::U16(uint_value) => uint_value.decrement(),
            Self::U32(uint_value) => uint_value.decrement(),
        }
    }

    /// Whether [`Self::increment`] would change the value.
    #[must_use]
    pub fn can_increment(&self) -> bool {
        let mut stepped = *self;
        stepped.increment()
    }

    /// Whether [`Self::decrement`] would change the value.
    #[must_use]
    pub fn can_decrement(&self) -> bool {
        let mut stepped = *self;
        stepped.decrement()
    }

    /// The value as an `f32`, if it is one.
    #[must_use]
    pub const fn as_f32(&self) -> Option<f32> {
        match self {
            Self::F32 { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// The value as an `u8`, if it is one.
    #[must_use]
    pub const fn as_u8(&self) -> Option<u8> {
        match self {
            Self::U8(uint_value) => Some(uint_value.value),
            _ => None,
        }
    }

    /// The value as an `u16`, if it is one.
    #[must_use]
    pub const fn as_u16(&self) -> Option<u16> {
        match self {
            Self::U16(uint_value) => Some(uint_value.value),
            _ => None,
        }
    }

    /// The value as an `u32`, if it is one.
    #[must_use]
    pub const fn as_u32(&self) -> Option<u32> {
        match self {
            Self::U32(uint_value) => Some(uint_value.value),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F32 { value, .. } => write!(f, "{value:.1}"),
            Self::U8(uint_value) => write!(f, "{}", uint_value.value),
            Self::U16(uint_value) => write!(f, "{}", uint_value.value),
            Self::U32(uint_value) => write!(f, "{}", uint_value.value),
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
    mut numeric_steppers: Query<(&mut Node, &mut BorderColor), With<NumericStepper>>,
    theme: Res<Theme>,
) {
    let it = if let Some(numeric_stepper_ref) = numeric_stepper_ref.as_ref() {
        Either::Left(numeric_steppers.get_mut(*numeric_stepper_ref).into_iter())
    } else {
        Either::Right(numeric_steppers.iter_mut())
    };

    for (mut node, mut border_color) in it {
        node.column_gap = Val::Px(10.0);
        node.border = UiRect::all(theme.border_thickness);
        node.border_radius = BorderRadius::all(theme.corner_radius);
        border_color.set_all(theme.colors.border);
    }
}
fn on_activate(
    trigger: On<Activate>,
    decrease: Query<&ValueRef, With<Decrease>>,
    increase: Query<&ValueRef, With<Increase>>,
    mut values: Query<&mut Value>,
) {
    let button = trigger.event().event_target();
    if let Ok(vr) = decrease.get(button) {
        let Ok(mut value) = values.get_mut(vr.0) else {
            return;
        };
        if value.can_decrement() {
            value.decrement();
        }
    } else if let Ok(vr) = increase.get(button) {
        let Ok(mut value) = values.get_mut(vr.0) else {
            return;
        };
        if value.can_increment() {
            value.increment();
        }
    }
}

fn on_change(
    changed_values: Query<(Entity, &Value), Changed<Value>>,
    mut numeric_stepper_labels: Query<(&ValueRef, &mut Text), With<NumericStepperLabel>>,
    decrease: Query<(Entity, &ValueRef, Has<InteractionDisabled>), With<Decrease>>,
    increase: Query<(Entity, &ValueRef, Has<InteractionDisabled>), With<Increase>>,
    mut commands: Commands,
) {
    for (value_ref, value) in &changed_values {
        for (text_value_ref, mut text) in &mut numeric_stepper_labels {
            if text_value_ref.0 == value_ref {
                text.0 = value.to_string();
            }
        }
        for (button, button_value_ref, disabled) in &decrease {
            if button_value_ref.0 == value_ref {
                set_disabled(&mut commands, button, disabled, !value.can_decrement());
            }
        }
        for (button, button_value_ref, disabled) in &increase {
            if button_value_ref.0 == value_ref {
                set_disabled(&mut commands, button, disabled, !value.can_increment());
            }
        }
        commands.trigger(ValueChange {
            entity: value_ref,
            value: *value,
        });
    }
}

/// Makes `button` [`InteractionDisabled`] or not, touching it only if that changes.
fn set_disabled(commands: &mut Commands, button: Entity, is_disabled: bool, should_be: bool) {
    if should_be && !is_disabled {
        commands.entity(button).insert(InteractionDisabled);
    } else if !should_be && is_disabled {
        commands.entity(button).remove::<InteractionDisabled>();
    }
}

/// Triggered on the entity holding a [`Value`] after a stepper button changed it.
#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct ValueChange {
    /// The entity holding the [`Value`].
    pub entity: Entity,
    /// The new value.
    pub value: Value,
}

/// An unsigned integer with bounds and a step size.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct UIntValue<T> {
    /// The current value.
    pub value: T,
    /// The lowest value.
    pub min: T,
    /// The highest value.
    pub max: T,
    /// The amount one press adds or subtracts.
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

/// An unsigned integer type usable in a [`UIntValue`].
pub trait UInt: Sized + Ord + Copy {
    /// Adds, or `None` on overflow.
    fn checked_add(self, rhs: Self) -> Option<Self>;
    /// Subtracts, or `None` on underflow.
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
