//! Keyboard focus: tab stops, roving groups and the focus ring.

#![expect(clippy::redundant_pub_crate, reason = "pub would trip unreachable_pub")]

use accesskit::Role;
use bevy::a11y::{AccessibilityNode, AccessibilitySystems};
use bevy::camera::visibility::VisibilitySystems;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{FocusCause, FocusedInput, InputFocus, InputFocusVisible};
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Selected, UiSystems};
use bevy::ui_widgets::{Activate, ActiveDescendant, ListItem, ScrollIntoView, Tab};

use crate::select::{SelectionParam, SelectionValue};
use crate::theme::Theme;

/// Width of the focus ring, in logical pixels.
pub(crate) const FOCUS_RING_WIDTH: f32 = 2.0;

/// Gap between a node and its focus ring, in logical pixels.
pub(crate) const FOCUS_RING_OFFSET: f32 = 1.0;

/// How far a focus ring reaches beyond its node, in logical pixels.
pub(crate) const FOCUS_RING_REACH: f32 = FOCUS_RING_OFFSET + FOCUS_RING_WIDTH;

/// Registers the systems of the keyboard focus.
#[derive(Debug, Copy, Clone)]
pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(navigate_roving_group)
            .add_systems(PostUpdate, draw_focus_ring.in_set(DrawFocusRing))
            .init_resource::<HiddenDirty>()
            .add_systems(
                PostUpdate,
                collect_hidden_changes.after(VisibilitySystems::VisibilityPropagate),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_tab_index,
                    sync_roving_groups.after(sync_tab_index),
                    sync_accessibility_hidden,
                )
                    .after(collect_hidden_changes),
            )
            .configure_sets(PostUpdate, DrawFocusRing.before(UiSystems::Layout))
            .configure_sets(
                PostUpdate,
                AccessibilitySync
                    .after(UiSystems::PostLayout)
                    .before(AccessibilitySystems::Update),
            );
    }
}

/// The system that draws the focus ring, before the layout.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DrawFocusRing;

/// The systems that set the role and states of the widgets' [`AccessibilityNode`]s.
///
/// They run after Bevy's own, which reset [`Role::Button`] and clear labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AccessibilitySync;

/// Sets the role of `node`, writing only on a difference.
pub(crate) fn sync_role(node: &mut Mut<AccessibilityNode>, role: Role) {
    if node.role() != role {
        node.set_role(role);
    }
}

/// Sets the toggled state of `node`, or clears it for `None`, writing only on a difference.
pub(crate) fn sync_toggled(node: &mut Mut<AccessibilityNode>, toggled: Option<bool>) {
    let wanted = toggled.map(accesskit::Toggled::from);
    if node.toggled() == wanted {
        return;
    }
    match wanted {
        Some(toggled) => node.set_toggled(toggled),
        None => node.clear_toggled(),
    }
}

/// Sets the label of `node`, writing only on a difference.
pub(crate) fn sync_label(node: &mut Mut<AccessibilityNode>, label: &str) {
    if node.label() != Some(label) {
        node.set_label(label);
    }
}

/// Sets the value of `node`, writing only on a difference.
pub(crate) fn sync_value(node: &mut Mut<AccessibilityNode>, value: &str) {
    if node.value() != Some(value) {
        node.set_value(value);
    }
}

/// Clears the value of `node`, writing only if it has one.
pub(crate) fn clear_value(node: &mut Mut<AccessibilityNode>) {
    if node.value().is_some() {
        node.clear_value();
    }
}

/// Clears the label of `node`, writing only if it has one.
pub(crate) fn clear_label(node: &mut Mut<AccessibilityNode>) {
    if node.label().is_some() {
        node.clear_label();
    }
}

/// Sets the node `node` controls, writing only on a difference.
pub(crate) fn sync_controls(node: &mut Mut<AccessibilityNode>, controlled: Entity) {
    let wanted = [node_id(controlled)];
    if node.controls() != wanted {
        node.set_controls(wanted);
    }
}

