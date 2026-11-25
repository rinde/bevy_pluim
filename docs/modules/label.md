# `label.rs`

`Label`: themed text in the styles of the type scale.

Read this before changing `src/label.rs`. When the module's behavior changes, update this file in the same PR.

## Details

`Label` with `Title` (`Label::title`, also `Label::header`), `Regular`, `Secondary`, `Caption` and `Overline` (shown in upper case, no letter spacing), sized and colored from the theme's type scale and text slots; it requires Bevy's `ui::widget::Label`, so it is a `Role::Label` accessibility node with its text as value, and the hook inserts that marker again so a new `Label` updates the value; immutable (`#[component(immutable)]`, with `Kbd`, `Pill`, `Toast` and `Badge`: build or clone a value and insert it to update, compile-fail doc tests pin it); empty text keeps the node and role styling
