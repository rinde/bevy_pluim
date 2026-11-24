//! What `Combo` and `ColorPaletteCombo` share: the button's colours, the side effects of
//! opening and closing the popup, the outside-click rule and the button's accessibility node.
//!
//! The two widgets differ in where their parts live (`ComboRefs` against `SubRefs`), in what
//! the popup is (a list, or a palette inside a float) and in when a click inside the popup
//! keeps it open; each passes those as arguments.

#![expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private, while its items keep pub(crate) to say how far they reach (pub would trip unreachable_pub), as in focus.rs"
)]

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::system::SystemParam;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::backend::HitData;
use bevy::picking::hover::{HoverMap, PreviousHoverMap};
use bevy::picking::pointer::{
    PointerAction, PointerButton, PointerId, PointerInput, PointerPressState,
};
use bevy::prelude::*;

use crate::MouseKind;
use crate::focus::{Unreachable, clear_label, sync_controls, sync_role, sync_value};
use crate::scroll_area::PopupScrollBar;
use crate::theme::Theme;

/// How a combo's button is coloured.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ButtonLook {
    /// `background_primary` and `border`: the button at rest.
    Resting,
    /// `background_secondary` and `border_highlight`: hovered, pressed or open.
    Highlighted,
}

/// The look the button takes for a pointer event, or `None` to leave it as it is.
///
/// A drag says nothing new about the pointer being on the button, and can arrive after the
/// pointer left it, so it does not restyle it. A release or a click on a closed combo does
/// not either. An open combo stays highlighted whatever the pointer does.
///
/// `hovered` is `MouseEvent::hovered`: a finger never hovers, so its `Over` changes nothing,
/// its press shows the highlight while it is down, and its release or cancel puts the button
/// back at rest.
pub(crate) const fn button_look(kind: MouseKind, hovered: bool, open: bool) -> Option<ButtonLook> {
    match (kind, open) {
        (MouseKind::Drag, _) => None,
        (_, true) | (MouseKind::Pressed, false) => Some(ButtonLook::Highlighted),
        (MouseKind::Over, false) => {
            if hovered {
                Some(ButtonLook::Highlighted)
            } else {
                None
            }
        }
        (MouseKind::Out | MouseKind::Cancel, false) => Some(ButtonLook::Resting),
        (MouseKind::Released | MouseKind::Click(_), false) => {
            if hovered {
                None
            } else {
                Some(ButtonLook::Resting)
            }
        }
    }
}

/// Colours the button for `look`.
pub(crate) fn apply_look(
    look: ButtonLook,
    theme: &Theme,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
) {
    match look {
        ButtonLook::Highlighted => {
            *background_color = BackgroundColor(theme.colors().background_secondary);
            *border_color = BorderColor::all(theme.colors().border_highlight);
        }
        ButtonLook::Resting => {
            *background_color = BackgroundColor(theme.colors().background_primary);
            *border_color = BorderColor::all(theme.colors().border);
        }
    }
}

/// The pointers that lifted their primary button (the left mouse button, a finger) since the
/// system last looked. Reading the messages takes them, so call it once per run.
pub(crate) fn primary_releases(inputs: &mut MessageReader<PointerInput>) -> Vec<PointerId> {
    inputs
        .read()
        .filter(|input| matches!(input.action, PointerAction::Release(PointerButton::Primary)))
        .map(|input| input.pointer_id)
        .collect()
}

/// Where the pointers are, whatever their kind: the hover map of picking, which has an entry
/// for a finger as well as for the mouse, where picking's `Hovered` follows the mouse only.
///
/// A node is under a pointer when it, or a descendant of it, is among the entities that
/// pointer hits, which is what `Hovered` says for the mouse.
#[derive(SystemParam)]
pub(crate) struct PopupPointers<'w, 's> {
    hover_map: Res<'w, HoverMap>,
    previous: Res<'w, PreviousHoverMap>,
    pressing: Query<'w, 's, (&'static PointerId, &'static PointerPressState)>,
    parents: Query<'w, 's, &'static ChildOf>,
    bars: Query<'w, 's, &'static PopupScrollBar>,
}

impl PopupPointers<'_, '_> {
    /// Whether `test` holds for a hit of `pointer` or one of its ancestors. `None` asks about
    /// every pointer that is there now. A finger that just lifted is gone from the hover
    /// map, and is looked up where it was a frame before.
    fn hit(&self, pointer: Option<PointerId>, test: impl Fn(Entity) -> bool) -> bool {
        let under = |hits: &EntityHashMap<HitData>| {
            hits.keys()
                .any(|&hit| test(hit) || self.parents.iter_ancestors(hit).any(&test))
        };
        pointer.map_or_else(
            || self.hover_map.values().any(under),
            |id| {
                self.hover_map
                    .get(&id)
                    .or_else(|| self.previous.get(&id))
                    .is_some_and(under)
            },
        )
    }

    /// Whether `pointer` (any pointer for `None`) is on `entity`.
    pub(crate) fn over(&self, pointer: Option<PointerId>, entity: Entity) -> bool {
        self.hit(pointer, |hit| hit == entity)
    }

