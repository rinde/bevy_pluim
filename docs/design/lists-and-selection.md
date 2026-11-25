# Lists and selection

How lists, item views and selections fit together. The selection model itself is in [`select.rs`](../modules/select.md).

When the behavior described here changes, update this file in the same PR.

## A list can render its own item views

`ItemViewCreator` (next to the
`List`, a registered system, like `Stack`'s `StackChildSpawner`) is run once
per item with an `ItemViewSpawner { item, view }` and spawns the contents
under the `ItemView`, which the list still owns, spawns and despawns with
`ItemsChanged` and themes. The items need not be `Item<String>`. Such a view
is `CustomView`: its children take the pointer, so hover colors come from
`Hovered` (`custom_view_hover`) and press and release from a global
`MouseEvent` observer that walks up from the target to the view
(`custom_view_of`), ignoring a target inside a nested `Button`, `List`, `Combo` or `FilterCombo`
(which handle their own press). A `Button` stops the click that hits it, but a
combo's root node and popup let the pointer through, and their click bubbles
through the row to the outer `ListBox`, which takes any click under a row as a
pick of that row; so each custom view stops a click whose original target lies
in a nested `Button`, `List`, `Combo` or `FilterCombo` (`keep_nested_clicks_from_row`,
`list_combo_in_custom_row_pick_leaves_outer_list.ron` and
`list_filter_combo_in_custom_row_pick_leaves_outer_list.ron`, the `ComboRows` and
`FilterComboRows` screens). Selection and `Selected` are as
for default views.

## Lists are `ListBox`es, and `Selection` keeps the last word

A `List` is a
`bevy::ui_widgets::ListBox` and its `ItemView`s are `ListItem`s (`Selectable`,
`Role::ListItem`); `WidgetsPlugin` adds `ListBoxPlugin` unless the app has it.
`Selected` is Bevy's marker for an item in the selection, and `selection_change`
in `list.rs` keeps it in step with the `Selection` (own or `Selection::Ref`), so
it is on exactly the items of the selection, and `ListBoxMultiSelect` is on the
list while the selection allows more than one item (`OneOrMore`, `ZeroOrMore`).
**The choice:** the `ListBox` observers send `ValueChange<Entity>` for a click
on a row (except the first selected row, see below), and for Enter / Space on
the active row, and insert and remove `Selected` only if the app registers `listbox_update_selection`.
We do not register it: `on_value_change` turns the `ValueChange` into a
`SelectionParam::update_selection` instead, so `One`, `ZeroOrOne`, `OneOrMore`
and `ZeroOrMore` (and `None` for a disallowed change) keep deciding, and
`Selected` can never disagree with the `Selection`. The `ListBox` ignores a click
only on the first selected row (`listbox_on_row_click` compares the row with the
current one); on any other selected row it still sends the `ValueChange`, and a click
on the first selected row, like Enter or Space on an active row that is already
`Selected`, sends a `ReselectListRow` instead. `on_value_change` and `on_reselect_row`
(`list.rs`) both go through `toggle_selected`, so a click or a key on a selected row
deselects it as far as the selection allows (`One` keeps its item) and the `ListBox`'s
events are the only path: no `MouseEvent::Released` deselects, or a click on the first
selected row would toggle twice (`list_click_deselects_once.ron`,
`list_keyboard_deselects.ron`, `color_palette_keyboard_deselects.ron`). A tap is a click here: the same path picks and deselects once by touch (`list_touch_taps_pick.ron`, `list_touch_looks.ron`, `list_touch_custom_views.ron`, `virtual_list_touch_taps_pick.ron`, `virtual_list_touch_separated.ron`, `color_palette_touch_taps_pick.ron`), and a tap on a nested button, list, combo or filter combo in a custom row is not the row's. Both ignore a row
whose `ItemView::list_ref` is not the event's source list: a nested list (a combo's popup
in a custom view) reports the row of the list around it, because `listbox_on_row_click`
searches above the popup when the click's target is the popup itself. Hover and
selected colors stay on `MouseEvent`. A `ListBox` also sets an
`ActiveDescendant` on a click and, once the list has focus (a `List` is one tab stop),
handles the arrow keys (all four move, wrapping round), Home and End by moving it, and
Enter / Space by sending the `ValueChange` of the active row, which goes through
`on_value_change` like a click, so the `Selection` decides, also a `Selection::Ref` and a
palette's. Moving the active row selects nothing.
The list's focus ring is drawn on the active row, not on the box (see "Keyboard
focus"). `ItemView`s keep `should_block_lower: false` so the wheel
reaches the scroll area, which also lets a click through to what is below: a popup
that floats over other widgets blocks the pointer itself (`Combo`'s list has
`Pickable::default()`).
A `ColorPalette` is a `List` with `NoDefaultItems`, so it is a `ListBox` of
`ListItem`s as well and follows the same pattern: `color_palette.rs` puts
`Selected` on its swatches (and `ListBoxMultiSelect` on the palette) from its
`on_selection_change`, using `list.rs`'s `sync_selected` / `sync_multi_select`,
`list.rs`'s `on_value_change` turns the `ListBox`'s pick into a selection update
for palettes too, and `on_reselect_row` deselects a swatch that is `Selected`.
A swatch has no text, so its accessible name is the name of its color (see
`color_palette.rs` in its [module doc](../modules/color_palette.md)); a `Role(role: ListItem)` locator without a label
still reaches all of them, so the scenarios use `Nth(..)` or
`App(Accessible(..))`.

## Selections are immutable components

Change one by going through
`SelectionParam::update_selection` / `update_selection_single`, whose
closures return `Option<SelectionValue>` from methods such as
`toggle_selected`, `push`, `pop`, `trim_to_length` and `set_selection`
(`None` means the change is not allowed for that variant). The new value is
inserted, which fires `OnInsert` observers.

## Shared selections

A widget holding `Selection::Ref(entity)` reads and
writes the selection on `entity`. `BreadCrumbs`, `List` and `Combo` accept
one; `Stack` takes the selection entity directly in `selection_ref`. **One hop
is the contract:** a `Ref` must point to an entity holding a `Selection::Value`.
A target that is missing, has no `Selection`, is the `Ref`'s own entity, or is
another `Ref` (a chain) resolves to nothing (`SelectionParam::find` returns
`None`, so `single` is an error and `get`, `get_ref`, `get_with_ref` return
`None`) and `find` logs an `error!` each time; chains are not followed
(`a_selection_ref_to_*` in `tests/headless.rs`).

## Shared items

`ItemsRef(entity)` lets several selections draw from one
`ItemValues`.
