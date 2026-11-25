//! `Stack`: shows the one child that matches the current selection.

use bevy::ecs::system::SystemId;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene::Ready;

use crate::select::{Selection, SelectionParam};

/// Registers the systems and observers of [`Stack`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct StackPlugin;

impl Plugin for StackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_insert_stack)
            .add_observer(on_selection_insert)
            .register_type::<Stack>()
            .register_type::<StackItem>()
            .register_type::<PrevDisplay>();
    }
}

/// Shows the one [`StackItem`] child that matches the selection on `selection_ref`.
///
/// Immutable: insert a new value to point it at another selection.
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::stack::Stack;
/// fn edit(mut stacks: Query<&mut Stack>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, FromTemplate, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(Name::new("Stack"), Node::default(), StackChildren)]
pub struct Stack {
    /// The entity holding the selection that picks the visible child.
    pub selection_ref: Entity,
}

/// The children one [`Stack`] has made, by the item whose creator made each.
///
/// An entry whose child is no longer a child of the stack is ignored and made again.
#[derive(Component, Debug, Default)]
struct StackChildren(HashMap<Entity, Entity>);

/// The `display` a hidden stack child had before the stack hid it.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
struct PrevDisplay(Display);

/// Shows the child of the selection of a stack that was just inserted or replaced.
fn on_insert_stack(t: On<Insert<Stack>>, stacks: Query<&Stack>, mut commands: Commands) {
    let stack_ref = t.event().event_target();
    let Ok(stack) = stacks.get(stack_ref).copied() else {
        return;
    };

    commands.run_system_cached_with(switch_view, (stack_ref, stack));
}

/// Switches every live stack showing the selection that was just inserted.
fn on_selection_insert(
    t: On<Insert<Selection>>,
    stacks: Query<(Entity, &Stack)>,
    selections: Query<&Selection>,
    mut commands: Commands,
) {
    let selection = t.event_target();
    for (stack_ref, stack) in &stacks {
        let refers_to_it = stack.selection_ref == selection
            || matches!(
                selections.get(stack.selection_ref),
                Ok(Selection::Ref(target)) if *target == selection
            );
        if refers_to_it {
            commands.run_system_cached_with(switch_view, (stack_ref, *stack));
        }
    }
}

fn switch_view(
    stack_and_ref: In<(Entity, Stack)>,
    selections: SelectionParam,
    items: Query<&StackItem>,
    caches: Query<&StackChildren>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    prev_displays: Query<&PrevDisplay>,
    mut commands: Commands,
) {
    enum Action {
        HideAll,
        CreateNew(Entity, StackChildCreator),
        SelectDifferent(Entity),
        SelectSame,
    }
    let (stack_ref, stack) = *stack_and_ref;
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

    let Some(selection) = selections.get(stack.selection_ref) else {
        show(None);
        return;
    };

    let cached_child = |creator_item: Entity| {
        let child = caches.get(stack_ref).ok()?.0.get(&creator_item).copied()?;
        children
            .get(stack_ref)
            .ok()?
            .iter()
            .any(|c| c == child)
            .then_some(child)
    };

    let action = (|| -> Option<Action> {
        let Some(selected_item_ref) = selection.last() else {
            return Some(Action::HideAll);
        };
        let selected_stack_item = items.get(selected_item_ref).ok()?;
        let (creator_item, create) = match selected_stack_item.source {
            StackSource::Creator { create, .. } => (selected_item_ref, create),
            StackSource::Reference { id } => {
                let referenced =
                    selections
                        .get_item_values(stack.selection_ref)
                        .and_then(|(_, item_values)| {
                            item_values.iter().find_map(|item_ref| {
                                match items.get(item_ref).ok()?.source {
                                    StackSource::Creator {
                                        create,
                                        id: Some(creator_id),
                                    } if creator_id == id => Some((item_ref, create)),
                                    _ => None,
                                }
                            })
                        });
                let Some(referenced) = referenced else {
                    error!(
                        "StackItem {selected_item_ref} refers to id {id}, which no StackItem::new_with_reference of the stack's items has"
                    );
                    return Some(Action::HideAll);
                };
                referenced
            }
        };
        Some(match cached_child(creator_item) {
            Some(existing) if currently_visible_entity == Some(existing) => Action::SelectSame,
            Some(existing) => Action::SelectDifferent(existing),
            None => Action::CreateNew(creator_item, create),
        })
    })();
    let Some(action) = action else {
        return;
    };
    match action {
        Action::HideAll => show(None),
        Action::CreateNew(creator_item, creator) => {
            show(None);
            let spawner = StackChildSpawner {
                parent: stack_ref,
                creator_item,
            };
            commands.run_system_with(creator, spawner);
        }
        Action::SelectDifferent(new_selected) => show(Some(new_selected)),
        Action::SelectSame => (),
    }
}