    /// Whether `pointer` (any pointer for `None`) is on the scroll bar of `list`, which counts
    /// as inside its popup.
    pub(crate) fn over_scroll_bar(&self, pointer: Option<PointerId>, list: Entity) -> bool {
        self.hit(pointer, |hit| {
            self.bars.get(hit).is_ok_and(|bar| bar.list() == list)
        })
    }

    /// Whether a pointer that has its primary button or finger down is on `entity`.
    pub(crate) fn pressed_over(&self, entity: Entity) -> bool {
        self.pressing
            .iter()
            .any(|(&id, press)| press.is_primary_pressed() && self.over(Some(id), entity))
    }
}

/// Whether a release of the primary button closes the popup: it is open and the pointer is on
/// neither the button nor the part of the popup that counts as inside, for the pointer that
/// lifted, a mouse or a finger.
///
/// What counts as inside differs: a `Combo`'s list only for a selection of many items, a
/// `ColorPaletteCombo`'s float always.
pub(crate) const fn closes_on_outside_release(
    open: bool,
    over_button: bool,
    over_inside_of_popup: bool,
) -> bool {
    open && !over_button && !over_inside_of_popup
}

/// Shows or hides the popup and makes the popup's focus target a tab stop only while open.
///
/// `popup` is the part that takes the focus (a combo's list, a palette combo's palette),
/// `visible_part` the part whose visibility changes (the same list, or the palette's float).
pub(crate) fn show_popup(commands: &mut Commands, open: bool, popup: Entity, visible_part: Entity) {
    commands.entity(visible_part).insert(if open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    });
    // A closed popup is not a tab stop.
    if open {
        commands.entity(popup).remove::<Unreachable>();
    } else {
        commands.entity(popup).insert(Unreachable);
    }
}

/// Moves the focus into the popup when it opens and back to the button when it closes with
/// the focus still on the popup, so it never stays on something hidden.
pub(crate) fn move_focus(focus: &mut InputFocus, open: bool, popup: Entity, button: Entity) {
    if open {
        focus.set(popup, FocusCause::Navigated);
    } else if focus.get() == Some(popup) {
        focus.set(button, FocusCause::Navigated);
    }
}

/// Says on the button's node whether the popup is open.
pub(crate) fn set_expanded(node: &mut AccessibilityNode, open: bool) {
    node.set_expanded(open);
}

/// Makes the button's node a combo box with `value`, controlling `popup`, without a label.
pub(crate) fn sync_combo_box(node: &mut Mut<AccessibilityNode>, value: &str, popup: Entity) {
    sync_role(node, Role::ComboBox);
    clear_label(node);
    sync_value(node, value);
    sync_controls(node, popup);
}

#[cfg(test)]
mod tests {
    use bevy::picking::pointer::PointerButton;

    use super::*;

    #[test]
    fn a_closed_button_highlights_on_over_and_press_and_rests_on_out() {
        assert_eq!(
            button_look(MouseKind::Over, true, false),
            Some(ButtonLook::Highlighted)
        );
        assert_eq!(
            button_look(MouseKind::Pressed, true, false),
            Some(ButtonLook::Highlighted)
        );
        assert_eq!(
            button_look(MouseKind::Out, true, false),
            Some(ButtonLook::Resting)
        );
    }

    #[test]
    fn a_closed_button_ignores_drags_releases_and_clicks() {
        let click = MouseKind::Click(PointerButton::Primary);
        for kind in [MouseKind::Drag, MouseKind::Released, click] {
            assert_eq!(button_look(kind, true, false), None);
        }
    }

    #[test]
    fn a_finger_highlights_a_closed_button_only_while_it_is_down() {
        let click = MouseKind::Click(PointerButton::Primary);
        assert_eq!(button_look(MouseKind::Over, false, false), None);
        assert_eq!(
            button_look(MouseKind::Pressed, false, false),
            Some(ButtonLook::Highlighted)
        );
        for kind in [
            MouseKind::Released,
            click,
            MouseKind::Out,
            MouseKind::Cancel,
        ] {
            assert_eq!(button_look(kind, false, false), Some(ButtonLook::Resting));
        }
    }

    #[test]
    fn an_open_button_stays_highlighted_except_for_a_drag() {
        let click = MouseKind::Click(PointerButton::Primary);
        for kind in [
            MouseKind::Over,
            MouseKind::Pressed,
            MouseKind::Released,
            click,
            MouseKind::Out,
            MouseKind::Cancel,
        ] {
            assert_eq!(
                button_look(kind, false, true),
                Some(ButtonLook::Highlighted)
            );
        }
        assert_eq!(button_look(MouseKind::Drag, false, true), None);
    }

    #[test]
    fn only_an_open_popup_with_the_pointer_outside_closes_on_release() {
        assert!(closes_on_outside_release(true, false, false));
        assert!(!closes_on_outside_release(false, false, false));
        assert!(!closes_on_outside_release(true, true, false));
        assert!(!closes_on_outside_release(true, false, true));
    }
}
