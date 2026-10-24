//! Keyboard focus: which widgets Tab reaches, and the ring that shows where the focus is.
//!
//! `TabStop` is required by the widgets a person can focus. `sync_tab_index` keeps their
//! [`TabIndex`] and `draw_focus_ring` draws the [`ThemeColors::focus_ring`](crate::theme::ThemeColors)
//! around the focused one while the focus is visible.

#![expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private so FocusPlugin stays out of the public API, while its items keep pub(crate) to say how far they reach (pub would trip unreachable_pub)"
)]

use accesskit::Role;
use bevy::a11y::{AccessibilityNode, AccessibilitySystems};
use bevy::camera::visibility::VisibilitySystems;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{FocusCause, InputFocus, InputFocusVisible};
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, UiSystems};
use bevy::ui_widgets::{ActiveDescendant, ListItem, Tab};

use crate::select::SelectionValue;
use crate::theme::Theme;

/// Width of the focus ring, in logical pixels.
pub(crate) const FOCUS_RING_WIDTH: f32 = 2.0;

/// Gap between a node and its focus ring, in logical pixels.
pub(crate) const FOCUS_RING_OFFSET: f32 = 1.0;

/// How far a focus ring reaches beyond its node, in logical pixels: its offset and width. A
/// scroll area pads its contents by this much so the ring is not clipped.
pub(crate) const FOCUS_RING_REACH: f32 = FOCUS_RING_OFFSET + FOCUS_RING_WIDTH;

