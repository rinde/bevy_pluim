# `breadcrumbs.rs`

`BreadCrumbs`: a trail of `Link`s for a `OneOrMore` selection; a click trims the selection.

Read this before changing `src/breadcrumbs.rs`. When the module's behavior changes, update this file in the same PR.

Cross-cutting design: [lists and selection](../design/lists-and-selection.md), [pointer, touch and buttons](../design/pointer-and-buttons.md).

## Details

`BreadCrumbs`: a `Link` trail for a `OneOrMore` selection; clicking a crumb trims the selection back to it. Adding, editing or removing the `Item<String>` of a selected item rebuilds the trail in full (`Changed<Item<String>>` and removals; the selection need not change), so labels, the current crumb and the position a click trims to stay current, also through a `Selection::Ref` (`bread_crumbs_follow_item_text_changes_and_trim_to_the_current_position`). Items without an `Item<String>` get no crumb; the last crumb built is the current one, a `Link` with `InteractionDisabled` (a crumb that stops being the last is built again without it), and a click trims to that crumb's position in the selection. `BreadCrumb` marks a crumb
