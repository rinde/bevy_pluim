//! `Stack`: shows the one child that matches the current selection.
//!
//! Children are created lazily by a registered system and hidden, not despawned,
//! when they are deselected.

use bevy::ecs::system::SystemId;
use bevy::prelude::*;

use crate::select::{Item, Selection, SelectionParam};

/// Registers the systems and observers of [`Stack`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct StackPlugin;

impl Plugin for StackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_stack).register_type::<Stack>();
    }
}

/// Shows the one [`StackItem`] child that matches the selection on `selection_ref`.
#[derive(Component, FromTemplate, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Name::new("Stack"), Node::default())]
pub struct Stack {
    /// The entity holding the selection that picks the visible child.
    pub selection_ref: Entity,
}

#[derive(Component)]
struct PrevDisplay(Display);

fn on_add_stack(
    t: On<Add<Stack>>,
    stacks: Query<&Stack>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let stack_ref = t.event().event_target();
    let Ok(stack) = stacks.get(stack_ref).copied() else {
        return;
    };

    if selections.get(stack.selection_ref).is_some() {
        commands.run_system_cached_with(switch_view, (stack_ref, stack));
    }

    commands.entity(stack.selection_ref).observe(
        move |_: On<Insert<Selection>>, mut commands: Commands| {
            commands.run_system_cached_with(switch_view, (stack_ref, stack));
        },
    );
}

fn switch_view(
    stack_and_ref: In<(Entity, Stack)>,
    selections: SelectionParam,
    mut items: Query<(&mut StackItem, Option<&Item<String>>)>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    prev_displays: Query<&PrevDisplay>,
    mut commands: Commands,
) {
    enum Action {
        HideAll,
        CreateNew(Entity, StackChildCreator),
        SelectDifferent(Entity),
        SelectDifferentRef(Entity, Entity),
        SelectSame,
    }
    let (stack_ref, stack) = *stack_and_ref;
    let Some(selection) = selections.get(stack.selection_ref) else {
        return;
    };

    let currently_visible_entity = children.get(stack_ref).ok().and_then(|children| {
        children.iter().find(|c| {
            nodes
                .get(*c)
                .is_ok_and(|node| node.display != Display::None)
        })
    });

    let mut show = |node: Option<Entity>| {
        if let Ok(children) = children.get(stack_ref) {
            for c in children.iter().filter(|c| node.is_none_or(|x| x != *c)) {
                let Ok(mut node) = nodes.get_mut(c) else {
                    continue;
                };
                if node.display != Display::None {
                    commands.entity(c).insert(PrevDisplay(node.display));
                    node.display = Display::None;
                }
            }
        }

        if let Some(node_ref) = node
            && let (Ok(mut node), Ok(prev)) = (nodes.get_mut(node_ref), prev_displays.get(node_ref))
        {
            node.display = prev.0;
        }
    };

    let action = (|| -> Option<Action> {
        let Some(selected_item_ref) = selection.last() else {
            return Some(Action::HideAll);
        };
        if selected_item_ref == Entity::PLACEHOLDER {
            return Some(Action::HideAll);
        }
        let selected_stack_item = items.get(selected_item_ref).ok()?.0;
        if let Some(existing_node_ref) = selected_stack_item.entity {
            return Some(if currently_visible_entity == Some(existing_node_ref) {
                Action::SelectSame
            } else {
                Action::SelectDifferent(existing_node_ref)
            });
        }
        if let Some(creator) = selected_stack_item.create {
            return Some(Action::CreateNew(selected_item_ref, creator));
        }
        let item_values = selections.get_item_values(stack.selection_ref)?.1;

        let (referenced_ref, referenced_stack_item) = item_values.iter().find_map(|item_ref| {
            let (stack_item, _) = items.get(item_ref).ok()?;
            (stack_item.id == selected_stack_item.id && stack_item.create.is_some())
                .then_some((item_ref, *stack_item))
        })?;

        if let Some(existing_entity) = referenced_stack_item.entity {
            Some(if currently_visible_entity == Some(existing_entity) {
                Action::SelectSame
            } else {
                Action::SelectDifferentRef(selected_item_ref, existing_entity)
            })
        } else {
            Some(Action::CreateNew(
                referenced_ref,
                referenced_stack_item.create?,
            ))
        }
    })();
    let Some(action) = action else {
        return;
    };
    match action {
        Action::HideAll => show(None),
        Action::CreateNew(selected_item_ref, creator) => {
            show(None);
            let spawner = StackChildSpawner {
                parent: stack_ref,
                stack_item_ref: selected_item_ref,
            };
            commands.run_system_with(creator, spawner);
        }
        Action::SelectDifferent(new_selected) => show(Some(new_selected)),
        Action::SelectDifferentRef(selected_item_ref, new_selected) => {
            show(Some(new_selected));
            if let Ok((mut mutable_selected_stack_item, _)) = items.get_mut(selected_item_ref) {
                mutable_selected_stack_item.entity = Some(new_selected);
            }
        }
        Action::SelectSame => (),
    }
}