/// Registers the systems of the keyboard focus.
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_focus_ring.in_set(DrawFocusRing))
            .add_systems(
                PostUpdate,
                (sync_tab_index, sync_accessibility_hidden)
                    .after(VisibilitySystems::VisibilityPropagate),
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

/// The system that draws the focus ring, in `PostUpdate` before the layout.
///
/// A widget that names its active row in `PostUpdate` (a `VirtualList`) runs before it, so
/// the ring is on the row the frame the row is named.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DrawFocusRing;

/// The systems that set the role and states of the widgets' [`AccessibilityNode`]s.
///
/// They run after Bevy's own, which set [`Role::Button`] on a node whose `Button` changed and
/// clear the label of a button without text, and before the tree goes to the platform.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AccessibilitySync;

/// Sets the role of `node` to `role`, writing it only when it differs.
pub(crate) fn sync_role(node: &mut Mut<AccessibilityNode>, role: Role) {
    if node.role() != role {
        node.set_role(role);
    }
}

/// Sets the toggled state of `node` to `toggled`, or clears it for `None`, writing it only
/// when it differs.
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

/// Sets the label of `node` to `label`, writing it only when it differs.
pub(crate) fn sync_label(node: &mut Mut<AccessibilityNode>, label: &str) {
    if node.label() != Some(label) {
        node.set_label(label);
    }
}

/// Sets the value of `node` to `value`, writing it only when it differs.
pub(crate) fn sync_value(node: &mut Mut<AccessibilityNode>, value: &str) {
    if node.value() != Some(value) {
        node.set_value(value);
    }
}

/// Clears the value of `node`, writing it only when it has one.
pub(crate) fn clear_value(node: &mut Mut<AccessibilityNode>) {
    if node.value().is_some() {
        node.clear_value();
    }
}

/// Clears the label of `node`, writing it only when it has one.
pub(crate) fn clear_label(node: &mut Mut<AccessibilityNode>) {
    if node.label().is_some() {
        node.clear_label();
    }
}

/// Says that `node` controls `controlled`, the widget it opens or acts on, writing it only
/// when it differs.
pub(crate) fn sync_controls(node: &mut Mut<AccessibilityNode>, controlled: Entity) {
    let wanted = [node_id(controlled)];
    if node.controls() != wanted {
        node.set_controls(wanted);
    }
}

/// Says that `node` is named by the text of `labelling`, writing it only when it differs.
pub(crate) fn sync_labelled_by(node: &mut Mut<AccessibilityNode>, labelling: Entity) {
    let wanted = [node_id(labelling)];
    if node.labelled_by() != wanted {
        node.set_labelled_by(wanted);
    }
}

/// The id of the accessibility node of `entity`, as Bevy builds it for the platform.
pub(crate) const fn node_id(entity: Entity) -> accesskit::NodeId {
    accesskit::NodeId(entity.to_bits())
}

/// Marks a widget that Tab reaches.
///
/// Its [`TabIndex`] is kept by `sync_tab_index`: 0 while the widget is usable, -1 while it
/// is [`InteractionDisabled`] (a disabled `Link` included), [`Unreachable`] (the list of a
/// closed combo) or hidden (its [`InheritedVisibility`] is hidden, or it is under a
/// `Display::None` node, such as a hidden `Stack` panel), so Tab skips it.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[require(TabIndex(0))]
pub(crate) struct TabStop;

/// Marks a [`TabStop`] that Tab skips although it is usable: the list of a closed combo.
///
/// Whoever hides the widget inserts it, and removes it again when the widget shows.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Unreachable;

/// Whether `entity` is not shown: its [`InheritedVisibility`] is hidden, or it or an
/// ancestor has `Display::None`, which takes a node out of the layout without touching its
/// visibility.
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

/// What changed this frame that can change whether something is hidden, shared by
/// `sync_tab_index` and `sync_accessibility_hidden` so that neither walks every entity each
/// frame.
///
/// Hidden is a function of an entity's [`InheritedVisibility`] and of `Display::None` on it or
/// an ancestor. [`InheritedVisibility`] propagates by itself (a change reaches the
/// descendants as changes of their own), but `Display::None` does not touch it, so when the
/// display of a node flips or a node is reparented the whole subtree is dirty too.
#[derive(SystemParam)]
struct HiddenChanges<'w, 's> {
    visibility: Query<'w, 's, Entity, Changed<InheritedVisibility>>,
    nodes: Query<'w, 's, (Entity, &'static Node), Changed<Node>>,
    reparented: Query<'w, 's, Entity, Changed<ChildOf>>,
    children: Query<'w, 's, &'static Children>,
    removed_nodes: RemovedComponents<'w, 's, Node>,
    removed_parents: RemovedComponents<'w, 's, ChildOf>,
    /// The entities known to have `Display::None`, to tell a flip from another edit.
    display_none: Local<'s, EntityHashSet>,
}

impl HiddenChanges<'_, '_> {
    /// The entities whose hidden state may have changed since the last call.
    fn dirty(&mut self) -> EntityHashSet {
        let mut dirty = EntityHashSet::default();
        let mut subtrees = Vec::new();
        dirty.extend(self.visibility.iter());
        for (entity, node) in &self.nodes {
            let none = node.display == Display::None;
            if none != self.display_none.contains(&entity) {
                if none {
                    self.display_none.insert(entity);
                } else {
                    self.display_none.remove(&entity);
                }
                subtrees.push(entity);
            }
        }
        for entity in self.removed_nodes.read() {
            if self.display_none.remove(&entity) {
                subtrees.push(entity);
            }
        }
        subtrees.extend(self.reparented.iter());
        subtrees.extend(self.removed_parents.read());
        for root in subtrees {
            dirty.insert(root);
            dirty.extend(self.children.iter_descendants(root));
        }
        dirty
    }
}

/// Keeps the [`TabIndex`] of every [`TabStop`] in step with whether it is disabled,
/// [`Unreachable`] or hidden.
///
/// It looks only at what changed: a new stop, a change of its [`InteractionDisabled`] or
/// [`Unreachable`] marker, and the stops whose visibility, `Display::None` (on them or an
/// ancestor) or parent changed.
fn sync_tab_index(
    mut changes: HiddenChanges,
    mut stops: Query<(&mut TabIndex, Has<InteractionDisabled>, Has<Unreachable>), With<TabStop>>,
    new_stops: Query<Entity, Added<TabStop>>,
    disabled_added: Query<Entity, Added<InteractionDisabled>>,
    unreachable_added: Query<Entity, Added<Unreachable>>,
    mut disabled_removed: RemovedComponents<InteractionDisabled>,
    mut unreachable_removed: RemovedComponents<Unreachable>,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    let mut dirty = changes.dirty();
    dirty.extend(new_stops.iter());
    dirty.extend(disabled_added.iter());
    dirty.extend(unreachable_added.iter());
    dirty.extend(disabled_removed.read());
    dirty.extend(unreachable_removed.read());
    for entity in dirty {
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

/// Marks the [`AccessibilityNode`] of everything that is hidden (by the same rule as a
/// [`TabStop`]'s) as hidden, and clears it when it shows again, so the platform's
/// accessibility tree leaves it out. A node is written only when its flag changes, and only
/// those whose node was added or changed, or whose visibility, `Display::None` (on them or
/// an ancestor) or parent changed, are looked at.
fn sync_accessibility_hidden(
    mut changes: HiddenChanges,
    mut nodes: ParamSet<(
        Query<Entity, Changed<AccessibilityNode>>,
        Query<&mut AccessibilityNode>,
    )>,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    let mut dirty = changes.dirty();
    dirty.extend(nodes.p0().iter());
    let mut accessible = nodes.p1();
    for entity in dirty {
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

/// Puts the focus ring of the entity on another one, a [`RingTarget`], and shows it
/// whenever the entity has the focus, also after a click: a text field's input rings the
/// field's box, so a person sees where the keys go.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct RingOn(pub(crate) Entity);

/// Marks an entity that carries the focus ring of a [`RingOn`] entity.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct RingTarget;

/// Marks an [`Outline`] that `draw_focus_ring` put there, so it takes off only its own.
#[derive(Component, Debug, Copy, Clone)]
struct FocusRingOutline;

/// The [`Outline`] an entity had before the focus ring replaced it, put back when the ring
/// leaves.
#[derive(Component, Debug, Copy, Clone)]
struct OutlineUnderRing(Outline);

/// Draws the focus ring as an [`Outline`] while the focus is visible, and takes it off
/// everything else.
///
/// The ring goes on the focused [`TabStop`] or [`Tab`] (a tab bar's tabs keep their own
/// roving [`TabIndex`], so they are no tab stops), except for a list: a `ListBox` has one tab stop
/// and moves an [`ActiveDescendant`] over its rows, so the ring is on that row and the box
/// itself shows none. A list that is not focused has no active descendant, so no ring. A
/// focused [`RingOn`] entity has its ring on its [`RingTarget`], visible focus or not.
///
/// The ring owns only the outline it drew: an [`Outline`] the app or a widget set is kept
/// aside while the ring shows and put back when it leaves.
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
    // Only the entity that had the ring and the one that has it now are touched, so the work
    // does not grow with the number of candidates (every row of every list).
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

/// Whether a key pressed with the focus on an open combo's popup closes it.
///
/// Escape closes it. So do Enter and Space when the selection holds one item at most: the
/// popup's list picks the active row (or leaves the selection as it is when that row is
/// selected already, as a click does), and the popup has done its job. A selection of many
/// keeps the popup open for the next pick.
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

/// What a popup's list losing the focus means for its combo.
///
/// Moves the focus where it belongs and says whether the combo closes. The focus going to the
/// combo's own button is not leaving, and nor is a click in the popup, which takes the focus
/// off the list (nothing in it can have it): the popup stays, and the list gets the focus
/// back. Elsewhere the focus goes back to the button when nothing took it, so a click on empty
/// space does not leave the keyboard nowhere; when it went to another widget it stays there.
/// `open` says whether the combo is open, `popup_hovered` whether the pointer is on the popup.
pub(crate) fn popup_focus_lost(
    focus: &mut InputFocus,
    open: bool,
    button: Entity,
    list: Entity,
    popup_hovered: bool,
) -> bool {
    if open {
        // The focus going to the combo's own button (a click on it, Shift+Tab) is not
        // leaving.
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
    // An outside click may have closed the popup already: the focus still goes back.
    if focus.get().is_none() {
        focus.set(button, FocusCause::Navigated);
    }
    open
}