/// The system that spawns the child of a [`StackItem`], through the [`StackChildSpawner`] it is given.
pub type StackChildCreator = SystemId<In<StackChildSpawner>>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum StackSource {
    Creator {
        create: StackChildCreator,
        id: Option<usize>,
    },
    Reference {
        id: usize,
    },
}

impl StackSource {
    /// What reflection builds: a reference no item has the id of.
    const fn unreflected() -> Self {
        Self::Reference { id: usize::MAX }
    }
}

/// An item that a [`Stack`] can show; its child is created lazily by its creator.
///
/// Selecting a reference whose id no item with a creator has logs an error and hides the stack's
/// current child.
#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct StackItem {
    #[reflect(ignore, default = "StackSource::unreflected")]
    source: StackSource,
}

impl StackItem {
    /// An item whose child is created by `create`.
    #[must_use]
    pub const fn new(create: StackChildCreator) -> Self {
        Self {
            source: StackSource::Creator { create, id: None },
        }
    }

    /// An item whose creator other items reuse by `id` through [`StackItem::new_reference`].
    #[must_use]
    pub const fn new_with_reference(id: usize, create: StackChildCreator) -> Self {
        Self {
            source: StackSource::Creator {
                create,
                id: Some(id),
            },
        }
    }

    /// An item that reuses the creator of the item made with [`StackItem::new_with_reference`] and the same `id`.
    #[must_use]
    pub const fn new_reference(id: usize) -> Self {
        Self {
            source: StackSource::Reference { id },
        }
    }

    /// The system that spawns this item's child, or `None` for a reference.
    #[must_use]
    pub const fn creator(&self) -> Option<StackChildCreator> {
        match self.source {
            StackSource::Creator { create, .. } => Some(create),
            StackSource::Reference { .. } => None,
        }
    }

    /// The id of the creator this item defines or refers to; `None` for [`StackItem::new`].
    #[must_use]
    pub const fn reference_id(&self) -> Option<usize> {
        match self.source {
            StackSource::Creator { id, .. } => id,
            StackSource::Reference { id } => Some(id),
        }
    }
}

/// Given to the creator system of a [`StackItem`] to spawn its child into the [`Stack`].
#[derive(Copy, Clone, Debug)]
pub struct StackChildSpawner {
    parent: Entity,
    creator_item: Entity,
}

impl StackChildSpawner {
    /// Spawns `child` as the child of the stack.
    pub fn spawn(self, commands: &mut Commands, child: impl Bundle) {
        let id = commands
            .spawn((Name::new("StackItem"), child, ChildOf(self.parent)))
            .id();
        add_node_if_absent(commands, id);
        commands.run_system_cached_with(update_stack_item, (self.parent, self.creator_item, id));
    }

    /// Spawns `scene` as the child of the stack: the scene's root is the child.
    ///
    /// The default `Node` is added only if the root has none once the scene has been applied.
    pub fn spawn_scene(self, commands: &mut Commands, scene: impl Scene) {
        let mut child = commands.spawn((Name::new("StackItem"), ChildOf(self.parent)));
        let id = child.id();
        child.queue_apply_scene(scene);
        child.observe(add_node_when_scene_is_ready);
        commands.run_system_cached_with(update_stack_item, (self.parent, self.creator_item, id));
    }
}

fn update_stack_item(
    input: In<(Entity, Entity, Entity)>,
    mut caches: Query<&mut StackChildren>,
    parents: Query<&ChildOf>,
) {
    let (stack, creator_item, child) = *input;
    if let Ok(mut cache) = caches.get_mut(stack) {
        cache
            .0
            .retain(|_, c| parents.get(*c).is_ok_and(|p| p.parent() == stack));
        cache.0.insert(creator_item, child);
    }
}

/// Like [`add_node_if_absent`], once the scene has been applied.
fn add_node_when_scene_is_ready(
    ready: On<Ready>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
) {
    let entity = ready.event_target();
    if !nodes.contains(entity) {
        commands.entity(entity).insert(Node::default());
    }
}

/// Gives the stack child a default `Node` unless it or its required components bring one.
fn add_node_if_absent(commands: &mut Commands, child: Entity) {
    commands.queue(move |world: &mut World| {
        if let Ok(mut entity) = world.get_entity_mut(child)
            && !entity.contains::<Node>()
        {
            entity.insert(Node::default());
        }
    });
}
