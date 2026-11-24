# Popups

How combo popups take focus, open and close, and where popups are placed.

When the behaviour described here changes, update this file in the same PR.

## A combo's popup takes the keyboard focus while it is open

Opening a `Combo` or a
`ColorPaletteCombo` (the hook of `ComboState`, the observer of `ColorPaletteComboState`) sets
`InputFocus` to the popup's list (the palette, for the palette combo) with
`FocusCause::Navigated`, so the `ListBox` arrows and Enter / Space work on it; closing
it, however it was closed, moves the focus back to the button if it was still on the
list. The observers sit on that list, not on the combo, because a palette combo's popup
is a child of its root: `FocusedInput<KeyboardInput>` closes the combo on Escape, and
on Enter / Space for a selection of one item at most (the `ListBox` picks the active
row, or leaves a selected one, and the popup is done; a selection of many stays open),
and `FocusLost` closes it when the focus leaves the list (Tab away, a click on
something else). A click on the button or in the popup also takes the focus off the
list (nothing in them is focusable), but that is not leaving: the pointer being on
either (their `Hovered`) makes the observer ignore it (and put the focus back on the
list for a click in the popup). On a close by losing the focus the focus goes back to the
button only if nothing took it. `ComboState` / `ColorPaletteComboState` stay the one source of truth. The
`combo_keyboard_*` and `color_palette_combo_keyboard_*` files cover it.

## Combo buttons open on `Activate` and say whether the popup is open

The
`ComboButton` and the `ColorPushButton` of a `ColorPaletteCombo` are `Button`s;
the combo toggles its `ComboState` (the palette combo its `ColorPaletteComboState`) on their
`Activate`, so on release and on Enter / Space for a focused button, and a
press alone opens nothing. Hover and press colours stay `MouseEvent`, observed
on the combo, which gets the pointer because the button has `should_block_lower:
false`. The outside-click systems read `ButtonInput<MouseButton>` and the
`Hovered` of the button and the popup, not `Activate`, and a click on the
button is inside the combo, so they never close what the same click opens. A
release on a palette combo's popup is inside as well, whatever the selection:
its scroll bar, or a drag of its thumb, leaves it open. The popup closes on a
pick by itself, not through the outside click: a global observer of
`MouseEvent`s acts on a `MouseKind::Click` (a press and a release on the same
swatch) on a swatch that belongs to the popup's `ColorPalette` and, for a
selection of one item at most (`One`, `ZeroOrOne`), inserts `ColorPaletteComboState::Closed`,
whether or not the click changed the selection (the selected colour of a `One`
combo, a deselecting click of a `ZeroOrOne` one). A selection of many keeps
the popup open, on a click and on Enter / Space (a palette combo over many
is supported: `color_palette_combo_many_stays_open.ron`). A `Combo` has not been moved over (#96 left it): a release on
its list still closes it, unless the selection may hold several items. The
state is also `expanded` on the button's `AccessibilityNode` (`set_expanded`), set
where `ComboState` / `ColorPaletteComboState` is inserted (the hook, the observer); the button is a
`Role::ComboBox` whose value is the selection and which `controls` the popup's list
(palette), set by each combo's `sync_accessibility` in `AccessibilitySync`. Scenarios
check `ComboState` / `ColorPaletteComboState`, and `App(Accessible(..))` reads the node's role, value
and `Expanded` / `Collapsed`; `controls` is checked headless. The popup is still the
home-made `Float`, not `bevy_ui_widgets`' `MenuButton` / popover: that would
change positioning, z-index and outside-click behaviour, so it is a ticket of
its own if wanted.

## Popups are placed against the UI target of their widget, not the primary window (#669)

`float.rs`'s crate-private `UiViewport::size_of(node)` returns the logical size of the render
target the node is drawn to: Bevy's `ComputedUiRenderTargetInfo` (the camera's viewport in
physical pixels over its scale factor, `UiScale` included), so a camera with a restricted
viewport, a second window or an image target clamps against its own space. UI coordinates are
viewport-local, so no offset is added. Unusable data (no camera, an empty viewport, a scale that
is not positive and finite) falls back to the primary window's size (`target_size`), and with
neither the popup is left where it is. `Combo`, `FilterCombo`, `ColorPaletteCombo`, `Tooltip`
and `VirtualList`'s fallback height (no scroll area around it) all use it
(`a_combo_list_stays_inside_the_ui_target_of_its_button_and_falls_back_to_the_window`,
`a_tooltip_popup_stays_inside_a_restricted_viewport_not_the_primary_window`; the headless app has
no camera, so the tests set the target data by reflection).

## A `Float` stays in its window (investigation #506, not built)

Putting a popup in a
window of its own, so it can leave the application window, was looked at against Bevy 0.20
and not implemented. What the Bevy source says: a UI tree is drawn by the camera of its
root. `propagate_ui_target_cameras` (`bevy_ui` `update.rs`) reads `UiTargetCamera` (else the
`IsDefaultUiCamera` camera, else the highest-order camera of the primary window) from root
nodes only, and propagates the result, `ComputedUiTargetCamera` and the scale factor and
size in `ComputedUiRenderTargetInfo`, to every descendant; `UiTargetCamera` on a non-root
node "will have no effect". `FixedNode` only makes a node lay out against its camera's
viewport rather than its parent, it keeps the parent's camera. So a `Float` that is a
child of its widget (what every popup is today) is always in the widget's window, and a
popup in another window needs its own `Window`, a `Camera2d` with
`RenderTarget::Window(WindowRef::Entity(..))` and a root `Node` with that camera as
`UiTargetCamera`, a node that has no parent. `UiRoot` already leaves such a root alone
(a `UiTargetCamera` in the bundle means no `UiRoot`, as the tester's cursor overlay shows).
What that breaks, and why it is a design of its own rather than a change to `Float`:
- the popup is no longer a descendant of its widget, so `Hovered` (the outside-click
  systems of `Combo` and `ColorPaletteCombo` rely on "the popup is a descendant"),
  visibility and `Display::None` inheritance (a popup of a hidden `Stack` panel), the
  `Within(..)` reach of the scenarios, the closing of the popup with its widget and event
  bubbling from the popup to the combo would all need an explicit link and cleanup;
- the popup window takes the OS focus from the application window, so `InputFocus`, the
  `FocusLost` that closes a combo and the keyboard hand-over of "A combo's popup takes the
  keyboard focus" change meaning; a tooltip must never take focus at all;
- every placement system (`place_open_lists`, the palette combo, `Tooltip`,
  `VirtualList`'s window height, `float.rs`'s `keep_in_window_x` / `below_or_above`) reads
  the UI target's logical size (`UiViewport`, see above), which a popup window would have to replace;
  a popup window needs screen coordinates (`Window::position` plus the widget's offset,
  the scale factor of whichever monitor it is on) and a reflow of the window when the popup
  resizes;
- Wayland does not let a client place a toplevel window, so the feature cannot be
  uniform across platforms, and web/mobile targets have one window only; the in-window
  `Float` must stay as the fallback;
- the headless tester has no real windows, so the scenario files, which pass at scale
  factors 1 and 2, `[text 1.5x]` and under shuffled schedules, could not cover it without
  extra support in the tester.
If wanted, a first step is an opt-in `PopupWindow` mode on `Tooltip` and `Combo`, with the
in-window `Float` as default and fallback; it is tracked as a follow-up ticket, not here.
