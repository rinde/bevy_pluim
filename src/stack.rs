use std::fmt;

use bevy::prelude::*;

use crate::select::{Item, Selection, SelectionParam};
use crate::theme::Theme;

#[derive(Copy, Clone, Debug)]
pub struct StackPlugin;

impl Plugin for StackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_stack);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("Stack"), Node::default())]
pub struct Stack {
    pub selection_ref: Entity,
}

#[derive(Component)]
struct PrevDisplay(Display);

fn on_add_stack(
    t: Trigger<OnAdd, Stack>,
    stacks: Query<&Stack>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let stack_ref = t.target();
    let Ok(stack) = stacks.get(stack_ref).copied() else {
        return;
    };

    if selections.get(stack.selection_ref).is_some() {
        commands.run_system_cached_with(switch_view, (stack_ref, stack));
    }

    commands.entity(stack.selection_ref).observe(
        move |_: Trigger<OnInsert, Selection>, mut commands: Commands| {
            commands.run_system_cached_with(switch_view, (stack_ref, stack));
        },
    );
}

fn switch_view(
    stack_and_ref: In<(Entity, Stack)>,
    selections: SelectionParam,
    mut items: Query<&mut StackItem, With<Item<String>>>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    prev_displays: Query<&PrevDisplay>,
    mut commands: Commands,
    theme: Res<Theme>,
) {
    let (stack_ref, stack) = *stack_and_ref;
    let selection = selections.get(stack.selection_ref).unwrap();

    if let Ok(children) = children.get(stack_ref) {
        // hide all children
        for c in children {
            let mut node = nodes.get_mut(*c).unwrap();
            if node.display != Display::None {
                commands.entity(*c).insert(PrevDisplay(node.display));
                node.display = Display::None;
            }
        }
    }

    let Some(selected_item_ref) = selection.last() else {
        return;
    };
    if selected_item_ref == Entity::PLACEHOLDER {
        return;
    }
    let selected_stack_item = items.get(selected_item_ref).unwrap();

    let mut show_node = |node_ref| {
        let mut node = nodes.get_mut(node_ref).unwrap();
        let prev = prev_displays.get(node_ref).unwrap();
        node.display = prev.0;
    };

    // create new child or make existing visible
    if let Some(existing_node_ref) = selected_stack_item.entity {
        show_node(existing_node_ref);
    } else {
        let spawner = StackChildSpawner {
            parent: stack_ref,
            commands,
        };

        let new_ref = if let Some(creator) = selected_stack_item.create {
            creator(spawner, &theme)
        } else {
            let item_values = selections.get_item_values(stack.selection_ref).unwrap().1;
            let mut reference = Entity::PLACEHOLDER;
            for item_ref in item_values.iter() {
                let stack_item = items.get(item_ref).unwrap();
                if stack_item.id == selected_stack_item.id {
                    if let Some(existing_entity) = stack_item.entity {
                        reference = existing_entity;
                        show_node(existing_entity);
                    } else {
                        reference = stack_item.create.unwrap()(spawner, &theme);
                        items.get_mut(item_ref).unwrap().entity = Some(reference);
                    }

                    break;
                }
            }
            assert!(reference != Entity::PLACEHOLDER);
            reference
        };

        let mut mutable_selected_stack_item = items.get_mut(selected_item_ref).unwrap();
        mutable_selected_stack_item.entity = Some(new_ref);
    }
}

#[derive(Component, Clone, Copy)]
pub struct StackItem {
    entity: Option<Entity>,
    pub create: Option<&'static (dyn Fn(StackChildSpawner, &Theme) -> Entity + Send + Sync)>,
    id: Option<usize>,
}

impl fmt::Debug for StackItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackItem")
            .field("entity", &self.entity)
            .finish_non_exhaustive()
    }
}

impl StackItem {
    #[must_use]
    pub fn new(
        create: &'static (dyn Fn(StackChildSpawner, &Theme) -> Entity + Send + Sync),
    ) -> Self {
        Self {
            entity: None,
            create: Some(create),
            id: None,
        }
    }

    /// Create a new `StackItem` with a reference such that it's create function
    /// can be reused by other `StackItem`s created with
    /// [`StackItem::new_reference`].
    #[must_use]
    pub fn new_with_reference(
        id: usize,
        create: &'static (dyn Fn(StackChildSpawner, &Theme) -> Entity + Send + Sync),
    ) -> Self {
        Self {
            entity: None,
            create: Some(create),
            id: Some(id),
        }
    }

    #[must_use]
    pub fn new_reference(id: usize) -> Self {
        Self {
            entity: None,
            create: None,
            id: Some(id),
        }
    }
}

pub struct StackChildSpawner<'w, 's> {
    parent: Entity,
    commands: Commands<'w, 's>,
}

impl fmt::Debug for StackChildSpawner<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackChildSpawner")
            .field("parent", &self.parent)
            .finish_non_exhaustive()
    }
}

impl StackChildSpawner<'_, '_> {
    pub fn spawn(mut self, child: impl Bundle) -> Entity {
        let mut id = Entity::PLACEHOLDER;
        self.commands.entity(self.parent).with_children(|parent| {
            id = parent
                .spawn((Name::new("StackItem"), AddNodeIfAbsent, child))
                .id();
        });
        id
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node::default())]
struct AddNodeIfAbsent;
