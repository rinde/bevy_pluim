//! `Stack`: shows the one child that matches the current selection.
//!
//! Children are created lazily by a registered system and hidden, not despawned,
//! when they are deselected.
//!
//! A [`Stack`] is immutable: replace it to point it at another selection.

use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::scene::Ready;

use crate::select::{Item, Selection, SelectionParam};

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
/// Immutable: `selection_ref` is read when the component is inserted, so build the value,
/// editing it before it is inserted, and insert a new one to change it. Replacing a live stack
/// shows the child of the new selection and hides the children of the former one; they are kept
/// and shown again if the stack points back at that selection. It cannot be queried mutably:
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
#[require(Name::new("Stack"), Node::default())]
pub struct Stack {
    /// The entity holding the selection that picks the visible child.
    pub selection_ref: Entity,
}

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
///
/// One global observer, so a despawned stack leaves nothing behind that reacts to the selection.
fn on_selection_insert(
    t: On<Insert<Selection>>,
    stacks: Query<(Entity, &Stack)>,
    mut commands: Commands,
) {
    let selection = t.event_target();
    for (stack_ref, stack) in &stacks {
        if stack.selection_ref == selection {
            commands.run_system_cached_with(switch_view, (stack_ref, *stack));
        }
    }
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
        // No selection (yet) to show: nothing of a former source stays visible.
        show(None);
        return;
    };

    let action = (|| -> Option<Action> {
        let Some(selected_item_ref) = selection.last() else {
            return Some(Action::HideAll);
        };
        let selected_stack_item = items.get(selected_item_ref).ok()?.0;
        if let Some(existing_node_ref) = selected_stack_item.entity {
            return Some(if currently_visible_entity == Some(existing_node_ref) {
                Action::SelectSame
            } else {
                Action::SelectDifferent(existing_node_ref)
            });
        }
        let id = match selected_stack_item.source {
            StackSource::Creator { create, .. } => {
                return Some(Action::CreateNew(selected_item_ref, create));
            }
            StackSource::Reference { id } => id,
        };
        let referenced =
            selections
                .get_item_values(stack.selection_ref)
                .and_then(|(_, item_values)| {
                    item_values.iter().find_map(|item_ref| {
                        let (stack_item, _) = items.get(item_ref).ok()?;
                        match stack_item.source {
                            StackSource::Creator {
                                create,
                                id: Some(creator_id),
                            } if creator_id == id => Some((item_ref, stack_item.entity, create)),
                            _ => None,
                        }
                    })
                });
        let Some((referenced_ref, referenced_entity, create)) = referenced else {
            error!(
                "StackItem {selected_item_ref} refers to id {id}, which no StackItem::new_with_reference of the stack's items has"
            );
            return Some(Action::HideAll);
        };

        Some(
            referenced_entity.map_or(Action::CreateNew(referenced_ref, create), |existing| {
                if currently_visible_entity == Some(existing) {
                    Action::SelectSame
                } else {
                    Action::SelectDifferentRef(selected_item_ref, existing)
                }
            }),
        )
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

/// The system that spawns the child of a [`StackItem`], through the [`StackChildSpawner`] it is given.
pub type StackChildCreator = SystemId<In<StackChildSpawner>>;

/// Where a [`StackItem`]'s child comes from: its own creator, or another item's.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum StackSource {
    /// A creator, with an `id` that [`StackSource::Reference`]s can reuse it by.
    Creator {
        create: StackChildCreator,
        id: Option<usize>,
    },
    /// The creator of the item that has this `id`.
    Reference { id: usize },
}

impl StackSource {
    /// What reflection builds a `StackItem` with: a creator cannot be reflected, so it is a
    /// reference no item has the id of, which shows nothing and logs an error when selected.
    const fn unreflected() -> Self {
        Self::Reference { id: usize::MAX }
    }
}

/// An item that a [`Stack`] can show; its child is created lazily by its creator.
///
/// An item has either a creator ([`StackItem::new`], [`StackItem::new_with_reference`]) or a
/// reference to the creator of another item ([`StackItem::new_reference`]); there is no item
/// with neither. Selecting a reference whose id no item with a creator has logs an error and
/// hides the stack's current child.
#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct StackItem {
    entity: Option<Entity>,
    #[reflect(ignore, default = "StackSource::unreflected")]
    source: StackSource,
}

impl StackItem {
    /// An item whose child is created by `create`.
    #[must_use]
    pub const fn new(create: StackChildCreator) -> Self {
        Self {
            entity: None,
            source: StackSource::Creator { create, id: None },
        }
    }

    /// Create a new `StackItem` with a reference such that it's create function
    /// can be reused by other `StackItem`s created with
    /// [`StackItem::new_reference`].
    #[must_use]
    pub const fn new_with_reference(id: usize, create: StackChildCreator) -> Self {
        Self {
            entity: None,
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
            entity: None,
            source: StackSource::Reference { id },
        }
    }

    /// The system that spawns this item's child, or `None` for a reference
    /// ([`StackItem::new_reference`]), which uses the creator of another item.
    #[must_use]
    pub const fn creator(&self) -> Option<StackChildCreator> {
        match self.source {
            StackSource::Creator { create, .. } => Some(create),
            StackSource::Reference { .. } => None,
        }
    }

    /// The id other items refer to this item's creator by (`Some` for
    /// [`StackItem::new_with_reference`]), or the id this item refers to (`Some` for
    /// [`StackItem::new_reference`]); `None` for an item made with [`StackItem::new`].
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
    stack_item_ref: Entity,
}

impl StackChildSpawner {
    /// Spawns `child` as the child of the stack.
    pub fn spawn(self, commands: &mut Commands, child: impl Bundle) {
        let id = commands
            .spawn((Name::new("StackItem"), child, ChildOf(self.parent)))
            .id();
        add_node_if_absent(commands, id);
        commands.run_system_cached_with(update_stack_item, (self.stack_item_ref, id));
    }

    /// Spawns `scene` as the child of the stack: the scene's root is the child.
    ///
    /// Like [`StackChildSpawner::spawn`] otherwise: the default `Node` is added only once the
    /// scene has been applied (bevy's `Ready`), and only if the scene's root still has none, so a
    /// root that requires its own `Node` (a `Panel`) keeps it, and a `Node` in the scene is used
    /// as written.
    pub fn spawn_scene(self, commands: &mut Commands, scene: impl Scene) {
        let mut child = commands.spawn((Name::new("StackItem"), ChildOf(self.parent)));
        let id = child.id();
        child.queue_apply_scene(scene);
        child.observe(add_node_when_scene_is_ready);
        commands.run_system_cached_with(update_stack_item, (self.stack_item_ref, id));
    }
}

fn update_stack_item(input: In<(Entity, Entity)>, mut stack_items: Query<&mut StackItem>) {
    let (stack_item_ref, new_child_ref) = *input;
    if let Ok(mut stack_item) = stack_items.get_mut(stack_item_ref) {
        stack_item.entity = Some(new_child_ref);
    }
}

/// Like [`add_node_if_absent`], for a scene child: runs when its scene has been applied, which
/// bevy does later than the command queue, so a `Node` the scene's root requires is already there.
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

/// Gives the stack child a default `Node` when neither it nor its required components brought
/// one, so the child's own required `Node` (a `Panel`'s width) is never overridden.
fn add_node_if_absent(commands: &mut Commands, child: Entity) {
    commands.queue(move |world: &mut World| {
        if let Ok(mut entity) = world.get_entity_mut(child)
            && !entity.contains::<Node>()
        {
            entity.insert(Node::default());
        }
    });
}
