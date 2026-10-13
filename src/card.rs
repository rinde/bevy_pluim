//! `Card`: a themed background, border, radius and shadow on a node.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::Theme;

/// Registers the systems and observers of [`Card`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_cards)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// A node with a themed background, border, corner radius and shadow.
#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(BoxShadow::default(), Node::default())]
#[component(on_insert=Self::on_insert)]
pub struct Card;

impl Card {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_cards, Some(context.entity));
    }
}

fn theme_cards(
    card: In<Option<Entity>>,
    theme: Res<Theme>,
    mut cards: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BoxShadow,
        ),
        With<Card>,
    >,
) {
    let iter = if let Some(card) = card.0 {
        Either::Left(cards.get_mut(card).into_iter())
    } else {
        Either::Right(cards.into_iter())
    };

    for (mut node, mut background_color, mut border_color, mut box_shadow) in iter {
        node.border = UiRect::all(theme.border_thickness);
        background_color.0 = theme.colors.background_primary;
        node.border_radius = BorderRadius::all(theme.panel_radius);
        border_color.set_all(theme.colors.border);
        *box_shadow = theme.large_shadow().into();
    }
}
