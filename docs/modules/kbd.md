# `kbd.rs`

`Kbd`: a display-only key cap.

Read this before changing `src/kbd.rs`. When the module's behaviour changes, update this file in the same PR.

## Details

`Kbd::new("Esc")`: a display-only key cap, a bordered box (`ThemeColors::border`, `text_secondary` text in the caption size; it reuses those slots and has none of its own); a `Role::Label` accessibility node with the key as value; immutable, re-insert to change the key; empty text keeps the empty box
