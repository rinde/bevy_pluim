use std::fmt;

use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::OnInsert;
use bevy::ui::Display;
use bevy::ui::Node;

use crate::select::Item;
use crate::select::Selection;
use crate::select::SelectionParam;
use crate::theme::Theme;

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("Stack"), Node::default())]
#[component(on_insert=Self::on_insert)]
pub struct Stack {
    pub selection_ref: Entity,
}

#[derive(Component)]
struct PrevDisplay(Display);

impl Stack {
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let stack = *world.entity(context.entity).get::<Self>().unwrap();

        world.commands().entity(stack.selection_ref).observe(
            move |event: Trigger<OnInsert, Selection>,
                  selections: SelectionParam,
                  mut items: Query<(&Item<String>, &mut StackItem)>,
                  children: Query<&Children>,
                  mut nodes: Query<&mut Node>,
                  prev_displays: Query<&PrevDisplay>,
                  mut commands: Commands,
                  theme: Res<Theme>| {
                let selection = selections.get(event.target()).unwrap();
                assert!(
                    !selection.multi,
                    "Multi-selection is not supported for Stack"
                );

                let selected_item = selection.items.first().unwrap();
                let (_selected_item, mut selected_stack_item) =
                    items.get_mut(*selected_item).unwrap();

                if let Ok(children) = children.get(context.entity) {
                    // hide all children
                    for c in children {
                        let mut node = nodes.get_mut(*c).unwrap();

                        if node.display != Display::None {
                            commands.entity(*c).insert(PrevDisplay(node.display));
                            node.display = Display::None;
                        }
                    }
                }

                // create new child or make existing visible
                if let Some(existing) = selected_stack_item.entity {
                    let mut node = nodes.get_mut(existing).unwrap();
                    let prev = prev_displays.get(existing).unwrap();
                    node.display = prev.0;
                } else {
                    let spawner = StackChildSpawner {
                        parent: context.entity,
                        commands,
                    };
                    let new_id = (selected_stack_item.create)(spawner, &theme);
                    selected_stack_item.entity = Some(new_id);
                }
            },
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct StackItem {
    entity: Option<Entity>,
    pub create: &'static (dyn Fn(StackChildSpawner, &Theme) -> Entity + Send + Sync),
}

impl fmt::Debug for StackItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StackItem")
            .field("entity", &self.entity)
            .finish_non_exhaustive()
    }
}

impl StackItem {
    pub fn new(
        create: &'static (dyn Fn(StackChildSpawner, &Theme) -> Entity + Send + Sync),
    ) -> Self {
        Self {
            entity: None,
            create,
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
