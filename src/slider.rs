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
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, from_reflect = false)]
#[component(on_insert = Self::on_insert)]
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
    /// The value it starts at.
    pub value: f32,
    /// The value at the left end.
    pub min: f32,
    /// The value at the right end.
    pub max: f32,
    /// The distance between the values it can take; `None` for any value.
    pub step: Option<f32>,
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
    #[must_use]
    pub const fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            step: None,
            format: None,
            label_min_width: DEFAULT_LABEL_MIN_WIDTH_PX,
            name: None,
        }
    }

    /// The slider moving in steps of `step` from `min`.
    #[must_use]
    pub const fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
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

    /// `value` within the range, on a step when the slider has one.
    #[must_use]
    pub fn snap(&self, value: f32) -> f32 {
        let clamped = value.max(self.min).min(self.max);
        let Some(step) = self.step.filter(|step| *step > 0.0) else {
            return clamped;
        };
        let steps = ((clamped - self.min) / step).round();
        let snapped = round_to_decimals(step.mul_add(steps, self.min), decimals_of(step));
        snapped.max(self.min).min(self.max)
    }

    /// The step the keys move by: `step`, or a hundredth of the range.
    fn key_step(&self) -> f32 {
        self.step
            .filter(|step| *step > 0.0)
            .unwrap_or_else(|| ((self.max - self.min) / 100.0).max(f32::EPSILON))
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
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
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

/// The number of decimals in `step`, up to six.
fn decimals_of(step: f32) -> i32 {
    (0..6)
        .find(|decimals| {
            let scaled = step * 10f32.powi(*decimals);
            (scaled - scaled.round()).abs() < 1e-3
        })
        .unwrap_or(6)
}

fn round_to_decimals(value: f32, decimals: i32) -> f32 {
    let factor = 10f32.powi(decimals);
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
            &self.theme.colors,
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
mod tests {
    use super::*;

    #[test]
    fn a_continuous_slider_only_clamps() {
        let slider = Slider::new(0.0, 0.0, 1.0);
        assert!((slider.snap(0.123_4) - 0.123_4).abs() < f32::EPSILON);
        assert!((slider.snap(-1.0) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap(2.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_stepped_slider_rounds_to_the_nearest_step_from_its_minimum() {
        let slider = Slider::new(0.0, 1.0, 11.0).with_step(2.0);
        assert!((slider.snap(4.2) - 5.0).abs() < f32::EPSILON);
        assert!((slider.snap(3.9) - 3.0).abs() < f32::EPSILON);
        assert!((slider.snap(100.0) - 11.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_decimal_step_gives_round_values() {
        let slider = Slider::new(0.0, 0.0, 1.0).with_step(0.1);
        assert!((slider.snap(0.299_999) - 0.3).abs() < f32::EPSILON);
        assert_eq!(decimals_of(0.1), 1);
        assert_eq!(decimals_of(0.25), 2);
        assert_eq!(decimals_of(5.0), 0);
    }
}
