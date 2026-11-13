//! The selection model.
//!
//! A [`Selection`] is an immutable component that either holds a
//! [`SelectionValue`] or refers to the selection on another entity. The items
//! to choose from live in an [`ItemValues`] relationship target, optionally shared
//! through [`ItemsRef`]. Widgets read and change selections through
//! [`SelectionParam`].

use std::sync::Arc;

use bevy::ecs::entity::EntityHashSet;
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
            .add_observer(on_add_selection)
            .add_observer(repair_pending_selections)
            .add_observer(on_replace_items_ref)
            .add_observer(forget_removed_selection)
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
    ///
    /// Exactly one hop is followed: the target must hold a
    /// [`Selection::Value`]. A target that is missing, has no `Selection`,
    /// is this very entity, or is itself a `Ref` (a chain) resolves to
    /// nothing, and [`SelectionParam`] logs an `error!`; point a `Ref` at the
    /// entity that holds the value instead.
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
            let items_entity = items_ref.map_or(selection_ref, |ItemsRef(items_ref)| *items_ref);
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
                r.add_pending(PendingSelection {
                    selection: selection_ref,
                    items: items_entity,
                });
            }
        }
        (Selection::Value(SelectionValue::OneOrMore(selected)), items_ref, item_values)
            if items_ref.is_some() != item_values.is_some() && selected.is_empty() =>
        {
            let items_entity = items_ref.map_or(selection_ref, |ItemsRef(items_ref)| *items_ref);
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
                r.add_pending(PendingSelection {
                    selection: selection_ref,
                    items: items_entity,
                });
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

/// A selection waiting for the first item of `items`.
#[derive(Debug, Clone, Copy)]
struct PendingSelection {
    selection: Entity,
    items: Entity,
}

/// The selections waiting for their first item.
///
/// Nothing polls it: [`repair_pending_selections`] runs on the [`ItemsChanged`] of an entry's
/// items and [`forget_removed_selection`] drops an entry when its selection loses its
/// [`Selection`] or is despawned.
#[derive(Resource, Default)]
struct SelectionResource {
    pending: Vec<PendingSelection>,
}

impl SelectionResource {
    /// Remembers `pending`, once per selection: a selection waits for one set of items only.
    fn add_pending(&mut self, pending: PendingSelection) {
        self.pending.retain(|p| p.selection != pending.selection);
        self.pending.push(pending);
    }
}

/// Starts a refresh of the selection on an entity whose [`ItemsRef`] was replaced.
///
/// `Discard` (before the old value goes) runs before the new value is in, so the refresh is queued to run after it; a removed
/// [`ItemsRef`] has nothing to refresh to.
fn on_replace_items_ref(t: On<Discard<ItemsRef>>, mut commands: Commands) {
    commands.run_system_cached_with(refresh_selection_items, t.event_target());
}

/// Points the selection on `entity` at the items of its (new) [`ItemsRef`].
///
/// Selected items that are not items of the new source are dropped, as a selection of one item
/// goes back to its first item (or to pending, when the source has none yet, see
/// [`SelectionValue::is_pending`]). The selection is inserted again whatever the result, so the
/// widgets that follow it draw the new items.
fn refresh_selection_items(
    In(entity): In<Entity>,
    selections: Query<(&Selection, &ItemsRef)>,
    item_values: Query<&ItemValues>,
    mut r: ResMut<SelectionResource>,
    mut commands: Commands,
) {
    let Ok((Selection::Value(value), ItemsRef(items_entity))) = selections.get(entity) else {
        return;
    };
    let items: &[Entity] = item_values.get(*items_entity).map_or(&[], |v| &v.0);
    let first = items.first().copied();
    let kept = |selected: &Entity| items.contains(selected);
    let new_value = match value {
        SelectionValue::One(x) if items.contains(x) => SelectionValue::One(*x),
        SelectionValue::One(_) => SelectionValue::One(first.unwrap_or(Entity::PLACEHOLDER)),
        SelectionValue::ZeroOrOne(x) => SelectionValue::ZeroOrOne(x.filter(kept)),
        SelectionValue::OneOrMore(selected) => {
            let mut kept_items: SelectionVec = selected.iter().copied().filter(kept).collect();
            if kept_items.is_empty() {
                kept_items.extend(first);
            }
            SelectionValue::OneOrMore(kept_items)
        }
        SelectionValue::ZeroOrMore(selected) => {
            SelectionValue::ZeroOrMore(selected.iter().copied().filter(kept).collect())
        }
    };
    if new_value.is_pending() {
        r.add_pending(PendingSelection {
            selection: entity,
            items: *items_entity,
        });
    }
    commands.entity(entity).insert(new_value.into_component());
}

/// Gives the pending selections of the changed items their first item.
fn repair_pending_selections(
    changed: On<ItemsChanged>,
    mut r: ResMut<SelectionResource>,
    selections: Query<&Selection>,
    mut commands: Commands,
) {
    let event = changed.event();
    let Some(first_item) = event.items.first().copied() else {
        return;
    };
    r.pending.retain(|pending| {
        if pending.items != event.entity {
            return true;
        }
        let new_value = match selections.get(pending.selection) {
            Ok(Selection::Value(SelectionValue::One(x))) if *x == Entity::PLACEHOLDER => {
                Some(SelectionValue::One(first_item))
            }
            Ok(Selection::Value(SelectionValue::OneOrMore(selected))) if selected.is_empty() => {
                Some(SelectionValue::OneOrMore(smallvec![first_item]))
            }
            _ => None,
        };
        if let Some(new_value) = new_value {
            commands
                .entity(pending.selection)
                .insert(Selection::Value(new_value));
        }
        false
    });
}

/// Drops the pending entry of a selection whose [`Selection`] is gone (also when it is despawned).
fn forget_removed_selection(removed: On<Remove<Selection>>, mut r: ResMut<SelectionResource>) {
    let entity = removed.event_target();
    r.pending.retain(|pending| pending.selection != entity);
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
    ///
    /// `One(Entity::PLACEHOLDER)` is the *pending* state of a selection whose item source has no
    /// first item yet (see [`SelectionValue::is_pending`]); it selects nothing.
    One(Entity),
    /// At most one item.
    #[educe(Default)]
    ZeroOrOne(Option<Entity>),
    /// At least one item.
    ///
    /// An empty `OneOrMore` is the *pending* state of a selection whose item source has no first
    /// item yet (see [`SelectionValue::is_pending`]); it selects nothing.
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

    /// Whether this is a mandatory selection ([`SelectionValue::One`] or
    /// [`SelectionValue::OneOrMore`]) still waiting for its first item: `One(Entity::PLACEHOLDER)`
    /// or an empty `OneOrMore`.
    ///
    /// Pending is not an error: it is how a selection over items that are spawned later starts.
    /// A pending selection has no selected items (`iter` yields nothing, `len` is 0, `is_empty`
    /// is true, `single` and `last` are `None`, nothing `is_selected`), and consumers treat it
    /// as empty and never look up a view for the placeholder. The first `ItemsChanged` that
    /// brings an item selects it, keeping the variant, and the normal update methods never
    /// return to pending: `One` and `OneOrMore` refuse to clear an initialised selection.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        match self {
            Self::One(selected) => *selected == Entity::PLACEHOLDER,
            Self::OneOrMore(items) => items.is_empty(),
            Self::ZeroOrOne(_) | Self::ZeroOrMore(_) => false,
        }
    }

    /// Whether `item` is selected.
    #[must_use]
    pub fn is_selected(&self, item: Entity) -> bool {
        match self {
            Self::One(selected) => *selected != Entity::PLACEHOLDER && selected == &item,
            Self::ZeroOrOne(selected) => selected.is_some_and(|s| s == item),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.contains(&item),
        }
    }

    /// The selected items, in selection order.
    pub fn iter(&self) -> impl Iterator<Item = Entity> {
        match self {
            Self::One(selected) => Either::Left(
                Some(*selected)
                    .filter(|selected| *selected != Entity::PLACEHOLDER)
                    .into_iter(),
            ),
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
            Self::One(selected) => *selected == Entity::PLACEHOLDER,
            Self::ZeroOrOne(selected) => selected.is_none(),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.is_empty(),
        }
    }

    /// The number of selected items.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::One(selected) => usize::from(*selected != Entity::PLACEHOLDER),
            Self::ZeroOrOne(selected) => usize::from(selected.is_some()),
            Self::OneOrMore(items) | Self::ZeroOrMore(items) => items.len(),
        }
    }

    /// The selected item, if exactly one item is selected.
    #[must_use]
    pub fn single(&self) -> Option<Entity> {
        match self {
            Self::One(selected) => Some(*selected).filter(|s| *s != Entity::PLACEHOLDER),
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
            Self::One(selected) => Some(*selected).filter(|s| *s != Entity::PLACEHOLDER),
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

    /// Deselects everything, where the variant allows an empty selection.
    ///
    /// `One` and `OneOrMore` must keep an item, so they refuse (`None`), as does a
    /// selection that is already empty.
    #[must_use]
    pub fn clear(&self) -> Option<Self> {
        match self {
            Self::One(_) | Self::OneOrMore(_) | Self::ZeroOrOne(None) => None,
            Self::ZeroOrOne(Some(_)) => Some(Self::ZeroOrOne(None)),
            Self::ZeroOrMore(s) if s.is_empty() => None,
            Self::ZeroOrMore(_) => Some(Self::ZeroOrMore(smallvec![])),
        }
    }

    /// Replaces the selection with `iter`, keeping its order (duplicates dropped).
    ///
    /// Order is significant for `OneOrMore` and `ZeroOrMore`, because [`Self::iter`]
    /// yields selection order and [`Self::trim_to_length`] keeps the first items: the
    /// same items in another order are a change and give `Some`. `None` means no
    /// change (the same items in the same order) or a refused change.
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
                } else if x.as_slice() == new_vals.as_slice() {
                    None
                } else {
                    Some(Self::OneOrMore(new_vals))
                }
            }
            Self::ZeroOrMore(x) => {
                let new_vals = iter.collect::<SelectionVec>();
                if x.as_slice() == new_vals.as_slice() {
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
    /// `Selection::Value` (logged as an `error!`).
    fn find<'a>(
        &'a self,
        entity: Entity,
        selection: &'a Selection,
    ) -> Option<(Entity, &'a SelectionValue)> {
        match selection {
            Selection::Ref(entity_ref) => {
                let value = self
                    .selection_refs
                    .get(*entity_ref)
                    .ok()
                    .and_then(Selection::as_selection);
                if value.is_none() {
                    error!(
                        "Selection::Ref on {entity} points to {entity_ref}, which is {}; \
                         a Ref must point to an entity holding a Selection::Value",
                        if *entity_ref == entity {
                            "itself"
                        } else if self.selection_refs.contains(*entity_ref) {
                            "another Selection::Ref (chains are not followed)"
                        } else {
                            "missing or without a Selection"
                        }
                    );
                }
                Some((*entity_ref, value?))
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

    /// The items that the selection on `selection_ref` chooses from, with the entity they are
    /// on. A source without [`ItemValues`] (no item added yet, or the last one removed) has
    /// no items, so the slice is empty then where [`Self::get_item_values`] is `None`; `None`
    /// only when the selection does not resolve.
    pub(crate) fn get_items(&self, selection_ref: Entity) -> Option<(Entity, &[Entity])> {
        let (_, items_entity) = self.sources_of(selection_ref)?;
        let items = self
            .item_values
            .get(items_entity)
            .map_or(&[][..], |(_, values)| values.as_slice());
        Some((items_entity, items))
    }

    /// The entity holding the value of the selection on `widget` and the entity its items are
    /// on, whether or not that one has any [`ItemValues`] yet.
    pub(crate) fn sources_of(&self, widget: Entity) -> Option<(Entity, Entity)> {
        let root = self.get_ref(widget)?;
        let items = self
            .items_refs
            .get(root)
            .map_or(root, |(_, ItemsRef(sel))| *sel);
        Some((root, items))
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
///
/// Bevy's relationship input: insert it (again) to add the item to other items, which updates the
/// [`ItemValues`] of both the old and the new entity through Bevy's relationship hooks. The
/// `ItemValues` collection is derived and has no public mutable access. The entity is not
/// checked: an `ItemOf` that points to a missing entity is Bevy's to report.
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

/// What changed since the system last ran that a widget `W` over items of type `T` builds its
/// children from, resolved to the widgets it affects.
///
/// [`Self::affected`] is empty in a frame where nothing changed, so a system that builds a child
/// per item does no per-item work then. In a frame with a change it names only the widgets
/// whose own entity, selection entity (a [`Selection::Ref`] followed) or items entity
/// ([`ItemsRef`] followed) was touched, so a change to one source leaves independent widgets
/// alone while every widget sharing the source is still named. A widget that changed itself is
/// always named.
#[derive(SystemParam)]
pub(crate) struct StructureChanges<'w, 's, W: Component, T: Send + Sync + 'static> {
    widgets: Query<'w, 's, Entity, Changed<W>>,
    all_widgets: Query<'w, 's, Entity, With<W>>,
    values: Query<'w, 's, Entity, Changed<ItemValues>>,
    items: Query<'w, 's, &'static ItemOf, Changed<Item<T>>>,
    selections: Query<'w, 's, Entity, Changed<Selection>>,
    items_refs: Query<'w, 's, Entity, Changed<ItemsRef>>,
    item_owners: Query<'w, 's, &'static ItemOf>,
    removed_values: RemovedComponents<'w, 's, ItemValues>,
    removed_items: RemovedComponents<'w, 's, Item<T>>,
    removed_item_of: RemovedComponents<'w, 's, ItemOf>,
}

impl<W: Component, T: Send + Sync + 'static> StructureChanges<'_, '_, W, T> {
    /// The widgets that need to look at their items again this frame.
    pub(crate) fn affected(&mut self, selections: &SelectionParam) -> Vec<Entity> {
        // Both readers are drained, so neither keeps the result non-empty in the next frame.
        // A despawned or moved item changed the `ItemValues` of its owner, which is seen
        // below; only an item that lost its `Item<T>` but kept its owner needs the lookup.
        let removed: Vec<Entity> = self
            .removed_items
            .read()
            .chain(self.removed_item_of.read())
            .collect();

        // Entities whose items changed, matched against a widget's items entity, and entities
        // whose selection changed, matched against its selection entity. `ItemValues` goes
        // when its last item does, which `Changed` does not see.
        let mut item_sources: EntityHashSet = EntityHashSet::default();
        item_sources.extend(self.values.iter());
        item_sources.extend(self.removed_values.read());
        item_sources.extend(self.items.iter().map(|owner| owner.0));
        item_sources.extend(
            removed
                .into_iter()
                .filter_map(|item| self.item_owners.get(item).ok().map(|owner| owner.0)),
        );
        let selection_sources: EntityHashSet = self.selections.iter().collect();
        // A widget whose own `ItemsRef` was replaced is named through its own entity.
        let replaced: EntityHashSet = self.items_refs.iter().collect();

        let mut affected: EntityHashSet = self.widgets.iter().collect();
        if !(item_sources.is_empty() && selection_sources.is_empty() && replaced.is_empty()) {
            for widget in &self.all_widgets {
                if affected.contains(&widget) {
                    continue;
                }
                let named = item_sources.contains(&widget)
                    || selection_sources.contains(&widget)
                    || replaced.contains(&widget)
                    || selections.sources_of(widget).is_some_and(|(root, items)| {
                        selection_sources.contains(&root) || item_sources.contains(&items)
                    });
                if named {
                    affected.insert(widget);
                }
            }
        }
        affected.into_iter().collect()
    }
}

/// Triggered on an [`ItemValues`] entity when its items change; holds the new items.
#[derive(EntityEvent, Debug, Clone)]
pub struct ItemsChanged {
    /// The [`ItemValues`] entity whose items changed.
    pub entity: Entity,
    /// The new items, one snapshot shared by every observer of the event: cloning the
    /// `Arc` copies no entities.
    pub items: Arc<[Entity]>,
}

/// Records the observer a widget put on its [`ItemValues`] entity to hear [`ItemsChanged`], on
/// the widget itself, so the observer goes when the widget does (removing or replacing this
/// component despawns it, so two watches set up before the first is applied leave one
/// observer) and goes with the values entity when that is despawned first.
#[derive(Component, Debug)]
#[component(on_discard = despawn_items_observer)]
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

/// The `children` that show `items`, in the order of `items` (the first child of an item), then
/// every other child in its place order. `item_of` says which item a child shows, `None` for a
/// child that is not a view. Linear: one map of the first child per item, one set of the
/// children placed.
pub(crate) fn items_in_order(
    children: impl IntoIterator<Item = Entity> + Clone,
    items: &[Entity],
    item_of: impl Fn(Entity) -> Option<Entity>,
) -> Vec<Entity> {
    let mut first_of = bevy::platform::collections::HashMap::new();
    for child in children.clone() {
        if let Some(item) = item_of(child) {
            first_of.entry(item).or_insert(child);
        }
    }
    let mut placed = bevy::platform::collections::HashSet::new();
    let mut wanted: Vec<Entity> = items
        .iter()
        .filter_map(|item| first_of.get(item).copied())
        .filter(|child| placed.insert(*child))
        .collect();
    wanted.extend(children.into_iter().filter(|c| !placed.contains(c)));
    wanted
}

/// What to do with the views a widget owns when its wanted views change: `slots` has one entry
/// per wanted view, the existing view to keep for it or `None` for one to spawn, and `stale` the
/// existing views to despawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ViewPlan {
    pub(crate) slots: Vec<Option<Entity>>,
    pub(crate) stale: Vec<Entity>,
}

/// Matches the `existing` views, in child order, with the `wanted` keys: a wanted key takes the
/// first unused existing view of that key, so views of unchanged items keep their entity.
/// An existing view with a `None` key is never kept (it is of the wrong kind), and so is a second
/// view of one key when only one is wanted. Linear: one map of the views per key.
pub(crate) fn plan_views<K: Eq + std::hash::Hash>(
    existing: impl IntoIterator<Item = (Entity, Option<K>)>,
    wanted: &[K],
) -> ViewPlan {
    let mut pool: bevy::platform::collections::HashMap<K, std::collections::VecDeque<Entity>> =
        bevy::platform::collections::HashMap::new();
    let mut stale = Vec::new();
    for (view, key) in existing {
        match key {
            Some(key) => pool.entry(key).or_default().push_back(view),
            None => stale.push(view),
        }
    }
    let slots = wanted
        .iter()
        .map(|key| {
            pool.get_mut(key)
                .and_then(std::collections::VecDeque::pop_front)
        })
        .collect();
    stale.extend(pool.into_values().flatten());
    ViewPlan { slots, stale }
}

/// The children a widget must be given so that its `views` (the kept ones and the entities just
/// spawned, in the order wanted) are in that order as one run, at the place of its first current
/// view and after the children that are not views, which keep their order. `None` when
/// despawning the `stale` views and appending the new ones, which is what the commands do alone,
/// already gives that order.
pub(crate) fn order_views(
    children: &[Entity],
    is_view: impl Fn(Entity) -> bool,
    views: &[Entity],
    stale: &[Entity],
) -> Option<Vec<Entity>> {
    let current: bevy::platform::collections::HashSet<Entity> = children.iter().copied().collect();
    let mut ordered = Vec::with_capacity(children.len() + views.len());
    let mut placed = false;
    for child in children {
        if is_view(*child) {
            if !placed {
                ordered.extend_from_slice(views);
                placed = true;
            }
        } else {
            ordered.push(*child);
        }
    }
    if !placed {
        ordered.extend_from_slice(views);
    }
    let stale: bevy::platform::collections::HashSet<Entity> = stale.iter().copied().collect();
    let without_stale = children
        .iter()
        .filter(|child| !stale.contains(*child))
        .copied()
        .chain(
            views
                .iter()
                .filter(|view| !current.contains(*view))
                .copied(),
        );
    (!ordered.iter().copied().eq(without_stale)).then_some(ordered)
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
    on_changed: fn(&mut Commands, Entity, Arc<[Entity]>),
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
                on_changed(&mut commands, widget, Arc::clone(&t.event().items));
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
            items: items.iter().collect(),
        });
    }
}

