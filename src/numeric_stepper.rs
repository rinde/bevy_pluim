//! `NumericStepper`: a value with a decrease and an increase button.
//!
//! # Migrating from the unvalidated `Value`
//!
//! - `Value::new_u8` / `new_u16` / `new_u32` / `new_f32` return `Result<Value, ValueError>`;
//!   handle or `?` the result. Reversed bounds, a zero integer step and non-finite or
//!   non-positive float input are errors, no longer silently repaired.
//! - `Value::F32` holds a [`FloatValue`], not a tuple of `f32`s; `UIntValue` and
//!   `FloatValue` fields are private. Read them with `value()`, `min()`, `max()` and
//!   `step()`; to change bounds or step, build a new value with `FloatValue::new` or
//!   `UIntValue::new` and wrap it in `Value::F32` / `Value::U8` / ..
//! - `increment` and `decrement` leave a float unchanged when it cannot strictly move.

use accesskit::{Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::asset::HandleTemplate;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::template::{SpecializeFromTemplate, Template, TemplateContext};
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusCause, FocusLost, FocusedInput, InputFocus};
use bevy::picking::cursor::{EntityCursor, OverrideCursor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;
use bevy::window::SystemCursorIcon;
use itertools::Either;

use crate::icon_button::{IconButton, IconButtonSize};
use crate::text_field::{TextField, TextFieldCommit, TextFieldRefs};
use crate::theme::Theme;
use crate::{HoverLock, MouseEvent, MouseKind};

/// Registers the systems and observers of [`NumericStepper`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct NumericStepperPlugin;

impl Plugin for NumericStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_activate)
            .add_observer(on_key_input)
            .add_observer(on_mouse)
            .add_observer(on_editor_commit)
            .add_observer(on_editor_focus_lost)
            .add_systems(Last, (finish_editors, end_scrub))
            .add_systems(
                Update,
                (
                    focus_new_editors,
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
/// do not move as the text changes. It is the one constant [`Slider`](crate::slider::Slider)
/// shares for its own label.
pub use crate::slider::DEFAULT_LABEL_MIN_WIDTH_PX;

/// A [`Value`] with a decrease and an increase button.
///
/// The value is either on the same entity or, through [`ValueRef`], on another one. A [`ValueChange`] is triggered on the value's entity when the user steps it with a button or an arrow key, not when the stepper is spawned or the app sets the value itself (the display follows those too). A stepper without a [`Value`], or whose [`ValueRef`] target has none, logs an `error!` and stays unbuilt.
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
/// The stepper can also be scrubbed like a slider. A background fill shows where the value is
/// between its bounds, and dragging sideways from anywhere on the stepper, the "-" and "+"
/// buttons included, moves the value across that range (a drag over the whole width of the
/// stepper covers the whole range), snapped to the step, with a [`ValueChange`] for each change.
/// A drag does not also step the button it started on. A click on the value (without dragging)
/// swaps it for a [`TextField`] holding the number: Enter, Tab or a click elsewhere sets the
/// value to the number typed (clamped to the bounds, whole for an integer; text after the number
/// is ignored, and text without a number leaves the value as it was), Escape drops the edit.
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
///
/// `NumericStepper` is an immutable component: it is configuration, replaced by inserting a new
/// one, never edited in place, so a system cannot take it as `&mut`. Migrating: a `Query<&mut NumericStepper>` becomes
/// `commands.entity(e).insert(..)` of a prepared `NumericStepper`. The stepper owns its children: inserting again despawns them and builds new ones, so do not add children of your own. Its `Value` stays mutable and keeps its `ValueRef`.
///
/// A `&mut` query does not compile:
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::numeric_stepper::NumericStepper;
///
/// fn edit(mut widgets: Query<&mut NumericStepper>) {
///     for mut widget in &mut widgets {
///         let _ = &mut *widget;
///     }
/// }
/// ```
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, from_reflect = false)]
#[component(immutable, on_insert = Self::on_insert)]
#[require(Node::default(), Scrub)]
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

/// What `bsn!` patches when a scene writes `NumericStepper { .. }`: the fields not
/// written are those of [`NumericStepper::default`], and the icons are
/// [`HandleTemplate`]s.
pub struct NumericStepperTemplate {
    /// See [`NumericStepper::format`].
    pub format: ValueFormat,
    /// See [`NumericStepper::decrease_icon`].
    pub decrease_icon: HandleTemplate<Image>,
    /// See [`NumericStepper::increase_icon`].
    pub increase_icon: HandleTemplate<Image>,
    /// See [`NumericStepper::decrease_name`].
    pub decrease_name: String,
    /// See [`NumericStepper::increase_name`].
    pub increase_name: String,
    /// See [`NumericStepper::label_min_width`].
    pub label_min_width: f32,
    /// See [`NumericStepper::name`].
    pub name: Option<String>,
}

impl core::fmt::Debug for NumericStepperTemplate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NumericStepperTemplate")
            .finish_non_exhaustive()
    }
}

impl Default for NumericStepperTemplate {
    fn default() -> Self {
        let stepper = NumericStepper::default();
        Self {
            format: stepper.format,
            decrease_icon: HandleTemplate::Handle(stepper.decrease_icon),
            increase_icon: HandleTemplate::Handle(stepper.increase_icon),
            decrease_name: stepper.decrease_name,
            increase_name: stepper.increase_name,
            label_min_width: stepper.label_min_width,
            name: stepper.name,
        }
    }
}

impl Template for NumericStepperTemplate {
    type Output = NumericStepper;

    fn build_template(&self, context: &mut TemplateContext) -> Result<NumericStepper> {
        Ok(NumericStepper {
            format: self.format,
            decrease_icon: self.decrease_icon.build_template(context)?,
            increase_icon: self.increase_icon.build_template(context)?,
            decrease_name: self.decrease_name.clone(),
            increase_name: self.increase_name.clone(),
            label_min_width: self.label_min_width,
            name: self.name.clone(),
        })
    }

    fn clone_template(&self) -> Self {
        Self {
            format: self.format,
            decrease_icon: self.decrease_icon.clone_template(),
            increase_icon: self.increase_icon.clone_template(),
            decrease_name: self.decrease_name.clone(),
            increase_name: self.increase_name.clone(),
            label_min_width: self.label_min_width,
            name: self.name.clone(),
        }
    }
}

// `NumericStepper` is `Default` and `Clone`, so without this `Unpin` impl (what `#[derive(FromTemplate)]`
// writes) bevy's blanket `FromTemplate` for such types would conflict with the one below.
#[expect(
    unused_lifetimes,
    reason = "the higher-ranked bound is what keeps this impl from applying generally"
)]
impl Unpin for NumericStepper where for<'a> [()]: SpecializeFromTemplate {}

