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
/// is [`InteractionDisabled`] (a disabled `Link` included), so Tab skips it.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[require(TabIndex(0))]
pub(crate) struct TabStop;

/// Keeps the [`TabIndex`] of every [`TabStop`] in step with whether it is disabled.
fn sync_tab_index(mut stops: Query<(&mut TabIndex, Has<InteractionDisabled>), With<TabStop>>) {
    for (mut index, disabled) in &mut stops {
        let wanted = if disabled { -1 } else { 0 };
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
