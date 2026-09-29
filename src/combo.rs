//! `Combo`: a button that opens a [`List`] in a [`Float`].
//!
//! The popup closes again on a click outside of it.

use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use itertools::Itertools;

use crate::MouseEvent;
use crate::float::Float;
use crate::list::{List, NoDefaultItems};
use crate::push_button::{NoDefaultHover, PushButton, PushButtonLabel, PushButtonRefs};
use crate::select::{Item, Selection, SelectionParam};
use crate::theme::Theme;

/// Registers the systems and observers of [`Combo`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (close_combos_on_outside_click, selection_change))
            .add_observer(on_insert_combo);
    }
}

/// A button that opens a [`List`] in a [`Float`].
///
/// The popup closes again on a click outside of it.
#[derive(Component, Debug, Clone, Copy)]
#[component(on_add=Self::on_add)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Name::new("Combo"),
    RelativeCursorPosition::default()
)]
pub struct Combo;

/// Marks the button of a [`Combo`].
#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboButton"))]
pub struct ComboButton;

/// The children of a [`Combo`].
#[derive(Component, Debug, Copy, Clone)]
pub struct ComboSubRefs {
    /// The button that opens and closes the popup.
    pub button_id: Entity,
    /// The list inside the popup.
    pub list_id: Entity,
}

/// Marks the list inside the popup of a [`Combo`].
#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboList"))]
pub struct ComboList;

/// Whether the popup of a [`Combo`] is showing.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
#[component(immutable, on_insert=Self::on_insert)]
pub enum ComboState {
    /// The popup is showing.
    Open,
    /// The popup is hidden.
    Closed,
}

impl ComboState {
    /// The other state.
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let (state, &ComboSubRefs { list_id, .. }) = world
            .entity(context.entity)
            .components::<(&Self, &ComboSubRefs)>();

        let visibility = match state {
            Self::Open => Visibility::Visible,
            Self::Closed => Visibility::Hidden,
        };
        world.commands().entity(list_id).insert(visibility);
    }
}

const PLACEHOLDER_TEXT: &str = "Select..";

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mut button_id = Entity::PLACEHOLDER;
        let mut list_id = Entity::PLACEHOLDER;
        world.commands().entity(context.entity).with_children(|x| {
            button_id = x
                .spawn((
                    PushButton::new(PLACEHOLDER_TEXT, "\\/"),
                    ComboButton,
                    NoDefaultHover,
                    // allow mouse events to pass through to parent
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
                ))
                .id();

            x.spawn(Node::DEFAULT).with_children(|spawner| {
                list_id = spawner
                    .spawn((
                        List,
                        Float,
                        Visibility::Hidden,
                        ComboList,
                        RelativeCursorPosition::default(),
                    ))
                    .id();
            });
        });

        world
            .commands()
            .entity(context.entity)
            .insert((ComboState::Closed, ComboSubRefs { button_id, list_id }))
            .observe(handle_combo_button_mouse_events);
    }
}

fn on_insert_combo(
    t: Trigger<OnInsert, Combo>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let combo_ref = t.target();
    let Some(combo_selection) = selections.get_ref(combo_ref) else {
        return;
    };
    commands.run_system_cached_with(update_selection, (combo_selection, combo_ref));
}

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    combos: Query<(Entity, &Selection), (With<Combo>, Without<NoDefaultItems>)>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (combo_ref, combo_selection) in combos {
            let is_changed = match combo_selection {
                Selection::Ref(entity) => changed_selection_ref == *entity,
                Selection::Value(_selection) => changed_selection_ref == combo_ref,
            };
            if is_changed {
                commands
                    .run_system_cached_with(update_selection, (changed_selection_ref, combo_ref));
            }
        }
    }
}

fn update_selection(
    input: In<(Entity, Entity)>,
    selections: SelectionParam,
    combo_sub_refs: Query<&ComboSubRefs>,
    pblr: Query<&PushButtonRefs>,
    items: Query<&Item<String>>,
    mut labels: Query<&mut Text, With<PushButtonLabel>>,
    mut commands: Commands,
) {
    let (selection_ref, combo_ref) = *input;
    let (Ok(sub_refs), Some(selection)) =
        (combo_sub_refs.get(combo_ref), selections.get(selection_ref))
    else {
        return;
    };

    if selections
        .get_ref(sub_refs.list_id)
        .is_none_or(|existing_ref| existing_ref != selection_ref)
    {
        // Selection id has changed -> propagate to the List
        commands
            .entity(sub_refs.list_id)
            .insert(Selection::reference(selection_ref));
    }

    let Ok(PushButtonRefs { label_id, .. }) = pblr.get(sub_refs.button_id) else {
        return;
    };
    let Ok(mut text) = labels.get_mut(*label_id) else {
        return;
    };
    let label = if selection.is_empty() {
        PLACEHOLDER_TEXT.to_string()
    } else {
        selection
            .iter()
            .map(|item| {
                if item == Entity::PLACEHOLDER {
                    ""
                } else {
                    items.get(item).map_or("", |item| item.data.as_str())
                }
            })
            .join(", ")
    };
    text.0 = label;
}

/// System that closes the combo when clicking outside of it.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<(Entity, &RelativeCursorPosition, &ComboState, &ComboSubRefs), With<Combo>>,
    selections: SelectionParam,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    combo_lists: Query<&RelativeCursorPosition, With<ComboList>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if mouse_button.just_released(MouseButton::Left) {
        for (combo_ref, rcp, &combo_state, subs) in combos.iter() {
            let Some(selection_value) = selections.get(combo_ref) else {
                continue;
            };
            let over_list = combo_lists
                .get(subs.list_id)
                .is_ok_and(RelativeCursorPosition::mouse_over);
            if combo_state == ComboState::Open
                && !rcp.mouse_over()
                && !(selection_value.allow_multiple() && over_list)
            {
                let Ok((mut background_color, mut border_color)) =
                    combo_buttons.get_mut(subs.button_id)
                else {
                    continue;
                };

                *background_color = BackgroundColor(theme.colors.background_primary);
                *border_color = BorderColor(theme.colors.border);
                commands.entity(combo_ref).insert(ComboState::Closed);
            }
        }
    }
}

fn handle_combo_button_mouse_events(
    trigger: Trigger<MouseEvent>,
    combos: Query<(&ComboSubRefs, &ComboState), With<Combo>>,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let Ok((ComboSubRefs { button_id, .. }, combo_state)) = combos.get(trigger.target()) else {
        return;
    };
    let Ok((mut background_color, mut border_color)) = combo_buttons.get_mut(*button_id) else {
        return;
    };

    match (trigger.event(), combo_state) {
        (MouseEvent::Pressed, _) => {
            commands
                .entity(trigger.target())
                .insert(combo_state.inverse());
        }
        (MouseEvent::Over, _) | (_, ComboState::Open) => {
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor(theme.colors.border_highlight);
        }

        (MouseEvent::Out, ComboState::Closed) => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor(theme.colors.border);
        }
        (MouseEvent::Released, ComboState::Closed) => {}
    }
}
