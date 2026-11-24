# `float.rs`

`Float`: the viewport-placed popup layer, and the helpers that keep popups inside the window.

Read this before changing `src/float.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [popups](../design/popups.md).

## Details

`Float`: requires Bevy's `FixedNode`, so it is a child of its widget but placed by its `left` / `top` in the viewport (logical pixels) and takes no layout, clipping or transform from its ancestors: a `ScrollArea` around the widget does not clip it. `GlobalZIndex(1000)`, large shadow; the popup layer of `Combo`, `ColorPaletteCombo` and `Tooltip`, each of which places its own, `Combo` and `ColorPaletteCombo` through the crate-private `keep_in_window_x` and `below_or_above` of `float.rs` (10 px `WINDOW_MARGIN` from the window's side edges, above the button when it would run off the bottom; `fit_vertically` returns the `top` and a `max_height`: when the popup fits on neither side it goes on the side with more room and is capped to that room, never below `MIN_POPUP_HEIGHT` (100 px) except in a window shorter than that, and scrolls with the wheel (`overflow.y` is `Scroll`), staying inside the window's top and bottom margins; the cap is always the chosen side's room, so the next frame's measured height picks the same side; keys in a capped list do not scroll it, `scroll_into_view` skips a `FixedNode`; `combo_taller_than_window_stays_inside_window.ron` on the `long combo`) (`combo_in_scroll_area_popup_is_not_clipped.ron`, `color_palette_combo_in_scroll_area_popup_is_not_clipped.ron`)