/// Sets the node that labels `node`, writing only on a difference.
pub(crate) fn sync_labelled_by(node: &mut Mut<AccessibilityNode>, labelling: Entity) {
    let wanted = [node_id(labelling)];
    if node.labelled_by() != wanted {
        node.set_labelled_by(wanted);
    }
}

/// The accessibility node id of `entity`.
pub(crate) const fn node_id(entity: Entity) -> accesskit::NodeId {
    accesskit::NodeId(entity.to_bits())
}

/// Marks a widget that Tab reaches; `sync_tab_index` sets its [`TabIndex`] to -1 while it is
/// disabled, [`Unreachable`] or hidden.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[require(TabIndex(0))]
pub(crate) struct TabStop;

/// Marks an item button of a [`RovingGroup`]; [`sync_roving_groups`] owns its [`TabIndex`].
#[derive(Component, Clone, Copy, Debug)]
#[component(immutable)]
pub(crate) struct RovingItem {
    owner: Entity,
    item: Entity,
}

impl RovingItem {
    /// The button for `item`, of the group `owner`.
    pub(crate) const fn new(owner: Entity, item: Entity) -> Self {
        Self { owner, item }
    }
}

/// Which item a [`RovingGroup`] offers to Tab when the focus is not in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RovingEntry {
    /// The selected item, else the first.
    Selected,
    /// The item with Bevy's `Selected` marker, else the first.
    Marked,
    /// The item that had the focus last, else the first.
    Last,
}

/// Which keys move the focus inside a [`RovingGroup`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RovingKeys {
    Radio,
    /// Left and right only; up and down are left to the app.
    Horizontal,
}

/// What moving the focus inside a [`RovingGroup`] does to the item it lands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RovingPick {
    Nothing,
    Selection,
    Activate,
}

/// A composite widget with one tab stop and arrow keys between its [`RovingItem`]s.
#[derive(Component, Clone, Debug)]
pub(crate) struct RovingGroup {
    entry: RovingEntry,
    keys: RovingKeys,
    pick: RovingPick,
    last: Option<Entity>,
    holds_focus: bool,
    /// The items in shown order, with whether the focus may land on each.
    order: Vec<(Entity, bool)>,
}

impl RovingGroup {
    /// A radio group over a `Selection` of at most one.
    pub(crate) const fn radio() -> Self {
        Self::new(
            RovingEntry::Selected,
            RovingKeys::Radio,
            RovingPick::Selection,
        )
    }

    /// A radio group whose state is the `Selected` marker of its items.
    pub(crate) const fn marked_radio() -> Self {
        Self::new(RovingEntry::Marked, RovingKeys::Radio, RovingPick::Activate)
    }

    /// A toolbar: the arrows only move the focus.
    pub(crate) const fn toolbar() -> Self {
        Self::new(
            RovingEntry::Last,
            RovingKeys::Horizontal,
            RovingPick::Nothing,
        )
    }

    const fn new(entry: RovingEntry, keys: RovingKeys, pick: RovingPick) -> Self {
        Self {
            entry,
            keys,
            pick,
            last: None,
            holds_focus: false,
            order: Vec::new(),
        }
    }

    fn step(&self, from: Entity, step: isize) -> Option<Entity> {
        let eligible: Vec<Entity> = self
            .order
            .iter()
            .filter(|(_, ok)| *ok)
            .map(|(entity, _)| *entity)
            .collect();
        let at = eligible.iter().position(|entity| *entity == from)?;
        let len = isize::try_from(eligible.len()).ok()?;
        let to = (isize::try_from(at).ok()? + step).rem_euclid(len);
        eligible.get(usize::try_from(to).ok()?).copied()
    }

    fn first(&self) -> Option<Entity> {
        self.order
            .iter()
            .find(|(_, ok)| *ok)
            .map(|(entity, _)| *entity)
    }

