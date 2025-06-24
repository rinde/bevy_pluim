#![allow(
    missing_copy_implementations,
    missing_debug_implementations,
    reason = "wip"
)]

use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::FlexDirection;
use bevy::ui::JustifyContent;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::utils::default;

use crate::colors::BORDER_PRIMARY;
use crate::colors::PRIMARY;
use crate::colors::TEAL_500;

// TODO could use a component hook to add more children perhaps?
//
//
// TODO ComboId could be required for Combo. Could implement FromWorld to get
// correct id from Widgets
//
// Combo requires ComboButton
// TODO implement FloatingPane that takes a child, Combo requires FloatingPane
// TODO implement SelectableItems as a Widget
//
// (Combo, SelectableItems(vec!["opt 1", "opt 2", "opt 3"]))
// (Combo, ColorPalette(vec![Color::RED, Color::GREEN, Color::BLUE]))

#[derive(Component, Default)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    // Text("Hello".to_string()),
    ComboButton
    // create_children(),
    // Children::spawn(SpawnIter(vec![ComboButton].into_iter()))
    // children![ComboButton]
)]
pub struct Combo;

#[derive(Component, Default)]
#[component(on_add = on_add_func)]
pub struct ComboButton;

fn on_add_func(mut world: DeferredWorld, context: HookContext) {
    // let t = world.entity(context.entity).get::<ComboButton>().unwrap();

    // let q = world.query_filtered::<&ChildOf, With<ComboButton>>();

    // if let Ok(mut x) = world.get_entity_mut(context.entity) {
    //  x.
    // }
    world
        .commands()
        .entity(context.entity)
        .with_child(ComboButtonView);

    // world.entity_mut(context.entity).insert(ComboButton);
    // world.entity spawn(ComboButton)
    // let children = create_children();
    // children.spawn();
}

// fn create_children() -> SpawnRelatedBundle<ChildOf, impl
// SpawnableList<ChildOf>> {     Children::spawn(SpawnIter(vec![ComboButton].
// into_iter())) }

#[derive(Component, Default)]
#[require(
    Button,
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(42.0),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    Text("Select".to_string()),
    TextColor(TEAL_500),
    BackgroundColor(PRIMARY),
    BorderColor(BORDER_PRIMARY),
    BorderRadius::all(Val::Px(6.0))
)]
pub struct ComboButtonView;

// fn fsadf() {
//     let x = children![];
// }