impl FromTemplate for NumericStepper {
    type Template = NumericStepperTemplate;
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
                error!(
                    "NumericStepper {} points to {} through ValueRef, which has no Value",
                    context.entity, value_ref.0
                );
                return;
            };
            world.commands().entity(context.entity).insert(value_ref);
            (value, value_ref)
        } else {
            error!(
                "NumericStepper {} has neither a Value nor a ValueRef; insert one with the stepper",
                context.entity
            );
            return;
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
        world.commands().entity(context.entity).insert((
            AccessibilityNode::from(access_node),
            // The column resize cursor, a double arrow with a bar: a drag scrubs the value.
            EntityCursor::System(SystemCursorIcon::ColResize),
        ));

        // Inserting again replaces the parts: the old ones would be left behind as orphans.
        let old_parts: Vec<Entity> = world
            .entity(context.entity)
            .get::<Children>()
            .map(|children| children.iter().collect())
            .unwrap_or_default();
        for part in old_parts {
            world.commands().entity(part).despawn();
        }
        world
            .commands()
            .entity(context.entity)
            .insert(Children::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<ChildOf>| {
                    // The track runs from the right edge of the decrease button to the left
                    // edge of the increase button: the fill is empty at the minimum and
                    // reaches the increase button at the maximum.
                    spawner
                        .spawn((
                            Name::new("ScrubTrack"),
                            Pickable::IGNORE,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(BUTTON_WIDTH_PX),
                                right: Val::Px(BUTTON_WIDTH_PX),
                                top: Val::Px(0.0),
                                bottom: Val::Px(0.0),
                                ..default()
                            },
                        ))
                        .with_child((
                            Name::new("ScrubFill"),
                            ScrubFill(stepper_entity),
                            Pickable::IGNORE,
                            Node {
                                height: Val::Percent(100.0),
                                width: Val::Percent(fraction_percent(&value)),
                                ..default()
                            },
                            value_ref,
                        ));
                    let mut decrease = spawner.spawn((
                        IconButton::new(
                            decrease_icon,
                            stepper.decrease_name.clone(),
                            IconButtonSize::Small,
                        ),
                        Name::new(stepper.decrease_name.clone()),
                        Decrease,
                        // Not the stepper's resize cursor: only the drag scrubs there.
                        EntityCursor::default(),
                        value_ref,
                    ));
                    if !value.can_decrement() {
                        decrease.insert(InteractionDisabled);
                    }
                    spawner.spawn((
                        Name::new("Value"),
                        Text((stepper.format)(&value)),
                        // The text cursor: a click here opens the number editor. The label
                        // blocks the stepper below it, so its cursor is the only one hovered.
                        EntityCursor::System(SystemCursorIcon::Text),
                        Pickable::default(),
                        Node {
                            min_width: Val::Px(stepper.label_min_width),
                            // Takes the room a wider stepper leaves, so the buttons stay at
                            // its ends.
                            flex_grow: 1.0,
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
                        EntityCursor::default(),
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
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ValueRef(pub Entity);

/// Why a [`Value`], [`UIntValue`] or [`FloatValue`] could not be built.
///
/// The configuration is rejected as given: bounds are never swapped and a step is never
/// replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueError {
    /// The minimum is above the maximum.
    MinAboveMax,
    /// An integer step is zero.
    ZeroStep,
    /// A float step is not finite or not greater than zero.
    InvalidStep,
    /// A float initial value, minimum or maximum is NaN or infinite.
    NotFinite,
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MinAboveMax => "the minimum is above the maximum",
            Self::ZeroStep => "the step is zero",
            Self::InvalidStep => "the step is not a finite number above zero",
            Self::NotFinite => "the value or a bound is not finite",
        })
    }
}

impl std::error::Error for ValueError {}

/// A number with bounds and a step size.
///
/// A `Value` is live data: the stepper and its mirrors change it in place with
/// [`Self::increment`] and [`Self::decrement`], and an app replaces it as a whole. The
/// payloads can only be built through their validating constructors, so the value always
/// lies within its bounds and a step is always positive.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub enum Value {
    /// A floating point value.
    F32(FloatValue),
    /// An unsigned 8-bit value.
    U8(UIntValue<u8>),
    /// An unsigned 16-bit value.
    U16(UIntValue<u16>),
    /// An unsigned 32-bit value.
    U32(UIntValue<u32>),
}

impl Value {
    /// A `u8` value starting at `initial_value` (clamped to the range), between `min` and `max`,
    /// moving by `step`.
    ///
    /// # Errors
    /// [`ValueError::MinAboveMax`] if `min > max`, [`ValueError::ZeroStep`] if `step` is zero.
    pub fn new_u8(initial_value: u8, min: u8, max: u8, step: u8) -> Result<Self, ValueError> {
        UIntValue::new(initial_value, min, max, step).map(Self::U8)
    }

    /// A `u16` value starting at `initial_value` (clamped to the range), between `min` and `max`,
    /// moving by `step`.
    ///
    /// # Errors
    /// [`ValueError::MinAboveMax`] if `min > max`, [`ValueError::ZeroStep`] if `step` is zero.
    pub fn new_u16(initial_value: u16, min: u16, max: u16, step: u16) -> Result<Self, ValueError> {
        UIntValue::new(initial_value, min, max, step).map(Self::U16)
    }

    /// A `u32` value starting at `initial_value` (clamped to the range), between `min` and `max`,
    /// moving by `step`.
    ///
    /// # Errors
    /// [`ValueError::MinAboveMax`] if `min > max`, [`ValueError::ZeroStep`] if `step` is zero.
    pub fn new_u32(initial_value: u32, min: u32, max: u32, step: u32) -> Result<Self, ValueError> {
        UIntValue::new(initial_value, min, max, step).map(Self::U32)
    }

    /// An `f32` value starting at `initial_value` (clamped to the range), between `min` and `max`,
    /// moving by `step`.
    ///
    /// # Errors
    /// [`ValueError::NotFinite`] if the value or a bound is NaN or infinite,
    /// [`ValueError::MinAboveMax`] if `min > max`, [`ValueError::InvalidStep`] if `step` is not
    /// finite and above zero.
    pub fn new_f32(initial_value: f32, min: f32, max: f32, step: f32) -> Result<Self, ValueError> {
        FloatValue::new(initial_value, min, max, step).map(Self::F32)
    }

    /// Adds one step. Returns `false`, leaving the value as it is, if that would pass the
    /// maximum, overflow, or (for a float) not change the value.
    pub fn increment(&mut self) -> bool {
        match self {
            Self::F32(float_value) => float_value.increment(),
            Self::U8(uint_value) => uint_value.increment(),
            Self::U16(uint_value) => uint_value.increment(),
            Self::U32(uint_value) => uint_value.increment(),
        }
    }

