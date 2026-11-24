# `lib.rs`

`WidgetsPlugin` (the one plugin an app adds), `UiRoot`, and the crate-private `MouseEvent` every widget reacts to.

Read this before changing `src/lib.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [pointer, touch and buttons](../design/pointer-and-buttons.md).

## Details

`WidgetsPlugin` (add this one plugin to an app), `UiRoot`, and `MouseEvent` (with `MouseKind::Cancel`, `touch` and `hovered()`, see [touch](../design/pointer-and-buttons.md#touch-793-795)) with the observers that produce it (the crate-private `HoverLock` resource drops `MouseKind::Over` while a `NumericStepper` scrub drag is on, so no other control highlights)
