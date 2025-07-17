use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::Event;
use bevy::ecs::name::Name;
use bevy::ecs::query::Changed;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::world::DeferredWorld;
use bevy::reflect::Reflect;
use derive_more::Deref;
use smallvec::SmallVec;

#[derive(Copy, Clone, Debug, Default)]
pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, items_change);
    }
}

#[derive(Component, Debug, Clone, Default)]
#[component(immutable)]
#[component(on_add=Self::on_add, on_insert = Self::on_insert)]
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

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .insert(SelectionRef(context.entity));
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        // let event = OnSelectionChange {
        //     selection_id: context.entity,
        // };
        // if let Some(views) = world.entity(context.entity).get::<Views>().cloned() {
        //     for view_id in views.iter() {
        //         world.commands().trigger_targets(event, *view_id);
        //     }
        // }
        // world.commands().trigger_targets(event, context.entity);
    }
}

// TODO remove and use Trigger<OnInsert, Selection> instead
#[derive(Event, Copy, Clone, Debug)]
pub struct OnSelectionChange {
    pub selection_id: Entity,
}
#[derive(Component, Debug, Clone, Copy)]
#[component(immutable)]
pub struct SelectionRef(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
#[component(immutable)]
pub struct ItemsRef(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = Items)]
pub struct ItemOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[component(on_add = Self::on_add)]
#[require(Selection::default(), Name::new("Items"))]
pub struct Items(Vec<Entity>);

impl Items {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .insert(ItemsRef(context.entity));
    }
}

#[derive(Component, Debug, Clone)]
#[require(Name::new("Item"))]
pub struct Item<T> {
    pub data: T,
}

impl Item<String> {
    pub fn new(label: impl Into<String>) -> Item<String> {
        Self { data: label.into() }
    }
}

impl<T: Into<String>> From<T> for Item<String> {
    fn from(value: T) -> Self {
        Self { data: value.into() }
    }
}

#[derive(Event, Debug, Clone)]
pub struct ItemsChanged(pub Items);

fn items_change(q: Query<(Entity, &Items), Changed<Items>>, mut commands: Commands) {
    for (items_ref, items) in q {
        commands.trigger_targets(ItemsChanged(items.clone()), items_ref);
    }
}
