use std::fmt;

use bevy::prelude::*;

use crate::select::{Selection, SelectionParam};
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
    mut items: Query<&mut StackItem>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    prev_displays: Query<&PrevDisplay>,
    mut commands: Commands,
    theme: Res<Theme>,
) {
    enum Action {
        HideAll,
        CreateNew(Entity, StackChildCreator),
        SelectDifferent(Entity),
        SelectDifferentRef(Entity, Entity),
        SelectSame,
    }
    let (stack_ref, stack) = *stack_and_ref;
    let selection = selections.get(stack.selection_ref).unwrap();

    let currently_visible_entity = children.get(stack_ref).ok().and_then(|children| {
        children
            .iter()
            .find(|c| nodes.get(*c).unwrap().display != Display::None)
    });

    let mut show = |node: Option<Entity>| {
        if let Ok(children) = children.get(stack_ref) {
            for c in children.iter().filter(|c| node.is_none_or(|x| x != *c)) {
                let mut node = nodes.get_mut(c).unwrap();
                if node.display != Display::None {
                    commands.entity(c).insert(PrevDisplay(node.display));
                    node.display = Display::None;
                }
            }
        }

        if let Some(node_ref) = node {
            let mut node = nodes.get_mut(node_ref).unwrap();
            let prev = prev_displays.get(node_ref).unwrap();
            node.display = prev.0;
        }
    };

    let action = (|| {
        let Some(selected_item_ref) = selection.last() else {
            return Action::HideAll;
        };
        if selected_item_ref == Entity::PLACEHOLDER {
            return Action::HideAll;
        }
        let selected_stack_item = items.get(selected_item_ref).unwrap_or_else(|_| {
            panic!("Selected stack item {selected_item_ref} not found");
        });
        if let Some(existing_node_ref) = selected_stack_item.entity {
            return if currently_visible_entity == Some(existing_node_ref) {
                Action::SelectSame
            } else {
                Action::SelectDifferent(existing_node_ref)
            };
        }
        if let Some(creator) = selected_stack_item.create {
            return Action::CreateNew(selected_item_ref, creator);
        }
        let item_values = selections.get_item_values(stack.selection_ref).unwrap().1;

        let referenced_stack_item = item_values
            .iter()
            .find_map(|item_ref| {
                let stack_item = items.get(item_ref).unwrap();
                (stack_item.id == selected_stack_item.id && stack_item.create.is_some())
                    .then_some(stack_item)
            })
            .unwrap();

        if let Some(existing_entity) = referenced_stack_item.entity {
            if currently_visible_entity == Some(existing_entity) {
                Action::SelectSame
            } else {
                Action::SelectDifferentRef(selected_item_ref, existing_entity)
            }
        } else {
            Action::CreateNew(selected_item_ref, referenced_stack_item.create.unwrap())
        }
    })();
    match action {
        Action::HideAll => show(None),
        Action::CreateNew(selected_item_ref, creator) => {
            show(None);
            let spawner = StackChildSpawner {
                parent: stack_ref,
                commands,
            };
            let new_reference = creator(spawner, &theme);
            let mut mutable_selected_stack_item = items.get_mut(selected_item_ref).unwrap();
            mutable_selected_stack_item.entity = Some(new_reference);
        }
        Action::SelectDifferent(new_selected) => show(Some(new_selected)),
        Action::SelectDifferentRef(selected_item_ref, new_selected) => {
            show(Some(new_selected));
            let mut mutable_selected_stack_item = items.get_mut(selected_item_ref).unwrap();
            mutable_selected_stack_item.entity = Some(new_selected);
        }
        Action::SelectSame => (),
    }
}

type StackChildCreator = &'static (dyn Fn(StackChildSpawner, &Theme) -> Entity + Send + Sync);

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
