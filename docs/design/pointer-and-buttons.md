# Pointer, touch and buttons

How pointer input reaches the widgets: the UI root, `MouseEvent`, touch, `Hovered` and the shared button.

When the behavior described here changes, update this file in the same PR.

## `UiRoot`

An observer on `OnAdd, Node` inserts `UiRoot` on a node
without a parent and without a `UiTargetCamera`, so root nodes aimed at
another camera (a second window's UI, an overlay) are not the app's and are
left alone. The camera is read when the `Node` is added, so a
`UiTargetCamera` inserted later is not seen: spawn it in the same bundle. An
observer of `Insert, ChildOf` removes `UiRoot` again from a node parented
after its `Node` was added (a scene writes `ChildOf` last), so the marker
follows the hierarchy. A `PostStartup` system logs an `error!` (it does not
panic) unless there is exactly one `UiRoot`; an app should spawn one. Two
roots both get `UiRoot`, so a widget must not take `Single<.., With<UiRoot>>`;
a popup is a `Float` child of its widget instead, which needs no root at all.
The tester's cursor overlay spawns its roots with a `UiTargetCamera`, so they
get no `UiRoot` (`the_cursor_overlays_roots_get_no_ui_root` in
`tests/scenarios.rs`).

## Mouse events

Global observers (registered by `WidgetsPlugin`, not on
`UiRoot`) watch Bevy's flat pointer events `PointerOver`, `PointerPress`,
`PointerRelease`, `PointerClick`, `PointerDrag` and `PointerOut` (one generic
observer over the `PointerEvent` trait) and re-trigger each as a `MouseEvent`
on the original target, on the first hop of the event only. `MouseEvent` is
crate-private: it is how widgets inside this crate react to the pointer. They
are global because Bevy's `Button` calls `propagate(false)` on `PointerPress`,
`PointerRelease`, `PointerClick`, `PointerDragEnd` and `PointerCancel`: an
observer on `UiRoot` would never see those for a button. `PointerOver`,
`PointerDrag` and `PointerOut` are not stopped. `MouseKind` says what the pointer did:
`Over`, `Pressed`, `Released`, `Click(button)` and `Drag`, plus `Out`. `Drag` carries the pointer's `distance` from where the press began (logical pixels), which the `NumericStepper` scrubs by.
`Click` is a press and a release on the same node, which `Released` cannot
say (it also ends a press that began elsewhere), and `Drag` is a move with a
button held, which widgets ignore but the `NumericStepper` (the scroll thumb's drag is the
`bevy_ui_widgets` `Scrollbar`'s). The popups' outside-click handlers
read `PointerInput` releases and the hover map, not a `MouseEvent`, so a `Button` cannot
starve them, and a tap works as a click (#796).

## Touch (#793, #795)

A finger is a `PointerId::Touch` pointer and sends every
`MouseKind` a mouse does, as `PointerButton::Primary`, so `MouseEvent.button` is
`Some(Primary)` and the `touch` flag says it was a finger. `MouseKind::Cancel`
(picking's `PointerCancel`, sent to every hovered node, nothing follows it) ends a
press like a release away from the node: no `Click`, no `Activate`. A finger does not
hover: **`MouseEvent::hovered()` is the one place that tells touch from mouse**
(false for a touch `Over`, and for `Out` and `Cancel`), `pressed()` is true for a
`Pressed`, and the shared look functions (`background_for`, `item_background`,
`popup.rs`'s `button_look(kind, hovered, open)`, which `Combo`, `FilterCombo` and `ColorPaletteCombo` call with `event.hovered()`, and the link, thumb and swatch handlers) use
them, never a per-widget `touch` test. So a finger down shows the pressed look, and after
a lift, slide off or cancel no hover look stays. Picking's `Hovered` follows
`PointerId::Mouse` only, so a system that reads it never sees a finger. A tap
activates once, a disabled button ignores it and a finger that slides off before
lifting activates nothing (`push_button_touch_*.ron` (looks, disabled, slide-off, cancel, toggle, compact, no default hover), `icon_button_touch_tap.ron`, `icon_button_touch_looks.ron`,
`breadcrumbs_touch_tap_trims.ron`, `breadcrumbs_touch_slide_off_and_disabled.ron`, `breadcrumbs_touch_looks.ron`, `panel_touch_close.ron`, `panel_touch_slide_off.ron`, `link_touch.ron`, `color_push_button_touch.ron`; the list widgets' `list_touch_*.ron`, `virtual_list_touch_*.ron` and `color_palette_touch_taps_pick.ron` (a tap picks as a click does, a custom row reads `Hovered`, which a finger never sets); the popups' (#796): `combo_touch_*.ron`, `filter_combo_touch_*.ron` and `color_palette_combo_touch_*.ron` (a tap opens, a second tap closes, a tap on an option picks, a tap outside closes, a tap on the popup does not reach what is beneath); the `TouchTaps` resource counts the `Activate`s of `compact button` and `no hover button` (end of the `Buttons` screen) and of `ButtonNames`' link and color button, and the `IconButtons` screen ends with a `disabled toggle button`; the other widgets' touch
work is the rest of epic #793).

**Long press on a `Tooltip` (#802).** A finger held `TOOLTIP_DELAY_SECS` on a `Tooltip` node without moving past `DRAG_THRESHOLD` shows the popup (above the node when it fits). When it shows, the press is canceled (`Pressed` removed, a `MouseKind::Cancel` for the look), so the lift fires no `Activate` and the widget picks nothing; a short tap is unchanged. See [`tooltip.rs`](../modules/tooltip.md).

**Drag controls (#799).** `MouseEvent.pointer` says which pointer (the mouse or one finger) sent an event. A `NumericStepper` scrubs with the pointer that went down first (`Scrub.pointer`); a second finger's press and drag are ignored, so it neither starts another scrub nor moves or ends the first. `end_scrub` reads `Touches`, so a lift or a cancel of the last finger releases `HoverLock` and the cursor. The headless `Slider` ignores `PointerCancel`, so the library's `end_canceled_presses` clears its `Pressed` and drag state when `Touches` reports a canceled finger. Scenarios: `slider_touch_*.ron`, `numeric_stepper_touch_*.ron` (scrub, tap, second finger, releases hover lock), `scroll_area_touch_thumb_*.ron`. The scenario tester's `TouchCancel` despawns the pointer (an `Out`), it sends no `PointerCancel`.

## "Is the pointer on it?" is picking's `Hovered`

A system that asks where
the pointer is now, outside a `MouseEvent`, reads `bevy::picking::hover::Hovered`
(required by the component it asks about): true while the mouse pointer is on
the entity or on any of its descendants, as picking's `HoverMap` says. The
`selection_change` of
`list.rs` and `color_palette.rs` on each `ItemView`, to keep the hover color of
the item under the pointer. Bevy's `RelativeCursorPosition` is not used: it is
only computed for cameras that render to a window, so UI drawn to an image (the
recorder, any render-to-texture UI) never had the pointer on it (#100).
`Hovered` follows the hover map, not the geometry: a node is only hovered when
nothing above it that blocks the pointer (`should_block_lower`) is in the way,
so an item under an open popup is not. A node with `should_block_lower: false`
(a combo's button, an `ItemView`) is hovered and lets what is below be hovered
too; a `Pickable::IGNORE` node (an item's label) is never in the hover map, so
only the node it lies on counts. It follows `PointerId::Mouse` only, not touch; the combo popups read the hover map of the pointer that released instead (`popup::PopupPointers`). It is updated in `PreUpdate`,
so an `Update` system sees this frame's pointer.

## Buttons are `bevy::ui_widgets::Button`

Every button
(`PushButton`, `ColorPushButton`, `Link`, `IconButton` and the rest) is a
headless `bevy_ui_widgets` button, and nothing uses the deprecated `bevy::ui`
`Button` or `Interaction`. The scroll thumb is no button: a `bevy_ui_widgets`
`ScrollbarThumb` without a `Node` of ours, sized and placed by its `Scrollbar`,
that requires `Pickable::default()` so it blocks the pointer and gets the
`MouseEvent`s of its hover color.
A button: `Pressed` while held, `Activate`
on release over the button and on Enter / Space for a focused one, never while
`InteractionDisabled`. `WidgetsPlugin` adds `ButtonPlugin` (and `ListBoxPlugin`, `ScrollbarPlugin`, `SliderPlugin` and `TabPlugin`) in
its `Plugin::finish`, unless the app has it by then (`DefaultPlugins` and the
tester's `HeadlessPlugins` add all of `UiWidgetsPlugins`; adding twice panics),
so the order of `WidgetsPlugin` and `UiWidgetsPlugins` does not matter; an app
driven by hand with `App::update` must call `App::finish` first. A
`ColorPushButton` with `InteractionDisabled` has its border in `text_disabled`
(it has no hover colors to ignore). **`Activate` and `MouseEvent`:**
"the button was used" is `Activate`, so an action observes `Activate` on the
button itself: it does not bubble (`bevy_ui_widgets`' `Activate` has no
`propagate`, `an_activate_does_not_bubble_to_an_observer_on_an_ancestor`), so an
observer on an ancestor never hears it; how the button *looks* under the
pointer (hover and press colors) is `MouseEvent`. Never do both for one
thing. A disabled `PushButton` (insert
`InteractionDisabled`) shows its text in `ThemeColors::text_disabled` and
ignores hover. A `Link` is disabled the same way: insert `InteractionDisabled`
to disable it and remove it to enable it again (there is no `LinkState`); it
keeps its `Button` and `Role::Link`, shows plain text in `text_primary`, ignores
hover and press, and `restyle_on_disabled_change` in `link.rs` restyles it when
the marker is added or removed, also under the pointer.
The crumbs of `BreadCrumbs` observe `Activate`; the current crumb is a
`Link` with `InteractionDisabled`.
