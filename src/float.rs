use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::Theme;

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
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
#[require(GlobalZIndex(1000), BoxShadow::default())]
#[component(on_insert=Self::on_insert)]
pub struct Float;

impl Float {
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
        Either::Left([query.get_mut(*float_id).unwrap()].into_iter())
    } else {
        Either::Right(query.iter_mut())
    };
    for (mut node, mut bs) in iter {
        node.position_type = PositionType::Absolute;
        // node.width = Val::Percent(100.0);
        *bs = BoxShadow::from(theme.large_shadow());
    }
}
