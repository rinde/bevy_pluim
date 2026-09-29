use bevy::ecs::query::{QueryData, QueryFilter, QuerySingleError, ROQueryItem};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use derive_more::Deref;
use educe::Educe;
use itertools::{Either, Itertools};
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
            .add_observer(on_add_selection)
            .register_type::<Item<String>>()
            .register_type::<ItemOf>()
            .register_type::<ItemValues>()
            .register_type::<ItemsRef>()
            .register_type::<ItemSelections>()
            .register_type::<Selection>();
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
            value: SelectionValue::OneOrMore(smallvec![]),
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
                r.buffer.push((selection_ref, selection_ref));
            }
        }
        (Selection::Value(SelectionValue::OneOrMore(selected)), items_ref, item_values)
            if items_ref.is_some() != item_values.is_some() && selected.is_empty() =>
        {
            let values = if let Some(ItemsRef(items_ref)) = items_ref {
                item_values_query.get(*items_ref).unwrap()
            } else {
                item_values.unwrap()
            };

            if let Some(first_item) = values.iter().next() {
                commands
                    .entity(selection_ref)
                    .insert(Selection::Value(SelectionValue::OneOrMore(smallvec![
                        first_item
                    ])));
            } else {
                r.buffer.push((selection_ref, selection_ref));
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
    buffer: Vec<(Entity, Entity)>,
}

fn process_event_buffer(
    mut r: ResMut<SelectionResource>,
    mut commands: Commands,
    selections: SelectionParam,
) {
    for (target_ref, selection_ref) in r.buffer.drain(..) {
        if let Some((_, values)) = selections.get_item_values(selection_ref) {
            let selection_value = selections.get(selection_ref).unwrap();
            let new_selection_value = match selection_value {
                SelectionValue::One(x) if *x == Entity::PLACEHOLDER => {
                    Some(SelectionValue::One(values.iter().next().unwrap()))
                }
                SelectionValue::OneOrMore(selected) if selected.is_empty() => {
                    Some(SelectionValue::OneOrMore(smallvec![
                        values.iter().next().unwrap()
                    ]))
                }
                SelectionValue::One(_) | SelectionValue::OneOrMore(_) => None,
                _ => unimplemented!(),
            };
            if let Some(new_selection_value) = new_selection_value {
                commands
                    .entity(target_ref)
                    .insert(Selection::Value(new_selection_value));
            }
        }
    }
}

type SelectionVec = SmallVec<[Entity; 4]>;

#[derive(Debug, Clone, Educe, Eq, PartialEq, EnumIs, Reflect, EnumDiscriminants)]
#[educe(Default)]
pub enum SelectionValue {
    One(Entity),
    #[educe(Default)]
    ZeroOrOne(Option<Entity>),
    OneOrMore(SelectionVec),
    ZeroOrMore(SelectionVec),
}

impl SelectionValue {
    #[must_use]
    pub const fn into_component(self) -> Selection {
        Selection::Value(self)
    }

    #[must_use]
    pub const fn allow_multiple(&self) -> bool {
        match self {
            Self::One(_) | Self::ZeroOrOne(_) => false,
            Self::OneOrMore(_) | Self::ZeroOrMore(_) => true,
        }
    }

    #[must_use]
    pub fn is_selected(&self, item: Entity) -> bool {
        match self {
            Self::One(selected) => selected == &item,
            Self::ZeroOrOne(selected) => selected.is_some_and(|s| s == item),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.contains(&item),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> {
        match self {
            Self::One(selected) => Either::Left(Some(selected).into_iter().copied()),
            Self::ZeroOrOne(selected) => {
                let x = match selected {
                    Some(item) => Some(item).into_iter().copied(),
                    None => None.into_iter().copied(),
                };
                Either::Left(x)
            }
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => {
                Either::Right(items.iter().copied())
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::One(_) => false,
            Self::ZeroOrOne(selected) => selected.is_none(),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.is_empty(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::ZeroOrOne(selected) => usize::from(selected.is_some()),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.len(),
        }
    }

    #[must_use]
    pub fn single(&self) -> Option<Entity> {
        match self {
            Self::One(selected) => Some(*selected),
            Self::ZeroOrOne(selected) => *selected,
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => {
                (items.len() == 1).then(|| items[0])
            }
        }
    }

    #[must_use]
    pub fn last(&self) -> Option<Entity> {
        match self {
            Self::One(selected) => Some(*selected),
            Self::ZeroOrOne(selected) => *selected,
            Self::OneOrMore(items) | Self::ZeroOrMore(items) if items.is_empty() => None,
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.last().copied(),
        }
    }

    #[must_use]
    pub fn toggle_selected(&self, new_selection: Entity) -> Option<Self> {
        match self {
            Self::One(selected) => {
                // is a no-op if attempted to toggle the current selection
                (*selected != new_selection).then_some(Self::One(new_selection))
            }
            Self::ZeroOrOne(selected) => match selected {
                Some(selected) if selected == &new_selection => Some(Self::ZeroOrOne(None)),
                _ => Some(Self::ZeroOrOne(Some(new_selection))),
            },
            Self::OneOrMore(selected) => {
                if selected.len() == 1 {
                    if selected[0] == new_selection {
                        None
                    } else {
                        let mut clone = selected.clone();
                        clone.push(new_selection);
                        Some(Self::OneOrMore(clone))
                    }
                } else {
                    // more
                    let mut clone = selected.clone();
                    if self.is_selected(new_selection) {
                        clone.retain(|x| *x != new_selection);
                    } else {
                        clone.push(new_selection);
                    }
                    Some(Self::OneOrMore(clone))
                }
            }
            Self::ZeroOrMore(selected) => {
                let mut clone = selected.clone();
                if self.is_selected(new_selection) {
                    clone.retain(|x| *x != new_selection);
                } else {
                    clone.push(new_selection);
                }
                Some(Self::ZeroOrMore(clone))
            }
        }
    }

    #[must_use]
    pub fn push(&self, new_selection: Entity) -> Option<Self> {
        self.append([new_selection])
    }

    #[must_use]
    pub fn append(&self, iter: impl IntoIterator<Item = Entity>) -> Option<Self> {
        match self {
            Self::One(_) => panic!("Cannot append to a SelectionValue::One"),
            Self::ZeroOrOne(Some(_)) => {
                panic!("Cannot append to a SelectionValue::ZeroOrOne if it already has a selection")
            }
            Self::ZeroOrOne(None) => {
                let res = iter.into_iter().unique().collect::<SelectionVec>();
                assert!(
                    res.len() <= 1,
                    "Cannot append more than one entity to a SelectionValue::ZeroOrOne"
                );
                if res.is_empty() {
                    None
                } else {
                    Some(Self::ZeroOrOne(res.first().copied()))
                }
            }
            Self::OneOrMore(selected) => {
                let new_selected = selected
                    .iter()
                    .copied()
                    .chain(iter)
                    .unique()
                    .collect::<SelectionVec>();
                if new_selected.len() == selected.len() {
                    None
                } else {
                    Some(Self::OneOrMore(new_selected))
                }
            }
            Self::ZeroOrMore(selected) => {
                let new_selected = selected
                    .iter()
                    .copied()
                    .chain(iter)
                    .unique()
                    .collect::<SelectionVec>();
                if new_selected.len() == selected.len() {
                    None
                } else {
                    Some(Self::ZeroOrMore(new_selected))
                }
            }
        }
    }

    #[must_use]
    pub fn pop(&self) -> Option<Self> {
        match self {
            Self::One(_) => panic!("Cannot pop a SelectionValue::One"),
            Self::ZeroOrOne(None) => None,
            Self::ZeroOrOne(Some(_)) => Some(Self::ZeroOrOne(None)),
            Self::OneOrMore(selection) if selection.len() <= 1 => None,
            Self::OneOrMore(s) => Some(Self::OneOrMore(
                s.iter().take(s.len() - 1).copied().collect(),
            )),
            Self::ZeroOrMore(selection) if selection.is_empty() => None,
            Self::ZeroOrMore(s) => Some(Self::ZeroOrMore(
                s.iter().take(s.len() - 1).copied().collect(),
            )),
        }
    }

    #[must_use]
    pub fn trim_to_length(&self, new_length: usize) -> Option<Self> {
        match self {
            Self::One(_) => panic!("Cannot trim a SelectionValue::One"),
            Self::ZeroOrOne(_) if new_length > 1 => {
                panic!("Cannot trim SelectionValue::ZeroOrOne to {new_length}")
            }
            Self::ZeroOrOne(_) if new_length == 1 => None,
            Self::ZeroOrOne(_) => Some(Self::ZeroOrOne(None)),
            Self::OneOrMore(_) if new_length == 0 => {
                panic!("Cannot trim SelectionValue::OneOrMore to 0")
            }
            Self::OneOrMore(small_vec) if new_length >= small_vec.len() => None,
            Self::OneOrMore(small_vec) => Some(Self::OneOrMore(
                small_vec[..new_length].iter().copied().collect(),
            )),
            Self::ZeroOrMore(small_vec) if new_length >= small_vec.len() => None,
            Self::ZeroOrMore(small_vec) => Some(Self::ZeroOrMore(
                small_vec[..new_length].iter().copied().collect(),
            )),
        }
    }

    #[must_use]
    pub fn clear(&self) -> Option<Self> {
        match self {
            Self::One(_) => panic!("Cannot clear SelectionValue::One"),
            Self::ZeroOrOne(None) => None,
            Self::ZeroOrOne(Some(_)) => Some(Self::ZeroOrOne(None)),
            Self::OneOrMore(_) => panic!("Cannot clear SelectionValue::OneOrMore"),
            Self::ZeroOrMore(s) if s.is_empty() => None,
            Self::ZeroOrMore(_) => Some(Self::ZeroOrMore(smallvec![])),
        }
    }

    #[must_use]
    pub fn set_selection(&self, iter: impl IntoIterator<Item = Entity>) -> Option<Self> {
        let mut iter = iter.into_iter().unique();
        match self {
            Self::One(x) => {
                let value = iter.next().unwrap();
                assert!(iter.next().is_none());
                if *x == value {
                    None
                } else {
                    Some(Self::One(value))
                }
            }
            Self::ZeroOrOne(x) => {
                let value = iter.next();
                assert!(iter.next().is_none());
                if *x == value {
                    None
                } else {
                    Some(Self::ZeroOrOne(value))
                }
            }
            Self::OneOrMore(x) => {
                let new_vals = iter.collect::<SelectionVec>();
                if new_vals.is_empty() {
                    panic!("SelectionValue::OneOrMore needs to have at least one element");
                } else if x.len() == new_vals.len() && x.iter().all(|v| new_vals.contains(v)) {
                    None
                } else {
                    Some(Self::OneOrMore(new_vals))
                }
            }
            Self::ZeroOrMore(x) => {
                let new_vals = iter.collect::<SelectionVec>();
                if x.len() == new_vals.len() && x.iter().all(|v| new_vals.contains(v)) {
                    None
                } else {
                    Some(Self::ZeroOrMore(new_vals))
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct SelectionParam<'w, 's, F: QueryFilter + 'static = ()> {
    selection_refs: Query<'w, 's, &'static Selection>,
    subset: Query<'w, 's, (Entity, &'static Selection), F>,
    item_values: Query<'w, 's, (Entity, &'static ItemValues)>,
    items_refs: Query<'w, 's, (Entity, &'static ItemsRef)>,
    commands: Commands<'w, 's>,
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

    #[expect(
        clippy::disallowed_methods,
        reason = "passes the `Result` on to the caller instead of panicking"
    )]
    pub fn single(&self) -> Result<(Entity, &SelectionValue), QuerySingleError> {
        self.subset
            .single()
            .map(|(entity, selection)| self.find(entity, selection))
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "passes the `Result` on to the caller instead of panicking"
    )]
    pub fn single_ref(&self) -> Result<Entity, QuerySingleError> {
        self.subset
            .single()
            .map(|(entity, selection)| self.find(entity, selection).0)
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
            let items_ref = if let Ok((_, ItemsRef(sel))) = self.items_refs.get(root_selection_ref)
            {
                *sel
            } else {
                root_selection_ref
            };

            self.item_values.get(items_ref).ok()
        } else {
            None
        }
    }

    pub fn update_selection_single(
        &mut self,
        updater: impl FnOnce(&SelectionValue) -> Option<SelectionValue>,
    ) -> bool {
        let Ok((selection_ref, selection)) = self.single() else {
            return false;
        };
        if let Some(new_selection) = updater(selection) {
            self.do_update_selection(selection_ref, new_selection);
            true
        } else {
            false
        }
    }

    pub fn update_selection(
        &mut self,
        selection_ref: Entity,
        updater: impl FnOnce(&SelectionValue) -> Option<SelectionValue>,
    ) -> bool {
        let (selection_ref, selection) = self.get_with_ref(selection_ref).unwrap();
        if let Some(new_selection) = updater(selection) {
            self.do_update_selection(selection_ref, new_selection);
            true
        } else {
            false
        }
    }

    fn do_update_selection(&mut self, selection_ref: Entity, new_selection: SelectionValue) {
        self.commands
            .entity(selection_ref)
            .insert(new_selection.into_component());
    }
}

impl<F> std::fmt::Debug for SelectionParam<'_, '_, F>
where
    F: QueryFilter + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionMutParam").finish_non_exhaustive()
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

/// T: The type of the item data.
/// D: Additional query data (optional).
/// F: Additional filter data (optional).
#[derive(SystemParam, Debug)]
pub struct ItemParam<'w, 's, T, D = (), F = ()>
where
    T: Send + Sync + 'static,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    items: Query<'w, 's, (Entity, &'static Item<T>, D), F>,
}

impl<T, D, F> ItemParam<'_, '_, T, D, F>
where
    T: Send + Sync + 'static,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    pub fn find_item(
        &self,
        predicate: impl Fn(&Item<T>, &ROQueryItem<D>) -> bool,
    ) -> Option<(Entity, &Item<T>, ROQueryItem<'_, D>)> {
        self.items
            .iter()
            .find(|(_, item, e_value)| predicate(item, e_value))
    }

    pub fn find_item_ref(
        &self,
        predicate: impl Fn(&Item<T>, &ROQueryItem<D>) -> bool,
    ) -> Option<Entity> {
        self.find_item(predicate).map(|(item_ref, _, _)| item_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(index: u32) -> Entity {
        Entity::from_raw(index)
    }

    // One

    #[test]
    fn toggling_the_current_selection_in_one_is_a_no_op() {
        let value = SelectionValue::One(entity(1));
        assert_eq!(value.toggle_selected(entity(1)), None);
    }

    #[test]
    fn toggling_a_different_entity_in_one_replaces_the_selection() {
        let value = SelectionValue::One(entity(1));
        assert_eq!(
            value.toggle_selected(entity(2)),
            Some(SelectionValue::One(entity(2)))
        );
    }

    #[test]
    fn one_is_never_empty() {
        assert!(!SelectionValue::One(entity(1)).is_empty());
        assert_eq!(SelectionValue::One(entity(1)).len(), 1);
    }

    #[test]
    fn one_reports_is_selected_only_for_its_entity() {
        let value = SelectionValue::One(entity(1));
        assert!(value.is_selected(entity(1)));
        assert!(!value.is_selected(entity(2)));
    }

    #[test]
    #[should_panic(expected = "Cannot append to a SelectionValue::One")]
    fn pushing_to_one_panics() {
        let _ = SelectionValue::One(entity(1)).push(entity(2));
    }

    #[test]
    #[should_panic(expected = "Cannot pop a SelectionValue::One")]
    fn popping_one_panics() {
        let _ = SelectionValue::One(entity(1)).pop();
    }

    #[test]
    #[should_panic(expected = "Cannot trim a SelectionValue::One")]
    fn trimming_one_panics() {
        let _ = SelectionValue::One(entity(1)).trim_to_length(0);
    }

    #[test]
    #[should_panic(expected = "Cannot clear SelectionValue::One")]
    fn clearing_one_panics() {
        let _ = SelectionValue::One(entity(1)).clear();
    }

    // ZeroOrOne

    #[test]
    fn zero_or_one_defaults_to_no_selection() {
        assert_eq!(SelectionValue::default(), SelectionValue::ZeroOrOne(None));
    }

    #[test]
    fn selecting_in_an_empty_zero_or_one_sets_the_selection() {
        let value = SelectionValue::ZeroOrOne(None);
        assert_eq!(
            value.toggle_selected(entity(1)),
            Some(SelectionValue::ZeroOrOne(Some(entity(1))))
        );
    }

    #[test]
    fn toggling_the_current_selection_in_zero_or_one_clears_it() {
        let value = SelectionValue::ZeroOrOne(Some(entity(1)));
        assert_eq!(
            value.toggle_selected(entity(1)),
            Some(SelectionValue::ZeroOrOne(None))
        );
    }

    #[test]
    fn toggling_a_different_entity_in_zero_or_one_replaces_the_selection() {
        let value = SelectionValue::ZeroOrOne(Some(entity(1)));
        assert_eq!(
            value.toggle_selected(entity(2)),
            Some(SelectionValue::ZeroOrOne(Some(entity(2))))
        );
    }

    #[test]
    fn zero_or_one_is_empty_only_without_a_selection() {
        assert!(SelectionValue::ZeroOrOne(None).is_empty());
        assert!(!SelectionValue::ZeroOrOne(Some(entity(1))).is_empty());
    }

    #[test]
    fn appending_one_entity_to_an_empty_zero_or_one_selects_it() {
        let value = SelectionValue::ZeroOrOne(None);
        assert_eq!(
            value.append([entity(1)]),
            Some(SelectionValue::ZeroOrOne(Some(entity(1))))
        );
    }

    #[test]
    fn appending_nothing_to_zero_or_one_is_a_no_op() {
        let value = SelectionValue::ZeroOrOne(None);
        assert_eq!(value.append(std::iter::empty()), None);
    }

    #[test]
    #[should_panic(expected = "Cannot append more than one entity to a SelectionValue::ZeroOrOne")]
    fn appending_two_entities_to_zero_or_one_panics() {
        let _ = SelectionValue::ZeroOrOne(None).append([entity(1), entity(2)]);
    }

    #[test]
    #[should_panic(
        expected = "Cannot append to a SelectionValue::ZeroOrOne if it already has a selection"
    )]
    fn appending_to_an_occupied_zero_or_one_panics() {
        let _ = SelectionValue::ZeroOrOne(Some(entity(1))).append([entity(2)]);
    }

    #[test]
    fn popping_an_empty_zero_or_one_is_a_no_op() {
        assert_eq!(SelectionValue::ZeroOrOne(None).pop(), None);
    }

    #[test]
    fn popping_zero_or_one_clears_the_selection() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).pop(),
            Some(SelectionValue::ZeroOrOne(None))
        );
    }

    #[test]
    fn clearing_zero_or_one_removes_the_selection() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).clear(),
            Some(SelectionValue::ZeroOrOne(None))
        );
        assert_eq!(SelectionValue::ZeroOrOne(None).clear(), None);
    }

    #[test]
    fn trimming_zero_or_one_to_zero_clears_it() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).trim_to_length(0),
            Some(SelectionValue::ZeroOrOne(None))
        );
    }

    #[test]
    fn trimming_zero_or_one_to_one_is_a_no_op() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).trim_to_length(1),
            None
        );
    }

    #[test]
    #[should_panic(expected = "Cannot trim SelectionValue::ZeroOrOne to 2")]
    fn trimming_zero_or_one_beyond_one_panics() {
        let _ = SelectionValue::ZeroOrOne(Some(entity(1))).trim_to_length(2);
    }

    // OneOrMore

    #[test]
    fn toggling_the_only_entity_in_one_or_more_is_a_no_op() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1)]);
        assert_eq!(value.toggle_selected(entity(1)), None);
    }

    #[test]
    fn toggling_a_new_entity_into_one_or_more_adds_it() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.toggle_selected(entity(2)),
            Some(SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]))
        );
    }

    #[test]
    fn toggling_a_selected_entity_out_of_one_or_more_removes_it() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(
            value.toggle_selected(entity(2)),
            Some(SelectionValue::OneOrMore(smallvec![entity(1)]))
        );
    }

    #[test]
    fn toggling_an_unselected_entity_into_one_or_more_adds_it() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(
            value.toggle_selected(entity(3)),
            Some(SelectionValue::OneOrMore(smallvec![
                entity(1),
                entity(2),
                entity(3)
            ]))
        );
    }

    #[test]
    fn pushing_a_duplicate_entity_to_one_or_more_is_a_no_op() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1)]);
        assert_eq!(value.push(entity(1)), None);
    }

    #[test]
    fn pushing_a_new_entity_to_one_or_more_appends_it() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.push(entity(2)),
            Some(SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]))
        );
    }

    #[test]
    fn popping_a_single_item_one_or_more_is_a_no_op() {
        assert_eq!(SelectionValue::OneOrMore(smallvec![entity(1)]).pop(), None);
    }

    #[test]
    fn popping_one_or_more_removes_the_last_entity() {
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]).pop(),
            Some(SelectionValue::OneOrMore(smallvec![entity(1)]))
        );
    }

    #[test]
    #[should_panic(expected = "Cannot clear SelectionValue::OneOrMore")]
    fn clearing_one_or_more_panics() {
        let _ = SelectionValue::OneOrMore(smallvec![entity(1)]).clear();
    }

    #[test]
    #[should_panic(expected = "Cannot trim SelectionValue::OneOrMore to 0")]
    fn trimming_one_or_more_to_zero_panics() {
        let _ = SelectionValue::OneOrMore(smallvec![entity(1)]).trim_to_length(0);
    }

    #[test]
    fn trimming_one_or_more_beyond_its_length_is_a_no_op() {
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]).trim_to_length(5),
            None
        );
    }

    #[test]
    fn trimming_one_or_more_truncates_the_trailing_entities() {
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1), entity(2), entity(3)]).trim_to_length(1),
            Some(SelectionValue::OneOrMore(smallvec![entity(1)]))
        );
    }

    #[test]
    #[should_panic(expected = "SelectionValue::OneOrMore needs to have at least one element")]
    fn setting_one_or_more_to_an_empty_selection_panics() {
        let _ = SelectionValue::OneOrMore(smallvec![entity(1)]).set_selection(std::iter::empty());
    }

    #[test]
    fn setting_one_or_more_to_the_same_entities_is_a_no_op() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(value.set_selection([entity(2), entity(1)]), None);
    }

    #[test]
    fn setting_one_or_more_to_different_entities_replaces_the_selection() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.set_selection([entity(2)]),
            Some(SelectionValue::OneOrMore(smallvec![entity(2)]))
        );
    }

    // ZeroOrMore

    #[test]
    fn zero_or_more_is_empty_only_without_selected_entities() {
        assert!(SelectionValue::ZeroOrMore(smallvec![]).is_empty());
        assert!(!SelectionValue::ZeroOrMore(smallvec![entity(1)]).is_empty());
    }

    #[test]
    fn toggling_an_unselected_entity_into_zero_or_more_adds_it() {
        let value = SelectionValue::ZeroOrMore(smallvec![]);
        assert_eq!(
            value.toggle_selected(entity(1)),
            Some(SelectionValue::ZeroOrMore(smallvec![entity(1)]))
        );
    }

    #[test]
    fn toggling_a_selected_entity_out_of_zero_or_more_removes_it() {
        let value = SelectionValue::ZeroOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.toggle_selected(entity(1)),
            Some(SelectionValue::ZeroOrMore(smallvec![]))
        );
    }

    #[test]
    fn popping_an_empty_zero_or_more_is_a_no_op() {
        assert_eq!(SelectionValue::ZeroOrMore(smallvec![]).pop(), None);
    }

    #[test]
    fn popping_zero_or_more_removes_the_last_entity() {
        assert_eq!(
            SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]).pop(),
            Some(SelectionValue::ZeroOrMore(smallvec![entity(1)]))
        );
    }

    #[test]
    fn clearing_an_empty_zero_or_more_is_a_no_op() {
        assert_eq!(SelectionValue::ZeroOrMore(smallvec![]).clear(), None);
    }

    #[test]
    fn clearing_zero_or_more_empties_the_selection() {
        assert_eq!(
            SelectionValue::ZeroOrMore(smallvec![entity(1)]).clear(),
            Some(SelectionValue::ZeroOrMore(smallvec![]))
        );
    }

    #[test]
    fn trimming_zero_or_more_beyond_its_length_is_a_no_op() {
        assert_eq!(
            SelectionValue::ZeroOrMore(smallvec![entity(1)]).trim_to_length(5),
            None
        );
    }

    #[test]
    fn trimming_zero_or_more_truncates_the_trailing_entities() {
        assert_eq!(
            SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]).trim_to_length(0),
            Some(SelectionValue::ZeroOrMore(smallvec![]))
        );
    }

    #[test]
    fn setting_zero_or_more_to_an_empty_selection_empties_it() {
        let value = SelectionValue::ZeroOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.set_selection(std::iter::empty()),
            Some(SelectionValue::ZeroOrMore(smallvec![]))
        );
    }

    #[test]
    fn setting_zero_or_more_to_the_same_entities_is_a_no_op() {
        let value = SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(value.set_selection([entity(2), entity(1)]), None);
    }

    // Cross-variant behaviour

    #[test]
    fn single_returns_the_only_selected_entity() {
        assert_eq!(SelectionValue::One(entity(1)).single(), Some(entity(1)));
        assert_eq!(SelectionValue::ZeroOrOne(None).single(), None);
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1)]).single(),
            Some(entity(1))
        );
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]).single(),
            None
        );
    }

    #[test]
    fn last_returns_the_most_recently_selected_entity() {
        assert_eq!(SelectionValue::ZeroOrMore(smallvec![]).last(), None);
        assert_eq!(
            SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]).last(),
            Some(entity(2))
        );
    }

    #[test]
    fn iter_yields_all_selected_entities_in_order() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(value.iter().collect::<Vec<_>>(), vec![entity(1), entity(2)]);
    }
}