type StackChildCreator = SystemId<In<StackChildSpawner>>;

/// An item that a [`Stack`] can show; its child is created lazily by `create`.
#[derive(Component, Copy, Clone, Debug)]
pub struct StackItem {
    entity: Option<Entity>,
    /// The system that spawns the child, through the [`StackChildSpawner`] it is given.
    pub create: Option<StackChildCreator>,
    id: Option<usize>,
}

impl StackItem {
    /// An item whose child is created by `create`.
    #[must_use]
    pub const fn new(create: StackChildCreator) -> Self {
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
    pub const fn new_with_reference(id: usize, create: StackChildCreator) -> Self {
        Self {
            entity: None,
            create: Some(create),
            id: Some(id),
        }
    }

    /// An item that reuses the `create` of the item made with [`StackItem::new_with_reference`] and the same `id`.
    #[must_use]
    pub const fn new_reference(id: usize) -> Self {
        Self {
            entity: None,
            create: None,
            id: Some(id),
        }
    }
}

/// Given to the creator system of a [`StackItem`] to spawn its child into the [`Stack`].
#[derive(Copy, Clone, Debug)]
pub struct StackChildSpawner {
    parent: Entity,
    stack_item_ref: Entity,
}

impl StackChildSpawner {
    /// Spawns `child` as the child of the stack.
    pub fn spawn(self, commands: &mut Commands, child: impl Bundle) {
        let mut id = Entity::PLACEHOLDER;
        commands.entity(self.parent).with_children(|parent| {
            id = parent
                .spawn((Name::new("StackItem"), AddNodeIfAbsent, child))
                .id();
        });
        commands.run_system_cached_with(update_stack_item, (self.stack_item_ref, id));
    }
}

impl StackChildSpawner {
    /// Spawns `scene` as the child of the stack: the scene's root is the child.
    ///
    /// Like [`StackChildSpawner::spawn`] otherwise; a `Node` in the scene patches the child's
    /// default one.
    pub fn spawn_scene(self, commands: &mut Commands, scene: impl Scene) {
        let mut child = commands.spawn((
            Name::new("StackItem"),
            AddNodeIfAbsent,
            ChildOf(self.parent),
        ));
        let id = child.id();
        child.queue_apply_scene(scene);
        commands.run_system_cached_with(update_stack_item, (self.stack_item_ref, id));
    }
}

fn update_stack_item(input: In<(Entity, Entity)>, mut stack_items: Query<&mut StackItem>) {
    let (stack_item_ref, new_child_ref) = *input;
    if let Ok(mut stack_item) = stack_items.get_mut(stack_item_ref) {
        stack_item.entity = Some(new_child_ref);
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node::default())]
struct AddNodeIfAbsent;
