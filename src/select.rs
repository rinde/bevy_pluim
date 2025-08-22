use bevy::ecs::component::HookContext;
use bevy::ecs::query::{QueryFilter, QuerySingleError};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use derive_more::Deref;
use educe::Educe;
use itertools::Either;
use smallvec::{SmallVec, smallvec};
use strum::EnumIs;

#[derive(Copy, Clone, Debug, Default)]
pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, items_change);
    }
}

#[derive(Component, Debug, Clone, Eq, PartialEq, Reflect)]
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

    #[must_use]
    pub const fn one() -> Self {
        Self::Value(SelectionValue::One(Entity::PLACEHOLDER))
    }

    #[must_use]
    pub const fn one_with(selected: Entity) -> Self {
        Self::Value(SelectionValue::One(selected))
    }

    #[must_use]
    pub const fn zero_or_one() -> Self {
        Self::Value(SelectionValue::ZeroOrOne(None))
    }

    #[must_use]
    pub const fn zero_or_more() -> Self {
        Self::Value(SelectionValue::ZeroOrMore(SmallVec::new_const()))
    }

    #[must_use]
    pub fn one_or_more() -> Self {
        Self::Value(SelectionValue::OneOrMore(smallvec![Entity::PLACEHOLDER]))
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::Value(default())
    }
}

#[derive(Debug, Clone, Educe, Eq, PartialEq, EnumIs, Reflect)]
#[educe(Default)]
pub enum SelectionValue {
    One(Entity),
    #[educe(Default)]
    ZeroOrOne(Option<Entity>),
    OneOrMore(SmallVec<[Entity; 4]>),
    ZeroOrMore(SmallVec<[Entity; 4]>),
}

impl SelectionValue {
    #[must_use]
    pub const fn into_component(self) -> Selection {
        Selection::Value(self)
    }

    pub const fn allow_multiple(&self) -> bool {
        match self {
            SelectionValue::One(_) | SelectionValue::ZeroOrOne(_) => false,
            SelectionValue::OneOrMore(_) | SelectionValue::ZeroOrMore(_) => true,
        }
    }

    #[must_use]
    pub fn is_selected(&self, item: Entity) -> bool {
        match self {
            SelectionValue::One(selected) => selected == &item,
            SelectionValue::ZeroOrOne(selected) => selected.is_some_and(|s| s == item),
            SelectionValue::OneOrMore(items) | SelectionValue::ZeroOrMore(items) => {
                items.contains(&item)
            }
        }
    }

    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = Entity> {
        match self {
            SelectionValue::One(selected) => Either::Left(Some(selected).into_iter().copied()),
            SelectionValue::ZeroOrOne(selected) => {
                let x = match selected {
                    Some(item) => Some(item).into_iter().copied(),
                    None => None.into_iter().copied(),
                };
                Either::Left(x)
            }
            SelectionValue::OneOrMore(items) | SelectionValue::ZeroOrMore(items) => {
                Either::Right(items.iter().copied())
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            SelectionValue::One(_) => false,
            SelectionValue::ZeroOrOne(selected) => selected.is_none(),
            SelectionValue::OneOrMore(items) | SelectionValue::ZeroOrMore(items) => {
                items.is_empty()
            }
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            SelectionValue::One(_) => 1,
            SelectionValue::ZeroOrOne(selected) => selected.is_some() as usize,
            SelectionValue::OneOrMore(items) | SelectionValue::ZeroOrMore(items) => items.len(),
        }
    }

    #[must_use]
    pub fn single(&self) -> Option<Entity> {
        match self {
            SelectionValue::One(selected) => Some(*selected),
            SelectionValue::ZeroOrOne(selected) => *selected,
            SelectionValue::OneOrMore(items) | SelectionValue::ZeroOrMore(items) => {
                (items.len() == 1).then(|| items[0])
            }
        }
    }

    #[must_use]
    pub fn toggle_selected(&self, new_selection: Entity) -> Option<Self> {
        match self {
            SelectionValue::One(selected) => {
                // is a no-op if attempted to toggle the current selection
                (*selected != new_selection).then_some(SelectionValue::One(new_selection))
            }
            SelectionValue::ZeroOrOne(selected) => match selected {
                Some(selected) if selected == &new_selection => {
                    Some(SelectionValue::ZeroOrOne(None))
                }
                _ => Some(SelectionValue::ZeroOrOne(Some(new_selection))),
            },
            SelectionValue::OneOrMore(selected) => {
                if selected.len() == 1 {
                    if selected[0] == new_selection {
                        None
                    } else {
                        Some(SelectionValue::OneOrMore(smallvec![new_selection]))
                    }
                } else {
                    // more
                    let mut clone = selected.clone();
                    if self.is_selected(new_selection) {
                        clone.retain(|x| *x != new_selection);
                    } else {
                        clone.push(new_selection);
                    }
                    Some(SelectionValue::OneOrMore(clone))
                }
            }
            SelectionValue::ZeroOrMore(selected) => {
                let mut clone = selected.clone();
                if self.is_selected(new_selection) {
                    clone.retain(|x| *x != new_selection);
                } else {
                    clone.push(new_selection);
                }
                Some(SelectionValue::ZeroOrMore(clone))
            }
        }
    }
}

#[derive(SystemParam, Debug)]
pub struct SelectionParam<'w, 's, F: QueryFilter + 'static = ()> {
    selection_refs: Query<'w, 's, &'static Selection>,
    subset: Query<'w, 's, (Entity, &'static Selection), F>,
}

impl<F> SelectionParam<'_, '_, F>
where
    F: QueryFilter + 'static,
{
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &SelectionValue)> {
        self.subset
            .iter()
            .map(|(entity, selection)| self.find(entity, selection))
    }

    pub fn single(&self) -> Result<(Entity, &SelectionValue), QuerySingleError> {
        self.subset
            .single()
            .map(|(entity, selection)| self.find(entity, selection))
    }

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
            .map(|x| self.find(entity, x))
            .ok()
    }

    fn find<'a>(
        &'a self,
        entity: Entity,
        selection: &'a Selection,
    ) -> (Entity, &'a SelectionValue) {
        match selection {
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
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[component(immutable)]
pub struct ItemsRef(pub Entity);

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[relationship(relationship_target = ItemValues)]
pub struct ItemOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[component(on_add = Self::on_add)]
#[require(Name::new("Items"))]
pub struct ItemValues(Vec<Entity>);

pub type ItemSpawner<'w> = RelatedSpawner<'w, ItemOf>;

impl ItemValues {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .insert(ItemsRef(context.entity));
    }
}

#[derive(Component, Debug, Clone, Reflect)]
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
