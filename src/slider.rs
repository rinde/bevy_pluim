//! `Slider`: a horizontal range control over Bevy's headless slider.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::{
    Slider as HeadlessSlider, SliderDragState, SliderOrientation, SliderRange, SliderStep,
    SliderThumb, SliderValue, TrackClick, ValueChange,
};
use itertools::Either;

use crate::focus::{
    AccessibilitySync, TabStop, clear_label, clear_value, sync_label, sync_role, sync_value,
};
use crate::theme::{Theme, ThemeColors};

/// Registers the systems and observers of [`Slider`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin), which also adds Bevy's headless
/// `SliderPlugin` unless the app has it.
#[derive(Copy, Clone, Debug)]
pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_value_change)
            .add_observer(keep_navigation_keys)
            .add_systems(
                Update,
                (
                    restyle_sliders,
                    (|| None::<Entity>)
                        .pipe(theme_sliders)
                        .run_if(resource_changed::<Theme>),
                ),
            )
            .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync));
    }
}

/// How a [`Slider`] shows its value.
pub type SliderFormat = fn(f32) -> String;

/// Why a [`Slider`] could not be built.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SliderError {
    /// The initial value is NaN or infinite.
    NonFiniteValue,
    /// The minimum or the maximum is NaN or infinite.
    NonFiniteBound,
    /// The minimum is greater than the maximum.
    ReversedBounds,
    /// The distance between the bounds is not a finite `f32`.
    RangeTooWide,
    /// The step is not positive and finite.
    InvalidStep,
}

impl std::fmt::Display for SliderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NonFiniteValue => "the slider value must be finite",
            Self::NonFiniteBound => "the slider bounds must be finite",
            Self::ReversedBounds => "the slider minimum must not exceed its maximum",
            Self::RangeTooWide => "the distance between the slider bounds must fit an f32",
            Self::InvalidStep => "the slider step must be positive and finite",
        })
    }
}

impl std::error::Error for SliderError {}

/// The default minimum width of the value label, in logical pixels, so that the
/// track does not move as the text changes.
pub const DEFAULT_LABEL_MIN_WIDTH_PX: f32 = 54.0;

/// The width of a slider whose `Node` leaves it `Auto`, in logical pixels.
pub const DEFAULT_WIDTH_PX: f32 = 200.0;

const HEIGHT_PX: f32 = 20.0;
const THUMB_PX: f32 = 16.0;
const TRACK_PX: f32 = 4.0;
const LABEL_GAP_PX: f32 = 12.0;

/// A horizontal slider: a track, a fill up to the value and a round thumb, with
/// the value as text beside it when it has a `format`.
///
/// It is Bevy's headless `bevy::ui_widgets::Slider` underneath. Pressing the
/// track moves the thumb there and dragging moves it, both through the headless
/// slider's `ValueChange<f32>`, which the app observes on the slider. The slider
/// keeps its own `SliderValue` in step with those changes, rounded to the `step`
/// first, and the `ValueChange` the app sees carries that rounded value. An app
/// that sets the value itself inserts a `SliderValue`.
///
/// The slider is one tab stop. While it has the focus, the left and right arrows
/// move it by `step` (by a hundredth of the range when it has none), Home and
/// End to its ends; it keeps those keys from bubbling on, and lets up and down
/// through. An [`InteractionDisabled`] slider is drawn in `text_disabled` and
/// ignores the pointer and the keys.
///
/// It is 200 px wide unless the app gives its `Node` a width; the thumb stays
/// inside it at both ends. The value label lies to the right of it, in a margin
/// of `label_min_width` plus a gap that the slider reserves: a slider with a
/// `format` owns its `Node`'s `margin.right`, one without leaves it to the app.
///
/// To a screen reader it is a `Role::Slider` with the value, bounds and step as
/// numbers, the text `format` shows as its value and `name` as its label. The
/// fields are read when the component is inserted; insert it again to change them.
///
/// The value, bounds and step are validated by [`Slider::new`] and
/// [`Slider::with_step`] and read through [`Slider::value`], [`Slider::min`],
/// [`Slider::max`] and [`Slider::step`]. A range of one value (`min == max`) is
/// valid: the slider then always holds that value.
///
/// `Slider` is an immutable component: it is configuration, replaced by inserting a new
/// one, never edited in place, so a system cannot take it as `&mut`. Migrating: a `Query<&mut Slider>` becomes
/// `commands.entity(e).insert(..)` of a prepared `Slider`. Inserting a new `Slider` rebuilds the parts and resets the value to the new one without a `ValueChange`.
///
/// A `&mut` query does not compile:
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use rin_bevy_ui::slider::Slider;
///
/// fn edit(mut widgets: Query<&mut Slider>) {
///     for mut widget in &mut widgets {
///         let _ = &mut *widget;
///     }
/// }
/// ```
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, from_reflect = false)]
#[component(immutable, on_insert = Self::on_insert)]
#[require(
    Node,
    Name::new("Slider"),
    HeadlessSlider {
        track_click: TrackClick::Snap,
        orientation: SliderOrientation::Horizontal,
    },
    Hovered,
    TabStop
)]
pub struct Slider {
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    /// Turns the value into the text beside the track; `None` for no text.
    #[reflect(ignore)]
    pub format: Option<SliderFormat>,
    /// The minimum width of the value label, in logical pixels.
    pub label_min_width: f32,
    /// The accessible name of the slider; a slider without one has no label.
    pub name: Option<String>,
}

