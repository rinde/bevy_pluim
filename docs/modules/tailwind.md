# `tailwind.rs`

The 242 Tailwind colors and `color_name`, for color pickers and accessible names.

Read this before changing `src/tailwind.rs`. When the module's behavior changes, update this file in the same PR.

## Details

`COLORS`, all 242 Tailwind colors as `TailwindColor { class, variant, color }`, and `color_name(color)` (Tailwind name, else hex: the accessible name of swatches and color buttons, and what `WidgetsCheck::Selected` uses), for color pickers (the order is classes slate to rose, shades 50 to 950, so `Nth(60, ..)` is red-500; `zinc-50` and `neutral-50` are the same color, which `color_name` calls `zinc-50`; guarded by unit tests); the theme itself uses `bevy::color::palettes::tailwind` directly
