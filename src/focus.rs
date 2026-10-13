//! Keyboard focus: which widgets Tab reaches, and the ring that shows where the focus is.
//!
//! `TabStop` is required by the widgets a person can focus. `sync_tab_index` keeps their
//! [`TabIndex`] and `draw_focus_ring` draws the [`ThemeColors::focus_ring`](crate::theme::ThemeColors)
//! around the focused one while the focus is visible.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{ActiveDescendant, ListItem};

use crate::theme::Theme;

/// Registers the systems of the keyboard focus.
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_tab_index, draw_focus_ring));
    }
}

/// Marks a widget that Tab reaches.
///
/// Its [`TabIndex`] is kept by `sync_tab_index`: 0 while the widget is usable, -1 while it
/// is [`InteractionDisabled`] (a disabled `Link` included), [`Unreachable`] (the list of a
/// closed combo) or hidden (`Visibility::Hidden`, or under a `Display::None` node, such
/// as a hidden `Stack` panel), so Tab skips it.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[require(TabIndex(0))]
pub(crate) struct TabStop;

/// Marks a [`TabStop`] that Tab skips although it is usable: the list of a closed combo.
///
/// Whoever hides the widget inserts it, and removes it again when the widget shows.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Unreachable;

/// Whether `entity` is not shown: it or an ancestor has `Display::None`, or its own
/// [`Visibility`] or that of the nearest ancestor that is not `Inherited` is `Hidden`.
///
/// Read from `Visibility` rather than `InheritedVisibility`, which only a rendering app
/// propagates.
fn is_hidden(
    entity: Entity,
    shown: &Query<(Option<&Node>, Option<&Visibility>)>,
    parents: &Query<&ChildOf>,
) -> bool {
    let mut visibility_decided = false;
    let mut current = entity;
    loop {
        if let Ok((node, visibility)) = shown.get(current) {
            if node.is_some_and(|node| node.display == Display::None) {
                return true;
            }
            if !visibility_decided {
                match visibility {
                    Some(Visibility::Hidden) => return true,
                    Some(Visibility::Visible) => visibility_decided = true,
                    _ => {}
                }
            }
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
    shown: Query<(Option<&Node>, Option<&Visibility>)>,
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
