# Audit 432: selection widgets

Scope: `segmented_control.rs`, `tab_bar.rs`, `filter_chips.rs`, `gallery.rs`, `toolbar.rs`. No code changed.

| Finding | Issue |
| --- | --- |
| Bug: `SegmentedControl` and `TabBar` never update a segment's or tab's text or order when an item changes | #479 |
| Bug: `Toolbar::sync_tooltips` overwrites an app's `Tooltip` | #480 |
| Invalid states: `ToolbarButton` duplicates the toolbar style, `ToolbarButtonRefs` holds style-dependent `Option`s | #483 |
| Inconsistency: `SegmentedControl` and `TabBar` duplicate the items-changed machinery (`Local` map of observers) | #485 |
| Inconsistency: `FilterChips`, `Gallery` and `Toolbar` rebuild `Vec`s of all items every frame | #488 |
| Inconsistency: refs and marker components not `Reflect` or in `tests/reflection.rs` | #489 |
| Encapsulation: back-reference fields named differently and all `pub` (needs human review) | #492 |
| Inconsistency: only `TabBar` roves, the others make every item a tab stop (needs human review) | #493 |
| Missing tests: no pointer or keyboard scenario files for `Toolbar` and `Gallery` | #494 |

Dismissed: no `position_in_set` on `Gallery` and the radio groups (promised only for `VirtualList`); no arrow navigation in `FilterChips` (documented, see #493); unenforced `One` selection (the selection variants decide, no invalid state); hook vs `#[require]` setup (matches the rule).
