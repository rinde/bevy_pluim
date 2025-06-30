use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::Event;
use bevy::ecs::world::DeferredWorld;
use bevy::reflect::Reflect;
use derive_more::Deref;
use smallvec::SmallVec;

#[derive(Component, Debug, Clone, Default)]
#[component(immutable)]
#[component(on_insert = Self::on_insert)]
pub struct Selection {
    pub items: SmallVec<[Entity; 4]>,
    pub multi: bool,
}

impl Selection {
    #[must_use]
    pub fn is_selected(&self, item: Entity) -> bool {
        self.items.contains(&item)
    }

    #[must_use]
    pub fn toggle_selected(&self, item: Entity) -> Option<Self> {
        let mut clone = self.clone();
        if self.multi {
            if self.is_selected(item) {
                clone.items.retain(|x| *x != item);
            } else {
                clone.items.push(item);
            }
        } else {
            // selecting the same item twice is a no-op if only one selection is allowed
            if self.is_selected(item) {
                return None;
            }
            clone.items.clear();
            clone.items.push(item);
        }
        Some(clone)
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let event = OnSelectionChange {
            selection_id: context.entity,
        };
        if let Some(views) = world.entity(context.entity).get::<Views>().cloned() {
            for view_id in views.iter() {
                world.commands().trigger_targets(event, *view_id);
            }
        }
        world.commands().trigger_targets(event, context.entity);
    }
}

#[derive(Event, Copy, Clone, Debug)]
pub struct OnSelectionChange {
    pub selection_id: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = Views)]
pub struct ViewOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ViewOf, linked_spawn)]
#[require(Selection::default())]
pub struct Views(Vec<Entity>);

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = Items)]
pub struct ItemOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[require(Selection::default())]
pub struct Items(Vec<Entity>);

#[derive(Component, Debug, Clone)]
#[component(on_insert = Self::on_insert)]
pub struct Item {
    pub label: String,
}

impl Item {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let ItemOf(items_id) = world
            .entity(context.entity)
            .get::<ItemOf>()
            .copied()
            .unwrap();

        let event = OnAddItem {
            item_id: context.entity,
            items_id,
        };

        if let Some(views) = world.entity(items_id).get::<Views>().cloned() {
            for view_id in views.iter() {
                world.commands().trigger_targets(event, *view_id);
            }
        }
        world.commands().trigger_targets(event, items_id);
    }
}

#[derive(Event, Copy, Clone, Debug)]
pub struct OnAddItem {
    pub item_id: Entity,
    pub items_id: Entity,
}

impl<T: Into<String>> From<T> for Item {
    fn from(value: T) -> Self {
        Self {
            label: value.into(),
        }
    }
}
