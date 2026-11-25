# `icon_button.rs`

`IconButton`: an icon-only button with sizes, tones and an optional toggle state.

Read this before changing `src/icon_button.rs`. When the module's behavior changes, update this file in the same PR.

Cross-cutting design: [pointer, touch and buttons](../design/pointer-and-buttons.md).

## Details

`IconButton::new(handle, name, size)` with `IconButtonSize::{Regular, Small, Large}` (44 px / 22 px icon, 30 px / 16 px icon, 54 px / 24 px icon): a borderless `Button` of control radius with an `Icon` child (`IconButtonRefs`); `name` is the label of its `AccessibilityNode`, kept there by a system because Bevy clears a text-less button's label on add; inserting `IconButton` again swaps icon, name or size; small icons are secondary text until hovered; an app observes `Activate`; an optional immutable `ToggleState(bool)` makes it an on/off toggle: `Activate` inserts the flipped value (an app can insert one too), and an on button shows `accent_subtle`, an `AccentSubtleText` icon and a 2 px `accent_subtle_bar` child along the bottom edge, inset by the control radius to stay on the straight part (spawned with the first `ToggleState`). `IconButton::with_tone(Tone)` (`Neutral` default, `Warning`, `Danger`; also the tone of a `Tool`): `Warning` has `warning_background`, a 1 px `warning_border`, a `WarningText` icon and the large shadow, mixes 25 % of the border color into the background when hovered or on, and when on shows a 3 px `warning_border` bar (a 54 px warning button with a `Badge` is the redesign's warnings button); `Danger` turns the icon `danger_hover_text` when hovered and is filled with `danger` and `DangerText` when on, without a bar. With a `ToggleState` its accessibility node is `toggled` with it; without one it has no `toggled` at all (`icon_button_accessible_toggle.ron`)

## Deferred construction

Two inserts applied in one command flush build one set of parts, the last insert's configuration winning (AGENTS.md, "Deferred construction decides at run time"; the `two_inserts_of_*_in_one_command_batch_*` tests in `tests/headless.rs`).
