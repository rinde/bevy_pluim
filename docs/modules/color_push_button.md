# `color_push_button.rs`

`ColorPushButton`: a button showing a color square.

Read this before changing `src/color_push_button.rs`. When the module's behavior changes, update this file in the same PR.

Cross-cutting design: [pointer, touch and buttons](../design/pointer-and-buttons.md).

## Details

`ColorPushButton`: a button showing a color square; its accessible name is the name of that color (same rule as a swatch), kept by a system because Bevy clears a text-less button's label on add; a `ColorPaletteCombo`'s button is not, its color is its value instead
