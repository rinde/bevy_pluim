//! `NumericStepper`: a value with a decrease and an increase button.

use accesskit::{Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::FocusedInput;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;
use itertools::Either;

use crate::icon_button::{IconButton, IconButtonSize};
use crate::theme::Theme;

/// Registers the systems and observers of [`NumericStepper`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct NumericStepperPlugin;

impl Plugin for NumericStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_activate)
            .add_observer(on_key_input)
            .add_systems(
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

/// How a [`NumericStepper`] shows its [`Value`].
pub type ValueFormat = fn(&Value) -> String;

/// The default width of the value label, in logical pixels, so that the buttons
/// do not move as the text changes.
pub const DEFAULT_LABEL_MIN_WIDTH_PX: f32 = 54.0;

/// A [`Value`] with a decrease and an increase button.
///
/// The value is either on the same entity or, through [`ValueRef`], on another one. A [`ValueChange`] is triggered on the value's entity when a button changes it.
///
/// The buttons are small [`IconButton`]s that change the value when they are
/// [`Activate`]d. A button that cannot step any further, at the minimum or maximum, is
/// [`InteractionDisabled`] until the value moves away from that bound again. The
/// stepper owns that component on its buttons: it enables a button whenever the
/// value allows a step, so do not disable one yourself.
///
/// The application chooses how the value is shown with `format` (the
/// default is [`Value`]'s `Display`), the icons and accessible names of the
/// buttons, and the minimum width of the value label. An icon left at its default
/// handle is replaced by a plain minus or plus drawn by the library. The
/// fields are read when the component is inserted.
///
/// The stepper is one spin button to a screen reader: its own node has the role
/// `SpinButton`, the value, bounds and step as numbers, the text `format` shows as its
/// value (kept in step when the value changes, also through a [`ValueRef`]) and `name` as
/// its label. The value text has no accessibility node of its own, so it is not read twice.
///
/// The arrow keys step the value as well, while the focus is on the stepper or on one of
/// its buttons (the keys bubble up to the stepper): the up and right arrows increase it,
/// the down and left arrows decrease it, within the same bounds. The stepper is not a
/// tab stop itself; its buttons are.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, from_reflect = false)]
#[component(on_insert=Self::on_insert)]
#[require(Node::default())]
pub struct NumericStepper {
    /// Turns the value into the text between the buttons.
    #[reflect(ignore)]
    pub format: ValueFormat,
    /// The icon of the decrease button, white on transparent (see
    /// [`Icon`](crate::icon::Icon)).
    pub decrease_icon: Handle<Image>,
    /// The icon of the increase button.
    pub increase_icon: Handle<Image>,
    /// The accessible name of the decrease button, also its `Name`.
    pub decrease_name: String,
    /// The accessible name of the increase button, also its `Name`.
    pub increase_name: String,
    /// The minimum width of the value label, in logical pixels.
    pub label_min_width: f32,
    /// The accessible name of the stepper itself; a stepper without one has no label.
    pub name: Option<String>,
}

fn default_format(value: &Value) -> String {
    value.to_string()
}

impl Default for NumericStepper {
    fn default() -> Self {
        Self {
            format: default_format,
            decrease_icon: Handle::default(),
            increase_icon: Handle::default(),
            decrease_name: "Decrease".into(),
            increase_name: "Increase".into(),
            label_min_width: DEFAULT_LABEL_MIN_WIDTH_PX,
            name: None,
        }
    }
}

/// A white-on-transparent 16 px minus or plus.
fn glyph(plus: bool) -> Image {
    const SIZE: u32 = 16;
    let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let horizontal = (7..9).contains(&y) && (2..14).contains(&x);
            let vertical = plus && (7..9).contains(&x) && (2..14).contains(&y);
            if horizontal || vertical {
                let at = ((y * SIZE + x) * 4) as usize;
                if let Some(pixel) = data.get_mut(at..at + 4) {
                    pixel.copy_from_slice(&[255; 4]);
                }
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

impl NumericStepper {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(stepper) = world.entity(context.entity).get::<Self>().cloned() else {
            return;
        };
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

        let mut decrease_icon = stepper.decrease_icon.clone();
        let mut increase_icon = stepper.increase_icon.clone();
        if let Some(mut images) = world.get_resource_mut::<Assets<Image>>() {
            if decrease_icon == Handle::default() {
                decrease_icon = images.add(glyph(false));
            }
            if increase_icon == Handle::default() {
                increase_icon = images.add(glyph(true));
            }
        }
        let stepper_entity = context.entity;
        let access_node = spin_button(&stepper, &value);
        world
            .commands()
            .entity(context.entity)
            .insert(AccessibilityNode::from(access_node));

        world
            .commands()
            .entity(context.entity)
            .insert(Children::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<ChildOf>| {
                    let mut decrease = spawner.spawn((
                        IconButton::new(
                            decrease_icon,
                            stepper.decrease_name.clone(),
                            IconButtonSize::Small,
                        ),
                        Name::new(stepper.decrease_name.clone()),
                        Decrease,
                        value_ref,
                    ));
                    if !value.can_decrement() {
                        decrease.insert(InteractionDisabled);
                    }
                    spawner.spawn((
                        Name::new("Value"),
                        Text((stepper.format)(&value)),
                        Pickable {
                            should_block_lower: false,
                            ..default()
                        },
                        Node {
                            min_width: Val::Px(stepper.label_min_width),
                            justify_content: JustifyContent::Center,
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        TextLayout::default().with_justify(Justify::Center),
                        NumericStepperLabel(stepper_entity),
                        value_ref,
                    ));
                    let mut increase = spawner.spawn((
                        IconButton::new(
                            increase_icon,
                            stepper.increase_name.clone(),
                            IconButtonSize::Small,
                        ),
                        Name::new(stepper.increase_name.clone()),
                        Increase,
                        value_ref,
                    ));
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

/// The accessibility node of a stepper showing `value`: a spin button.
fn spin_button(stepper: &NumericStepper, value: &Value) -> AccessNode {
    let mut node = AccessNode::new(Role::SpinButton);
    if let Some(name) = &stepper.name {
        node.set_label(name.as_str());
    }
    set_spin_value(&mut node, stepper.format, value);
    node
}

/// Writes the number, bounds, step and text of `value` to `node`.
fn set_spin_value(node: &mut AccessNode, format: ValueFormat, value: &Value) {
    let (now, min, max, step) = value.numbers();
    node.set_numeric_value(now);
    node.set_min_numeric_value(min);
    node.set_max_numeric_value(max);
    node.set_numeric_value_step(step);
    node.set_value(format(value));
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

    /// The value, minimum, maximum and step as `f64`s.
    fn numbers(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::F32 {
                value,
                min,
                max,
                step,
            } => (
                f64::from(*value),
                f64::from(*min),
                f64::from(*max),
                f64::from(*step),
            ),
            Self::U8(v) => (
                f64::from(v.value),
                f64::from(v.min),
                f64::from(v.max),
                f64::from(v.step),
            ),
            Self::U16(v) => (
                f64::from(v.value),
                f64::from(v.min),
                f64::from(v.max),
                f64::from(v.step),
            ),
            Self::U32(v) => (
                f64::from(v.value),
                f64::from(v.min),
                f64::from(v.max),
                f64::from(v.step),
            ),
        }
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
/// The value label of a stepper, which knows the stepper for its `format`.
#[derive(Component, Clone, Copy, Debug)]
struct NumericStepperLabel(Entity);

fn theme_numeric_stepper(
    numeric_stepper_ref: In<Option<Entity>>,
    mut numeric_steppers: Query<(&mut Node, &mut BorderColor), With<NumericStepper>>,
    mut value_texts: Query<(&NumericStepperLabel, &mut TextColor, &mut TextFont)>,
    theme: Res<Theme>,
) {
    for (label, mut text_color, mut text_font) in &mut value_texts {
        if numeric_stepper_ref.is_some_and(|stepper| stepper != label.0) {
            continue;
        }
        *text_color = TextColor(theme.colors.text_primary);
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(theme.font_size_secondary),
            ..default()
        };
    }

    let it = if let Some(numeric_stepper_ref) = numeric_stepper_ref.as_ref() {
        Either::Left(numeric_steppers.get_mut(*numeric_stepper_ref).into_iter())
    } else {
        Either::Right(numeric_steppers.iter_mut())
    };

    for (mut node, mut border_color) in it {
        node.column_gap = Val::Px(10.0);
        node.border = UiRect::all(theme.border_thickness);
        node.border_radius = BorderRadius::all(theme.control_radius);
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

/// Steps the value of the stepper the key reached with the arrow keys.
fn on_key_input(
    trigger: On<FocusedInput<KeyboardInput>>,
    steppers: Query<(), With<NumericStepper>>,
    value_refs: Query<&ValueRef>,
    mut values: Query<&mut Value>,
) {
    let input = &trigger.event().input;
    if input.state != ButtonState::Pressed {
        return;
    }
    let up = match input.key_code {
        KeyCode::ArrowUp | KeyCode::ArrowRight => true,
        KeyCode::ArrowDown | KeyCode::ArrowLeft => false,
        _ => return,
    };
    let stepper = trigger.event_target();
    if !steppers.contains(stepper) {
        return;
    }
    let value_entity = value_refs
        .get(stepper)
        .map_or(stepper, |value_ref| value_ref.0);
    let Ok(mut value) = values.get_mut(value_entity) else {
        return;
    };
    if up && value.can_increment() {
        value.increment();
    } else if !up && value.can_decrement() {
        value.decrement();
    }
}

fn on_change(
    changed_values: Query<(Entity, &Value), Changed<Value>>,
    mut numeric_stepper_labels: Query<(&ValueRef, &NumericStepperLabel, &mut Text)>,
    steppers: Query<&NumericStepper>,
    mut spin_buttons: Query<(
        Entity,
        Option<&ValueRef>,
        &NumericStepper,
        &mut AccessibilityNode,
    )>,
    decrease: Query<(Entity, &ValueRef, Has<InteractionDisabled>), With<Decrease>>,
    increase: Query<(Entity, &ValueRef, Has<InteractionDisabled>), With<Increase>>,
    mut commands: Commands,
) {
    for (value_ref, value) in &changed_values {
        for (text_value_ref, label, mut text) in &mut numeric_stepper_labels {
            if text_value_ref.0 == value_ref {
                let format = steppers
                    .get(label.0)
                    .map_or(default_format as ValueFormat, |s| s.format);
                text.0 = format(value);
            }
        }
        for (stepper_entity, stepper_value_ref, stepper, mut node) in &mut spin_buttons {
            // A stepper holding its own value has no `ValueRef`.
            if stepper_value_ref.map_or(stepper_entity, |r| r.0) == value_ref {
                set_spin_value(&mut node, stepper.format, value);
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