    fn final_one(&self) -> Option<Entity> {
        self.order
            .iter()
            .rev()
            .find(|(_, ok)| *ok)
            .map(|(entity, _)| *entity)
    }
}

/// Marks a [`TabStop`] that Tab skips although it is usable.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Unreachable;

/// Whether `entity` is hidden by its [`InheritedVisibility`] or a `Display::None` ancestor.
pub(crate) fn is_hidden(
    entity: Entity,
    shown: &Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: &Query<&ChildOf>,
) -> bool {
    if let Ok((_, Some(visibility))) = shown.get(entity)
        && !visibility.get()
    {
        return true;
    }
    let mut current = entity;
    loop {
        if let Ok((Some(node), _)) = shown.get(current)
            && node.display == Display::None
        {
            return true;
        }
        match parents.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => return false,
        }
    }
}

/// The entities whose hidden state may have changed this frame, collected by
/// [`collect_hidden_changes`].
#[derive(Resource, Debug, Default)]
pub(crate) struct HiddenDirty(EntityHashSet);

impl HiddenDirty {
    pub(crate) fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }

    pub(crate) fn contains(&self, entity: Entity) -> bool {
        self.0.contains(&entity)
    }

    /// The set chained with the entities of `extra` not in it.
    pub(crate) fn iter_with<'a>(
        &'a self,
        extra: &'a EntityHashSet,
    ) -> impl Iterator<Item = Entity> + 'a {
        self.iter().chain(
            extra
                .iter()
                .copied()
                .filter(|entity| !self.contains(*entity)),
        )
    }
}

