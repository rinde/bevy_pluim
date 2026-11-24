# `link.rs`

`Link`: clickable text, a button with `Role::Link`.

Read this before changing `src/link.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [pointer, touch and buttons](../design/pointer-and-buttons.md).

## Details

`Link::new(label)`: clickable text, a `Button` with `Role::Link` and a tab stop that triggers `Activate`; `InteractionDisabled` is its one disabled state (insert to disable, remove to enable; `restyle_on_disabled_change` restyles on either): a disabled link is plain `text_primary` text, ignores the pointer and is skipped by Tab (`link_disabled_is_skipped_by_tab.ron`). Insert a `Link` again to change its label. Hover and theme find the label by its private `LinkText` marker among the children, not by position, so an app child before it is kept and does not stop the label from restyling (`a_link_with_an_app_child_before_its_text_still_themes_hovers_and_disables`)
