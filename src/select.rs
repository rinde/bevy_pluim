use bevy::ecs::query::{QueryFilter, QuerySingleError};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use derive_more::Deref;
use educe::Educe;
use itertools::Either;
use smallvec::{SmallVec, smallvec};
use strum::{EnumDiscriminants, EnumIs};

#[derive(Copy, Clone, Debug, Default)]
pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionResource>()
            .add_systems(Update, items_change)
            .add_systems(
                Update,
                process_event_buffer.run_if(resource_changed::<SelectionResource>),
            )
            .add_observer(on_add_selection);
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
    pub const fn builder() -> SelectionBuilder {
        SelectionBuilder
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SelectionBuilder;
#[derive(Clone, Debug)]
pub struct SelectionBuilderWithValue {
    value: SelectionValue,
}

impl SelectionBuilder {
    #[must_use]
    pub const fn one(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::One(Entity::PLACEHOLDER),
        }
    }

    #[must_use]
    pub const fn one_with(self, selection: Entity) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::One(selection),
        }
    }

    #[must_use]
    pub const fn zero_or_one(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::ZeroOrOne(None),
        }
    }

    #[must_use]
    pub const fn zero_or_more(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::ZeroOrMore(SmallVec::new_const()),
        }
    }

    #[must_use]
    pub fn one_or_more(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::OneOrMore(smallvec![Entity::PLACEHOLDER]),
        }
    }
}

impl SelectionBuilderWithValue {
    #[must_use]
    pub fn build_with_items<I, B>(self, items: I) -> impl Bundle
    where
        B: Bundle,
        I: IntoIterator<Item = B>,
        I::IntoIter: Send + Sync + 'static,
    {
        (
            Selection::Value(self.value),
            ItemValues::spawn(SpawnIter(items.into_iter())),
        )
    }

    #[must_use]
    pub fn build_with_items_ref(self, items_ref: Entity) -> impl Bundle {
        (Selection::Value(self.value), ItemsRef(items_ref))
    }
}

fn on_add_selection(
    t: Trigger<OnInsert, Selection>,
    selections: Query<(&Selection, Option<&ItemsRef>, Option<&ItemValues>)>,
    item_values_query: Query<&ItemValues>,
    mut commands: Commands,
    mut r: ResMut<SelectionResource>,
) {
    let selection_ref = t.target();
    let (selection, selection_of, item_values) = selections.get(selection_ref).unwrap();

    match (selection, selection_of, item_values) {
        (Selection::Value(SelectionValue::One(one)), items_ref, item_values)
            if items_ref.is_some() != item_values.is_some() && one == &Entity::PLACEHOLDER =>
        {
            let values = if let Some(ItemsRef(items_ref)) = items_ref {
                item_values_query.get(*items_ref).unwrap()
            } else {
                item_values.unwrap()
            };

            if let Some(first_item) = values.iter().next() {
                commands
                    .entity(selection_ref)
                    .insert(Selection::Value(SelectionValue::One(first_item)));
            } else {
                r.buffer.push((
                    selection_ref,
                    SelectionValue::One(Entity::PLACEHOLDER),
                    selection_ref,
                ));
            }
        }
        (Selection::Ref(_), _, _)
        | (Selection::Value(_), Some(_), None)
        | (Selection::Value(_), None, Some(_)) => (), // no action required
        (Selection::Value(_), None, None) => {
            panic!("Selection has no item values {selection_ref}")
        }
        (Selection::Value(_), Some(_), Some(_)) => panic!("Ambiguous selection cofig"),
    }
}

#[derive(Resource, Default)]
struct SelectionResource {
    buffer: Vec<(Entity, SelectionValue, Entity)>,
}

fn process_event_buffer(
    mut r: ResMut<SelectionResource>,
    mut commands: Commands,
    selections: SelectionParam,
) {
    for (target_ref, selection_value, selection_ref) in r.buffer.drain(..) {
        if let Some((_, values)) = selections.get_item_values(selection_ref) {
            let new_selection_value = match selection_value {
                SelectionValue::One(_) => SelectionValue::One(values.iter().next().unwrap()),
                _ => unimplemented!(),
            };
            commands
                .entity(target_ref)
                .insert(Selection::Value(new_selection_value));
        }
    }
}

#[derive(Debug, Clone, Educe, Eq, PartialEq, EnumIs, Reflect, EnumDiscriminants)]
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

    #[must_use]
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
            SelectionValue::ZeroOrOne(selected) => usize::from(selected.is_some()),
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
    item_values: Query<'w, 's, (Entity, &'static ItemValues)>,
    selections_of: Query<'w, 's, (Entity, &'static ItemsRef)>,
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
    pub fn get_with_ref(&self, selection_ref: Entity) -> Option<(Entity, &SelectionValue)> {
        self.selection_refs
            .get(selection_ref)
            .map(|x| self.find(selection_ref, x))
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

    #[must_use]
    pub fn get_item_values(&self, selection_ref: Entity) -> Option<(Entity, &ItemValues)> {
        if let Some(root_selection_ref) = self.get_ref(selection_ref) {
            let items_ref =
                if let Ok((_, ItemsRef(sel))) = self.selections_of.get(root_selection_ref) {
                    *sel
                } else {
                    root_selection_ref
                };

            self.item_values.get(items_ref).ok()
        } else {
            None
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[relationship(relationship_target = ItemValues)]
pub struct ItemOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[require(Name::new("Items"))]
pub struct ItemValues(Vec<Entity>);

pub type ItemSpawner<'w> = RelatedSpawner<'w, ItemOf>;

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

// a selection value should have this component, should not be added to any
// other component
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[relationship(relationship_target = ItemSelections)]
pub struct ItemsRef(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemsRef, linked_spawn)]
pub struct ItemSelections(Vec<Entity>);
