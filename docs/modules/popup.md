# `popup.rs`

Crate-private parts the combo popups share: button look, outside click, open/close side effects, combo-box accessibility.

Read this before changing `src/popup.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [popups](../design/popups.md).

## Details

crate-private (`pub(crate)` items, like `focus.rs`): what `Combo`, `ColorPaletteCombo` and `FilterCombo` share, each passing the parts it differs in as arguments: `button_look` (the pure colour match of a pointer event and the open state, unit tested) and `apply_look`, `closes_on_outside_release` (open, pointer on neither button nor the inside part of the popup), `primary_releases` (the pointers that lifted their primary button or finger, read from `PointerInput` messages in an `Update` system, which is no observer) and the `PopupPointers` system param (picking's `HoverMap`, with `PreviousHoverMap` as the fallback for a finger whose pointer entity is gone on the release frame, so mouse and touch share one path; `over`, `over_scroll_bar`, `pressed_over`), `show_popup` / `move_focus` / `set_expanded` (the open/close side effects) and `sync_combo_box` (`Role::ComboBox`, value, `controls`, no label)
