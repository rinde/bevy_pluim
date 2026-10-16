//! `Divider`: a themed 1 px rule, horizontal or vertical.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::scene::{Scene, bsn};
use itertools::Either;

use crate::theme::Theme;

/// Registers the systems and observers of [`Divider`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct DividerPlugin;

impl Plugin for DividerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_dividers)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// A 1 px rule in the theme's divider colour.
///
/// Spawn [`Divider::horizontal`] or [`Divider::vertical`]. It stretches along
/// the other axis by default; set the `Node`'s `width` or `height` to give it a
/// length.
#[derive(Component, Copy, Clone, Debug, Reflect, FromTemplate)]
#[reflect(Component)]
#[require(Node::default(), BackgroundColor::default(), Name::new("Divider"))]
#[component(on_insert=Self::on_insert)]
pub struct Divider;

impl Divider {
    /// A rule 1 px tall that stretches across its parent's width.
    #[must_use]
    pub fn horizontal() -> impl Bundle {
        (
            Self,
            Node {
                height: Val::Px(1.0),
                align_self: AlignSelf::Stretch,
                flex_shrink: 0.0,
                ..default()
            },
        )
    }

    /// A rule 1 px wide that stretches along its parent's height.
    #[must_use]
    pub fn vertical() -> impl Bundle {
        (
            Self,
            Node {
                width: Val::Px(1.0),
                align_self: AlignSelf::Stretch,
                flex_shrink: 0.0,
                ..default()
            },
        )
    }

    /// [`Divider::horizontal`] as a scene, for `@{Divider::horizontal_scene()}` in `bsn!`.
    #[must_use]
    pub fn horizontal_scene() -> impl Scene {
        bsn! {
            Self
            Node {
                height: Val::Px(1.0),
                align_self: AlignSelf::Stretch,
                flex_shrink: 0.0,
            }
        }
    }

    /// [`Divider::vertical`] as a scene, for `@{Divider::vertical_scene()}` in `bsn!`.
    #[must_use]
    pub fn vertical_scene() -> impl Scene {
        bsn! {
            Self
            Node {
                width: Val::Px(1.0),
                align_self: AlignSelf::Stretch,
                flex_shrink: 0.0,
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_dividers, Some(context.entity));
    }
}

fn theme_dividers(
    divider: In<Option<Entity>>,
    theme: Res<Theme>,
    mut dividers: Query<&mut BackgroundColor, With<Divider>>,
) {
    let iter = if let Some(divider) = divider.0 {
        Either::Left(dividers.get_mut(divider).into_iter())
    } else {
        Either::Right(dividers.iter_mut())
    };

    for mut background_color in iter {
        background_color.0 = theme.colors.divider;
    }
}
