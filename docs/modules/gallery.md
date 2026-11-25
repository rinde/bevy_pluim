# `gallery.rs`

`Gallery`: grouped picture cards that pick one item.

Read this before changing `src/gallery.rs`. When the module's behavior changes, update this file in the same PR.

Cross-cutting design: [keyboard focus and navigation](../design/keyboard-focus.md), [lists and selection](../design/lists-and-selection.md).

## Details

`Gallery` over a `One` `Selection` (own or `Selection::Ref`) of `Item<GalleryCard>` (`GalleryCard { image, label, group }`): neighboring items with the same `group` form a group, an overline `Label` over a row of 104 x 92 px `GalleryCardView`s (6 px gap; each a `Button` that picks its item on `Activate` with `toggle_selected`; the gallery is one tab stop with arrow keys through the cards in the order shown, a `RovingGroup`), groups side by side with a vertical `Divider` between them. A card is a `card_surface` with a 1 px `border` and control radius, a 72 x 30 `Icon` image above a 12 px semibold label that wraps; hover is `hovered_item`; the picked card has a 2 px `selection_ring` border and the `hovered_selected_item` background. The gallery does not shrink, so put it in a horizontal `ScrollArea` to scroll it. Changing the items or their groups rebuilds the cheap group scaffolding (dividers, overline labels, rows) but keeps the card of every item that stays, re-parented into its new row, so its entity, focus and hover survive (`plan_views` over the `GalleryOwned` descendants; `a_structural_change_to_a_gallery_keeps_the_cards_of_the_items_that_stay`; `gallery-editing` bench workload, #690); changing a `GalleryCard` updates in place.

**Child ownership:** the gallery owns only the groups and dividers it spawns, marked with the crate-private `GalleryOwned`; a rebuild despawns just those, so children the app puts under the gallery entity (before the first build or later) are kept, with their state. Those present before the first build stay before the gallery's own; a child added later is not reordered, and a rebuild appends the new groups after it (`a_gallery_keeps_children_of_the_app_through_the_first_build_and_rebuilds`). `sync_cards` looks only at the widgets `StructureChanges::affected` names, so a quiet frame builds no groups to compare. To a screen reader the gallery is a `Role::RadioGroup` and each card a `Role::RadioButton` labeled with its item's label, toggled while picked

Touch: a tap picks as a click does, a finger that slides off before lifting picks nothing, and no hover look stays after the lift (a finger never hovers); the focus, and so the roving stop, follows the tapped item (`gallery_touch.ron`).
