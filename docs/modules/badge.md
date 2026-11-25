# `badge.rs`

`Badge`: a count or warning pill in the corner of a button.

Read this before changing `src/badge.rs`. When the module's behavior changes, update this file in the same PR.

## Details

`Badge { text, tone }` (`BadgeTone::Neutral` or `Warning`; `Badge::count(n)`): a 16 px high pill (min width 16, 4 px side padding, 10 px semibold text; height, min width and radius are `Val::Rem`, so the pill grows with its text when `RemSize` grows, which in pixels clipped the count) spawned as an absolute, pointer-ignoring child in the top-right corner of its host, on an `IconButton` or on the `Item<Tool>` of a `Toolbar` (mirrored onto that item's button). Neutral uses `pill_background` / `pill_text`, warning `warning_text` as fill with `warning_background` text. Immutable: insert a new `Badge` to change it (on an `Item<Tool>` too, `mirror_item_badges` copies it); remove it, or give it empty text, to hide it (no overlay, host description cleared; a nonempty one brings it back); the badge node is not in the accessibility tree, its text is the `description` of the host's `AccessibilityNode` (cleared when the badge is removed or empty). `mirror_item_badges` and `sync_badges` look only at what changed: hosts whose `Badge` changed or was removed, new nodes, changed `AccessibilityNode`s (Bevy rewrites them) and the buttons of a changed or new `Item<Tool>` or new `ToolbarButton`; a `Theme` change looks at all, a quiet frame does no work (`a_badge_follows_its_item_to_the_buttons_made_when_the_toolbar_style_changes`, `a_theme_change_recolors_a_badge_that_did_not_change`; `badges-idle` and `badges-changing` bench workloads, #685)
