# Keyboard focus and navigation

Tab stops, the focus ring, composite widgets with arrow keys, directional navigation and the accessibility tree.

When the behaviour described here changes, update this file in the same PR.

## Keyboard focus: tab stops and the focus ring

`WidgetsPlugin` adds Bevy's
`InputFocusPlugin`, `InputDispatchPlugin` and `TabNavigationPlugin` in
`Plugin::finish` unless the app has them by then (the tester's `HeadlessPlugins` has the first two, not the third), and
`UiRoot` requires `TabGroup::new(0)`, so the widget tree is one non-modal tab group.
`PushButton`, `ColorPushButton`, `Link`, `Slider` and `List` (so a `ColorPalette` as well)
require the crate-private `TabStop`, which requires `TabIndex`. `sync_tab_index`
owns the value: 0 for a stop that is not `InteractionDisabled` (a disabled
`Link` has it), -1 otherwise or while it has the crate-private `Unreachable` marker (the
list of a closed combo, put on and taken off with `ComboState` / `ColorPaletteComboState`) or while it is
hidden (its `InheritedVisibility` is hidden, as Bevy's UI renderer reads it, or itself or an
ancestor is `Display::None`, as in a hidden `Stack` panel; it is Tab-reachable again once
shown). `sync_tab_index` runs in `PostUpdate` after `VisibilitySystems::VisibilityPropagate`,
so an app needs Bevy's `VisibilityPlugin` (in `DefaultPlugins`, and the tester's
`HeadlessPlugins` through `CameraPlugin`), or every stop counts as hidden; **no widget writes `TabIndex` by hand** (the roving groups below are `focus.rs`'s too, and `TabBar` keeps Bevy's). Within the group
the order is `TabIndex`, then tree order, which is spawn order. The focused `TabStop`
gets an `Outline` (2 px wide, 1 px offset, `ThemeColors::focus_ring`) while
`InputFocusVisible` is true (an `Outline` the app or a widget set on it is kept aside and put back when the ring leaves, so the ring owns only its own), except that a list's ring is on its `ActiveDescendant` row
(a `ListItem`) and the box has none, and a focused `Tab` of a `TabBar` gets the ring too
(its `TabIndex` is Bevy's `TabPlugin`'s, kept at -1 while the bar is hidden by `tab_bar.rs`, not `focus.rs`'s `sync_tab_index`); the `ListBox` clears the active row when the list
loses focus, and the ring goes with it, and `ActiveDescendant` is registered in the type
registry for checks (`App(Active(of: .., is: ..))`, `WidgetsCheck::Active`); a click focuses without showing it (Bevy's
`TabNavigationPlugin`), Tab shows it. Scenarios check the focus with
`App(Focused(of: ..))` (`WidgetsCheck::Focused`) and press Tab with `Tap(key: Tab)`
(`tests/ron/keyboard_tab_order.ron`). A focused entity with the crate-private `RingOn(target)` (a `TextField`'s input) has the ring on its `RingTarget` (the field's box) whether the focus is visible or not, so a clicked field shows where the keys go. `keyboard_focus_ring_in_both_themes.ron` Tabs to "Change theme" and toggles the theme, to record the ring on both. The `*_keyboard_*.ron` files drive lists, palettes and the tab bar (`tabs_keyboard_switches_panel.ron`) with the arrow keys. The lists of closed combos are skipped by Tab, so the palette scenario Tabs 8 times.

## Composite widgets are one tab stop with arrow keys

A `SegmentedControl`, `FilterChips`,
`Gallery`, `Toolbar` and `LightDarkToggle` each require a crate-private `RovingGroup` (`RovingGroup::radio()`
for the first three, `::toolbar()`, `::marked_radio()` for the toggle) and each item button (`Segment`, `ChipView`,
`GalleryCardView`, `ToolbarButton`, `LightDarkOption`) carries a `RovingItem { owner, item }`, put on when the
widget spawns it. `sync_tab_index` leaves those buttons alone; `sync_roving_groups` (same
module, `PostUpdate` after it) owns their `TabIndex`: 0 for exactly one eligible button, -1 for
the rest, and the container is no stop. Eligible means not `InteractionDisabled`, not
`Unreachable` and not hidden (the same `is_hidden` rule); a group with no eligible button, or
none at all, has no stop. The stop is the focused item (the roving choice follows the focus,
also after a click), else the group's entry: `RovingEntry::Selected` (the radio widgets) the
selected eligible item, else the first; `Marked` (the `LightDarkToggle`, whose state is the `ThemePreference` resource, not a `Selection`) the eligible option that has Bevy's `Selected` marker, which `theme_toggles` keeps in step with the preference (a `Selected` added or removed on an item counts as a change of its group, so the entry follows an app that inserts `ThemePreference`), else the first; `Last` (the toolbar) the tool that had the focus last
while eligible, else the first. Items are found by walking the group's descendants in tree
order, which is the order shown (a gallery's groups included), so wrapping and Home / End are
by item order, never by geometry. Only the groups that changed are looked at: an item added,
disabled, hidden or removed, the group's children, a change of the `Selection` the group reads (its own or the one its
`Selection::Ref` resolves to; radio groups) or a move of the focus into or out of the group (the group that held it, the
owner of the newly focused item); nothing walks every frame, and an unrelated group is never looked at
(`a_selection_or_focus_change_looks_only_at_the_groups_it_concerns`,
`a_shared_selection_change_looks_at_every_group_that_reads_it_and_no_other`; the `roving-groups-*` bench workloads). `navigate_roving_group`, a global observer of
`FocusedInput<KeyboardInput>`, acts only when the `InputFocus` is on one of the group's own
items: the radio widgets use left / up and right / down (previous and next, wrapping) and Home
/ End, the toolbar left / right and Home / End (up and down go on to the app). It stops those
keys from bubbling (pressed, repeated and released, also with one eligible item), moves the
focus with `FocusCause::Navigated`, makes it visible, asks `ScrollIntoView` for the new item
and, for a radio group over a selection of at most one (`One`, `ZeroOrOne`), picks it with
`set_selection([item])`, never toggling, so wrapping to the same item keeps it; the toggle's `RovingPick::Activate` instead triggers `Activate` on the option landed on, which sets `ThemePreference` exactly as a click does (and changes nothing when it is the current one). With
`OneOrMore` / `ZeroOrMore` the arrows move the focus only; Enter / Space keep their
`Activate` behaviour (`toggle_selected`), as does a toolbar's. When the focused item goes
(despawned, disabled, hidden) the focus moves to the next eligible item, else the previous,
and is cleared when none is left; focus that went to another widget, or that something cleared,
is never taken or put back (`holds_focus` remembers that the group had it; a despawned item
keeps the `InputFocus` until the next key, which is how a stale focus is told from a cleared one). A `Selection::Ref` shared by several groups
is read per group. Tests: `*_roving_tab_stop.ron` for each widget (Tab, Shift+Tab with
`Key(ShiftLeft)` held, because the tester's `Tap(.., modifiers: [Shift])` releases Shift in
the frame it presses it, which Bevy's Tab navigation does not see), and the
`composite_screen` tests of `tests/scenarios.rs` (held arrows, an app's directional
navigation, disabled / hidden / empty groups, removal, reorder, a pick that lands on the
already selected item, shared selections). The keyboard
behaviour of these four widgets changed in #493; nopjes tabs once into each and uses the
arrows inside.

## Directional navigation is the app's

The library adds none: an app that wants the
arrow keys (or a gamepad stick) to move the focus between widgets puts Bevy's
`AutoDirectionalNavigation` on the widgets that take part, adds
`DirectionalNavigationPlugin`, and calls `AutoDirectionalNavigator::navigate` from an
observer of `FocusedInput<KeyboardInput>` on the primary window, which gets only the keys
no focused widget kept (a system reading `ButtonInput<KeyCode>` would see them all); it
sets `InputFocusVisible` itself, as Bevy's `directional_navigation` example does. What the
widgets guarantee: a widget that uses the arrows stops them from bubbling on, so they
never reach that observer. A focused `List` (so a palette and an open combo's
list too) keeps the arrows, Home and End (the `ListBox` stops a first press,
`keep_navigation_keys` in `list.rs` the repeats and releases, only while the list itself
is `InputFocus`), a focused tab of a `TabBar` keeps left, right, Home and End (Bevy's tab
list stops a first press, `keep_navigation_keys` in `tab_bar.rs` the repeats and
releases) and lets up and down through, a focused `VirtualList` keeps the arrows, Page Up,
Page Down, Home, End, Enter and Space (pressed, repeated or released; `navigate` in `virtual_list.rs`,
only while the list itself is `InputFocus`, so the keys of a focused descendant, a button
in a creator's row, go on past it), a focused `Slider` keeps left, right, Home and End
(`keep_navigation_keys` in `slider.rs`, also at a limit) and lets up and down through, and a
`NumericStepper`
keeps the arrows that reach it from its buttons, also at a limit. The focus ring follows
`InputFocus` whatever moved it, while `InputFocusVisible` is true. A `SegmentedControl`, `FilterChips` and
`Gallery` and `LightDarkToggle` keep the arrows, Home and End while one of their own items has the focus, a `Toolbar`
left, right, Home and End (up and down go through), all through `navigate_roving_group` (see
"Composite widgets"). Every other widget lets
the arrows through. A focusable child of a list row (a button in a `CustomView`) is not
the list: its arrows, Home and End all go on past the list. Bevy's `ListBox` would take a
first press of one as its own, because the `focused_entity` it checks is the hop the event
has reached, not the focus, so `pass_first_presses_from_descendants` in `list.rs` stops
that press on the list's child it comes through and triggers it again from the list's
parent (the window, for a list without one); the active row stays. `tests/scenarios.rs`
turns it on for two buttons, a list and a stepper (`directional_navigation_screen`), for
buttons in list rows (`button_in_a_list_row_app`), and for a tab bar
(`a_held_arrow_key_does_not_move_the_focus_out_of_a_tab_bar`).

## Hidden widgets are hidden from the accessibility tree

`sync_accessibility_hidden`
in `focus.rs` runs next to `sync_tab_index` and marks the `AccessibilityNode` of
anything hidden by the tab stop rule (`is_hidden`) as hidden, clearing it on show.