impl Slider {
    /// A continuous slider at `value` between `min` and `max`, without a label.
    ///
    /// A finite `value` outside the range is clamped into it.
    ///
    /// # Errors
    ///
    /// [`SliderError::NonFiniteBound`] when a bound is NaN or infinite,
    /// [`SliderError::ReversedBounds`] when `min > max`,
    /// [`SliderError::RangeTooWide`] when `max - min` overflows an `f32` and
    /// [`SliderError::NonFiniteValue`] when `value` is NaN or infinite.
    pub fn new(value: f32, min: f32, max: f32) -> Result<Self, SliderError> {
        if !min.is_finite() || !max.is_finite() {
            return Err(SliderError::NonFiniteBound);
        }
        if min > max {
            return Err(SliderError::ReversedBounds);
        }
        if !(max - min).is_finite() {
            return Err(SliderError::RangeTooWide);
        }
        if !value.is_finite() {
            return Err(SliderError::NonFiniteValue);
        }
        Ok(Self {
            value: value.clamp(min, max),
            min,
            max,
            step: None,
            format: None,
            label_min_width: DEFAULT_LABEL_MIN_WIDTH_PX,
            name: None,
        })
    }

    /// The value it starts at.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// The value at the left end.
    #[must_use]
    pub const fn min(&self) -> f32 {
        self.min
    }

    /// The value at the right end.
    #[must_use]
    pub const fn max(&self) -> f32 {
        self.max
    }

    /// The distance between the values it can take; `None` for any value.
    #[must_use]
    pub const fn step(&self) -> Option<f32> {
        self.step
    }

    /// The slider moving in steps of `step` from `min`. A step larger than the
    /// range is valid: only the minimum can then be picked.
    ///
    /// # Errors
    ///
    /// [`SliderError::InvalidStep`] unless `step` is positive and finite.
    pub fn with_step(mut self, step: f32) -> Result<Self, SliderError> {
        if !(step.is_finite() && step > 0.0) {
            return Err(SliderError::InvalidStep);
        }
        self.step = Some(step);
        Ok(self)
    }

