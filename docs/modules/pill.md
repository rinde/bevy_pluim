# `pill.rs`

`Pill`: a small uppercase status tag.

Read this before changing `src/pill.rs`. When the module's behaviour changes, update this file in the same PR.

## Details

`Pill::new(text)` with a `PillTone` (`Neutral`, `Info`, `Warning` = waiting, `Problem`): a small uppercase status tag; the app picks the tone; immutable, re-insert a `Pill` to change tone or text (empty text keeps the empty pill); a `Role::Status` accessibility node labelled with the text as given
