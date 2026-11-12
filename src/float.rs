//! `Float`: a popup layer placed in the viewport, used by combos and tooltips.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::{ComputedUiRenderTargetInfo, FixedNode};
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::theme::Theme;

/// Registers the systems and observers of [`Float`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct FloatPlugin;

impl Plugin for FloatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_float)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// Component that modifies the position and appearance of a node to create a
/// floating effect.
///
/// A `Float` is a [`FixedNode`]: it is a child of its widget, but placed by its `left` and
/// `top` in the viewport, in logical pixels, and it takes no layout, clipping or transform
/// from its ancestors, so a scroll area or card around the widget neither clips it nor
/// makes room for it. It draws over the rest of the UI ([`GlobalZIndex`] 1000), and its
/// visibility still follows its ancestors'.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[require(FixedNode, GlobalZIndex(1000), BoxShadow::default())]
#[component(on_insert=Self::on_insert)]
pub struct Float;

impl Float {
    /// Component hook: positions the node absolutely and gives it the large theme shadow.
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_float, Some(context.entity));
    }
}

fn theme_float(
    float_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut query: Query<(&mut Node, &mut BoxShadow), With<Float>>,
) {
    let iter = if let Some(float_id) = float_id.as_ref() {
        Either::Left(query.get_mut(*float_id).into_iter())
    } else {
        Either::Right(query.iter_mut())
    };
    for (mut node, mut bs) in iter {
        node.position_type = PositionType::Absolute;
        node.border_radius = BorderRadius::all(theme.panel_radius);
        *bs = BoxShadow::from(theme.large_shadow());
    }
}

/// The size, in logical pixels, of the UI target a node is drawn to: its camera's viewport
/// divided by the target's scale factor (with `UiScale`), which is the space `left`, `top` and
/// `UiGlobalTransform` of a [`FixedNode`] are in.
///
/// `target` is the physical viewport size and scale factor Bevy computed for the node, `window`
/// the primary window's logical size. An unusable target (no camera, an empty viewport, a
/// scale that is not positive and finite) falls back to the primary window, and with neither
/// there is no size: the caller leaves the popup where it is.
pub(crate) fn target_size(target: Option<(UVec2, f32)>, window: Option<Vec2>) -> Option<Vec2> {
    target
        .filter(|(size, scale)| size.x > 0 && size.y > 0 && scale.is_finite() && *scale > 0.0)
        .map(|(size, scale)| size.as_vec2() / scale)
        .or(window)
}

/// Looks up the UI target size of a node for placing its popup, see [`target_size`].
#[derive(SystemParam)]
pub(crate) struct UiViewport<'w, 's> {
    targets: Query<'w, 's, &'static ComputedUiRenderTargetInfo>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

impl UiViewport<'_, '_> {
    /// The logical size of the UI target `node` is drawn to, or of the primary window when the
    /// node has no usable target data, or `None` with neither.
    pub(crate) fn size_of(&self, node: Entity) -> Option<Vec2> {
        let target = self
            .targets
            .get(node)
            .ok()
            .map(|info| (info.physical_size(), info.scale_factor()));
        target_size(
            target,
            self.windows
                .iter()
                .next()
                .map(|window| Vec2::new(window.width(), window.height())),
        )
    }
}

/// The distance a popup keeps from the window's edges, in logical pixels.
pub(crate) const WINDOW_MARGIN: f32 = 10.0;

/// The `left` of a popup `width` wide that wants its left edge at `left`, moved in until it is
/// [`WINDOW_MARGIN`] from the window's left and right edges. A popup wider than the window
/// keeps its left edge on the margin.
pub(crate) fn keep_in_window_x(left: f32, width: f32, window_width: f32) -> f32 {
    left.min(window_width - WINDOW_MARGIN - width)
        .max(WINDOW_MARGIN)
}

/// The least height a popup keeps when the window leaves it less room, in logical pixels.
pub(crate) const MIN_POPUP_HEIGHT: f32 = 100.0;

/// Where a popup goes vertically: its `top` and the `max_height` it may grow to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct VerticalFit {
    /// The popup's `top`, in the viewport.
    pub top: f32,
    /// The popup's `max_height`: all the room on its side of the button, never less than
    /// [`MIN_POPUP_HEIGHT`] (or the window less its margins, when that is smaller).
    pub max_height: f32,
}