    /// The slider showing its value as `format` gives it, beside the track.
    #[must_use]
    pub const fn with_format(mut self, format: SliderFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// The slider with `name` as its accessible name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The slider with a value label at least `width` logical pixels wide.
    #[must_use]
    pub const fn with_label_min_width(mut self, width: f32) -> Self {
        self.label_min_width = width;
        self
    }

    /// `value` within the range, on a step when the slider has one; the minimum
    /// for a NaN or infinite `value`.
    #[must_use]
    pub fn snap(&self, value: f32) -> f32 {
        if !value.is_finite() {
            return self.min;
        }
        // f64 so that a tiny step cannot overflow the step count.
        let (min, max) = (f64::from(self.min), f64::from(self.max));
        let clamped = f64::from(value).clamp(min, max);
        let Some(step) = self.step.map(f64::from) else {
            return clamped as f32;
        };
        let steps = ((clamped - min) / step).round();
        let mut snapped = step.mul_add(steps, min);
        if let Some(decimals) = decimals_of(step) {
            snapped = round_to_decimals(snapped, decimals);
        }
        snapped.clamp(min, max) as f32
    }

    /// The step the keys move by: `step`, or a hundredth of the range.
    fn key_step(&self) -> f32 {
        let step = self
            .step
            .unwrap_or_else(|| ((self.max - self.min) / 100.0).max(f32::EPSILON));
        // A step wider than the range moves nothing but must stay finite.
        step.min(f32::MAX)
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        let Some(slider) = world.entity(entity).get::<Self>().cloned() else {
            return;
        };
        let old_parts = world.entity(entity).get::<SliderRefs>().copied();
        // Reserve the label's margin, or give it back when the label went away.
        let margin = match (slider.format, old_parts.and_then(|old| old.label)) {
            (Some(_), _) => Some(Val::Px(slider.label_min_width + LABEL_GAP_PX)),
            (None, Some(_)) => Some(Val::Px(0.0)),
            (None, None) => None,
        };
        if let (Some(margin), Some(mut node)) = (margin, world.get_mut::<Node>(entity)) {
            node.margin.right = margin;
        }
        let mut commands = world.commands();
        if let Some(old) = old_parts {
            for part in [Some(old.track), Some(old.fill), Some(old.travel), old.label]
                .into_iter()
                .flatten()
            {
                commands.entity(part).despawn();
            }
        }

        let value = slider.snap(slider.value);
        commands.entity(entity).insert((
            SliderValue(value),
            SliderRange::new(slider.min, slider.max),
            SliderStep(slider.key_step()),
        ));

        let ignore = Pickable::IGNORE;
        let bar = |left: Val, width: Val, right: Val| Node {
            position_type: PositionType::Absolute,
            left,
            right,
            width,
            top: Val::Px((HEIGHT_PX - TRACK_PX) / 2.0),
            height: Val::Px(TRACK_PX),
            border_radius: BorderRadius::all(Val::Px(TRACK_PX / 2.0)),
            ..default()
        };
        let track = commands
            .spawn((
                Name::new("SliderTrack"),
                bar(Val::Px(0.0), Val::Auto, Val::Px(0.0)),
                BackgroundColor::default(),
                ignore,
                ChildOf(entity),
            ))
            .id();
        let fill = commands
            .spawn((
                Name::new("SliderFill"),
                bar(Val::Px(0.0), Val::Percent(0.0), Val::Auto),
                BackgroundColor::default(),
                ignore,
                ChildOf(entity),
            ))
            .id();
        // The thumb's left edge moves over the slider's width less its own, as the
        // headless slider's pointer maths assume.
        let travel = commands
            .spawn((
                Name::new("SliderTravel"),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(THUMB_PX),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                ignore,
                ChildOf(entity),
            ))
            .id();
        let thumb = commands
            .spawn((
                Name::new("SliderThumb"),
                SliderThumb,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Px((HEIGHT_PX - THUMB_PX) / 2.0),
                    width: Val::Px(THUMB_PX),
                    height: Val::Px(THUMB_PX),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor::default(),
                BorderColor::default(),
                BoxShadow::default(),
                ChildOf(travel),
            ))
            .id();
        let label = slider.format.map(|_| {
            let label = commands
                .spawn((
                    Name::new("SliderLabel"),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(100.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        margin: UiRect::left(Val::Px(LABEL_GAP_PX)),
                        min_width: Val::Px(slider.label_min_width),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ignore,
                    ChildOf(entity),
                ))
                .id();
            commands.spawn((
                Text::default(),
                TextLayout::default().with_justify(Justify::Center),
                ignore,
                ChildOf(label),
            ));
            label
        });
        commands.entity(entity).insert(SliderRefs {
            track,
            fill,
            travel,
            thumb,
            label,
        });
        commands.run_system_cached_with(theme_sliders, Some(entity));
    }
}

/// The parts of a [`Slider`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SliderRefs {
    /// The track, the full width of the slider.
    pub track: Entity,
    /// The fill over the track, from the left end to the thumb.
    pub fill: Entity,
    /// The node the thumb moves along.
    pub travel: Entity,
    /// The thumb, a `SliderThumb`.
    pub thumb: Entity,
    /// The node of the value label, when the slider has a `format`.
    pub label: Option<Entity>,
}

/// The number of decimals in `step`, up to six; `None` for a step that is not
/// a multiple of a millionth.
fn decimals_of(step: f64) -> Option<i32> {
    (0..=6).find(|decimals| {
        let scaled = step * 10f64.powi(*decimals);
        scaled.round() >= 1.0 && (scaled - scaled.round()).abs() < 1e-3
    })
}

fn round_to_decimals(value: f64, decimals: i32) -> f64 {
    let factor = 10f64.powi(decimals);
    (value * factor).round() / factor
}

/// The colours of a slider in one state.
struct Look {
    thumb: Color,
    ink: Color,
    track: Color,
    text: Color,
}

/// The colours of a slider that is `disabled`, `pressed` (on the track or
/// dragged) or `hovered`.
const fn look(colors: &ThemeColors, disabled: bool, pressed: bool, hovered: bool) -> Look {
    let thumb = if disabled {
        colors.card_surface
    } else if pressed {
        colors.accent
    } else if hovered {
        colors.accent_subtle
    } else {
        colors.card_surface
    };
    Look {
        thumb,
        ink: if disabled {
            colors.text_disabled
        } else {
            colors.slider_fill
        },
        track: colors.slider_track,
        text: if disabled {
            colors.text_disabled
        } else {
            colors.text_primary
        },
    }
}

/// What styling a slider touches.
#[derive(SystemParam)]
struct SliderStyler<'w, 's> {
    theme: Res<'w, Theme>,
    nodes: Query<'w, 's, &'static mut Node, Without<Slider>>,
    backgrounds: Query<'w, 's, &'static mut BackgroundColor>,
    borders: Query<'w, 's, &'static mut BorderColor>,
    shadows: Query<'w, 's, &'static mut BoxShadow>,
    children: Query<'w, 's, &'static Children>,
    texts: Query<
        'w,
        's,
        (
            &'static mut Text,
            &'static mut TextColor,
            &'static mut TextFont,
        ),
    >,
}

type SliderState = (
    Entity,
    &'static Slider,
    &'static SliderRefs,
    &'static SliderValue,
    &'static SliderRange,
    &'static Hovered,
    &'static SliderDragState,
    Has<Pressed>,
    Has<InteractionDisabled>,
);

impl SliderStyler<'_, '_> {
    fn style(
        &mut self,
        (_, slider, refs, value, range, hovered, drag, pressed, disabled): <SliderState as bevy::ecs::query::QueryData>::Item<'_, '_>,
    ) {
        let look = look(
            self.theme.colors(),
            disabled,
            pressed || drag.dragging,
            hovered.get(),
        );
        let position = range.thumb_position(value.0) * 100.0;
        if let Ok(mut node) = self.nodes.get_mut(refs.thumb) {
            node.left = Val::Percent(position);
        }
        if let Ok(mut node) = self.nodes.get_mut(refs.fill) {
            node.width = Val::Percent(position);
        }
        for (part, color) in [
            (refs.track, look.track),
            (refs.fill, look.ink),
            (refs.thumb, look.thumb),
        ] {
            if let Ok(mut background) = self.backgrounds.get_mut(part) {
                background.0 = color;
            }
        }
        if let Ok(mut border) = self.borders.get_mut(refs.thumb) {
            border.set_all(look.ink);
        }
        let (Some(label), Some(format)) = (refs.label, slider.format) else {
            return;
        };
        let Some(text_entity) = self
            .children
            .get(label)
            .ok()
            .and_then(|children| children.first().copied())
        else {
            return;
        };
        if let Ok((mut text, mut color, _)) = self.texts.get_mut(text_entity) {
            let shown = format(value.0);
            if text.0 != shown {
                text.0 = shown;
            }
            color.0 = look.text;
        }
    }