/// Lets a selection choose from the [`ItemValues`] on another entity.
///
/// Application-authored input, and immutable (as every Bevy relationship): replace it by
/// inserting a new `ItemsRef` to change the item source. The selection on the entity is then
/// refreshed from the new source: selected items that do not belong to it are dropped, a `One`
/// or `OneOrMore` selection takes the new source's first item (waiting for it when the source
/// has none yet) and the widgets that follow the selection draw the new items. It cannot be
/// queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::select::ItemsRef;
/// fn edit(mut refs: Query<&mut ItemsRef>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
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
    use bevy::ecs::relationship::RelationshipTarget;
    use bevy::ecs::system::RunSystemOnce;

    #[derive(Component)]
    struct Widget;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap_or(Entity::PLACEHOLDER)
    }

    #[test]
    fn a_view_plan_keeps_the_view_of_each_wanted_key_and_drops_the_rest() {
        let plan = plan_views(
            [(e(1), Some('a')), (e(2), Some('b')), (e(3), Some('a'))],
            &['b', 'c', 'a'],
        );
        assert_eq!(plan.slots, vec![Some(e(2)), None, Some(e(1))]);
        assert_eq!(plan.stale, vec![e(3)]);
    }

    #[test]
    fn a_view_plan_never_keeps_a_view_without_a_key_and_reuses_equal_keys_in_order() {
        let plan = plan_views([(e(1), None), (e(2), Some(0)), (e(3), Some(0))], &[0, 0, 0]);
        assert_eq!(plan.slots, vec![Some(e(2)), Some(e(3)), None]);
        assert_eq!(plan.stale, vec![e(1)]);
    }

    #[test]
    fn views_are_ordered_as_one_run_at_the_first_view_around_the_apps_children() {
        // children: app(1), view(2), app(3), view(4); wanted views 4, 5 (new), 2.
        let views = [e(4), e(5), e(2)];
        let is_view = |c: Entity| c == e(2) || c == e(4);
        let ordered = order_views(&[e(1), e(2), e(3), e(4)], is_view, &views, &[]);
        assert_eq!(ordered, Some(vec![e(1), e(4), e(5), e(2), e(3)]));
    }

    #[test]
    fn no_reorder_is_asked_for_when_commands_alone_give_the_order() {
        let is_view = |c: Entity| c != e(1);
        // Stale view 3 despawned, new view 5 appended: 1, 2, 4, 5.
        assert_eq!(
            order_views(
                &[e(1), e(2), e(3), e(4)],
                is_view,
                &[e(2), e(4), e(5)],
                &[e(3)]
            ),
            None
        );
        // No views yet.
        assert_eq!(order_views(&[e(1)], is_view, &[e(5)], &[]), None);
    }

    #[derive(Resource, Default)]
    struct Affected(Vec<Entity>);

    fn structure_app() -> App {
        let mut app = App::new();
        app.init_resource::<Affected>().add_systems(
            Update,
            |mut changes: StructureChanges<Widget, String>,
             selections: SelectionParam,
             mut affected: ResMut<Affected>| {
                affected.0 = changes.affected(&selections);
                affected.0.sort();
            },
        );
        app
    }

    /// The widgets named by the next frame, sorted.
    fn affected_in_next_frame(app: &mut App) -> Vec<Entity> {
        app.update();
        app.world()
            .get_resource::<Affected>()
            .map(|a| a.0.clone())
            .unwrap_or_default()
    }

    fn sorted<const N: usize>(mut entities: [Entity; N]) -> Vec<Entity> {
        entities.sort();
        entities.to_vec()
    }

    /// A widget over a selection of its own with one item.
    fn widget_with_items(app: &mut App, label: &str) -> (Entity, Entity) {
        let widget = app
            .world_mut()
            .spawn((Widget, Selection::Value(SelectionValue::ZeroOrOne(None))))
            .id();
        let item = app
            .world_mut()
            .spawn((Item::new(label), ItemOf(widget)))
            .id();
        (widget, item)
    }

    #[test]
    fn the_structure_changes_name_no_widget_in_a_frame_without_a_change_and_the_widget_after_one() {
        let mut app = structure_app();
        let (widget, item) = widget_with_items(&mut app, "a");
        assert_eq!(
            affected_in_next_frame(&mut app),
            vec![widget],
            "new entities are changes"
        );
        for _ in 0..3 {
            assert!(affected_in_next_frame(&mut app).is_empty());
        }

        if let Some(mut changed) = app.world_mut().get_mut::<Widget>(widget) {
            changed.set_changed();
        }
        assert_eq!(affected_in_next_frame(&mut app), vec![widget]);
        assert!(affected_in_next_frame(&mut app).is_empty());

        if let Some(mut changed) = app.world_mut().get_mut::<Item<String>>(item) {
            changed.set_changed();
        }
        assert_eq!(affected_in_next_frame(&mut app), vec![widget]);
        assert!(affected_in_next_frame(&mut app).is_empty());

        app.world_mut().entity_mut(item).despawn();
        assert_eq!(
            affected_in_next_frame(&mut app),
            vec![widget],
            "a removed item is a change"
        );
        assert!(affected_in_next_frame(&mut app).is_empty());
    }

    #[test]
    fn a_change_to_one_source_names_only_the_widgets_that_draw_from_it() {
        let mut app = structure_app();
        let (first, first_item) = widget_with_items(&mut app, "a");
        let (second, _) = widget_with_items(&mut app, "b");
        // Two more widgets sharing the first one's selection and its items.
        let shared = app
            .world_mut()
            .spawn((Widget, Selection::reference(first)))
            .id();
        // One with a selection of its own over the first one's items.
        let own = app
            .world_mut()
            .spawn((
                Widget,
                Selection::Value(SelectionValue::ZeroOrOne(None)),
                ItemsRef(first),
            ))
            .id();
        let _ = affected_in_next_frame(&mut app);
        assert!(affected_in_next_frame(&mut app).is_empty());

        // A new item of the first source: its widget, the sharing one and the one over its items.
        app.world_mut().spawn((Item::new("c"), ItemOf(first)));
        assert_eq!(
            affected_in_next_frame(&mut app),
            sorted([first, shared, own])
        );

        // An item changed in place.
        if let Some(mut changed) = app.world_mut().get_mut::<Item<String>>(first_item) {
            changed.set_changed();
        }
        assert_eq!(
            affected_in_next_frame(&mut app),
            sorted([first, shared, own])
        );

        // The second source is independent of all of them.
        app.world_mut().spawn((Item::new("d"), ItemOf(second)));
        assert_eq!(affected_in_next_frame(&mut app), vec![second]);

        // A selection change on the one over the first items touches only it.
        app.world_mut()
            .entity_mut(own)
            .insert(Selection::Value(SelectionValue::ZeroOrOne(None)));
        assert_eq!(affected_in_next_frame(&mut app), vec![own]);

        // The shared one follows the selection it refers to.
        app.world_mut()
            .entity_mut(first)
            .insert(Selection::Value(SelectionValue::ZeroOrOne(None)));
        assert_eq!(affected_in_next_frame(&mut app), sorted([first, shared]));
    }

    #[test]
    fn replacing_the_items_ref_names_the_widget_and_a_removed_item_that_kept_its_owner_names_the_source()
     {
        let mut app = structure_app();
        let (first, first_item) = widget_with_items(&mut app, "a");
        let (second, _) = widget_with_items(&mut app, "b");
        let _ = affected_in_next_frame(&mut app);
        let own = app
            .world_mut()
            .spawn((
                Widget,
                Selection::Value(SelectionValue::ZeroOrOne(None)),
                ItemsRef(first),
            ))
            .id();
        let _ = affected_in_next_frame(&mut app);

        app.world_mut().entity_mut(own).insert(ItemsRef(second));
        assert_eq!(affected_in_next_frame(&mut app), vec![own]);
        assert!(affected_in_next_frame(&mut app).is_empty());

        // An item that stays under its owner but stops being an `Item<String>`.
        app.world_mut()
            .entity_mut(first_item)
            .remove::<Item<String>>();
        assert_eq!(affected_in_next_frame(&mut app), vec![first]);
        assert!(affected_in_next_frame(&mut app).is_empty());
    }

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
    fn setting_one_or_more_to_the_same_entities_in_the_same_order_is_a_no_op() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(value.set_selection([entity(1), entity(2)]), None);
    }

    #[test]
    fn setting_one_or_more_to_the_same_entities_in_another_order_reorders_it() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(
            value.set_selection([entity(2), entity(1)]),
            Some(SelectionValue::OneOrMore(smallvec![entity(2), entity(1)]))
        );
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
    fn setting_zero_or_more_to_the_same_entities_in_the_same_order_is_a_no_op() {
        let value = SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(value.set_selection([entity(1), entity(2)]), None);
    }

    #[test]
    fn setting_zero_or_more_to_the_same_entities_in_another_order_reorders_it() {
        let value = SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(
            value.set_selection([entity(2), entity(1)]),
            Some(SelectionValue::ZeroOrMore(smallvec![entity(2), entity(1)]))
        );
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

    // set_selection and append, the gaps of the first tests

    #[test]
    fn setting_one_to_a_different_entity_replaces_it() {
        let value = SelectionValue::One(entity(1));
        assert_eq!(
            value.set_selection([entity(2)]),
            Some(SelectionValue::One(entity(2)))
        );
    }

    #[test]
    fn setting_one_to_its_own_entity_is_a_no_op() {
        assert_eq!(
            SelectionValue::One(entity(1)).set_selection([entity(1)]),
            None
        );
    }

    #[test]
    fn setting_one_to_nothing_or_to_two_entities_is_refused() {
        let value = SelectionValue::One(entity(1));
        assert_eq!(value.set_selection([]), None);
        assert_eq!(value.set_selection([entity(2), entity(3)]), None);
    }

    #[test]
    fn setting_one_to_the_same_entity_twice_selects_it_once() {
        let value = SelectionValue::One(entity(1));
        assert_eq!(
            value.set_selection([entity(2), entity(2)]),
            Some(SelectionValue::One(entity(2)))
        );
    }

    #[test]
    fn setting_an_empty_zero_or_one_selects_the_entity() {
        assert_eq!(
            SelectionValue::ZeroOrOne(None).set_selection([entity(1)]),
            Some(SelectionValue::ZeroOrOne(Some(entity(1))))
        );
    }

    #[test]
    fn setting_zero_or_one_to_nothing_clears_it() {
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).set_selection([]),
            Some(SelectionValue::ZeroOrOne(None))
        );
    }

    #[test]
    fn setting_zero_or_one_to_what_it_holds_is_a_no_op() {
        assert_eq!(SelectionValue::ZeroOrOne(None).set_selection([]), None);
        assert_eq!(
            SelectionValue::ZeroOrOne(Some(entity(1))).set_selection([entity(1)]),
            None
        );
    }

    #[test]
    fn setting_zero_or_one_to_two_entities_is_refused() {
        assert_eq!(
            SelectionValue::ZeroOrOne(None).set_selection([entity(1), entity(2)]),
            None
        );
    }

    #[test]
    fn appending_duplicates_to_one_or_more_adds_each_entity_once() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.append([entity(2), entity(2), entity(1)]),
            Some(SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]))
        );
    }

    #[test]
    fn appending_only_selected_entities_to_one_or_more_is_a_no_op() {
        let value = SelectionValue::OneOrMore(smallvec![entity(1), entity(2)]);
        assert_eq!(value.append([entity(2), entity(1)]), None);
    }

    #[test]
    fn appending_duplicates_to_zero_or_more_adds_each_entity_once() {
        let value = SelectionValue::ZeroOrMore(smallvec![entity(1)]);
        assert_eq!(
            value.append([entity(2), entity(2), entity(1)]),
            Some(SelectionValue::ZeroOrMore(smallvec![entity(1), entity(2)]))
        );
        assert_eq!(value.append([entity(1)]), None);
    }

    #[test]
    fn appending_the_same_entity_twice_to_an_empty_zero_or_one_selects_it() {
        assert_eq!(
            SelectionValue::ZeroOrOne(None).append([entity(1), entity(1)]),
            Some(SelectionValue::ZeroOrOne(Some(entity(1))))
        );
    }

    #[test]
    fn appending_to_one_is_refused() {
        assert_eq!(SelectionValue::One(entity(1)).append([entity(2)]), None);
    }

    // Pending selections

    fn pending_values() -> [SelectionValue; 2] {
        [
            SelectionValue::One(Entity::PLACEHOLDER),
            SelectionValue::OneOrMore(smallvec![]),
        ]
    }

    #[test]
    fn pending_selections_have_no_selected_items() {
        for value in pending_values() {
            assert!(value.is_pending(), "{value:?}");
            assert_eq!(value.iter().count(), 0, "{value:?}");
            assert_eq!(value.len(), 0, "{value:?}");
            assert!(value.is_empty(), "{value:?}");
            assert_eq!(value.single(), None, "{value:?}");
            assert_eq!(value.last(), None, "{value:?}");
            assert!(!value.is_selected(Entity::PLACEHOLDER), "{value:?}");
        }
    }

    #[test]
    fn initialised_selections_are_not_pending() {
        for value in [
            SelectionValue::One(entity(1)),
            SelectionValue::OneOrMore(smallvec![entity(1)]),
            SelectionValue::ZeroOrOne(None),
            SelectionValue::ZeroOrMore(smallvec![]),
        ] {
            assert!(!value.is_pending(), "{value:?}");
        }
    }

    #[test]
    fn a_pending_selection_cannot_be_cleared_back_to_pending_by_the_update_methods() {
        let initialised = [
            SelectionValue::One(entity(1)),
            SelectionValue::OneOrMore(smallvec![entity(1)]),
        ];
        for value in initialised {
            assert_eq!(value.clear(), None);
            assert_eq!(value.pop(), None);
            assert_eq!(value.trim_to_length(0), None);
            assert_eq!(value.set_selection([]), None);
            assert_eq!(value.toggle_selected(entity(1)), None);
        }
    }

    #[test]
    fn a_pending_selection_is_given_an_item_of_the_same_variant() {
        assert_eq!(
            SelectionValue::One(Entity::PLACEHOLDER).toggle_selected(entity(1)),
            Some(SelectionValue::One(entity(1)))
        );
        assert_eq!(
            SelectionValue::OneOrMore(smallvec![]).push(entity(1)),
            Some(SelectionValue::OneOrMore(smallvec![entity(1)]))
        );
    }

    #[test]
    fn a_pending_one_or_more_is_repaired_by_the_first_item_and_keeps_its_cardinality() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one_or_more()
                    .build_with_items(std::iter::empty::<Item<String>>()),
            )
            .id();
        app.update();
        assert!(selection_value(&app, id).is_some_and(|v| v.is_pending() && v.is_empty()));

        let first = app.world_mut().spawn((Item::new("a"), ItemOf(id))).id();
        app.update();
        let repaired = SelectionValue::OneOrMore(smallvec![first]);
        assert_eq!(selection_value(&app, id), Some(repaired.clone()));
        assert!(!repaired.is_pending());
        // Still a `OneOrMore`: it refuses to clear.
        assert_eq!(repaired.clear(), None);
        assert_eq!(repaired.pop(), None);
        assert!(repaired.allow_multiple());
    }

    #[test]
    fn a_pending_one_repaired_by_the_first_item_stays_a_one() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one()
                    .build_with_items(std::iter::empty::<Item<String>>()),
            )
            .id();
        app.update();
        assert!(selection_value(&app, id).is_some_and(|v| v.is_pending()));
        let first = app.world_mut().spawn((Item::new("a"), ItemOf(id))).id();
        app.update();
        let repaired = SelectionValue::One(first);
        assert_eq!(selection_value(&app, id), Some(repaired.clone()));
        assert_eq!(repaired.single(), Some(first));
        assert!(!repaired.allow_multiple());
        assert_eq!(repaired.clear(), None);
    }

    // The builder, the items and `ItemParam`, in an app with the selection model

    fn items_app() -> App {
        let mut app = App::new();
        app.add_plugins(ItemsPlugin);
        app
    }

    fn selection_value(app: &App, entity: Entity) -> Option<SelectionValue> {
        match app.world().get::<Selection>(entity) {
            Some(Selection::Value(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn item_entities(app: &App, owner: Entity) -> Vec<Entity> {
        app.world()
            .get::<ItemValues>(owner)
            .map(|items| items.iter().collect())
            .unwrap_or_default()
    }

    fn pending_count(app: &App) -> usize {
        app.world()
            .get_resource::<SelectionResource>()
            .map_or(0, |r| r.pending.len())
    }

    #[test]
    fn a_pending_one_is_repaired_by_the_first_item_and_then_forgotten() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one()
                    .build_with_items(std::iter::empty::<Item<String>>()),
            )
            .id();
        app.update();
        assert_eq!(pending_count(&app), 1);
        let item = app.world_mut().spawn((Item::new("a"), ItemOf(id))).id();
        app.update();
        assert_eq!(selection_value(&app, id), Some(SelectionValue::One(item)));
        assert_eq!(pending_count(&app), 0);
    }

    #[test]
    fn a_pending_selection_that_is_despawned_is_dropped_without_polling() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one_or_more()
                    .build_with_items(std::iter::empty::<Item<String>>()),
            )
            .id();
        app.update();
        assert_eq!(pending_count(&app), 1);
        app.world_mut().despawn(id);
        assert_eq!(pending_count(&app), 0);
        app.update();
        assert_eq!(pending_count(&app), 0);
    }

    #[test]
    fn a_pending_selection_whose_shared_items_are_despawned_goes_with_them() {
        let mut app = items_app();
        let items = app.world_mut().spawn(ItemValues::default()).id();
        let sharing = app
            .world_mut()
            .spawn(Selection::builder().one().build_with_items_ref(items))
            .id();
        app.update();
        assert_eq!(pending_count(&app), 1);
        app.world_mut().despawn(items);
        assert!(app.world().get_entity(sharing).is_err());
        assert_eq!(pending_count(&app), 0);
    }

    #[test]
    fn a_one_builder_selects_the_first_item() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        app.update();
        let items = item_entities(&app, id);
        assert_eq!(items.len(), 2);
        assert_eq!(
            selection_value(&app, id),
            items.first().map(|first| SelectionValue::One(*first))
        );
    }

    #[test]
    fn a_one_with_builder_selects_the_given_entity() {
        let mut app = items_app();
        let chosen = app.world_mut().spawn_empty().id();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one_with(chosen)
                    .build_with_items(["a"].map(Item::new)),
            )
            .id();
        app.update();
        assert_eq!(selection_value(&app, id), Some(SelectionValue::One(chosen)));
    }

    #[test]
    fn a_zero_or_one_builder_starts_empty_with_its_items() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_one()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        app.update();
        assert_eq!(
            selection_value(&app, id),
            Some(SelectionValue::ZeroOrOne(None))
        );
        assert_eq!(item_entities(&app, id).len(), 2);
    }

    #[test]
    fn a_zero_or_more_builder_starts_empty_with_its_items() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(["a", "b", "c"].map(Item::new)),
            )
            .id();
        app.update();
        assert_eq!(
            selection_value(&app, id),
            Some(SelectionValue::ZeroOrMore(smallvec![]))
        );
        assert_eq!(item_entities(&app, id).len(), 3);
    }

    #[test]
    fn a_one_or_more_builder_selects_the_first_item() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one_or_more()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        app.update();
        let items = item_entities(&app, id);
        assert_eq!(
            selection_value(&app, id),
            items
                .first()
                .map(|first| SelectionValue::OneOrMore(smallvec![*first]))
        );
    }

    #[test]
    fn a_builder_with_items_ref_chooses_from_the_items_of_another_entity() {
        let mut app = items_app();
        let owner = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_one()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        let sharing = app
            .world_mut()
            .spawn(Selection::builder().one().build_with_items_ref(owner))
            .id();
        app.update();
        let items = item_entities(&app, owner);
        assert!(app.world().get::<ItemValues>(sharing).is_none());
        assert_eq!(
            app.world().get::<ItemsRef>(sharing).map(|r| r.0),
            Some(owner)
        );
        assert_eq!(
            app.world()
                .get::<ItemSelections>(owner)
                .map(RelationshipTarget::len),
            Some(1)
        );
        assert_eq!(
            selection_value(&app, sharing),
            items.first().map(|first| SelectionValue::One(*first))
        );
    }

    #[test]
    fn a_one_selection_over_no_items_selects_the_first_item_added_later() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .one()
                    .build_with_items(Vec::<Item<String>>::new()),
            )
            .id();
        app.update();
        assert_eq!(
            selection_value(&app, id),
            Some(SelectionValue::One(Entity::PLACEHOLDER))
        );
        let added = app.world_mut().spawn((Item::new("a"), ItemOf(id))).id();
        app.update();
        app.update();
        assert_eq!(selection_value(&app, id), Some(SelectionValue::One(added)));
    }

    #[test]
    fn changing_the_items_triggers_items_changed_with_the_new_items() {
        #[derive(Resource, Default)]
        struct Seen(Vec<(Entity, usize)>);

        fn seen(app: &App) -> Vec<(Entity, usize)> {
            app.world()
                .get_resource::<Seen>()
                .map(|seen| seen.0.clone())
                .unwrap_or_default()
        }

        let mut app = items_app();
        app.init_resource::<Seen>();
        app.add_observer(|changed: On<ItemsChanged>, mut seen: ResMut<Seen>| {
            seen.0.push((changed.entity, changed.items.len()));
        });
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        app.update();
        assert_eq!(seen(&app), vec![(id, 2)]);

        app.world_mut().spawn((Item::new("c"), ItemOf(id)));
        app.update();
        assert_eq!(seen(&app), vec![(id, 2), (id, 3)]);

        app.update();
        assert_eq!(seen(&app).len(), 2);
    }

    #[test]
    fn every_observer_of_one_change_shares_one_snapshot_of_the_items() {
        #[derive(Resource, Default)]
        struct Snapshots(Vec<Arc<[Entity]>>);

        let mut app = items_app();
        app.init_resource::<Snapshots>();
        for _ in 0..3 {
            app.add_observer(|changed: On<ItemsChanged>, mut seen: ResMut<Snapshots>| {
                seen.0.push(Arc::clone(&changed.items));
            });
        }
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        app.update();
        if let Some(mut seen) = app.world_mut().get_resource_mut::<Snapshots>() {
            seen.0.clear();
        }

        // Two additions in one frame reach the observers as one change holding both.
        let first = app.world_mut().spawn((Item::new("c"), ItemOf(id))).id();
        let second = app.world_mut().spawn((Item::new("d"), ItemOf(id))).id();
        app.update();
        let snapshots = app
            .world()
            .get_resource::<Snapshots>()
            .map(|seen| seen.0.clone())
            .unwrap_or_default();
        assert_eq!(snapshots.len(), 3);
        for snapshot in &snapshots {
            assert!(snapshots.iter().all(|other| Arc::ptr_eq(snapshot, other)));
            assert_eq!(snapshot.len(), 4);
            assert_eq!(snapshot.last_chunk::<2>(), Some(&[first, second]));
        }
    }

    #[test]
    fn a_snapshot_is_not_changed_by_a_later_mutation_of_the_items() {
        #[derive(Resource, Default)]
        struct Kept(Vec<Arc<[Entity]>>);

        let mut app = items_app();
        app.init_resource::<Kept>();
        app.add_observer(|changed: On<ItemsChanged>, mut kept: ResMut<Kept>| {
            kept.0.push(Arc::clone(&changed.items));
        });
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(["a", "b"].map(Item::new)),
            )
            .id();
        app.update();
        let added = app.world_mut().spawn((Item::new("c"), ItemOf(id))).id();
        app.update();
        app.world_mut().despawn(added);
        app.update();
        let kept = app
            .world()
            .get_resource::<Kept>()
            .map(|kept| kept.0.clone())
            .unwrap_or_default();
        assert_eq!(
            kept.iter().map(|items| items.len()).collect::<Vec<_>>(),
            vec![2, 3, 2]
        );
    }

    #[test]
    fn an_item_param_finds_the_first_item_matching_the_predicate() {
        let mut app = items_app();
        let id = app
            .world_mut()
            .spawn(
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(["a", "b", "c"].map(Item::new)),
            )
            .id();
        app.update();
        let items = item_entities(&app, id);
        let found = app
            .world_mut()
            .run_system_once(|param: ItemParam<String>| {
                (
                    param.find_item_ref(|item, ()| item.data == "b"),
                    param.find_item_ref(|item, ()| item.data == "z"),
                    param
                        .find_item(|item, ()| item.data == "c")
                        .map(|(_, item, ())| item.data.clone()),
                )
            })
            .ok();
        assert_eq!(
            found,
            Some((items.get(1).copied(), None, Some("c".to_owned())))
        );
    }
}
