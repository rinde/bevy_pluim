# `color_palette_combo.rs`

`ColorPaletteCombo`: a `ColorPalette` in a combo-style popup.

Read this before changing `src/color_palette_combo.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [popups](../design/popups.md), [lists and selection](../design/lists-and-selection.md).

## Details

`ColorPaletteCombo`: a `ColorPalette` inside a combo-style popup; the popup, a `Float` child of the combo, is centred under the button when it opens, 10 px inside the window, above the button when it would run off the bottom; its palette gets `Selection::Ref` to the combo's selection when it is spawned, and `SubRefs` keeps its entity. Its button is a `Role::ComboBox` like a `Combo`'s, without a label, its value the colour names of the selected items that have an `Item<Color>` joined with ", " in selection order (empty for none, also a pending mandatory selection), `controls` the popup's palette. Over a selection of many (`OneOrMore`, `ZeroOrMore`) the button's square shows the first selected colour in selection order (`Srgba::NONE` when none is usable) and nothing else (no split swatch or counter); a pick or Enter / Space leaves the popup open, Escape, an outside click, focus loss and the button still close it `ColorPaletteComboState` (`Open` / `Closed`, immutable, with `inverse()`; formerly `State`), `ColorPaletteComboRefs` on the combo (private fields, getters `button()`, `popup()` and `palette()`; immutable, reflected, built only by the widget; the routing records on the button and palette stay private), and the public reflected markers `ColorPaletteComboButton` (named `ColorPaletteComboButton`) and `ColorPaletteComboFloat` (the popup, named `<combo name> popup`) It shares the button colours, the open/close side effects, the outside-click rule and the combo box node with `Combo` through `popup.rs`, and keeps what differs: the float always counts as inside, a swatch click closes it, it runs `position_float` on open, and its visible part is the float while its focus target is the palette..

**Edited in place (#691):** `sync_edited_item_colors` (`Changed<Item<Color>>`) refreshes the button square when the first selected item's colour is edited, not only on a `Selection` change.

**Lazy popup (#680):** opening wakes the palette before layout and keeps the popup `Visibility::Hidden` until `place_opened_popups` (`PostUpdate`, after `UiSystems::PostLayout`) has placed it, so it appears one frame after the click (scenarios `Wait(1)` after opening by click); closing hides it and a `PendingSleep` marker despawns the swatches in `Last`, after the `ListBox`'s `ValueChange` of the click that closed it was handled. A closed popup has no swatches, so scenarios cannot check swatch state there (`a_closed_color_palette_combo_builds_no_swatches_until_it_opens`, `despawning_color_palette_combos_open_or_dormant_leaves_no_swatches_behind`)
