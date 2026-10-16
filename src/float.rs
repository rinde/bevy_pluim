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