/// What changed this frame that can change whether something is hidden.
#[derive(SystemParam)]
pub(crate) struct HiddenChanges<'w, 's> {
    visibility: Query<'w, 's, Entity, Changed<InheritedVisibility>>,
    nodes: Query<'w, 's, (Entity, &'static Node), Changed<Node>>,
    reparented: Query<'w, 's, Entity, Changed<ChildOf>>,
    children: Query<'w, 's, &'static Children>,
    removed_nodes: RemovedComponents<'w, 's, Node>,
    removed_parents: RemovedComponents<'w, 's, ChildOf>,
    display_none: Local<'s, EntityHashSet>,
    roots: Local<'s, Vec<Entity>>,
    walked: Local<'s, EntityHashSet>,
    stack: Local<'s, Vec<Entity>>,
}

/// Collects into [`HiddenDirty`] the entities whose hidden state may have changed.
pub(crate) fn collect_hidden_changes(mut changes: HiddenChanges, mut dirty: ResMut<HiddenDirty>) {
    let dirty = &mut dirty.0;
    dirty.clear();
    changes.roots.clear();
    changes.walked.clear();
    dirty.extend(changes.visibility.iter());
    for (entity, node) in &changes.nodes {
        let none = node.display == Display::None;
        if none != changes.display_none.contains(&entity) {
            if none {
                changes.display_none.insert(entity);
            } else {
                changes.display_none.remove(&entity);
            }
            changes.roots.push(entity);
        }
    }
    for entity in changes.removed_nodes.read() {
        if changes.display_none.remove(&entity) {
            changes.roots.push(entity);
        }
    }
    changes.roots.extend(changes.reparented.iter());
    changes.roots.extend(changes.removed_parents.read());
    for index in 0..changes.roots.len() {
        let Some(&root) = changes.roots.get(index) else {
            continue;
        };
        if !changes.walked.insert(root) {
            continue;
        }
        dirty.insert(root);
        changes.stack.push(root);
        while let Some(entity) = changes.stack.pop() {
            let Ok(children) = changes.children.get(entity) else {
                continue;
            };
            for &child in children {
                if changes.walked.insert(child) {
                    dirty.insert(child);
                    changes.stack.push(child);
                }
            }
        }
    }
}

/// Sets the [`TabIndex`] of every changed [`TabStop`] from whether it is disabled,
/// [`Unreachable`] or hidden.
fn sync_tab_index(
    hidden_changes: Res<HiddenDirty>,
    mut stops: Query<
        (&mut TabIndex, Has<InteractionDisabled>, Has<Unreachable>),
        (With<TabStop>, Without<RovingItem>),
    >,
    new_stops: Query<Entity, Added<TabStop>>,
    disabled_added: Query<Entity, Added<InteractionDisabled>>,
    unreachable_added: Query<Entity, Added<Unreachable>>,
    mut disabled_removed: RemovedComponents<InteractionDisabled>,
    mut unreachable_removed: RemovedComponents<Unreachable>,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    let mut extra = EntityHashSet::default();
    extra.extend(new_stops.iter());
    extra.extend(disabled_added.iter());
    extra.extend(unreachable_added.iter());
    extra.extend(disabled_removed.read());
    extra.extend(unreachable_removed.read());
    for entity in hidden_changes.iter_with(&extra) {
        let Ok((mut index, disabled, unreachable)) = stops.get_mut(entity) else {
            continue;
        };
        let hidden = is_hidden(entity, &shown, &parents);
        let wanted = if disabled || unreachable || hidden {
            -1
        } else {
            0
        };
        if index.0 != wanted {
            index.0 = wanted;
        }
    }
}

/// Marks the [`AccessibilityNode`] of hidden entities as hidden, and clears it when they show.
fn sync_accessibility_hidden(
    hidden_changes: Res<HiddenDirty>,
    mut nodes: ParamSet<(
        Query<Entity, Changed<AccessibilityNode>>,
        Query<&mut AccessibilityNode>,
    )>,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    let changed: EntityHashSet = nodes.p0().iter().collect();
    let mut accessible = nodes.p1();
    for entity in hidden_changes.iter_with(&changed) {
        let Ok(mut node) = accessible.get_mut(entity) else {
            continue;
        };
        let hidden = is_hidden(entity, &shown, &parents);
        if node.is_hidden() != hidden {
            if hidden {
                node.set_hidden();
            } else {
                node.clear_hidden();
            }
        }
    }
}

/// Puts the focus ring of the entity on a [`RingTarget`], whenever the entity has the focus.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct RingOn(pub(crate) Entity);

/// Marks an entity that carries the focus ring of a [`RingOn`] entity.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct RingTarget;

/// Marks an [`Outline`] that `draw_focus_ring` put there.
#[derive(Component, Debug, Copy, Clone)]
struct FocusRingOutline;

/// The [`Outline`] an entity had before the focus ring replaced it.
#[derive(Component, Debug, Copy, Clone)]
struct OutlineUnderRing(Outline);

/// Draws the focus ring as an [`Outline`] on the focused entity, or on its [`ActiveDescendant`]
/// row for a list. An existing [`Outline`] is kept aside while the ring shows.
fn draw_focus_ring(
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    theme: Res<Theme>,
    stops: Query<
        (
            Entity,
            Option<&Outline>,
            Has<FocusRingOutline>,
            Option<&OutlineUnderRing>,
        ),
        Or<(With<TabStop>, With<ListItem>, With<Tab>, With<RingTarget>)>,
    >,
    active: Query<&ActiveDescendant>,
    ring_on: Query<&RingOn>,
    mut ringed_before: Local<Option<Entity>>,
    mut commands: Commands,
) {
    let ringed = focus.get().and_then(|entity| {
        if let Ok(RingOn(target)) = ring_on.get(entity) {
            return Some(*target);
        }
        if !visible.0 {
            return None;
        }
        match active.get(entity) {
            Ok(ActiveDescendant(row)) => *row,
            Err(_) => Some(entity),
        }
    });
    // Touch only the previous and the current entity: candidates include every list row.
    if let Some(previous) = *ringed_before
        && ringed != Some(previous)
    {
        if let Ok((_, _, true, saved)) = stops.get(previous) {
            let mut entity_commands = commands.entity(previous);
            entity_commands.remove::<(Outline, FocusRingOutline, OutlineUnderRing)>();
            if let Some(OutlineUnderRing(own)) = saved {
                entity_commands.insert(*own);
            }
        }
        *ringed_before = None;
    }
    let Some(entity) = ringed else {
        return;
    };
    let Ok((_, outline, is_ring, _)) = stops.get(entity) else {
        return;
    };
    *ringed_before = Some(entity);
    let ring = Outline {
        width: Val::Px(FOCUS_RING_WIDTH),
        offset: Val::Px(FOCUS_RING_OFFSET),
        color: theme.colors().focus_ring,
    };
    if is_ring {
        if outline != Some(&ring) {
            commands.entity(entity).insert(ring);
        }
    } else {
        let mut entity_commands = commands.entity(entity);
        if let Some(own) = outline {
            entity_commands.insert(OutlineUnderRing(*own));
        }
        entity_commands.insert((ring, FocusRingOutline));
    }
}

/// Whether a key pressed in an open combo popup closes it: Escape, or Enter and Space unless
/// the selection allows many.
pub(crate) fn popup_key_closes(input: &KeyboardInput, selection: Option<&SelectionValue>) -> bool {
    if !input.state.is_pressed() {
        return false;
    }
    match input.key_code {
        KeyCode::Escape => true,
        KeyCode::Enter | KeyCode::Space => {
            selection.is_some_and(|selection| !selection.allow_multiple())
        }
        _ => false,
    }
}

/// Moves the focus after a popup's list lost it and says whether the combo closes.
///
/// A click inside the popup keeps it open and returns the focus to the list; focus nobody
/// took goes back to the button.
pub(crate) fn popup_focus_lost(
    focus: &mut InputFocus,
    open: bool,
    button: Entity,
    list: Entity,
    popup_hovered: bool,
) -> bool {
    if open {
        if focus.get() == Some(button) {
            return false;
        }
        if popup_hovered {
            if focus.get().is_none() {
                focus.set(list, FocusCause::Navigated);
            }
            return false;
        }
    }
    if focus.get().is_none() {
        focus.set(button, FocusCause::Navigated);
    }
    open
}

#[derive(SystemParam)]
struct RovingChanges<'w, 's> {
    new_items: Query<'w, 's, &'static RovingItem, Added<RovingItem>>,
    disabled_added: Query<'w, 's, &'static RovingItem, Added<InteractionDisabled>>,
    unreachable_added: Query<'w, 's, &'static RovingItem, Added<Unreachable>>,
    disabled_removed: RemovedComponents<'w, 's, InteractionDisabled>,
    unreachable_removed: RemovedComponents<'w, 's, Unreachable>,
    items_removed: RemovedComponents<'w, 's, RovingItem>,
    changed_children: Query<'w, 's, Entity, (Changed<Children>, With<RovingGroup>)>,
    changed_selections: Query<'w, 's, Entity, Changed<crate::select::Selection>>,
    marked_added: Query<'w, 's, &'static RovingItem, Added<Selected>>,
    marked_removed: RemovedComponents<'w, 's, Selected>,
}

/// Gives one eligible item per [`RovingGroup`] a `TabIndex` of 0, and moves the focus off an
/// item that became ineligible. Only groups that changed are visited.
fn sync_roving_groups(
    hidden_changes: Res<HiddenDirty>,
    mut roving: RovingChanges,
    mut groups: Query<(Entity, &mut RovingGroup)>,
    items: Query<(&RovingItem, Has<InteractionDisabled>, Has<Unreachable>)>,
    mut indices: Query<&mut TabIndex, With<RovingItem>>,
    selections: SelectionParam,
    marked: Query<(), With<Selected>>,
    mut focus: ResMut<InputFocus>,
    children: Query<&Children>,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    let mut extra_items = EntityHashSet::default();
    extra_items.extend(roving.disabled_removed.read());
    extra_items.extend(roving.unreachable_removed.read());
    let gone: Vec<Entity> = roving.items_removed.read().collect();
    let mut dirty_groups = EntityHashSet::default();
    for entity in hidden_changes.iter_with(&extra_items) {
        if let Ok((item, _, _)) = items.get(entity) {
            dirty_groups.insert(item.owner);
        }
    }
    dirty_groups.extend(roving.new_items.iter().map(|item| item.owner));
    dirty_groups.extend(roving.disabled_added.iter().map(|item| item.owner));
    dirty_groups.extend(roving.unreachable_added.iter().map(|item| item.owner));
    dirty_groups.extend(roving.changed_children.iter());
    dirty_groups.extend(roving.marked_added.iter().map(|item| item.owner));
    for entity in roving.marked_removed.read() {
        if let Ok((item, _, _)) = items.get(entity) {
            dirty_groups.insert(item.owner);
        }
    }
    let focus_moved = focus.is_changed();
    let focused_owner = focus
        .get()
        .and_then(|entity| items.get(entity).ok())
        .map(|(item, _, _)| item.owner);
    let changed_selections: EntityHashSet = roving.changed_selections.iter().collect();
    if focus_moved || !changed_selections.is_empty() || !gone.is_empty() {
        for (entity, group) in &groups {
            let focus_concerns =
                focus_moved && (group.holds_focus || focused_owner == Some(entity));
            let selection_concerns = group.entry == RovingEntry::Selected
                && !changed_selections.is_empty()
                && (changed_selections.contains(&entity)
                    || selections
                        .get_ref(entity)
                        .is_some_and(|source| changed_selections.contains(&source)));
            let item_gone =
                !gone.is_empty() && group.order.iter().any(|(item, _)| gone.contains(item));
            if focus_concerns || selection_concerns || item_gone {
                dirty_groups.insert(entity);
            }
        }
    }

    for owner in dirty_groups {
        let Ok((_, mut group)) = groups.get_mut(owner) else {
            continue;
        };
        let order: Vec<(Entity, bool)> = children
            .iter_descendants_depth_first(owner)
            .filter_map(|entity| {
                let (item, disabled, unreachable) = items.get(entity).ok()?;
                (item.owner == owner).then(|| {
                    let eligible =
                        !disabled && !unreachable && !is_hidden(entity, &shown, &parents);
                    (entity, eligible)
                })
            })
            .collect();
        let eligible_of =
            |entity: Entity| order.iter().find(|(e, _)| *e == entity).map(|(_, ok)| *ok);
        let focused = focus.get();
        let focused_item = focused.filter(|entity| eligible_of(*entity).is_some());

        let mut chosen = None;
        if let Some(item) = focused_item {
            if eligible_of(item) == Some(true) {
                group.last = Some(item);
                group.holds_focus = true;
                chosen = Some(item);
            } else {
                let replacement = replacement_for(&group.order, item, &order);
                match replacement {
                    Some(next) => focus.set(next, FocusCause::Navigated),
                    None => focus.clear(),
                }
                group.last = replacement;
                group.holds_focus = replacement.is_some();
                chosen = replacement;
            }
        } else if group.holds_focus {
            let last = group.last;
            let last_eligible = last.is_some_and(|last| eligible_of(last) == Some(true));
            // A despawned item keeps `InputFocus` until the next key; focus that moved elsewhere is not ours.
            let focus_is_stale = focused.is_some() && focused == last;
            if last_eligible || !focus_is_stale {
                group.holds_focus = false;
            } else if let Some(last) = last {
                let replacement = replacement_for(&group.order, last, &order);
                match replacement {
                    Some(next) => focus.set(next, FocusCause::Navigated),
                    None => focus.clear(),
                }
                group.last = replacement;
                group.holds_focus = replacement.is_some();
                chosen = replacement;
            } else {
                group.holds_focus = false;
            }
        }
        if chosen.is_none() {
            chosen = match group.entry {
                RovingEntry::Selected => {
                    let selection = selections.get(owner);
                    order
                        .iter()
                        .find(|(entity, ok)| {
                            *ok && selection.is_some_and(|selection| {
                                items
                                    .get(*entity)
                                    .is_ok_and(|(item, _, _)| selection.is_selected(item.item))
                            })
                        })
                        .or_else(|| order.iter().find(|(_, ok)| *ok))
                        .map(|(entity, _)| *entity)
                }
                RovingEntry::Marked => order
                    .iter()
                    .find(|(entity, ok)| *ok && marked.contains(*entity))
                    .or_else(|| order.iter().find(|(_, ok)| *ok))
                    .map(|(entity, _)| *entity),
                RovingEntry::Last => group
                    .last
                    .filter(|last| eligible_of(*last) == Some(true))
                    .or_else(|| order.iter().find(|(_, ok)| *ok).map(|(entity, _)| *entity)),
            };
        }
        for (entity, _) in &order {
            let wanted = if Some(*entity) == chosen { 0 } else { -1 };
            if let Ok(mut index) = indices.get_mut(*entity)
                && index.0 != wanted
            {
                index.0 = wanted;
            }
        }
        group.order = order;
    }
}

/// The next item after `lost` that is eligible in `now`, else the previous one.
fn replacement_for(
    before: &[(Entity, bool)],
    lost: Entity,
    now: &[(Entity, bool)],
) -> Option<Entity> {
    let eligible = |entity: Entity| now.iter().any(|(e, ok)| *e == entity && *ok);
    let Some(at) = before.iter().position(|(entity, _)| *entity == lost) else {
        return now.iter().find(|(_, ok)| *ok).map(|(entity, _)| *entity);
    };
    let (earlier, later) = before.split_at(at);
    later
        .iter()
        .skip(1)
        .map(|(entity, _)| *entity)
        .find(|entity| eligible(*entity))
        .or_else(|| {
            earlier
                .iter()
                .rev()
                .map(|(entity, _)| *entity)
                .find(|entity| eligible(*entity))
        })
}

/// Moves the focus between the items of a [`RovingGroup`] with its keys and stops those keys bubbling.
fn navigate_roving_group(
    mut input: On<FocusedInput<KeyboardInput>>,
    items: Query<&RovingItem>,
    groups: Query<&RovingGroup>,
    mut focus: ResMut<InputFocus>,
    mut visible: ResMut<InputFocusVisible>,
    mut selections: SelectionParam,
    mut commands: Commands,
) {
    // `focused_entity` follows the event up the tree; read the focus itself.
    let target = input.event_target();
    if focus.get() != Some(target) {
        return;
    }
    let Ok(item) = items.get(target) else {
        return;
    };
    let Ok(group) = groups.get(item.owner) else {
        return;
    };
    let key = input.event().input.key_code;
    let uses = match group.keys {
        RovingKeys::Radio => matches!(
            key,
            KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::Home
                | KeyCode::End
        ),
        RovingKeys::Horizontal => matches!(
            key,
            KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::Home | KeyCode::End
        ),
    };
    if !uses {
        return;
    }
    input.propagate(false);
    if !input.event().input.state.is_pressed() {
        return;
    }
    let destination = match key {
        KeyCode::ArrowLeft | KeyCode::ArrowUp => group.step(target, -1),
        KeyCode::ArrowRight | KeyCode::ArrowDown => group.step(target, 1),
        KeyCode::Home => group.first(),
        _ => group.final_one(),
    };
    let Some(destination) = destination else {
        return;
    };
    focus.set(destination, FocusCause::Navigated);
    visible.0 = true;
    if let Ok(destination_item) = items.get(destination) {
        match group.pick {
            RovingPick::Nothing => {}
            RovingPick::Selection => {
                let picked = destination_item.item;
                selections.update_selection(item.owner, |selection| {
                    (!selection.allow_multiple()).then(|| selection.set_selection([picked]))?
                });
            }
            RovingPick::Activate => commands.trigger(Activate {
                entity: destination,
            }),
        }
    }
    commands.trigger(ScrollIntoView {
        entity: destination,
    });
}
