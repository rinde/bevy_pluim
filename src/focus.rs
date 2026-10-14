//! Keyboard focus: which widgets Tab reaches, and the ring that shows where the focus is.
//!
//! `TabStop` is required by the widgets a person can focus. `sync_tab_index` keeps their
//! [`TabIndex`] and `draw_focus_ring` draws the [`ThemeColors::focus_ring`](crate::theme::ThemeColors)
//! around the focused one while the focus is visible.

use bevy::camera::visibility::VisibilitySystems;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{FocusCause, InputFocus, InputFocusVisible};
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{ActiveDescendant, ListItem};

use crate::select::SelectionValue;
use crate::theme::Theme;

/// Registers the systems of the keyboard focus.
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_focus_ring).add_systems(
            PostUpdate,
            sync_tab_index.after(VisibilitySystems::VisibilityPropagate),
        );
    }
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
fn is_hidden(
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

/// Keeps the [`TabIndex`] of every [`TabStop`] in step with whether it is disabled,
/// [`Unreachable`] or hidden.
fn sync_tab_index(
    mut stops: Query<
        (
            Entity,
            &mut TabIndex,
            Has<InteractionDisabled>,
            Has<Unreachable>,
        ),
        With<TabStop>,
    >,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    for (entity, mut index, disabled, unreachable) in &mut stops {
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

/// Draws the focus ring as an [`Outline`] while the focus is visible, and takes it off
/// everything else.
///
/// The ring goes on the focused [`TabStop`], except for a list: a `ListBox` has one tab stop
/// and moves an [`ActiveDescendant`] over its rows, so the ring is on that row and the box
/// itself shows none. A list that is not focused has no active descendant, so no ring.
fn draw_focus_ring(
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    theme: Res<Theme>,
    stops: Query<(Entity, Option<&Outline>), Or<(With<TabStop>, With<ListItem>)>>,
    active: Query<&ActiveDescendant>,
    mut commands: Commands,
) {
    let focused = if visible.0 { focus.get() } else { None };
    let ringed = focused.and_then(|entity| match active.get(entity) {
        Ok(ActiveDescendant(row)) => *row,
        Err(_) => Some(entity),
    });
    for (entity, outline) in &stops {
        if ringed == Some(entity) {
            let ring = Outline {
                width: Val::Px(2.0),
                offset: Val::Px(1.0),
                color: theme.colors.focus_ring,
            };
            if outline != Some(&ring) {
                commands.entity(entity).insert(ring);
            }
        } else if outline.is_some() {
            commands.entity(entity).remove::<Outline>();
        }
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
