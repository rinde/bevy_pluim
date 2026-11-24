# `tooltip.rs`

`Tooltip`: a popup shown after the pointer rests on a node.

Read this before changing `src/tooltip.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [popups](../design/popups.md).

## Details

`Tooltip { text }`: after the pointer (`Hovered`) rests on the node for `TOOLTIP_DELAY_SECS` (0.5 s) a `TooltipPopup` `Float` (`card_surface`, `border`, small `text_primary`) is spawned as a child of the node, hidden until laid out, then centred under the node, kept 6 px inside the primary window and put above the node when it would run off the bottom, its `top` never above the 6 px margin; a `Tooltip` with empty text shows no popup, and a showing one goes when the text becomes empty; it never takes the pointer. Leaving the node or a `Pressed` hides it, and a press keeps it hidden until the pointer has left and returned. Only active tooltips are visited: `track_active_tooltips` puts the private `TooltipActive` marker on a node when its `Hovered` or `Tooltip` changes and it is hovered, `update_tooltips` queries only marked nodes and takes the marker off once the pointer is gone and no popup shows, so a tooltip the pointer is not on is not visited (a hovered one with empty text still is, while the pointer stays; the marker arrives through `Commands`, so a hover is first counted in the frame after it starts), guarded by the `tooltip.rs` unit tests `only_a_hovered_or_showing_tooltip_is_visited` and `an_unmarked_tooltip_is_not_visited_in_a_quiet_frame`. While a `NumericStepper` scrub drag holds the crate-private `HoverLock`, no tooltip shows or counts its delay and a showing one goes (`a_tooltip_stays_hidden_while_a_numeric_stepper_is_scrubbed`). `IconButton` inserts one with its name; the buttons of an underlined `Toolbar` (labels hidden) get one with their tool's label.
