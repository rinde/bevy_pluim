use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::Theme;

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

#[derive(Component, Copy, Clone, Debug)]
#[require(BoxShadow::default(), BorderRadius::default(), Node::default())]
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
            &mut BorderRadius,
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

    for (mut node, mut background_color, mut border_radius, mut border_color, mut box_shadow) in
        iter
    {
        node.border = UiRect::all(theme.border_thickness);
        background_color.0 = theme.colors.background_primary;
        *border_radius = BorderRadius::all(theme.corner_radius);
        border_color.0 = theme.colors.border;
        *box_shadow = theme.large_shadow().into();
    }
}
