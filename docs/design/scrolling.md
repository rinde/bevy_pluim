# Scrolling

Scroll areas: focus-ring room, natural content size, the wheel, a finger's pan and scroll-into-view.

When the behaviour described here changes, update this file in the same PR.

## A scroll area leaves room for a focus ring

`ScrollContents` has 3 px of
padding on every side (`FOCUS_RING_REACH`: `focus.rs`'s `FOCUS_RING_WIDTH` of 2 px
plus `FOCUS_RING_OFFSET` of 1 px, which `draw_focus_ring`, `scroll_area.rs` and
`virtual_list.rs` all use), because the area clips at its padding box and the ring of a row at
the edge would be cut off (#293). `scroll_into_view` counts the same 3 px as
part of the entity, so a row scrolled into view shows its ring. The maximum
scroll of every area is 6 px more than the contents' overflow alone.

## Scroll area contents keep their natural size

`ScrollContents` uses
`align_items: FlexStart` instead of stretching its children. Taffy 0.10
(Bevy 0.19) reported no overflow for a grid whose items overflow it, so a
`List` or palette stretched to the area's height left nothing to scroll;
taffy 0.14 (Bevy 0.20) was tried (#265): with `Stretch` the content still
scrolls, but `list_wheel_scrolls` and
`scroll_area_keyboard_scrolls_item_into_view` (both scale factors) fail, the
maximum scroll 2 px short (296, not 298), so the workaround stays. For the
same reason, don't give scroll contents `height: 100%`. The
`scroll_area_*.ron` scenarios guard this on taffy 0.10: put `Stretch` back
and the wheel and thumb scenarios failed there.

## A scroll area's scroll position is kept in range

The wheel handler
(`update_scroll_position` in `scroll_area.rs`) moves only an axis whose
`Node::overflow` is `Scroll` and leaves `ScrollPosition` inside `[0, max]`
per axis, `max` being how far the contents overflow in logical pixels (0 when
they fit); a node that does not scroll along an axis is not moved along it,
except that, after the Ctrl swap of the wheel's two deltas, a node that scrolls
horizontally but not vertically takes an event with no horizontal delta
(`dx == 0`) as a horizontal scroll by its vertical delta. **Nested areas:** the wheel
goes to the entities in picking's hover map, and a `ScrollContents` blocks the
pointer for what is below it, so over a nested area only that one scrolls: the
area around it is not hovered and gets nothing, also when the inner one is at its
limit (nothing is passed on). The outer area scrolls where the pointer is on it
beside the inner one (`scroll_area_wheel_nested_horizontal_in_vertical.ron`, the
`outer scroll area` of the `ScrollAreas` screen).
A scenario reads `ScrollPosition.0.y`, the value that was set (logical
pixels). `ComputedNode.scroll_position.y` is what the layout shows, floored,
in physical pixels and one layout pass behind, so it doubles at a scale
factor of 2 and scenario files do not read it. A thumb drag and a click on
the track are the `bevy_ui_widgets` `Scrollbar`'s, which clamps to the same
range; a click on the track beside the thumb pages by one visible length
towards the click (`scroll_area_track_click_pages.ron`). The wheel stays
ours rather than `bevy_ui_widgets`' `ScrollArea`:
picking sends a `PointerScroll` to every hovered entity and each bubbles up to
the area, and an `ItemView`, its `List` and the `ScrollContents` are all
hovered (`should_block_lower: false`), so one turn would scroll two or three
times; it also scrolls by `MouseScrollPixelsPerLine` (100 px a line, not our
21) and has no Ctrl axis swap.

A one-finger pan (`pan_with_finger`, #800) moves the same `ScrollPosition` through
the same `scroll_by`, so it is clamped to `[0, max]` per scrolling axis alike and writes
only a position that moves. It goes to the nearest scrollable ancestor of the node the finger
went down on (found on the press, a `FixedNode` ending the search), never past a limit, and
does not start on a `Slider`, `NumericStepper` or `ScrollBar`. A press that moves more than
`DRAG_THRESHOLD` becomes a pan and ends there: the buttons it went down on lose `Pressed`, a
`Cancel` restores their look, and `TouchPans` makes the producer in `lib.rs` drop the
`MouseKind::Click` that would follow and the combos ignore its lift, so nothing it started on
is activated, picked or closed.

A pan that lifts while moving flings (#801): the velocity of the last 100 ms per axis, if at
least 100 px a second, becomes a crate-private `Fling` on the area, and `step_flings` decays it
by 0.998 per millisecond of elapsed time (exact integration, so the distance does not depend on
the frame rate) until it is under 30 px a second, clamping per axis through `scroll_by`. A new
touch, a wheel turn, a scroll-bar drag or any other change of the `ScrollPosition` stops it; a
touch that stops it is not a tap. A step that moves the position closes an open combo in the
area, like the wheel; one that does not writes nothing. Non-goal: no rubber band or bounce past
the limits, the fling stops at `[0, max]`.

## A scroll area scrolls a focused row into view

`scroll_into_view` in
`scroll_area.rs` observes `bevy_ui_widgets::ScrollIntoView`, which a `ListBox`
triggers when the arrow keys, Home or End move its active descendant. It finds
the `ScrollContents` above the entity and moves its `ScrollPosition` by the
least that shows the entity, per axis only where the contents scroll and within
`[0, max]`; an entity outside any scroll area is left alone, and so is one in a
`FixedNode` (a combo's popup) inside one, since scrolling the area would not
show it; a `FixedNode` that itself scrolls (a `Combo` or `ColorPaletteCombo` popup
capped to the window height) has its own `ScrollPosition` moved instead, never the
area's (`combo_taller_than_window_keyboard_scrolls_the_list.ron`). It reads the
entity's laid-out position, so it is one layout pass behind like the wheel's
`ComputedNode`. The wheel is untouched, and so is the `Scrollbar`, which
follows the `ScrollPosition` it sets
(`scroll_area_keyboard_scrolls_item_into_view.ron`).
