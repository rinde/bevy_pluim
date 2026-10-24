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
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use crate::MouseKind;
use crate::focus::{Unreachable, clear_label, sync_controls, sync_role, sync_value};
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
pub(crate) const fn button_look(kind: MouseKind, open: bool) -> Option<ButtonLook> {
    match (kind, open) {
        (MouseKind::Drag, _) | (MouseKind::Released | MouseKind::Click(_), false) => None,
        (MouseKind::Over | MouseKind::Pressed, _) | (_, true) => Some(ButtonLook::Highlighted),
        (MouseKind::Out, false) => Some(ButtonLook::Resting),
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
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor::all(theme.colors.border_highlight);
        }
        ButtonLook::Resting => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor::all(theme.colors.border);
        }
    }
}

/// Whether a release of the left button closes the popup: it is open and the pointer is on
/// neither the button nor the part of the popup that counts as inside.
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
            button_look(MouseKind::Over, false),
            Some(ButtonLook::Highlighted)
        );
        assert_eq!(
            button_look(MouseKind::Pressed, false),
            Some(ButtonLook::Highlighted)
        );
        assert_eq!(
            button_look(MouseKind::Out, false),
            Some(ButtonLook::Resting)
        );
    }

    #[test]
    fn a_closed_button_ignores_drags_releases_and_clicks() {
        let click = MouseKind::Click(PointerButton::Primary);
        for kind in [MouseKind::Drag, MouseKind::Released, click] {
            assert_eq!(button_look(kind, false), None);
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
        ] {
            assert_eq!(button_look(kind, true), Some(ButtonLook::Highlighted));
        }
        assert_eq!(button_look(MouseKind::Drag, true), None);
    }

    #[test]
    fn only_an_open_popup_with_the_pointer_outside_closes_on_release() {
        assert!(closes_on_outside_release(true, false, false));
        assert!(!closes_on_outside_release(false, false, false));
        assert!(!closes_on_outside_release(true, true, false));
        assert!(!closes_on_outside_release(true, false, true));
    }
}
