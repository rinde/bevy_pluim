# `color_palette.rs`

`ColorPalette`: a grid of colour swatches over a selection of `Item<Color>`.

Read this before changing `src/color_palette.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [lists and selection](../design/lists-and-selection.md).

## Details

`ColorPalette`: a grid of colour swatches (`ListItem`s, `Selected` when in the selection) over a selection of `Item<Color>`; a swatch's accessible name (label of its `AccessibilityNode`, kept up to date) is the name of its colour, `tailwind::color_name`; its `on_items_changed` reconciles linearly like a `List`'s (a set of wanted and a set of shown items, a second swatch of one item is despawned, #679; `palette-items-adding` bench workload): the Tailwind name (`blue-500`) when it equals one of `COLORS`, else its lowercase sRGB hex (`#3b82f6`, alpha appended only when not 1).

**Edited in place (#691):** `sync_item_colors` (change-driven, `Changed<Item<Color>>`) rewrites the square, highlight and hover state of only the swatches of the changed items, without rebuilding them; the accessible name already follows (`editing_an_item_colour_in_place_updates_its_swatch_highlight_combo_preview_and_name`).

**Dormant while closed (#680):** a palette that a `ColorPaletteCombo` owns carries the crate-private `PaletteDormant` while its popup is closed and has no `ItemView` children (`on_items_changed` returns early); `wake` removes the marker and builds the swatches, `sleep` inserts it and despawns them, so selection and items stay on the combo and a reopened popup shows the current selection and items
