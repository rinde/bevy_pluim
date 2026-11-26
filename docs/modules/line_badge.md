# `line_badge.rs`

`LineBadge`: short text on a circle or pill in a color the application gives.

Read this before changing `src/line_badge.rs`. When the module's behavior changes, update this file in the same PR.

## Details

`LineBadge::new(text, color)` with `with_size(LineBadgeSize)` and `with_label(..)`: the badge of a train line, such as `L3`, `IC12` or `7` (up to five characters; the text is shown as given, bold, centered, never wrapped). Up to `LINE_BADGE_MAX_CIRCLE_CHARS` (3) characters, counted with `chars()`, it is a circle (`width` = `height`); longer text makes a pill of the same height (`width` auto, `min_width` = height, side padding of 0.4 of the height), so the text keeps its size and is not clipped. `is_circle()` says which. Heights are `Val::Rem` and the text is a rem `FontSize`, so the badge follows `RemSize`: `Regular` (default, for list rows) is 2 rem high with 14 px text at the default rem size, `Small` (inline) 1.5 rem with 10 px text.

The fill is the `color` field and does not follow the theme. The text color is white or near-black (Tailwind gray-950), whichever has the higher WCAG contrast ratio with the fill (`LineBadge::text_color_for`, built on `theme::readable_ink`), so the badge reads in both themes. A `Theme` change restyles only the font (`theme_line_badges` writes only on a difference).

For a screen reader it is a `Role::Image` labeled with `label`, or with the text when there is none. Display only and pointer-transparent. Immutable: insert a new `LineBadge` to change text, color, size or label.

Tests: `a_line_badge_is_a_circle_up_to_three_characters_*`, `a_line_badge_text_is_white_or_near_black_*` and `a_line_badge_is_an_image_labeled_*` in `tests/headless.rs`, unit tests in the module, and the `LineBadges` screen with `line_badge_is_a_circle_*.ron`.
