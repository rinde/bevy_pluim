# `icon.rs`

`Icon`: an app-provided image tinted with a theme role.

Read this before changing `src/icon.rs`. When the module's behaviour changes, update this file in the same PR.

## Details

`Icon::bundle(handle, size)` / `Icon::bundle_with_color`: an `ImageNode` from an app-provided white-on-transparent `Handle<Image>`, tinted with the `IconColor` theme role of its `Icon` (primary text by default); a widget changes the role by inserting `Icon` again
