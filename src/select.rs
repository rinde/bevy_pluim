//! The selection model.
//!
//! A [`Selection`] is an immutable component that either holds a
//! [`SelectionValue`] or refers to the selection on another entity. The items
//! to choose from live in an [`ItemValues`] relationship target, optionally shared
//! through [`ItemsRef`]. Widgets read and change selections through
//! [`SelectionParam`].

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::{QueryData, QueryFilter, QuerySingleError, ROQueryItem};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::scene::{EmptySceneEffect, Scene, bsn};
use derive_more::Deref;
use educe::Educe;
use itertools::{Either, Itertools};
use smallvec::{SmallVec, smallvec};
use strum::EnumIs;

/// Registers the systems and observers of the selection model.
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
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

/// What is selected: either a [`SelectionValue`] on this entity, or a reference to the selection on another entity so that several widgets share one.
///
/// Immutable: change it through [`SelectionParam`], which inserts the new value.
#[derive(Component, Debug, Clone, Eq, PartialEq, Reflect)]
#[component(immutable)]
pub enum Selection {
    /// Use the selection on that entity.
    Ref(Entity),
    /// The selection itself.
    Value(SelectionValue),
}

impl Selection {
    /// A selection that refers to the selection on `selection_ref`.
    #[must_use]
    pub const fn reference(selection_ref: Entity) -> Self {
        Self::Ref(selection_ref)
    }

    /// The value, unless this is a reference.
    #[must_use]
    pub const fn as_selection(&self) -> Option<&SelectionValue> {
        match self {
            Self::Ref(_) => None,
            Self::Value(selection) => Some(selection),
        }
    }

    /// The referenced entity, unless this holds a value.
    #[must_use]
    pub const fn as_ref(&self) -> Option<&Entity> {
        match self {
            Self::Ref(entity) => Some(entity),
            Self::Value(_) => None,
        }
    }

    /// Starts a [`SelectionBuilder`].
    #[must_use]
    pub const fn builder() -> SelectionBuilder {
        SelectionBuilder
    }
}

/// Chooses the kind of [`SelectionValue`] to build.
#[derive(Copy, Clone, Debug)]
pub struct SelectionBuilder;
/// A [`SelectionValue`] chosen; finish it with an item source.
#[derive(Clone, Debug)]
pub struct SelectionBuilderWithValue {
    value: SelectionValue,
}

impl SelectionBuilder {
    /// Exactly one item is selected; the first item until the user picks one.
    #[must_use]
    pub const fn one(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::One(Entity::PLACEHOLDER),
        }
    }

    /// Exactly one item is selected, initially `selection`.
    #[must_use]
    pub const fn one_with(self, selection: Entity) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::One(selection),
        }
    }

    /// At most one item is selected, initially none.
    #[must_use]
    pub const fn zero_or_one(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::ZeroOrOne(None),
        }
    }

    /// Any number of items are selected, initially none.
    #[must_use]
    pub const fn zero_or_more(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::ZeroOrMore(SmallVec::new_const()),
        }
    }

    /// At least one item is selected; the first item until the user picks others.
    #[must_use]
    pub fn one_or_more(self) -> SelectionBuilderWithValue {
        SelectionBuilderWithValue {
            value: SelectionValue::OneOrMore(smallvec![]),
        }
    }
}

impl SelectionBuilderWithValue {
    /// The selection together with `items`, spawned as its [`ItemValues`].
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

    /// [`build_with_items`](Self::build_with_items) as a scene, for `@{..}` in `bsn!`.
    ///
    /// The scene can be spawned more than once, so the items are cloned each time.
    #[must_use]
    pub fn scene_with_items<I, B>(self, items: I) -> impl Scene
    where
        B: Bundle + Clone,
        I: IntoIterator<Item = B>,
    {
        let items: Vec<B> = items.into_iter().collect();
        bsn! {
            template(move |context| {
                context.entity.insert(self.clone().build_with_items(items.clone()));
                Ok(EmptySceneEffect)
            })
        }
    }

    /// [`build_with_items_ref`](Self::build_with_items_ref) as a scene, for `@{..}` in `bsn!`.
    #[must_use]
    pub fn scene_with_items_ref(self, items_ref: Entity) -> impl Scene {
        bsn! {
            template(move |context| {
                context.entity.insert(self.clone().build_with_items_ref(items_ref));
                Ok(EmptySceneEffect)
            })
        }
    }