    /// Subtracts one step. Returns `false`, leaving the value as it is, if that would pass the
    /// minimum, overflow, or (for a float) not change the value.
    pub fn decrement(&mut self) -> bool {
        match self {
            Self::F32(float_value) => float_value.decrement(),
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

    /// How far the value is between its bounds: 0 at the minimum, 1 at the maximum (0 when the
    /// bounds are equal).
    #[must_use]
    pub fn fraction(&self) -> f64 {
        let (now, min, max, _) = self.numbers();
        if max > min {
            ((now - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Moves the value to `fraction` of the way between its bounds, snapped to the step from the
    /// minimum. Returns whether the value changed.
    fn set_fraction(&mut self, fraction: f64) -> bool {
        let (_, min, max, _) = self.numbers();
        self.set_number(fraction.clamp(0.0, 1.0).mul_add(max - min, min), true)
    }

    /// Sets the value to `number`, clamped to the bounds, rounded for an integer and, with
    /// `snap`, moved to the nearest step from the minimum. Returns whether the value changed.
    #[expect(
        clippy::cast_sign_loss,
        reason = "the number is clamped to the bounds, which an unsigned integer keeps at or above 0"
    )]
    fn set_number(&mut self, number: f64, snap: bool) -> bool {
        let (_, min, max, step) = self.numbers();
        if !number.is_finite() {
            return false;
        }
        let mut number = number.clamp(min, max);
        if snap {
            number = ((number - min) / step)
                .round()
                .mul_add(step, min)
                .clamp(min, max);
        }
        macro_rules! set_uint {
            ($v:expr, $ty:ty) => {{
                let new = (number.round() as $ty).clamp($v.min, $v.max);
                let changed = new != $v.value;
                $v.value = new;
                changed
            }};
        }
        match self {
            Self::F32(v) => {
                let new = (number as f32).clamp(v.min, v.max);
                let changed = new.total_cmp(&v.value).is_ne();
                v.value = new;
                changed
            }
            Self::U8(v) => set_uint!(v, u8),
            Self::U16(v) => set_uint!(v, u16),
            Self::U32(v) => set_uint!(v, u32),
        }
    }

    /// The value as the number typed into the editor shows it: every digit, no unit.
    fn edit_text(&self) -> String {
        match self {
            Self::F32(v) => v.value.to_string(),
            Self::U8(v) => v.value.to_string(),
            Self::U16(v) => v.value.to_string(),
            Self::U32(v) => v.value.to_string(),
        }
    }

    /// The value, minimum, maximum and step as `f64`s.
    fn numbers(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::F32(v) => (
                f64::from(v.value),
                f64::from(v.min),
                f64::from(v.max),
                f64::from(v.step),
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
            Self::F32(float_value) => Some(float_value.value),
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
            Self::F32(float_value) => write!(f, "{:.1}", float_value.value),
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
/// The background fill of the stepper it holds, as wide as the value is far between its bounds.
#[derive(Component, Clone, Copy, Debug)]
struct ScrubFill(Entity);
/// What a stepper remembers about the press on it.
#[derive(Component, Clone, Copy, Debug, Default)]
struct Scrub {
    /// Where the value was, between 0 and 1, when the pointer went down on the stepper.
    start: Option<f64>,
    /// Whether that press became a drag, so its release is no click and no step.
    dragged: bool,
}
/// The text field that replaces the value label while the user types a number.
#[derive(Component, Clone, Copy, Debug)]
struct ScrubEditor {
    stepper: Entity,
    /// The part of the stepper that had the keyboard focus when the editor was opened with the
    /// keyboard, which gets it back when the editor closes with the focus unclaimed.
    restore_focus: Option<Entity>,
}
/// An editor whose field has given up the focus, to be closed after its commit was handled.
#[derive(Component, Clone, Copy, Debug)]
struct EditorDone;
/// The value label of a stepper, which knows the stepper for its `format`.
#[derive(Component, Clone, Copy, Debug)]
struct NumericStepperLabel(Entity);

/// How opaque the fill behind the value is over the stepper's own background.
const FILL_ALPHA: f32 = 0.25;
/// How far the pointer has to move, in logical pixels, before a press is a drag.
const SCRUB_CURSOR: EntityCursor = EntityCursor::System(SystemCursorIcon::ColResize);
const DRAG_THRESHOLD: f32 = 3.0;
/// The width of the small icon buttons at the ends of the stepper, in logical pixels.
const BUTTON_WIDTH_PX: f32 = 30.0;

fn theme_numeric_stepper(
    numeric_stepper_ref: In<Option<Entity>>,
    mut numeric_steppers: Query<(&mut Node, &mut BorderColor), With<NumericStepper>>,
    mut value_texts: Query<(&NumericStepperLabel, &mut TextColor, &mut TextFont)>,
    mut fills: Query<(&ScrubFill, &mut BackgroundColor)>,
    theme: Res<Theme>,
) {
    for (fill, mut background) in &mut fills {
        if numeric_stepper_ref.is_some_and(|stepper| stepper != fill.0) {
            continue;
        }
        background.0 = theme.colors().slider_fill.with_alpha(FILL_ALPHA);
    }

    for (label, mut text_color, mut text_font) in &mut value_texts {
        if numeric_stepper_ref.is_some_and(|stepper| stepper != label.0) {
            continue;
        }
        *text_color = TextColor(theme.colors().text_primary);
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
        border_color.set_all(theme.colors().border);
    }
}
fn on_activate(
    trigger: On<Activate>,
    decrease: Query<&ValueRef, With<Decrease>>,
    increase: Query<&ValueRef, With<Increase>>,
    mut values: Query<&mut Value>,
    parents: Query<&ChildOf>,
    scrubs: Query<&Scrub>,
    mut commands: Commands,
) {
    let button = trigger.event().event_target();
    if parents
        .get(button)
        .ok()
        .and_then(|parent| scrubs.get(parent.parent()).ok())
        .is_some_and(|scrub| scrub.dragged)
    {
        return;
    }
    if let Ok(vr) = decrease.get(button) {
        let Ok(mut value) = values.get_mut(vr.0) else {
            return;
        };
        if value.can_decrement() {
            value.decrement();
            commands.trigger(ValueChange {
                entity: vr.0,
                value: *value,
            });
        }
    } else if let Ok(vr) = increase.get(button) {
        let Ok(mut value) = values.get_mut(vr.0) else {
            return;
        };
        if value.can_increment() {
            value.increment();
            commands.trigger(ValueChange {
                entity: vr.0,
                value: *value,
            });
        }
    }
}

/// Steps the value of the stepper the key reached with the arrow keys.
///
/// The stepper owns the arrow keys while one of its buttons has the focus, so it stops them
/// there (pressed, repeated or released, also at a limit) and an app's directional
/// navigation never sees them.
fn on_key_input(
    mut trigger: On<FocusedInput<KeyboardInput>>,
    steppers: Query<(), With<NumericStepper>>,
    editors: Query<&ScrubEditor>,
    value_refs: Query<&ValueRef>,
    mut values: Query<&mut Value>,
    labels: Query<(Entity, &NumericStepperLabel)>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    focus: Res<InputFocus>,
    mut commands: Commands,
) {
    let stepper = trigger.event_target();
    if !steppers.contains(stepper) {
        return;
    }
    // The arrow keys of the number being typed belong to its text field.
    if editors.iter().any(|editor| editor.stepper == stepper) {
        return;
    }
    let input = &trigger.event().input;
    // F2 opens the number editor, as it does to edit a cell in a spreadsheet. Enter is the
    // focused button's own: it steps.
    if input.key_code == KeyCode::F2 {
        let opens = input.state == ButtonState::Pressed && !input.repeat;
        trigger.propagate(false);
        if opens {
            let value_entity = value_refs
                .get(stepper)
                .map_or(stepper, |value_ref| value_ref.0);
            if let Some((label, _)) = labels.iter().find(|(_, label)| label.0 == stepper) {
                open_editor(
                    stepper,
                    label,
                    focus.get(),
                    value_entity,
                    &children,
                    &values,
                    &mut nodes,
                    &mut commands,
                );
            }
        }
        return;
    }
    let up = match input.key_code {
        KeyCode::ArrowUp | KeyCode::ArrowRight => true,
        KeyCode::ArrowDown | KeyCode::ArrowLeft => false,
        _ => return,
    };
    let pressed = input.state == ButtonState::Pressed;
    trigger.propagate(false);
    if !pressed {
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
    } else {
        return;
    }
    commands.trigger(ValueChange {
        entity: value_entity,
        value: *value,
    });
}

/// The [`NumericStepper`] that `entity` is or lies in.
fn stepper_of(
    entity: Entity,
    parents: &Query<&ChildOf>,
    steppers: &Query<(), With<NumericStepper>>,
) -> Option<Entity> {
    let mut at = entity;
    // The deepest part (the icon of a button) is a few levels below the stepper.
    for _ in 0..8 {
        if steppers.contains(at) {
            return Some(at);
        }
        at = parents.get(at).ok()?.parent();
    }
    None
}

/// Scrubs the value when the pointer is dragged on a stepper, and opens the number editor on a
/// click on the value.
fn on_mouse(
    trigger: On<MouseEvent>,
    parents: Query<&ChildOf>,
    steppers: Query<(), With<NumericStepper>>,
    computed: Query<(&ComputedNode, Option<&ValueRef>)>,
    mut scrubs: Query<&mut Scrub>,
    labels: Query<&NumericStepperLabel>,
    editors: Query<&ScrubEditor>,
    children: Query<&Children>,
    mut values: Query<&mut Value>,
    mut nodes: Query<&mut Node>,
    mut commands: Commands,
    mut cursor_override: Option<ResMut<OverrideCursor>>,
    mut hover_lock: Option<ResMut<HoverLock>>,
) {
    let event = *trigger.event();
    if !matches!(
        event.kind,
        MouseKind::Pressed | MouseKind::Drag | MouseKind::Click(_)
    ) || event.button != Some(PointerButton::Primary)
    {
        return;
    }
    let Some(stepper) = stepper_of(event.entity, &parents, &steppers) else {
        return;
    };
    // While a number is typed, the pointer belongs to the text field.
    if editors.iter().any(|editor| editor.stepper == stepper) {
        return;
    }
    let Ok((computed_node, value_ref)) = computed.get(stepper) else {
        return;
    };
    let value_entity = value_ref.map_or(stepper, |value_ref| value_ref.0);
    let Ok(mut scrub) = scrubs.get_mut(stepper) else {
        return;
    };
    match event.kind {
        MouseKind::Pressed => {
            scrub.dragged = false;
            scrub.start = values.get(value_entity).ok().map(Value::fraction);
        }
        MouseKind::Drag => {
            let Some(start) = scrub.start else {
                return;
            };
            if !scrub.dragged && event.distance.x.abs() < DRAG_THRESHOLD {
                return;
            }
            if !scrub.dragged {
                // For the whole drag the cursor is the resize cursor wherever the pointer is,
                // and the other controls do not react to it.
                if let Some(cursor) = cursor_override.as_deref_mut() {
                    cursor.0 = Some(SCRUB_CURSOR);
                }
                if let Some(lock) = hover_lock.as_deref_mut() {
                    lock.0 = true;
                }
            }
            scrub.dragged = true;
            let width = f64::from(computed_node.size().x * computed_node.inverse_scale_factor());
            if width <= 0.0 {
                return;
            }
            let Ok(mut value) = values.get_mut(value_entity) else {
                return;
            };
            let target = start + f64::from(event.distance.x) / width;
            // Look first, so a drag that does not move the value does not mark it changed.
            let mut moved = *value;
            if moved.set_fraction(target) {
                *value = moved;
                commands.trigger(ValueChange {
                    entity: value_entity,
                    value: moved,
                });
            }
        }
        _ => {
            if scrub.dragged {
                return;
            }
            let Ok(label) = labels.get(event.entity) else {
                return;
            };
            open_editor(
                label.0,
                event.entity,
                None,
                value_entity,
                &children,
                &values,
                &mut nodes,
                &mut commands,
            );
        }
    }
}

/// Replaces the value label of a stepper with the number editor.
fn open_editor(
    stepper: Entity,
    label: Entity,
    restore_focus: Option<Entity>,
    value_entity: Entity,
    children: &Query<&Children>,
    values: &Query<&mut Value>,
    nodes: &mut Query<&mut Node>,
    commands: &mut Commands,
) {
    let Ok(value) = values.get(value_entity) else {
        return;
    };
    let Some(index) = children
        .get(stepper)
        .ok()
        .and_then(|kids| kids.iter().position(|kid| kid == label))
    else {
        return;
    };
    let Ok(mut label_node) = nodes.get_mut(label) else {
        return;
    };
    let width = match label_node.min_width {
        Val::Px(px) => Val::Px(px),
        _ => Val::Px(DEFAULT_LABEL_MIN_WIDTH_PX),
    };
    label_node.display = Display::None;
    let editor = commands
        .spawn((
            TextField::new("")
                .with_placeholder(value.edit_text())
                .with_name("Value")
                .with_min_width(width)
                .with_max_width(width),
            ScrubEditor {
                stepper,
                restore_focus,
            },
            EntityCursor::System(SystemCursorIcon::Text),
        ))
        .id();
    commands.entity(stepper).insert_children(index, &[editor]);
}

/// Sets the value to the number typed in the editor, if there is one.
fn on_editor_commit(
    trigger: On<TextFieldCommit>,
    editors: Query<&ScrubEditor>,
    value_refs: Query<&ValueRef>,
    mut values: Query<&mut Value>,
    mut commands: Commands,
) {
    let Ok(editor) = editors.get(trigger.event().entity) else {
        return;
    };
    let Some(number) = parse_number(&trigger.event().value) else {
        return;
    };
    let value_entity = value_refs
        .get(editor.stepper)
        .map_or(editor.stepper, |value_ref| value_ref.0);
    let Ok(mut value) = values.get_mut(value_entity) else {
        return;
    };
    let mut typed = *value;
    if typed.set_number(number, false) {
        *value = typed;
        commands.trigger(ValueChange {
            entity: value_entity,
            value: typed,
        });
    }
}

/// Marks the editor whose field gave up the focus, to be closed in `Last`, after the commit
/// that comes with the same loss was handled.
fn on_editor_focus_lost(
    trigger: On<FocusLost>,
    editors: Query<(Entity, &TextFieldRefs), With<ScrubEditor>>,
    mut commands: Commands,
) {
    for (editor, refs) in &editors {
        if refs.input == trigger.event_target() {
            commands.entity(editor).insert(EditorDone);
        }
    }
}

/// Gives a freshly built number editor the keyboard focus.
fn focus_new_editors(
    new: Query<&TextFieldRefs, (Added<TextFieldRefs>, With<ScrubEditor>)>,
    mut focus: ResMut<InputFocus>,
) {
    for refs in &new {
        focus.set(refs.input, FocusCause::Navigated);
    }
}

/// Closes the editors that are done and shows the value again.
fn finish_editors(
    done: Query<(Entity, &ScrubEditor), With<EditorDone>>,
    mut labels: Query<(&NumericStepperLabel, &mut Node)>,
    buttons: Query<(), (With<bevy::ui_widgets::Button>, Without<InteractionDisabled>)>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    for (editor, scrub_editor) in &done {
        commands.entity(editor).despawn();
        // Focus nobody claimed goes back to where the keys were before the editor.
        if let Some(back) = scrub_editor.restore_focus
            && focus.get().is_none()
            && buttons.contains(back)
        {
            focus.set(back, FocusCause::Navigated);
        }
        for (label, mut node) in &mut labels {
            if label.0 == scrub_editor.stepper {
                node.display = Display::DEFAULT;
            }
        }
    }
}

/// Forgets a press once the mouse button is up and no finger is down, so the next one starts
/// afresh and a stale drag never swallows a later keyboard `Activate`.
fn end_scrub(
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Option<Res<Touches>>,
    mut scrubs: Query<&mut Scrub>,
    mut cursor_override: Option<ResMut<OverrideCursor>>,
    mut hover_lock: Option<ResMut<HoverLock>>,
) {
    if buttons.pressed(MouseButton::Left)
        || touches.is_some_and(|touches| touches.iter().next().is_some())
    {
        return;
    }
    for mut scrub in &mut scrubs {
        if scrub.start.is_some() || scrub.dragged {
            *scrub = Scrub::default();
        }
    }
    if let Some(cursor) = cursor_override.as_deref_mut()
        && cursor.0 == Some(SCRUB_CURSOR)
    {
        cursor.0 = None;
    }
    if let Some(lock) = hover_lock.as_deref_mut()
        && lock.0
    {
        lock.0 = false;
    }
}

/// The leading number of `text`: the longest start of it that is a finite number.
///
/// The number grammar is Rust's `f64` one (sign, digits with an optional point,
/// optional exponent), so the text is scanned once to find where a number could
/// end, and only a few candidate prefixes are parsed: the longest, then (only
/// when that overflows to infinity) a binary search over the exponent digits
/// and over the mantissa digits, whose values grow with their length. That is
/// O(n log n) in the worst case, however long the pasted text is, where trying
/// every prefix was O(n^2).
fn parse_number(text: &str) -> Option<f64> {
    let text = text.trim();
    let bytes = text.as_bytes();
    let digits_from = |from: usize| {
        bytes.get(from..).map_or(0, |rest| {
            rest.iter().take_while(|b| b.is_ascii_digit()).count()
        })
    };
    let finite = |end: usize| {
        text.get(..end)
            .and_then(|prefix| prefix.parse::<f64>().ok())
            .filter(|number| number.is_finite())
    };

    let mut at = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let int_digits = digits_from(at);
    at += int_digits;
    let mut frac_digits = 0;
    if bytes.get(at) == Some(&b'.') {
        frac_digits = digits_from(at + 1);
        at += 1 + frac_digits;
    }
    if int_digits + frac_digits == 0 {
        return None;
    }
    let mantissa_end = at;

    // An exponent needs at least one digit after `e` and an optional sign.
    let mut exponent = None;
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        let sign_at = at + 1;
        let negative = bytes.get(sign_at) == Some(&b'-');
        let digits_at = sign_at + usize::from(matches!(bytes.get(sign_at), Some(b'+' | b'-')));
        let count = digits_from(digits_at);
        if count > 0 {
            exponent = Some((digits_at, count, negative));
        }
    }

    if let Some((digits_at, count, negative)) = exponent {
        let full = digits_at + count;
        if let Some(number) = finite(full) {
            return Some(number);
        }
        // Infinite. A positive exponent grows with its digits, so find the
        // most digits that stay finite. A negative one only shrinks the
        // number, so no exponent prefix helps.
        if !negative {
            let (mut low, mut high) = (0, count);
            while low + 1 < high {
                let mid = usize::midpoint(low, high);
                if finite(digits_at + mid).is_some() {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            if low > 0 {
                return finite(digits_at + low);
            }
        }
    }
    if let Some(number) = finite(mantissa_end) {
        return Some(number);
    }
    // The mantissa alone overflows. Its prefixes grow with their length, and
    // every one with a digit is valid, so find the longest finite one.
    let first_digit = bytes.iter().position(u8::is_ascii_digit)?;
    let (mut low, mut high) = (first_digit + 1, mantissa_end);
    while low + 1 < high {
        let mid = usize::midpoint(low, high);
        if finite(mid).is_some() {
            low = mid;
        } else {
            high = mid;
        }
    }
    finite(low)
}

/// How far `value` is between its bounds, as a CSS percentage.
fn fraction_percent(value: &Value) -> f32 {
    (value.fraction() * 100.0) as f32
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
    mut fills: Query<(&ValueRef, &mut Node), (With<ScrubFill>, Without<NumericStepper>)>,
    mut commands: Commands,
) {
    for (value_ref, value) in &changed_values {
        for (fill_value_ref, mut node) in &mut fills {
            if fill_value_ref.0 == value_ref {
                node.width = Val::Percent(fraction_percent(value));
            }
        }
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

/// Triggered on the entity holding a [`Value`] after the user stepped it with a stepper button or
/// an arrow key. Not triggered when the stepper spawns or the app sets the value.
#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct ValueChange {
    /// The entity holding the [`Value`].
    pub entity: Entity,
    /// The new value.
    pub value: Value,
}

/// A float with bounds and a step size, built only by [`Self::new`].
#[derive(Debug, Clone, Copy, Reflect)]
pub struct FloatValue {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
}

impl FloatValue {
    /// A float starting at `initial_value` (clamped to the range), between `min` and `max`,
    /// moving by `step`.
    ///
    /// # Errors
    /// [`ValueError::NotFinite`] if the value or a bound is NaN or infinite,
    /// [`ValueError::MinAboveMax`] if `min > max`, [`ValueError::InvalidStep`] if `step` is not
    /// finite and above zero.
    pub fn new(initial_value: f32, min: f32, max: f32, step: f32) -> Result<Self, ValueError> {
        if !(initial_value.is_finite() && min.is_finite() && max.is_finite()) {
            return Err(ValueError::NotFinite);
        }
        if min > max {
            return Err(ValueError::MinAboveMax);
        }
        if !(step.is_finite() && step > 0.0) {
            return Err(ValueError::InvalidStep);
        }
        Ok(Self {
            value: initial_value.clamp(min, max),
            min,
            max,
            step,
        })
    }

    /// The current value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// The lowest value.
    #[must_use]
    pub const fn min(&self) -> f32 {
        self.min
    }

    /// The highest value.
    #[must_use]
    pub const fn max(&self) -> f32 {
        self.max
    }

    /// The amount one press adds or subtracts.
    #[must_use]
    pub const fn step(&self) -> f32 {
        self.step
    }

    fn increment(&mut self) -> bool {
        let new_value = self.value + self.step;
        if new_value.is_finite() && new_value <= self.max && new_value > self.value {
            self.value = new_value;
            return true;
        }
        false
    }

    fn decrement(&mut self) -> bool {
        let new_value = self.value - self.step;
        if new_value.is_finite() && new_value >= self.min && new_value < self.value {
            self.value = new_value;
            return true;
        }
        false
    }
}

/// An unsigned integer with bounds and a step size, built only by [`Self::new`].
#[derive(Debug, Clone, Copy, Reflect)]
pub struct UIntValue<T> {
    value: T,
    min: T,
    max: T,
    step: T,
}

impl<T: UInt> UIntValue<T> {
    /// An integer starting at `initial_value` (clamped to the range), between `min` and `max`,
    /// moving by `step`.
    ///
    /// # Errors
    /// [`ValueError::MinAboveMax`] if `min > max`, [`ValueError::ZeroStep`] if `step` is zero.
    pub fn new(initial_value: T, min: T, max: T, step: T) -> Result<Self, ValueError> {
        if min > max {
            return Err(ValueError::MinAboveMax);
        }
        if step.is_zero() {
            return Err(ValueError::ZeroStep);
        }
        Ok(Self {
            value: initial_value.clamp(min, max),
            min,
            max,
            step,
        })
    }

    /// The current value.
    #[must_use]
    pub const fn value(&self) -> T {
        self.value
    }

    /// The lowest value.
    #[must_use]
    pub const fn min(&self) -> T {
        self.min
    }

    /// The highest value.
    #[must_use]
    pub const fn max(&self) -> T {
        self.max
    }

    /// The amount one press adds or subtracts.
    #[must_use]
    pub const fn step(&self) -> T {
        self.step
    }

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
    /// Whether this is zero.
    fn is_zero(self) -> bool;
    /// Adds, or `None` on overflow.
    fn checked_add(self, rhs: Self) -> Option<Self>;
    /// Subtracts, or `None` on underflow.
    fn checked_sub(self, rhs: Self) -> Option<Self>;
}

macro_rules! impl_uint {
    ($($i:ty),+ $(,)?) => {
        $(
            impl UInt for $i {
                fn is_zero(self) -> bool {
                    self == 0
                }
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests may abort loudly")]
mod tests {
    use super::*;

    macro_rules! integer_tests {
        ($($name:ident: $ty:ty, $new:ident, $as:ident;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(Value::$new(1, 5, 4, 1).unwrap_err(), ValueError::MinAboveMax);
                    assert_eq!(Value::$new(1, 0, 4, 0).unwrap_err(), ValueError::ZeroStep);
                    assert_eq!(Value::$new(1, 5, 4, 0).unwrap_err(), ValueError::MinAboveMax);
                    // The initial value is clamped to the inclusive range.
                    assert_eq!(Value::$new(0, 2, 4, 1).unwrap().$as(), Some(2));
                    assert_eq!(Value::$new(9, 2, 4, 1).unwrap().$as(), Some(4));
                    // Equal bounds and an oversized step are valid but cannot step.
                    for value in [Value::$new(3, 3, 3, 1).unwrap(), Value::$new(1, 0, 2, 5).unwrap()] {
                        assert!(!value.can_increment() && !value.can_decrement());
                        let mut stepped = value;
                        assert!(!stepped.increment() && !stepped.decrement());
                        assert_eq!(stepped.$as(), value.$as());
                    }
                    // Overflow and underflow leave the value alone.
                    let mut top = Value::$new(<$ty>::MAX, 0, <$ty>::MAX, 2).unwrap();
                    assert!(!top.increment() && top.$as() == Some(<$ty>::MAX));
                    assert!(top.decrement() && top.$as() == Some(<$ty>::MAX - 2));
                    let mut bottom = Value::$new(1, 0, <$ty>::MAX, 2).unwrap();
                    assert!(!bottom.decrement() && bottom.$as() == Some(1));
                }
            )+
        };
    }

    integer_tests! {
        a_u8_value_is_validated_clamped_and_steps_checked: u8, new_u8, as_u8;
        a_u16_value_is_validated_clamped_and_steps_checked: u16, new_u16, as_u16;
        a_u32_value_is_validated_clamped_and_steps_checked: u32, new_u32, as_u32;
    }

    #[test]
    fn a_float_value_rejects_non_finite_numbers_and_bad_steps() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                Value::new_f32(bad, 0.0, 1.0, 0.1).unwrap_err(),
                ValueError::NotFinite
            );
            assert_eq!(
                Value::new_f32(0.5, bad, 1.0, 0.1).unwrap_err(),
                ValueError::NotFinite
            );
            assert_eq!(
                Value::new_f32(0.5, 0.0, bad, 0.1).unwrap_err(),
                ValueError::NotFinite
            );
            assert_eq!(
                Value::new_f32(0.5, 0.0, 1.0, bad).unwrap_err(),
                ValueError::InvalidStep
            );
        }
        for step in [0.0, -0.0, -1.0] {
            assert_eq!(
                Value::new_f32(0.5, 0.0, 1.0, step).unwrap_err(),
                ValueError::InvalidStep
            );
        }
        assert_eq!(
            Value::new_f32(0.5, 2.0, 1.0, 0.1).unwrap_err(),
            ValueError::MinAboveMax
        );
        assert!(FloatValue::new(0.5, 0.0, 1.0, f32::MIN_POSITIVE).is_ok());
    }

    #[test]
    fn a_float_value_is_clamped_and_equal_or_oversized_ranges_cannot_step() {
        assert_eq!(
            Value::new_f32(-3.0, 0.0, 1.0, 0.5).unwrap().as_f32(),
            Some(0.0)
        );
        assert_eq!(
            Value::new_f32(3.0, 0.0, 1.0, 0.5).unwrap().as_f32(),
            Some(1.0)
        );
        for value in [
            Value::new_f32(2.0, 2.0, 2.0, 0.5).unwrap(),
            Value::new_f32(0.5, 0.0, 1.0, 4.0).unwrap(),
        ] {
            assert!(!value.can_increment() && !value.can_decrement());
        }
        let mut value = Value::new_f32(0.0, 0.0, 1.0, 0.5).unwrap();
        assert!(value.increment() && value.increment() && !value.increment());
        assert_eq!(value.as_f32(), Some(1.0));
        assert!(value.decrement());
        assert_eq!(value.as_f32(), Some(0.5));
    }

    #[test]
    fn a_float_step_that_rounds_to_the_same_value_is_not_a_step() {
        // 1e8 + 1 is 1e8 in f32, so the step changes nothing in either direction.
        let value = Value::new_f32(1.0e8, 0.0, 2.0e8, 1.0).unwrap();
        assert!(!value.can_increment() && !value.can_decrement());
        let mut stepped = value;
        assert!(!stepped.increment() && !stepped.decrement());
        assert_eq!(stepped.as_f32(), Some(1.0e8));
    }

    #[test]
    fn a_float_step_that_overflows_is_not_a_step() {
        let mut value = Value::new_f32(f32::MAX, -f32::MAX, f32::MAX, f32::MAX).unwrap();
        assert!(!value.increment());
        assert_eq!(value.as_f32(), Some(f32::MAX));
        assert!(value.decrement());
        assert_eq!(value.as_f32(), Some(0.0));
        let mut low = Value::new_f32(-f32::MAX, -f32::MAX, f32::MAX, f32::MAX).unwrap();
        assert!(!low.decrement());
    }

    #[test]
    fn a_value_shows_integers_whole_and_floats_with_one_decimal() {
        assert_eq!(Value::new_u8(7, 0, 9, 1).unwrap().to_string(), "7");
        assert_eq!(Value::new_u16(700, 0, 900, 1).unwrap().to_string(), "700");
        assert_eq!(
            Value::new_u32(70_000, 0, 90_000, 1).unwrap().to_string(),
            "70000"
        );
        assert_eq!(
            Value::new_f32(0.25, 0.0, 1.0, 0.25).unwrap().to_string(),
            "0.2"
        );
        assert_eq!(
            Value::new_f32(0.35, 0.0, 1.0, 0.25).unwrap().to_string(),
            "0.3"
        );
        assert_eq!(
            Value::new_f32(2.0, 0.0, 5.0, 1.0).unwrap().to_string(),
            "2.0"
        );
        assert_eq!(
            Value::new_f32(-1.46, -5.0, 5.0, 1.0).unwrap().to_string(),
            "-1.5"
        );
    }

    #[test]
    fn repeated_tenth_steps_show_every_tenth_despite_accumulated_error() {
        let mut value = Value::new_f32(0.0, 0.0, 1.0, 0.1).unwrap();
        let mut shown = vec![value.to_string()];
        while value.increment() {
            shown.push(value.to_string());
        }
        // The display rounds the accumulated error away, whatever the step count.
        let expected: Vec<String> = (0..shown.len())
            .map(|i| format!("{:.1}", i as f32 / 10.0))
            .collect();
        assert_eq!(shown, expected);
        // Rounding error may cost the last step, never more than that.
        assert!(shown.len() >= 10, "{shown:?}");
        let mut last = value;
        while last.decrement() {}
        assert!(last.as_f32().unwrap() <= 0.1 + 1e-6);
    }

    #[test]
    fn stepping_up_and_down_a_float_stays_inside_the_range() {
        let mut value = Value::new_f32(0.0, 0.0, 1.0, 0.1).unwrap();
        for _ in 0..30 {
            value.increment();
            let now = value.as_f32().unwrap();
            assert!((0.0..=1.0).contains(&now), "{now}");
        }
        for _ in 0..30 {
            value.decrement();
            let now = value.as_f32().unwrap();
            assert!((0.0..=1.0).contains(&now), "{now}");
        }
    }

    #[test]
    fn a_step_larger_than_the_range_cannot_step_any_value_type() {
        for mut value in [
            Value::new_u8(2, 1, 3, 200).unwrap(),
            Value::new_u16(2, 1, 3, 60_000).unwrap(),
            Value::new_u32(2, 1, 3, 4_000_000_000).unwrap(),
            Value::new_f32(2.0, 1.0, 3.0, 10.0).unwrap(),
        ] {
            assert!(!value.can_increment() && !value.can_decrement());
            assert!(!value.increment() && !value.decrement());
        }
    }

    #[test]
    fn an_integer_value_steps_exactly_to_both_limits_and_back() {
        let mut value = Value::new_u8(8, 5, 11, 3).unwrap();
        assert!(value.increment() && value.as_u8() == Some(11));
        assert!(!value.can_increment() && value.can_decrement());
        assert!(value.decrement() && value.decrement() && value.as_u8() == Some(5));
        assert!(!value.can_decrement() && value.can_increment());
        // A step that does not reach the maximum stops short of it.
        let mut short = Value::new_u8(5, 5, 10, 3).unwrap();
        assert!(short.increment() && !short.increment() && short.as_u8() == Some(8));
    }

    #[test]
    fn a_wide_integer_step_that_overflows_or_underflows_changes_nothing() {
        let mut high = UIntValue::<u8>::new(250, 0, 255, 10).unwrap();
        assert!(!high.increment() && high.value() == 250);
        let mut low = UIntValue::<u8>::new(5, 0, 255, 10).unwrap();
        assert!(!low.decrement() && low.value() == 5);
        assert!(u8::MAX.checked_add(1).is_none() && 0_u32.checked_sub(1).is_none());
        assert!(UInt::checked_add(250_u8, 10).is_none() && UInt::checked_sub(5_u8, 10).is_none());
    }

    #[test]
    fn a_value_error_has_a_message() {
        assert_eq!(ValueError::ZeroStep.to_string(), "the step is zero");
        let error: &dyn std::error::Error = &ValueError::MinAboveMax;
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn a_value_knows_how_far_it_is_between_its_bounds() {
        assert!((Value::new_u8(5, 0, 10, 1).unwrap().fraction() - 0.5).abs() < 1e-9);
        assert!(Value::new_u8(0, 0, 10, 1).unwrap().fraction().abs() < 1e-9);
        assert!((Value::new_f32(15.0, 5.0, 15.0, 0.5).unwrap().fraction() - 1.0).abs() < 1e-9);
        assert!(Value::new_u8(3, 3, 3, 1).unwrap().fraction().abs() < 1e-9);
    }

    #[test]
    fn scrubbing_snaps_to_the_step_and_stays_inside_the_bounds() {
        let mut value = Value::new_u32(1, 0, 10, 2).unwrap();
        assert!(value.set_fraction(0.33));
        assert_eq!(value.to_string(), "4");
        assert!(!value.set_fraction(0.35), "the same step is no change");
        assert!(value.set_fraction(7.0));
        assert_eq!(value.to_string(), "10");
        assert!(value.set_fraction(-3.0));
        assert_eq!(value.to_string(), "0");
        let mut float = Value::new_f32(10.0, 5.0, 15.0, 0.5).unwrap();
        assert!(float.set_fraction(0.26));
        assert_eq!(float.to_string(), "7.5");
    }

    #[test]
    fn a_typed_number_is_clamped_and_a_non_number_changes_nothing() {
        let mut value = Value::new_u8(5, 2, 8, 2).unwrap();
        assert!(
            value.set_number(7.0, false),
            "typing is not snapped to the step"
        );
        assert_eq!(value.to_string(), "7");
        assert!(value.set_number(100.0, false));
        assert_eq!(value.to_string(), "8");
        assert!(value.set_number(-4.0, false));
        assert_eq!(value.to_string(), "2");
        assert!(!value.set_number(f64::NAN, false));
        assert!(!value.set_number(f64::INFINITY, false));
        assert_eq!(value.to_string(), "2");
    }

    #[test]
    fn typed_text_is_read_as_its_leading_number() {
        assert_eq!(parse_number(" 12 "), Some(12.0));
        assert_eq!(parse_number("7.5 %"), Some(7.5));
        assert_eq!(parse_number("-3"), Some(-3.0));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("inf"), None);
    }

    /// The definition: try every prefix, longest first.
    fn longest_finite_prefix(text: &str) -> Option<f64> {
        let text = text.trim();
        (1..=text.len())
            .rev()
            .filter(|end| text.is_char_boundary(*end))
            .find_map(|end| {
                text.get(..end)
                    .and_then(|prefix| prefix.parse::<f64>().ok())
                    .filter(|number| number.is_finite())
            })
    }

    #[test]
    fn parse_number_reads_exponents_suffixes_and_malformed_text() {
        assert_eq!(parse_number("1e3"), Some(1000.0));
        assert_eq!(parse_number("1E+3x"), Some(1000.0));
        assert_eq!(parse_number("2.5e-1 units"), Some(0.25));
        assert_eq!(parse_number("1e"), Some(1.0));
        assert_eq!(parse_number("1e+"), Some(1.0));
        assert_eq!(parse_number("1e3e4"), Some(1000.0));
        assert_eq!(parse_number("5."), Some(5.0));
        assert_eq!(parse_number(".5"), Some(0.5));
        assert_eq!(parse_number("1.2.3"), Some(1.2));
        assert_eq!(parse_number("+"), None);
        assert_eq!(parse_number("-."), None);
        assert_eq!(parse_number("."), None);
        assert_eq!(parse_number(".e5"), None);
        assert_eq!(parse_number("e5"), None);
        assert_eq!(parse_number("-infinity"), None);
        assert_eq!(parse_number("1nan"), Some(1.0));
        assert_eq!(parse_number("1_000"), Some(1.0));
    }

    #[test]
    fn parse_number_stops_at_unicode_boundaries() {
        assert_eq!(parse_number("12\u{e9}3"), Some(12.0));
        assert_eq!(parse_number("7\u{ff10}"), Some(7.0));
        assert_eq!(parse_number("\u{2212}3"), None);
        assert_eq!(parse_number("\u{a0}4\u{2003}"), Some(4.0));
        assert_eq!(parse_number("3.5\u{1f600}"), Some(3.5));
    }

    #[test]
    fn parse_number_keeps_the_longest_finite_prefix_of_overflowing_text() {
        assert_eq!(parse_number("1e999"), longest_finite_prefix("1e999"));
        assert_eq!(parse_number("1e999"), Some(1e99));
        assert_eq!(parse_number("1e-999"), Some(0.0));
        let huge = format!("1{}", "0".repeat(400));
        assert_eq!(parse_number(&huge), longest_finite_prefix(&huge));
        assert_eq!(parse_number(&huge), Some(1e308));
        let shrunk = format!("1{}e-100", "0".repeat(400));
        assert_eq!(parse_number(&shrunk), Some(1e300));
        assert_eq!(parse_number("-1e400"), longest_finite_prefix("-1e400"));
    }

    #[test]
    fn parse_number_matches_trying_every_prefix() {
        let pieces = [
            "",
            "1",
            "9",
            "0",
            "-",
            "+",
            ".",
            "e",
            "E",
            "3",
            "5",
            "00",
            "e-",
            "e+",
            " ",
            "x",
            "\u{e9}",
            "inf",
            "nan",
            "99999999999999999999",
        ];
        let mut cases = vec![String::new()];
        for _ in 0..4 {
            let mut next = cases.clone();
            for case in &cases {
                for piece in pieces {
                    next.push(format!("{case}{piece}"));
                }
            }
            next.sort();
            next.dedup();
            next.truncate(60_000);
            cases = next;
        }
        for case in cases {
            assert_eq!(
                parse_number(&case),
                longest_finite_prefix(&case),
                "{case:?}"
            );
        }
        for case in [
            "1e308",
            "1e309",
            "1.7976931348623157e308",
            "1.7976931348623159e308",
            "9e307e5",
            "0e999999",
            "1e00999",
            "123456789e300",
        ] {
            assert_eq!(parse_number(case), longest_finite_prefix(case), "{case:?}");
        }
    }

    #[test]
    fn a_very_long_paste_is_parsed_in_bounded_work() {
        let start = std::time::Instant::now();
        let n = 2_000_000;
        let digits = "9".repeat(n);
        let cases = [
            digits.clone(),
            format!("1e{digits}"),
            format!("1e-{digits}"),
            format!("1.{digits}"),
            format!("1{} trailing", "x".repeat(n)),
            format!("{}1", "0".repeat(n)),
            "\u{e9}".repeat(n),
            format!("1e5{}", "\u{e9}".repeat(n)),
        ];
        let results: Vec<_> = cases.iter().map(|case| parse_number(case)).collect();
        assert_eq!(results.get(4), Some(&Some(1.0)));
        assert_eq!(results.get(5), Some(&Some(1.0)));
        assert_eq!(results.get(6), Some(&None));
        assert_eq!(results.get(7), Some(&Some(1e5)));
        assert_eq!(results.get(1), Some(&Some(1e99)));
        assert!(matches!(results.first(), Some(Some(n)) if n.is_finite()));
        assert!(start.elapsed().as_secs() < 20, "took {:?}", start.elapsed());
    }
}
