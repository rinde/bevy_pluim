use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::Event;
use bevy::ecs::name::Name;
use bevy::ecs::query::Changed;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::reflect::Reflect;
use bevy::utils::default;
use derive_more::Deref;
use smallvec::SmallVec;

#[derive(Copy, Clone, Debug, Default)]
pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, items_change);
    }
}

#[derive(Component, Debug, Clone)]
#[component(immutable)]
pub enum Selection {
    Ref(Entity),
    Value(SelectionValue),
}

impl Selection {
    #[must_use]
    pub const fn reference(selection_ref: Entity) -> Selection {
        Selection::Ref(selection_ref)
    }

    #[must_use]
    pub const fn as_selection(&self) -> Option<&SelectionValue> {
        match self {
            Selection::Ref(_) => None,
            Selection::Value(selection) => Some(selection),
        }
    }

    #[must_use]
    pub const fn as_ref(&self) -> Option<&Entity> {
        match self {
            Selection::Ref(entity) => Some(entity),
            Selection::Value(_) => None,
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::Value(default())
    }
}

#[derive(SystemParam, Debug)]
pub struct SelectionParam<'w, 's> {
    selection_refs: Query<'w, 's, &'static Selection>,
}

impl SelectionParam<'_, '_> {
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<&SelectionValue> {
        self.get_with_ref(entity).map(|(_, s)| s)
    }

    #[must_use]
    pub fn get_ref(&self, entity: Entity) -> Option<Entity> {
        self.get_with_ref(entity).map(|(e, _)| e)
    }

    #[must_use]
    pub fn get_with_ref(&self, entity: Entity) -> Option<(Entity, &SelectionValue)> {
        self.selection_refs
            .get(entity)
            .map(|x| match x {
                Selection::Ref(entity_ref) => (
                    *entity_ref,
                    self.selection_refs
                        .get(*entity_ref)
                        .unwrap_or_else(|e| {
                            panic!("Selection reference {entity} -> {entity_ref} -> ? does not point to a Selection: {e}")
                        })
                        .as_selection()
                        .unwrap(),
                ),
                Selection::Value(selection) => (entity, selection),
            })
            .ok()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectionValue {
    pub items: SmallVec<[Entity; 4]>,
    pub multi: bool,
}

impl SelectionValue {
    #[must_use]
    pub const fn into_component(self) -> Selection {
        Selection::Value(self)
    }

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
}

#[derive(Component, Debug, Clone, Copy)]
#[component(immutable)]
pub struct ItemsRef(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = ItemValues)]
pub struct ItemOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[component(on_add = Self::on_add)]
#[require(Name::new("Items"))]
pub struct ItemValues(Vec<Entity>);

pub type ItemsSpawner<'w> = RelatedSpawner<'w, ItemOf>;

impl ItemValues {
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
pub struct ItemsChanged(pub ItemValues);

fn items_change(q: Query<(Entity, &ItemValues), Changed<ItemValues>>, mut commands: Commands) {
    for (items_ref, items) in q {
        commands.trigger_targets(ItemsChanged(items.clone()), items_ref);
    }
}
