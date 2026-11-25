# `card.rs`

`Card`: themed background, border, radius and shadow; the smallest theming example.

Read this before changing `src/card.rs`. When the module's behavior changes, update this file in the same PR.

## Details

`Card`: a themed background, border, radius and shadow on a node; a `Role::Group` without a label (`themed_card_accessible_group.ron`). A `ScrollArea` at its bottom edge needs `ClearOfCorners` to stay clear of the rounded corners (see `scroll_area.rs`)