    /// The selection choosing from the items of `items_ref`, so several selections can share one set of items.
    #[must_use]
    pub fn build_with_items_ref(self, items_ref: Entity) -> impl Bundle {
        (Selection::Value(self.value), ItemsRef(items_ref))
    }
}

fn on_add_selection(
    t: On<Insert<Selection>>,
    selections: Query<(&Selection, Option<&ItemsRef>, Option<&ItemValues>)>,
    item_values_query: Query<&ItemValues>,
    mut commands: Commands,
    mut r: ResMut<SelectionResource>,
) {
    let selection_ref = t.event().event_target();
    let Ok((selection, selection_of, item_values)) = selections.get(selection_ref) else {
        return;
    };
    match (selection, selection_of, item_values) {
        (Selection::Value(SelectionValue::One(one)), items_ref, item_values)
            if items_ref.is_some() != item_values.is_some() && one == &Entity::PLACEHOLDER =>
        {
            let Some(values) = items_ref
                .and_then(|ItemsRef(items_ref)| item_values_query.get(*items_ref).ok())
                .or(item_values)
            else {
                return;
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
            let Some(values) = items_ref
                .and_then(|ItemsRef(items_ref)| item_values_query.get(*items_ref).ok())
                .or(item_values)
            else {
                return;
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
            error!("Selection has no item values {selection_ref}");
        }
        (Selection::Value(_), Some(_), Some(_)) => {
            error!("Ambiguous selection config for {selection_ref}");
        }
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
            let Some(selection_value) = selections.get(selection_ref) else {
                continue;
            };
            let Some(first_item) = values.iter().next() else {
                continue;
            };
            let new_selection_value = match selection_value {
                SelectionValue::One(x) if *x == Entity::PLACEHOLDER => {
                    Some(SelectionValue::One(first_item))
                }
                SelectionValue::OneOrMore(selected) if selected.is_empty() => {
                    Some(SelectionValue::OneOrMore(smallvec![first_item]))
                }
                SelectionValue::One(_)
                | SelectionValue::OneOrMore(_)
                | SelectionValue::ZeroOrOne(_)
                | SelectionValue::ZeroOrMore(_) => None,
            };
            if let Some(new_selection_value) = new_selection_value {
                commands
                    .entity(target_ref)
                    .insert(Selection::Value(new_selection_value));
            }
        }
    }
}

const DANGLING_SELECTION_REF: &str = "a `Selection::Ref` that points to a `Selection::Value`";

type SelectionVec = SmallVec<[Entity; 4]>;

/// The selected items, and how many may be selected.
///
/// Methods that change it return the new value, or `None` when the change is not allowed for the variant or would not change anything.
#[derive(Debug, Clone, Educe, Eq, PartialEq, EnumIs, Reflect)]
#[educe(Default)]
pub enum SelectionValue {
    /// Exactly one item.
    One(Entity),
    /// At most one item.
    #[educe(Default)]
    ZeroOrOne(Option<Entity>),
    /// At least one item.
    OneOrMore(SelectionVec),
    /// Any number of items.
    ZeroOrMore(SelectionVec),
}

impl SelectionValue {
    /// This value as a [`Selection`] component.
    #[must_use]
    pub const fn into_component(self) -> Selection {
        Selection::Value(self)
    }

    /// Whether more than one item can be selected.
    #[must_use]
    pub const fn allow_multiple(&self) -> bool {
        match self {
            Self::One(_) | Self::ZeroOrOne(_) => false,
            Self::OneOrMore(_) | Self::ZeroOrMore(_) => true,
        }
    }

    /// Whether `item` is selected.
    #[must_use]
    pub fn is_selected(&self, item: Entity) -> bool {
        match self {
            Self::One(selected) => selected == &item,
            Self::ZeroOrOne(selected) => selected.is_some_and(|s| s == item),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.contains(&item),
        }
    }

    /// The selected items, in selection order.
    pub fn iter(&self) -> impl Iterator<Item = Entity> {
        match self {
            Self::One(selected) => Either::Left(Some(*selected).into_iter()),
            Self::ZeroOrOne(selected) => Either::Left((*selected).into_iter()),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => {
                Either::Right(items.iter().copied())
            }
        }
    }

    /// Whether no item is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::One(_) => false,
            Self::ZeroOrOne(selected) => selected.is_none(),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.is_empty(),
        }
    }

    /// The number of selected items.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::ZeroOrOne(selected) => usize::from(selected.is_some()),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.len(),
        }
    }

    /// The selected item, if exactly one item is selected.
    #[must_use]
    pub fn single(&self) -> Option<Entity> {
        match self {
            Self::One(selected) => Some(*selected),
            Self::ZeroOrOne(selected) => *selected,
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => match items.as_slice() {
                [only] => Some(*only),
                _ => None,
            },
        }
    }

    /// The item that was selected last.
    #[must_use]
    pub fn last(&self) -> Option<Entity> {
        match self {
            Self::One(selected) => Some(*selected),
            Self::ZeroOrOne(selected) => *selected,
            Self::OneOrMore(items) | Self::ZeroOrMore(items) if items.is_empty() => None,
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.last().copied(),
        }
    }

    /// Selects `new_selection`, or deselects it if it is selected.
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
                if let [only] = selected.as_slice() {
                    if *only == new_selection {
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

    /// Adds one item to the selection.
    #[must_use]
    pub fn push(&self, new_selection: Entity) -> Option<Self> {
        self.append([new_selection])
    }

    /// Adds items to the selection, skipping the ones already selected.
    #[must_use]
    pub fn append(&self, iter: impl IntoIterator<Item = Entity>) -> Option<Self> {
        match self {
            // A `One`, or a `ZeroOrOne` that already has a selection, has no room.
            Self::One(_) | Self::ZeroOrOne(Some(_)) => None,
            Self::ZeroOrOne(None) => {
                let res = iter.into_iter().unique().collect::<SelectionVec>();
                match res.as_slice() {
                    [only] => Some(Self::ZeroOrOne(Some(*only))),
                    // nothing to add, or more than a `ZeroOrOne` can hold
                    _ => None,
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

    /// Removes the item that was selected last.
    #[must_use]
    pub fn pop(&self) -> Option<Self> {
        match self {
            Self::One(_) | Self::ZeroOrOne(None) => None,
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

    /// Keeps only the first `new_length` selected items.
    #[must_use]
    pub fn trim_to_length(&self, new_length: usize) -> Option<Self> {
        match self {
            Self::One(_) => None,
            Self::OneOrMore(_) if new_length == 0 => None,
            Self::ZeroOrOne(_) if new_length >= 1 => None,
            Self::ZeroOrOne(_) => Some(Self::ZeroOrOne(None)),
            Self::OneOrMore(small_vec) | Self::ZeroOrMore(small_vec)
                if new_length >= small_vec.len() =>
            {
                None
            }
            Self::OneOrMore(small_vec) => Some(Self::OneOrMore(
                small_vec.iter().take(new_length).copied().collect(),
            )),
            Self::ZeroOrMore(small_vec) => Some(Self::ZeroOrMore(
                small_vec.iter().take(new_length).copied().collect(),
            )),
        }
    }

    /// Deselects everything.
    #[must_use]
    pub fn clear(&self) -> Option<Self> {
        match self {
            Self::One(_) | Self::OneOrMore(_) | Self::ZeroOrOne(None) => None,
            Self::ZeroOrOne(Some(_)) => Some(Self::ZeroOrOne(None)),
            Self::ZeroOrMore(s) if s.is_empty() => None,
            Self::ZeroOrMore(_) => Some(Self::ZeroOrMore(smallvec![])),
        }
    }

    /// Replaces the selection with `iter`.
    #[must_use]
    pub fn set_selection(&self, iter: impl IntoIterator<Item = Entity>) -> Option<Self> {
        let mut iter = iter.into_iter().unique();
        match self {
            Self::One(x) => {
                let value = iter.next()?;
                if iter.next().is_some() || *x == value {
                    None
                } else {
                    Some(Self::One(value))
                }
            }
            Self::ZeroOrOne(x) => {
                let value = iter.next();
                if iter.next().is_some() || *x == value {
                    None
                } else {
                    Some(Self::ZeroOrOne(value))
                }
            }
            Self::OneOrMore(x) => {
                let new_vals = iter.collect::<SelectionVec>();
                if new_vals.is_empty() {
                    // a `OneOrMore` must keep at least one element
                    None
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

/// System parameter to read and change selections, following any [`Selection::Ref`].
///
/// `F` narrows down the selections that [`Self::iter`] and [`Self::single`] see.
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
    /// The selections matching the filter, with the entity holding each value.
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &SelectionValue)> {
        self.subset
            .iter()
            .filter_map(|(entity, selection)| self.find(entity, selection))
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "passes the `Result` on to the caller instead of panicking"
    )]
    /// The one selection in the query, resolved through any `Selection::Ref`.
    ///
    /// # Errors
    ///
    /// If the query does not match exactly one entity, or that entity's
    /// `Selection::Ref` does not point to a value.
    pub fn single(&self) -> Result<(Entity, &SelectionValue), QuerySingleError> {
        self.subset.single().and_then(|(entity, selection)| {
            self.find(entity, selection)
                .ok_or(QuerySingleError::NoEntities(DebugName::borrowed(
                    DANGLING_SELECTION_REF,
                )))
        })
    }

    /// The entity that holds the value of [`Self::single`].
    ///
    /// # Errors
    ///
    /// See [`Self::single`].
    pub fn single_ref(&self) -> Result<Entity, QuerySingleError> {
        self.single().map(|(selection_ref, _)| selection_ref)
    }

    /// The value of the selection on `entity`, following a reference.
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<&SelectionValue> {
        self.get_with_ref(entity).map(|(_, s)| s)
    }

    /// The entity holding the value of the selection on `entity`.
    #[must_use]
    pub fn get_ref(&self, entity: Entity) -> Option<Entity> {
        self.get_with_ref(entity).map(|(e, _)| e)
    }

    /// [`Self::get_ref`] and [`Self::get`] in one.
    #[must_use]
    pub fn get_with_ref(&self, selection_ref: Entity) -> Option<(Entity, &SelectionValue)> {
        let selection = self.selection_refs.get(selection_ref).ok()?;
        self.find(selection_ref, selection)
    }

    /// Resolves `selection` to the entity holding the value and the value
    /// itself. `None` if a `Selection::Ref` does not point to a
    /// `Selection::Value`.
    fn find<'a>(
        &'a self,
        entity: Entity,
        selection: &'a Selection,
    ) -> Option<(Entity, &'a SelectionValue)> {
        match selection {
            Selection::Ref(entity_ref) => {
                let value = self.selection_refs.get(*entity_ref).ok()?.as_selection()?;
                Some((*entity_ref, value))
            }
            Selection::Value(selection) => Some((entity, selection)),
        }
    }

    /// The items that the selection on `selection_ref` chooses from, with the entity holding them.
    #[must_use]
    pub fn get_item_values(&self, selection_ref: Entity) -> Option<(Entity, &ItemValues)> {
        let root_selection_ref = self.get_ref(selection_ref)?;
        let items_ref = self
            .items_refs
            .get(root_selection_ref)
            .map_or(root_selection_ref, |(_, ItemsRef(sel))| *sel);

        self.item_values.get(items_ref).ok()
    }

    /// Changes the one selection matching the filter.
    ///
    /// `updater` returns the new value, or `None` to leave it as it is. Returns whether the selection changed.
    pub fn update_selection_single(
        &mut self,
        updater: impl FnOnce(&SelectionValue) -> Option<SelectionValue>,
    ) -> bool {
        let Ok((selection_ref, selection)) = self.single() else {
            return false;
        };
        let Some(new_selection) = updater(selection) else {
            return false;
        };
        self.do_update_selection(selection_ref, new_selection);
        true
    }

    /// Changes the selection on `selection_ref`.
    ///
    /// `updater` returns the new value, or `None` to leave it as it is. Returns whether the selection changed.
    pub fn update_selection(
        &mut self,
        selection_ref: Entity,
        updater: impl FnOnce(&SelectionValue) -> Option<SelectionValue>,
    ) -> bool {
        let Some((selection_ref, selection)) = self.get_with_ref(selection_ref) else {
            return false;
        };
        let Some(new_selection) = updater(selection) else {
            return false;
        };
        self.do_update_selection(selection_ref, new_selection);
        true
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

/// Marks an entity as an item of the [`ItemValues`] on the given entity.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[relationship(relationship_target = ItemValues)]
pub struct ItemOf(pub Entity);

/// The items that a selection chooses from.
#[derive(Component, Debug, Default, Clone, Reflect, Deref)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[require(Name::new("Items"))]
pub struct ItemValues(Vec<Entity>);

/// Spawns items into an [`ItemValues`].
pub type ItemSpawner<'w> = RelatedSpawner<'w, ItemOf>;

/// An item that can be selected, carrying `data`.
#[derive(Component, Debug, Clone, Reflect)]
#[require(Name::new("Item"))]
pub struct Item<T> {
    /// What the item shows or stands for.
    pub data: T,
}

impl Item<String> {
    /// A text item.
    pub fn new(label: impl Into<String>) -> Self {
        Self { data: label.into() }
    }
}

impl<T: Into<String>> From<T> for Item<String> {
    fn from(value: T) -> Self {
        Self { data: value.into() }
    }
}

/// Triggered on an [`ItemValues`] entity when its items change; holds the new items.
#[derive(EntityEvent, Debug, Clone)]
pub struct ItemsChanged {
    /// The [`ItemValues`] entity whose items changed.
    pub entity: Entity,
    /// The new items.
    pub items: ItemValues,
}

/// Records the observer a widget put on its [`ItemValues`] entity to hear [`ItemsChanged`], on
/// the widget itself, so the observer goes when the widget does (removing this component
/// despawns it) and goes with the values entity when that is despawned first.
#[derive(Component, Debug)]
#[component(on_remove = despawn_items_observer)]
pub(crate) struct ItemsObserver {
    values: Entity,
    observer: Entity,
}

fn despawn_items_observer(mut world: DeferredWorld, context: HookContext) {
    let Some(watch) = world.get::<ItemsObserver>(context.entity) else {
        return;
    };
    let observer = watch.observer;
    world.commands().entity(observer).try_despawn();
}

/// Has `on_changed(commands, widget, items)` called when the items of `values` change, once
/// per widget and values entity: returns `false`, doing nothing, when `widget` already
/// watches `values`. Watching other values replaces the earlier observer.
pub(crate) fn watch_items(
    commands: &mut Commands,
    watches: &Query<&ItemsObserver>,
    observers: &Query<(), With<Observer>>,
    widget: Entity,
    values: Entity,
    on_changed: fn(&mut Commands, Entity, Vec<Entity>),
) -> bool {
    if let Ok(old) = watches.get(widget) {
        if old.values == values && observers.contains(old.observer) {
            return false;
        }
        commands.entity(old.observer).try_despawn();
    }
    let observer = commands
        .spawn(
            Observer::new(move |t: On<ItemsChanged>, mut commands: Commands| {
                on_changed(&mut commands, widget, t.event().items.iter().collect());
            })
            .with_entity(values),
        )
        .id();
    commands
        .entity(widget)
        .insert(ItemsObserver { values, observer });
    true
}

fn items_change(q: Query<(Entity, &ItemValues), Changed<ItemValues>>, mut commands: Commands) {
    for (items_ref, items) in q {
        commands.trigger(ItemsChanged {
            entity: items_ref,
            items: items.clone(),
        });
    }
}

// a selection value should have this component, should not be added to any
// other component
/// Lets a selection choose from the [`ItemValues`] on another entity.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[relationship(relationship_target = ItemSelections)]
pub struct ItemsRef(pub Entity);

/// The selections that choose from these items through [`ItemsRef`].
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
    /// The first item, with its extra query data, for which `predicate` holds.
    pub fn find_item(
        &self,
        predicate: impl Fn(&Item<T>, &ROQueryItem<D>) -> bool,
    ) -> Option<(Entity, &Item<T>, ROQueryItem<'_, '_, D>)> {
        self.items
            .iter()
            .find(|(_, item, e_value)| predicate(item, e_value))
    }

    /// The entity of the first item for which `predicate` holds.
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
        Entity::from_raw_u32(index).unwrap_or(Entity::PLACEHOLDER)
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
    fn pushing_to_one_is_refused() {
        assert_eq!(SelectionValue::One(entity(1)).push(entity(2)), None);
    }

    #[test]
    fn popping_one_is_refused() {
        assert_eq!(SelectionValue::One(entity(1)).pop(), None);
    }

    #[test]
    fn trimming_one_is_refused() {
        assert_eq!(SelectionValue::One(entity(1)).trim_to_length(0), None);
    }

    #[test]
    fn clearing_one_is_refused() {
        assert_eq!(SelectionValue::One(entity(1)).clear(), None);
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
    fn appending_two_entities_to_zero_or_one_is_refused() {
        assert_eq!(
            SelectionValue::ZeroOrOne(None).append([entity(1), entity(2)]),
            None
        );
    }

    #[test]
    fn appending_to_an_occupied_zero_or_one_is_refused() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).append([entity(2)]),
            None
        );
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
    fn trimming_zero_or_one_beyond_one_is_refused() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).trim_to_length(2),
            None
        );
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
    fn clearing_one_or_more_is_refused() {
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1)]).clear(),
            None
        );
    }

    #[test]
    fn trimming_one_or_more_to_zero_is_refused() {
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1)]).trim_to_length(0),
            None
        );
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
    fn setting_one_or_more_to_an_empty_selection_is_refused() {
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![entity(1)]).set_selection(std::iter::empty()),
            None
        );
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
