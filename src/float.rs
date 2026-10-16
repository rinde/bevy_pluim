//! `Float`: a popup layer placed in the viewport, used by combos and tooltips.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::FixedNode;
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

/// The distance a popup keeps from the window's edges, in logical pixels.
pub(crate) const WINDOW_MARGIN: f32 = 10.0;

/// The `left` of a popup `width` wide that wants its left edge at `left`, moved in until it is
/// [`WINDOW_MARGIN`] from the window's left and right edges. A popup wider than the window
/// keeps its left edge on the margin.
pub(crate) fn keep_in_window_x(left: f32, width: f32, window_width: f32) -> f32 {
    left.min(window_width - WINDOW_MARGIN - width)
        .max(WINDOW_MARGIN)
}

/// The `top` of a popup `height` high for a button spanning `button_top` to `button_bottom`:
/// `gap` under the button, or `gap` above it when the popup would come within
/// [`WINDOW_MARGIN`] of the window's bottom.
pub(crate) fn below_or_above(
    button_top: f32,
    button_bottom: f32,
    gap: f32,
    height: f32,
    window_height: f32,
) -> f32 {
    let below = button_bottom + gap;
    if below + height + WINDOW_MARGIN > window_height {
        button_top - gap - height
    } else {
        below
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_popup_is_kept_inside_the_window_margins() {
        assert!((keep_in_window_x(500.0, 100.0, 1000.0) - 500.0).abs() < f32::EPSILON);
        assert!((keep_in_window_x(950.0, 100.0, 1000.0) - 890.0).abs() < f32::EPSILON);
        assert!((keep_in_window_x(-5.0, 100.0, 1000.0) - 10.0).abs() < f32::EPSILON);
        assert!((keep_in_window_x(0.0, 2000.0, 1000.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_popup_goes_above_its_button_when_it_would_run_off_the_bottom() {
        assert!((below_or_above(100.0, 130.0, 3.0, 200.0, 720.0) - 133.0).abs() < f32::EPSILON);
        assert!((below_or_above(600.0, 630.0, 3.0, 200.0, 720.0) - 397.0).abs() < f32::EPSILON);
    }
}