/// Places a popup `height` high for a button spanning `button_top` to `button_bottom`: `gap`
/// under the button, or `gap` above it when it would come within [`WINDOW_MARGIN`] of the
/// window's bottom and fits above. When it fits on neither side it goes on the side with more
/// room and its `max_height` is that room, at least [`MIN_POPUP_HEIGHT`], so it stays inside
/// the window's top and bottom margins (and so it needs to scroll, as a [`ScrollPosition`]
/// node does). The `max_height` is always the room of the chosen side, so applying it does not
/// change which side the next frame's measured height picks.
pub(crate) fn fit_vertically(
    button_top: f32,
    button_bottom: f32,
    gap: f32,
    height: f32,
    window_height: f32,
) -> VerticalFit {
    let below = button_bottom + gap;
    let room_below = window_height - WINDOW_MARGIN - below;
    let room_above = button_top - gap - WINDOW_MARGIN;
    let window_room = (window_height - WINDOW_MARGIN - WINDOW_MARGIN).max(0.0);
    let go_below = height <= room_below || (height > room_above && room_below >= room_above);
    let room = if go_below { room_below } else { room_above };
    let max_height = room.max(MIN_POPUP_HEIGHT).min(window_room);
    let height = height.min(max_height);
    let top = if go_below {
        below.min(window_height - WINDOW_MARGIN - height)
    } else {
        (button_top - gap - height).max(WINDOW_MARGIN)
    };
    VerticalFit { top, max_height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_target_size_is_the_viewport_in_logical_pixels_else_the_window() {
        let window = Some(Vec2::new(1280.0, 720.0));
        // A restricted viewport of a scale 2 target.
        assert_eq!(
            target_size(Some((UVec2::new(600, 400), 2.0)), window),
            Some(Vec2::new(300.0, 200.0))
        );
        // Unavailable or unusable target data uses the primary window, else nothing.
        assert_eq!(target_size(None, window), window);
        assert_eq!(target_size(Some((UVec2::ZERO, 1.0)), window), window);
        assert_eq!(target_size(Some((UVec2::new(10, 10), 0.0)), window), window);
        assert_eq!(
            target_size(Some((UVec2::new(10, 10), f32::NAN)), window),
            window
        );
        assert_eq!(target_size(None, None), None);
    }

    #[test]
    fn a_popup_is_kept_inside_the_window_margins() {
        assert!((keep_in_window_x(500.0, 100.0, 1000.0) - 500.0).abs() < f32::EPSILON);
        assert!((keep_in_window_x(950.0, 100.0, 1000.0) - 890.0).abs() < f32::EPSILON);
        assert!((keep_in_window_x(-5.0, 100.0, 1000.0) - 10.0).abs() < f32::EPSILON);
        assert!((keep_in_window_x(0.0, 2000.0, 1000.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_popup_goes_above_its_button_when_it_would_run_off_the_bottom() {
        let fit = fit_vertically(100.0, 130.0, 3.0, 200.0, 720.0);
        assert!((fit.top - 133.0).abs() < f32::EPSILON);
        let fit = fit_vertically(600.0, 630.0, 3.0, 200.0, 720.0);
        assert!((fit.top - 397.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_popup_too_tall_for_either_side_is_capped_to_the_larger_room_inside_the_window() {
        // Room below 720 - 10 - 133 = 577, above 600 - 3 - 10 = 587 (taller wins).
        let fit = fit_vertically(600.0, 630.0, 3.0, 2000.0, 720.0);
        assert!((fit.max_height - 587.0).abs() < f32::EPSILON);
        assert!((fit.top - 10.0).abs() < f32::EPSILON);
        let fit = fit_vertically(100.0, 130.0, 3.0, 2000.0, 720.0);
        assert!((fit.max_height - 577.0).abs() < f32::EPSILON);
        assert!((fit.top - 133.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_popup_keeps_a_minimum_height_and_stays_in_the_window_where_there_is_no_room() {
        // Room below 720 - 10 - 683 = 27, above 650 - 3 - 10 = 637 would fit: above.
        let fit = fit_vertically(650.0, 680.0, 3.0, 2000.0, 720.0);
        assert!(fit.max_height >= MIN_POPUP_HEIGHT);
        // A short window: 150 high, button in the middle.
        let fit = fit_vertically(60.0, 90.0, 0.0, 500.0, 150.0);
        assert!((fit.max_height - MIN_POPUP_HEIGHT).abs() < f32::EPSILON);
        assert!(fit.top >= WINDOW_MARGIN);
        assert!(fit.top + fit.max_height <= 150.0 - WINDOW_MARGIN + f32::EPSILON);
    }

    #[test]
    fn the_side_chosen_is_stable_once_the_height_is_capped() {
        let first = fit_vertically(100.0, 130.0, 3.0, 2000.0, 720.0);
        let again = fit_vertically(100.0, 130.0, 3.0, first.max_height, 720.0);
        assert_eq!(first, again);
        let first = fit_vertically(650.0, 680.0, 3.0, 2000.0, 720.0);
        let again = fit_vertically(650.0, 680.0, 3.0, first.max_height, 720.0);
        assert_eq!(first, again);
    }
}