    /// Gives the thumb of a slider the theme's control radius.
    fn set_radius(&mut self, refs: &SliderRefs) {
        if let Ok(mut node) = self.nodes.get_mut(refs.thumb) {
            node.border_radius = BorderRadius::all(self.theme.control_radius);
        }
    }

    /// Gives the thumb of a slider the theme's small shadow.
    fn set_shadow(&mut self, refs: &SliderRefs) {
        if let Ok(mut shadow) = self.shadows.get_mut(refs.thumb) {
            *shadow = self.theme.small_shadow().into();
        }
    }

    /// Gives the value label of a slider the theme's font.
    fn set_font(&mut self, refs: &SliderRefs) {
        let Some(text_entity) = refs
            .label
            .and_then(|label| self.children.get(label).ok())
            .and_then(|children| children.first().copied())
        else {
            return;
        };
        if let Ok((_, _, mut font)) = self.texts.get_mut(text_entity) {
            *font = TextFont {
                font: self.theme.body_font.clone().into(),
                font_size: Theme::font_size(self.theme.font_size_secondary),
                ..default()
            };
        }
    }
}

/// Styles one new slider, or all of them when the theme changes.
fn theme_sliders(
    slider_ref: In<Option<Entity>>,
    mut sliders: Query<(SliderState, &mut Node)>,
    mut styler: SliderStyler,
) {
    let it = if let Some(slider_ref) = slider_ref.as_ref() {
        Either::Left(sliders.get_mut(*slider_ref).into_iter())
    } else {
        Either::Right(sliders.iter_mut())
    };
    for (state, mut node) in it {
        if node.width == Val::Auto {
            node.width = Val::Px(DEFAULT_WIDTH_PX);
        }
        node.height = Val::Px(HEIGHT_PX);
        styler.set_font(state.2);
        styler.set_shadow(state.2);
        styler.set_radius(state.2);
        styler.style(state);
    }
}

/// Restyles the sliders whose value, hover, press or disabled state changed.
fn restyle_sliders(
    sliders: Query<SliderState>,
    changed: Query<
        (),
        Or<(
            Changed<Hovered>,
            Changed<SliderValue>,
            Changed<SliderDragState>,
            Added<Pressed>,
            Added<InteractionDisabled>,
        )>,
    >,
    mut released: RemovedComponents<Pressed>,
    mut enabled: RemovedComponents<InteractionDisabled>,
    mut styler: SliderStyler,
) {
    let removed: Vec<Entity> = released.read().chain(enabled.read()).collect();
    for state in &sliders {
        if changed.contains(state.0) || removed.contains(&state.0) {
            styler.style(state);
        }
    }
}

/// Rounds a change of one of our sliders to its step and keeps its value in step.
///
/// A global observer runs before the app's observers on the slider, so the app
/// sees the rounded value.
fn on_value_change(
    mut change: On<ValueChange<f32>>,
    sliders: Query<(&Slider, &SliderValue)>,
    mut commands: Commands,
) {
    let source = change.event().source;
    let Ok((slider, current)) = sliders.get(source) else {
        return;
    };
    let value = slider.snap(change.event().value);
    change.event_mut().value = value;
    if current.0.to_bits() != value.to_bits() {
        commands.entity(source).insert(SliderValue(value));
    }
}

/// Keeps the keys a focused slider uses from bubbling on, pressed, repeated or
/// released, also at an end.
fn keep_navigation_keys(
    mut input: On<FocusedInput<KeyboardInput>>,
    sliders: Query<(), With<Slider>>,
    focus: Res<InputFocus>,
) {
    // `focused_entity` follows the event up the tree, so the focus itself is read.
    let slider = input.event_target();
    if focus.get() != Some(slider) || !sliders.contains(slider) {
        return;
    }
    if matches!(
        input.event().input.key_code,
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::Home | KeyCode::End
    ) {
        input.propagate(false);
    }
}

/// Gives the slider's accessibility node its role, label and value text; the
/// headless slider sets the numbers.
fn sync_accessibility(
    mut sliders: Query<
        (&Slider, &SliderValue, &mut AccessibilityNode),
        Or<(Changed<Slider>, Changed<SliderValue>)>,
    >,
) {
    for (slider, value, mut node) in &mut sliders {
        sync_role(&mut node, Role::Slider);
        match &slider.name {
            Some(name) => sync_label(&mut node, name),
            None => clear_label(&mut node),
        }
        match slider.format {
            Some(format) => sync_value(&mut node, &format(value.0)),
            None => clear_value(&mut node),
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests may abort loudly")]
mod tests {
    use super::*;

    const NAN: f32 = f32::NAN;
    const INF: f32 = f32::INFINITY;

    fn slider(value: f32, min: f32, max: f32) -> Slider {
        Slider::new(value, min, max).unwrap()
    }

    fn stepped(value: f32, min: f32, max: f32, step: f32) -> Slider {
        slider(value, min, max).with_step(step).unwrap()
    }

    #[test]
    fn a_continuous_slider_only_clamps() {
        let slider = slider(0.0, 0.0, 1.0);
        assert!((slider.snap(0.123_4) - 0.123_4).abs() < f32::EPSILON);
        assert!((slider.snap(-1.0) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap(2.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_stepped_slider_rounds_to_the_nearest_step_from_its_minimum() {
        let slider = stepped(1.0, 1.0, 11.0, 2.0);
        assert!((slider.snap(4.2) - 5.0).abs() < f32::EPSILON);
        assert!((slider.snap(3.9) - 3.0).abs() < f32::EPSILON);
        assert!((slider.snap(100.0) - 11.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_decimal_step_gives_round_values() {
        let slider = stepped(0.0, 0.0, 1.0, 0.1);
        assert!((slider.snap(0.299_999) - 0.3).abs() < f32::EPSILON);
        assert_eq!(decimals_of(0.1), Some(1));
        assert_eq!(decimals_of(0.25), Some(2));
        assert_eq!(decimals_of(5.0), Some(0));
        assert_eq!(decimals_of(1e-9), None);
    }

    #[test]
    fn a_fractional_step_snaps_from_a_fractional_minimum() {
        let slider = stepped(0.5, 0.5, 2.0, 0.25);
        assert!((slider.snap(0.8) - 0.75).abs() < f32::EPSILON);
        assert!((slider.snap(1.9) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reversed_bounds_are_rejected_not_swapped() {
        assert_eq!(
            Slider::new(0.5, 1.0, 0.0).err(),
            Some(SliderError::ReversedBounds)
        );
    }

    #[test]
    fn non_finite_numbers_are_rejected_in_every_argument() {
        for bad in [NAN, INF, -INF] {
            assert_eq!(
                Slider::new(bad, 0.0, 1.0).err(),
                Some(SliderError::NonFiniteValue)
            );
            assert_eq!(
                Slider::new(0.5, bad, 1.0).err(),
                Some(SliderError::NonFiniteBound)
            );
            assert_eq!(
                Slider::new(0.5, 0.0, bad).err(),
                Some(SliderError::NonFiniteBound)
            );
        }
    }

    #[test]
    fn invalid_steps_are_rejected() {
        for bad in [0.0, -1.0, -0.0, NAN, INF, -INF] {
            assert_eq!(
                slider(0.0, 0.0, 1.0).with_step(bad).err(),
                Some(SliderError::InvalidStep),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_finite_initial_value_is_clamped_into_the_range() {
        assert!((slider(5.0, 0.0, 1.0).value() - 1.0).abs() < f32::EPSILON);
        assert!((slider(-5.0, 0.0, 1.0).value() - 0.0).abs() < f32::EPSILON);
        assert!((slider(f32::MAX, 0.0, 1.0).value() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn getters_return_what_was_configured() {
        let stepped_slider = stepped(2.0, 1.0, 3.0, 0.5);
        assert!((stepped_slider.value() - 2.0).abs() < f32::EPSILON);
        assert!((stepped_slider.min() - 1.0).abs() < f32::EPSILON);
        assert!((stepped_slider.max() - 3.0).abs() < f32::EPSILON);
        assert_eq!(stepped_slider.step(), Some(0.5));
        assert_eq!(slider(2.0, 1.0, 3.0).step(), None);
    }

    #[test]
    fn equal_bounds_are_valid_and_hold_the_bound() {
        for slider in [slider(7.0, 3.0, 3.0), stepped(7.0, 3.0, 3.0, 1.0)] {
            assert!((slider.value() - 3.0).abs() < f32::EPSILON);
            assert!((slider.snap(100.0) - 3.0).abs() < f32::EPSILON);
            assert!((slider.snap(-100.0) - 3.0).abs() < f32::EPSILON);
            assert!(slider.key_step().is_finite() && slider.key_step() > 0.0);
        }
    }

    #[test]
    fn extreme_finite_bounds_are_valid_when_their_span_fits() {
        let half = f32::MAX / 2.0;
        let wide = slider(0.0, -half, half);
        assert!(wide.key_step().is_finite());
        assert!((wide.snap(f32::MAX) - half).abs() <= f32::EPSILON * half);
        let whole = slider(0.0, f32::MAX, f32::MAX);
        assert!((whole.snap(0.0) - f32::MAX).abs() < f32::EPSILON);
    }

    #[test]
    fn a_range_whose_span_overflows_is_rejected() {
        assert_eq!(
            Slider::new(0.0, f32::MIN, f32::MAX).err(),
            Some(SliderError::RangeTooWide)
        );
    }

    #[test]
    fn a_step_larger_than_the_range_only_allows_the_minimum() {
        let slider = stepped(0.5, 0.0, 1.0, 10.0);
        assert!((slider.snap(0.9) - 0.0).abs() < f32::EPSILON);
        assert!(slider.key_step().is_finite());
    }

    #[test]
    fn a_tiny_step_over_a_huge_range_snaps_to_a_finite_value() {
        let half = f32::MAX / 2.0;
        let slider = stepped(0.0, -half, half, f32::from_bits(1));
        for value in [-half, 0.0, 1.0, half] {
            assert!(slider.snap(value).is_finite());
        }
    }

    #[test]
    fn snapping_a_non_finite_value_gives_the_minimum() {
        let slider = stepped(0.0, 1.0, 5.0, 1.0);
        for bad in [NAN, INF, -INF] {
            assert!((slider.snap(bad) - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn the_keys_move_a_stepped_slider_by_its_step() {
        assert!((stepped(0.0, 0.0, 10.0, 2.5).key_step() - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn the_keys_move_a_continuous_slider_by_a_hundredth_of_its_range() {
        assert!((slider(0.0, 0.0, 1.0).key_step() - 0.01).abs() < 1e-6);
        assert!((slider(0.0, -50.0, 150.0).key_step() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn the_keys_of_a_zero_width_continuous_slider_step_by_epsilon() {
        assert!((slider(3.0, 3.0, 3.0).key_step() - f32::EPSILON).abs() < f32::EPSILON);
    }

    #[test]
    fn the_keys_of_a_step_larger_than_the_range_use_the_step() {
        assert!((stepped(0.0, 0.0, 1.0, 10.0).key_step() - 10.0).abs() < f32::EPSILON);
        assert!((stepped(0.0, 0.0, 1.0, f32::MAX).key_step() - f32::MAX).abs() < f32::EPSILON);
    }

    #[test]
    fn snapping_nan_gives_the_minimum_with_or_without_a_step() {
        assert!((slider(0.5, 2.0, 4.0).snap(NAN) - 2.0).abs() < f32::EPSILON);
        assert!((stepped(2.0, 2.0, 4.0, 0.5).snap(NAN) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn errors_display_and_are_errors() {
        let error: Box<dyn std::error::Error> = Box::new(SliderError::InvalidStep);
        assert!(error.to_string().contains("step"));
    }
}
