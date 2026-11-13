# Working on bevy-pluim

Guidance for coding agents. This file explains the codebase to whoever is
changing it. If it and the code disagree, the code wins and this file gets
fixed.

## What this is

A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons,
lists, combos, scroll areas, tabs, breadcrumbs, colour palettes, a slider and a
numeric stepper, all themed from one `Theme` resource. It is used by nopjes,
but it is a library in its own right.

**No application or game state.** The crate takes Bevy types only, so its
widgets can be exercised without an application behind them. Anything that
knows what a picked item *means* belongs in the application that uses the
widget, not here.

## Core beliefs

1. **The repo is the source of truth.** If it is not in the code or this
   file, do not assume it. Read before you act.
2. **Read before write.** Read the module you are changing and a sibling
   widget before adding anything; match what is already there.
3. **Small, verifiable changes.** One PR does one thing.
4. **Docs stay in sync.** When behaviour or a convention changes, update this
   file in the same PR. A stale doc is worse than none.
5. **No gold-plating.** Implement what is asked; no abstractions, config or
   features "just in case".
6. **Explicit over implicit.** Name things clearly and let the type system
   carry the meaning.

## Issues and coordination

Work is tracked in GitHub issues. Issues ready for implementation use the
`ready for agent` label. When working on an issue, follow its title and
description; ask for clarification if its scope is genuinely ambiguous.
Changes should be made through pull requests.

Run `just ci` locally to check the same lint, test and unused-dependency
recipes used by CI.

## Where to find what

| Module | Holds |
| --- | --- |
| `lib.rs` | `WidgetsPlugin` (add this one plugin to an app), `UiRoot`, and `MouseEvent` with the observers that produce it (the crate-private `HoverLock` resource drops `MouseKind::Over` while a `NumericStepper` scrub drag is on, so no other control highlights) |
| `theme.rs` | `Theme` resource (colours, `body_font` and `display_font` handles defaulting to Bevy's font with the application loading its own, the `font_size_*` type scale, in logical pixels at Bevy's default rem size of 20 px, which every widget turns into `FontSize::Rem` with `Theme::font_size(px)` (its own constant sizes too), so the text follows Bevy's `RemSize`: an app that raises `RemSize` scales the widgets' text, at the default nothing moves; sizes tied to text stay `px` unless a `[text 1.5x]` scenario shows them clipping (only the badge is in rem), `control_radius` for buttons, list items, steppers and scroll bars and `panel_radius` for `Card` and `Float`), `ThemeColors` (also the text, divider, accent, danger, warning, info, pill, segmented, card and chip slots of the redesign, and `slider_track` / `slider_fill` for a `Slider`; both defaults use a dark `large_shadow` and `small_shadow`, black, so neither theme glows; both meet WCAG AA contrast (4.5:1 for their text and ink pairs, 3:1 for `focus_ring` and `selection_ring`), guarded by `the_default_themes_meet_wcag_aa_contrast`, which lists every pair a widget draws (an `ItemNote`'s ink on the item backgrounds is `list.rs`'s `note_ink`, which falls back to `text_primary`; `theme.rs`'s crate-private `contrast_ratio` is shared by both), which exempts `text_disabled`, `text_tertiary` and the decorative borders; the shadow constants set colour only, offsets stay with the widgets; `Pill` and `Badge` share `pill_background` / `pill_text`), `DEFAULT_LIGHT_THEME_COLORS` / `DEFAULT_DARK_THEME_COLORS`, `ThemeMode`, `Theme::with_colors(light, dark)` for an app's own pair, `set_mode` / `mode` / `light_dark_toggle` (switch the set in use) and `set_colors` (replace one half of the pair); the set in use is read with `Theme::colors()`, a getter, as the field is private so it cannot fall out of step with the mode (a compile-fail doc example guards that; reflection still reaches `Theme.colors`, as an inspection facility, not an invariant boundary, so scenario files keep their `Theme.colors` paths); `Theme::font_size(px)` returns `FALLBACK_FONT_SIZE_PX` (the body's 15 px, in rem) for a NaN, infinite, zero or negative size, since the public `font_size_*` fields can hold one and it is the boundary to text layout, `colors_of(mode)` / `set_colors(mode, colors)` (read or replace one half of the pair; the latter also updates `colors` when that half is in use, which is how the `ThemeEditor` edits a slot live), the shadow constants; also keeps `ClearColor` in step with the theme |
| `theme_editor.rs` | `ThemeEditor`: a `Card` (440 px wide unless the node says otherwise) that edits the `Theme` in use while the widgets follow. A mode label ("Editing the light theme") over a scroll area (it grows to 520 px, then scrolls) holding **one column** of rows (`rows_column`: the contents of a scroll area lay their children out in a row, which once put every row beside the first two); a "Colours" section with a row per colour slot that is opaque in both default themes (`editable_color_slots`, reflected from `ThemeColors`): a `ThemeColorSwatch(slot)` and a `ColorPaletteCombo` marked `ThemeColorSlot(slot)` over `tailwind::COLORS` (the items entity is a child of the editor, `ThemeEditorRefs::colors()`); a pick sets that slot of the half in use through `Theme::set_colors`; and one section per group of `theme_parameters()` with a `NumericStepper` marked `ThemeParameterEditor(name)` per number: radii, border, scroll bar and compact button sizes ("Shapes"), the `font_size_*` type scale ("Text sizes"), the four sides of `item_padding`, `compact_button_padding` and `color_button_padding` ("Padding") and the shadow offsets, spread and blur ("Shadow", through crate-private `Theme::shadow_style_mut`). A `ThemeParameter { name, label, section, min, max }` reads and writes one number (`value` / `apply`, clamped, a `Val` in `Val::Px`, a non-pixel `Val` reads as 0); `every_number_of_the_theme_has_a_row_in_the_theme_editor` walks `Theme` by reflection and fails for a new `Val`, `f32`, `UiRect` or `ShadowStyle` field without a row, so the editor keeps up with the theme. A step (`ValueChange`) sets the number, `show_theme` (when the `Theme` changed or a row was just built) brings swatches, steppers and the mode label back in step, also for changes made elsewhere. Not editable: the fonts (handles to assets) and the translucent shadow colours; there is no export (the `overview` page has it). The rows, labels and swatches are `Pickable { should_block_lower: false }` so the wheel reaches the scroll area over them (`update_scroll_position` only scrolls what is in the hover map; `theme_editor_steps_a_number.ron`). Tests: `WidgetsSetup::ThemeEditor` (a "Change theme" button and the editor), `theme_editor_rows_are_stacked.ron`, `theme_editor_picks_a_colour.ron`, `theme_editor_steps_a_number.ron`, and the `theme_editor` headless tests  A row never goes missing because of its value: a number outside its row's range gets a row whose stepper range is widened to include it (`ThemeParameter::range_in`; a step clamps to the same widened range), a number that is not finite (NaN, infinity) shows the row's minimum with a `warn!` naming the parameter and the theme keeps it until a step sets it, and a `Val` that is not in pixels reads as 0 and gets a `warn!` naming the parameter the first time `show_theme` reads it (once per parameter and kind, kept in a `Local`); the first step turns it into pixels (`a_theme_number_above_its_range_still_has_a_row_with_the_range_widened_to_it`, `a_theme_number_that_is_nan_still_has_a_row_at_its_minimum_and_a_warning_names_it`, `a_theme_number_that_is_not_in_pixels_shows_0_and_is_warned_about_once`) |
| `tailwind.rs` | `COLORS`, all 242 Tailwind colours as `TailwindColor { class, variant, color }`, and `color_name(color)` (Tailwind name, else hex: the accessible name of swatches and colour buttons, and what `WidgetsCheck::Selected` uses), for colour pickers (the order is classes slate to rose, shades 50 to 950, so `Nth(60, ..)` is red-500; `zinc-50` and `neutral-50` are the same colour, which `color_name` calls `zinc-50`; guarded by unit tests); the theme itself uses `bevy::color::palettes::tailwind` directly |
| `select.rs` | the selection model: `Selection` (`Value` or `Ref`), `SelectionValue` (`One` / `ZeroOrOne` / `OneOrMore` / `ZeroOrMore`), with two documented pending states, `One(Entity::PLACEHOLDER)` and an empty `OneOrMore` (`SelectionValue::is_pending`): a pending selection has no selected items (`iter()` yields nothing, `len()` is 0, `is_empty()` is true, `single()` and `last()` are `None`, `is_selected(PLACEHOLDER)` is false), so widgets treat it as empty without checks of their own; it keeps its cardinality, the first item repairs it through `ItemsChanged` (`repair_pending_selections`, no polling) and the update methods never clear an initialised `One` / `OneOrMore` back to pending; the `Selection::builder()`, `Item<T>`, `ItemOf` / `ItemValues` / `ItemsRef`, `ItemsChanged`, and the `SelectionParam` / `ItemParam` system params; the crate-private `StructureChanges::<Widget, T>` system param for a system that builds a child per item (`FilterChips`, `Gallery`, `Toolbar`): `affected()` names the widgets to look at in this frame and nothing else, so a change to one source does not make every widget compare its items. A widget is affected when it changed, when its root selection entity (after one `Selection::Ref` hop) or its items entity (`ItemsRef` or its own) had a changed `Selection`, `ItemValues` (also removed, when the last item went), `Item<T>` or `ItemOf`, an `Item<T>` was removed from an item of that source, or its own `ItemsRef` was replaced; widgets sharing a source are all named, an unrelated one never is, and a quiet frame does no work (`a_change_to_one_chips_source_leaves_independent_chips_alone`, `chips_sharing_a_selection_or_items_are_all_rebuilt_by_a_change_to_the_source`, `replacing_the_items_ref_moves_the_chips_to_the_new_source_and_removals_are_seen`, `a_pending_selection_is_repaired_when_its_items_arrive_and_the_chips_follow`; `SelectionParam::sources_of` resolves a widget's two entities; `SelectionParam::get_items` returns the items of a widget's source as a slice, empty for an existing source with no `ItemValues` yet (`get_item_values` is `None` there), which `List`, `VirtualList`, `SegmentedControl`, `TabBar`, `Toolbar`, `Gallery`, `FilterChips` and `ColorPalette` use, so replacing `ItemsRef` with such a source detaches the old observer, clears the views and the active row, and the source's first item appears later (`switching_to_a_source_without_items_clears_the_views_and_follows_its_first_item`, #695). `items_change` also triggers `ItemsChanged` with no items for a live entity that loses its last `ItemValues` (`RemovedComponents`), so widgets drawn from it clear instead of keeping stale views; a despawned source is not reported (its observers go with it). Covered per widget by `a_filter_chips_row_`, `a_toolbar_`, `a_gallery_` and `a_color_palette_clears_and_refills_as_its_source_is_replaced_emptied_and_restored`, `a_dormant_palette_woken_over_an_empty_source_has_no_swatches_until_its_first_item` and `a_list_drops_selected_from_its_views_when_cleared_and_gives_a_one_selection_its_first_item`; a `ColorPaletteCombo` re-watches its palette when the combo's `ItemsRef` is replaced; the `filter-chips-one-changing` bench workload measures it); `ItemsChanged::items` is one `Arc<[Entity]>` snapshot taken when the change is detected and shared by every observer (cloning the `Arc` copies no entities; `every_observer_of_one_change_shares_one_snapshot_of_the_items`, `a_snapshot_is_not_changed_by_a_later_mutation_of_the_items`; `items-shared-adding` bench workload, #681); the crate-private `plan_views(existing, wanted)` (matches existing views to wanted keys, returning a `ViewPlan` of reusable entities per slot and stale ones; the first view per key is kept) and `order_views(children, is_view, views, stale)` (the new children order, `None` when it is already right; the app's children keep their place) used by `FilterChips`, `Toolbar` and `Gallery` to reconcile instead of rebuilding (#690), and the crate-private `items_in_order(children, items, item_of)` (children in item order, the first child per item, then the children that show no item, the app's, in their order; shared by `order_tabs` and `order_segments`, linear) and the crate-private `watch_items` / `ItemsObserver` put one `ItemsChanged` observer on a values entity per widget (`SegmentedControl`, `TabBar`), recorded on the widget so it is despawned with it, also when the record is replaced (two `watch_items` calls before the first one's commands apply leave one observer), and with the values entity, never in a `Local` map; a `one()` / `one_or_more()` selection with no item yet waits in the private `SelectionResource::pending` and is repaired by an observer of `ItemsChanged` for its items entity (nothing polls it), and dropped by an observer of `Remove<Selection>`, so a despawned selection leaves no entry; ownership: `Item<T>::data` is public and app-owned (edit it in place, widgets observe the change), `ItemOf(pub Entity)` is Bevy's relationship input (insert it again to move an item, the `ItemValues` of the old and new owner follow; `ItemValues` has no mutable access), `ItemsRef(pub Entity)` is immutable (replace it, never edit it): an observer of `Discard<ItemsRef>` queues `refresh_selection_items`, which drops selected items that are not the new source's, gives a `One` / `OneOrMore` the new first item (pending, and repaired by `ItemsChanged`, when the source has none yet; the widgets draw it once its first items arrive, a source with no `ItemValues` having nothing to redraw from) and inserts the `Selection` again so the widgets redraw (`replacing_items_ref_refreshes_the_selection_and_the_widget`); the pending list holds one entry per selection; **order is significant** for `OneOrMore` / `ZeroOrMore`: `iter()` yields selection order, `trim_to_length` keeps the first items, and `set_selection` compares the items in order, so the same items in another order replace the selection (the way to reorder; `BreadCrumbs` follows) and only the same items in the same order give `None` (`bread_crumbs_follow_a_reordered_selection_and_trim_from_the_new_order`) |
| `stack.rs` | `Stack { selection_ref }` shows the one `StackItem` child matching the selection; a `StackItem` is either a creator (`StackItem::new`, optionally with an id through `new_with_reference`) or a reference to another item's creator by id (`new_reference`), a private enum behind the read-only `creator()` and `reference_id()` accessors (the fields are not public) with no child entity stored on it: the children a stack has made are cached per stack, in a private `StackChildren` component that `Stack` requires, keyed by the item whose creator made the child, so two stacks sharing a selection and items each build and show a child of their own and a despawned stack takes its cache with it (a cached child counts only while it is still a child of that stack; `update_stack_item` prunes the others when a new child is recorded; a reference resolves to the creator item by id and reuses that stack's child of it, never another stack's; `two_stacks_sharing_a_selection_each_get_a_child_of_their_own`, `two_stacks_sharing_a_selection_each_build_their_own_scene_child`, `despawning_a_stack_leaves_the_other_one_and_a_recreated_stack_gets_new_children`); a selected reference with no item that has its id logs an `error!` and hides the current child; a pending selection shows nothing; `StackChildCreator` is the public `SystemId` of a creator, and `StackChildSpawner` (`parent` the stack, `creator_item` the item whose creator runs) records the id of the entity it spawned in that stack's cache; children are created lazily by a registered system (`StackChildSpawner`, whose `spawn` takes a bundle and `spawn_scene` a scene, the stack child being the scene's root) and hidden, not despawned, when deselected; one global observer of `Insert<Selection>` switches every live `Stack` whose `selection_ref` is that entity or holds a `Selection::Ref` to it (one hop, so a stack over a reference follows the selection it refers to: `a_stack_over_a_selection_ref_follows_the_selection_it_refers_to`), so a despawned stack reacts to nothing; `Stack` is immutable (`#[component(immutable)]`, `Stack { selection_ref }` and its `FromTemplate` unchanged, a `compile_fail` doctest guards `&mut Stack`): inserting a new one on a live stack (an observer of `Insert<Stack>`) shows the child of the new selection, hides the children of the former source (kept, shown again if the stack points back) and hides all when the new entity has no selection, and the former source no longer switches it (`replacing_a_live_stacks_selection_ref_shows_the_new_selection_and_ignores_the_old`); `spawn` gives the child a default `Node` after spawning it and `spawn_scene` on the scene's `Ready` (bevy applies queued scenes later), in both only when it has none after its own required components, so a `Panel`'s width is kept |
| `link.rs` | `Link::new(label)`: clickable text, a `Button` with `Role::Link` and a tab stop that triggers `Activate`; `InteractionDisabled` is its one disabled state (insert to disable, remove to enable; `restyle_on_disabled_change` restyles on either): a disabled link is plain `text_primary` text, ignores the pointer and is skipped by Tab (`link_disabled_is_skipped_by_tab.ron`). Insert a `Link` again to change its label. Hover and theme find the label by its private `LinkText` marker among the children, not by position, so an app child before it is kept and does not stop the label from restyling (`a_link_with_an_app_child_before_its_text_still_themes_hovers_and_disables`) |
| `breadcrumbs.rs` | `BreadCrumbs`: a `Link` trail for a `OneOrMore` selection; clicking a crumb trims the selection back to it. Adding, editing or removing the `Item<String>` of a selected item rebuilds the trail in full (`Changed<Item<String>>` and removals; the selection need not change), so labels, the current crumb and the position a click trims to stay current, also through a `Selection::Ref` (`bread_crumbs_follow_item_text_changes_and_trim_to_the_current_position`). Items without an `Item<String>` get no crumb; the last crumb built is the current one, a `Link` with `InteractionDisabled` (a crumb that stops being the last is built again without it), and a click trims to that crumb's position in the selection. `BreadCrumb` marks a crumb |
| `combo.rs` | `Combo`: a button (a label-only `PushButton` showing the optional `ComboPlaceholder(String)` of the combo, "Select.." by default, while nothing is selected: put with the combo or inserted later it changes a live combo; the accessible value stays empty for no selection, `a_combo_shows_its_placeholder_while_nothing_is_selected_and_the_selection_after`; with a `Chevron` `Icon` child, a 16 px triangle image the library draws and tints with `text_primary`) that opens a `List` in a `Float`, `ComboState` (open/closed), closes on an outside click or when an ancestor scrolls (`panel_scroll_closes_combo.ron`). While open, `place_open_lists` (`PostUpdate`, before layout) puts the list right under the button, left edges aligned, so it follows the button when a scroll area around it scrolls; the list is kept 10 px inside the window's left and right edges and goes above the button when it would run off the bottom, like a palette combo's popup (`combo_keeps_list_inside_window.ron`) A rename of a selected item updates the button text (`item_text_change`, which re-runs `update_selection`; the placeholder stays while nothing is selected; `renaming_the_selected_item_updates_a_combos_button_and_popup_but_not_its_placeholder`). To a screen reader the `ComboButton` is a `Role::ComboBox` without a label, its value the text of the selected items joined with ", " (empty for none; not the placeholder or the chevron), `expanded` while open and `controls` its `ComboList` (`combo_accessible_combo_box.ron`) `ComboRefs` on the combo (private fields, getters `button()` and `list()`; immutable, reflected, built only by the widget); `ComboButton` and `ComboList` are public markers The button colours (`handle_combo_button_mouse_events`), the open/close side effects (`ComboState::on_insert`: `Unreachable`, visibility, `InputFocus`, `expanded`), the outside-click rule and the accessibility node are shared with `ColorPaletteCombo` through `popup.rs`; a `Combo`'s list counts as inside an outside click only for a selection of many items, which stays here. |
| `filter_combo.rs` | `FilterCombo` (`FilterCombo::default().with_name("City")`): an opt-in sibling of `Combo` (not a mode of it, so `Combo` is untouched) whose popup has a search box above the options. It is used like a `Combo` (a node with a `Selection`, own or `Selection::Ref`; a `ComboPlaceholder` while nothing is selected) and reuses `ComboButton`, `ComboList`, `ComboState` and `popup.rs`, plus `combo.rs`'s crate-private `chevron` / `selected_text` / `PLACEHOLDER_TEXT`. `FilterComboRefs` (private fields; getters `button()`, `popup()`, `field()`, `input()`, `list()`, `empty()`) is built by `finish_setup` once the search `TextField` has built its input. The popup is a `Float` card (`background_primary`, `border`) holding the search `TextField`, a `List` (`CornerStyle::Separated`, `Unreachable` so Tab never lands on it, `overflow.y` scroll with its own `ScrollPosition`, so the popup caps to the window like `Combo`'s via `place_open_popups`) and a `FilterEmpty` "No matching items" line (`Role::Status`, polite). **The list is the full list**: its `Selection` is `Selection::reference` to the combo's, so rows are the original item entities, `Selected` and the items stay the list's own business, and the filter only sets `Node::display` on each `ItemView` (`Display::None` also takes a row out of Tab and the accessibility tree through `focus.rs`'s `is_hidden`); another widget on the same items is never filtered, and nothing replaces the `ItemsRef` (that could repair a mandatory selection). State is private: `FilterState { query, active, fresh, settle }` on the combo, a `FilterDirty` marker, and `ActiveOption` on the active row (its focus-ring `Outline`, drawn by `style_active_options` and re-coloured with the theme). A rename of a selected item also updates the button text (`selection_change`; `filter_combo_follows_a_rename.ron`, `renaming_an_item_updates_a_filter_combos_button_rows_and_matches`; the `FilterCombos` screen has a "Rename Delft" button, absolute and spawned last). The work is per change, never per idle frame: `mark_dirty` (list `Children` changed, or an `Item<String>` changed whose `ItemOf` owner is the items entity of an open combo's list, found with `SelectionParam::sources_of`, so a rename in an unrelated source rematches nothing and shared sources still do; `renaming_an_item_of_another_source_does_not_rematch_an_open_filter_combo`) and `on_field_change` (`TextFieldChange`) mark, `apply_filters` re-matches (case-insensitive substring of `Item<String>::data`, notes not searched, source order), keeps the active item if it still matches, else the first selected match on the first pass after opening (`fresh`), else the first match, else none. Opening (`on_state`, an observer of `Insert<ComboState>`) resets the query (a new `TextField` insert, which sends no change) and focuses the search input; closing moves the focus to the button only if it was on the input or list. Keyboard: the input has the keys (`on_key`, an observer on it, registered after Bevy's own `TextInput` observer, which runs first and queues a caret move for Up / Down that `on_key` pops): Arrow Down / Up move through the shown rows, wrapping, repeats included, and stop (`propagate(false)`, with Enter and Escape) so an app's directional navigation never sees them; Enter and numpad Enter (not repeats) toggle the active item through `SelectionParam::update_selection` on the list, exactly as a click, and close for a selection of one at most; Escape (by key code, since a source of key events may not fill in the logical key Bevy's input reads) closes without picking; nothing acts while `EditableText::is_composing()`. The search `TextField` input carries the crate-private `OwnKeys` marker so `text_field.rs`'s `on_key_input` leaves Enter and Escape alone. Focus: `on_focus_lost` closes when the focus leaves other than to the popup (a press in it gets the input back, a press on the button is left to its `Activate`, Shift+Tab to the button closes), `close_on_outside_click` closes on a release outside, and inside means the button, the popup's card, search box and empty line, and the list only for a selection of many. `keep_active_in_view` (`PostUpdate`, after layout, for `SETTLE_FRAMES` frames after a change) scrolls the list's own `ScrollPosition` to the active row plus the list padding and clamps it after the results shrink; it never scrolls a `ScrollArea` around the combo. To a screen reader the button is a `Role::ComboBox` (value the selection, `controls` the list, `expanded`) and the input a `Role::ComboBox` too (label `name` or "Search", value the query, `controls` the list, `expanded`, `active_descendant` the active row, also Bevy's `ActiveDescendant` for `App(Active(..))`); hidden rows, the closed popup and an empty-line that does not show are hidden nodes. Limits: a deleting key cannot be played by the scenarios (the tester's `Tap` carries no logical key, which Bevy's text input reads) and the headless tests have no font layout, so `backspace` there sets the text as a deleting edit would leave it (`FilterApp::backspace`); an IME's commit key is guarded only by `is_composing`. |
| `popup.rs` | crate-private (`pub(crate)` items, like `focus.rs`): what `Combo`, `ColorPaletteCombo` and `FilterCombo` share, each passing the parts it differs in as arguments: `button_look` (the pure colour match of a pointer event and the open state, unit tested) and `apply_look`, `closes_on_outside_release` (open, pointer on neither button nor the inside part of the popup), `show_popup` / `move_focus` / `set_expanded` (the open/close side effects) and `sync_combo_box` (`Role::ComboBox`, value, `controls`, no label) |
| `glyph_images.rs` | crate-private: `GlyphImages`, a per-app resource (initialised by `WidgetsPlugin`, so independent `App`s keep their own) caching one `Handle<Image>` per `GlyphKey` (`Chevron`, `Minus`, `Plus`, `Sun`, `Computer`, `Moon`), and `shared(world, key, generate)`, which `Combo`, `FilterCombo`, a `NumericStepper` left at its default icons and `LightDarkToggle` use instead of adding an image per instance: many widgets show 6 images in all. Without `Assets<Image>` it returns `Handle::default()` and caches nothing, so a widget made before the assets exist gets images only when inserted again after; an image the app removed from `Assets<Image>` is generated again; the cache holds a strong handle, so the images live as long as the app. A custom stepper icon is never touched (`many_widgets_share_one_image_per_generated_glyph`, `a_custom_stepper_icon_is_kept_and_only_the_default_one_is_shared`, `an_app_without_image_assets_gets_default_handles_and_the_images_once_assets_exist`, `separate_apps_do_not_share_glyph_images`, `an_image_the_app_removed_is_generated_again`) |
| `float.rs` | `Float`: requires Bevy's `FixedNode`, so it is a child of its widget but placed by its `left` / `top` in the viewport (logical pixels) and takes no layout, clipping or transform from its ancestors: a `ScrollArea` around the widget does not clip it. `GlobalZIndex(1000)`, large shadow; the popup layer of `Combo`, `ColorPaletteCombo` and `Tooltip`, each of which places its own, `Combo` and `ColorPaletteCombo` through the crate-private `keep_in_window_x` and `below_or_above` of `float.rs` (10 px `WINDOW_MARGIN` from the window's side edges, above the button when it would run off the bottom; `fit_vertically` returns the `top` and a `max_height`: when the popup fits on neither side it goes on the side with more room and is capped to that room, never below `MIN_POPUP_HEIGHT` (100 px) except in a window shorter than that, and scrolls with the wheel (`overflow.y` is `Scroll`), staying inside the window's top and bottom margins; the cap is always the chosen side's room, so the next frame's measured height picks the same side; keys in a capped list do not scroll it, `scroll_into_view` skips a `FixedNode`; `combo_taller_than_window_stays_inside_window.ron` on the `long combo`) (`combo_in_scroll_area_popup_is_not_clipped.ron`, `color_palette_combo_in_scroll_area_popup_is_not_clipped.ron`) |
| `scroll_area.rs` | `ScrollArea::bundle` (for code that spawns through `Commands`) and `ScrollArea::scene(contents, vertical, horizontal)`, the same area as a scene whose `contents` is a factory, `impl Fn() -> impl SceneList + Send + Sync + 'static` (`@ScrollArea::scene(move || bsn_list! { .. }, true, false)` in a `bsn!`; it inserts the bundle after the scene is applied). The factory runs for every build of the template, so each area, also from one resolved scene spawned again, gets fresh contents and its own scroll bar targets, with nothing consumed on the first build; a captured handle or data is cloned inside the closure (`a_scroll_area_scene_gives_every_build_its_own_fresh_contents`, `a_cached_scroll_area_scene_builds_fresh_contents_each_time_it_is_spawned`, `a_scroll_area_factory_with_a_captured_handle_builds_asset_scenes_again_and_again`), `ScrollContents`, the `ScrollBar` marker (a zero-sized immutable component, reflected; the axis is not stored on it but is the `orientation` of the `bevy_ui_widgets` `Scrollbar` on the same entity, which grid cell, thickness, visibility and theming all read; the crate has no `Orientation` of its own; its `on_insert` hook is private and queues one idempotent command that reuses the bar's `ScrollThumb` or spawns exactly one, so inserting it again, also twice before the commands are applied, never duplicates it; `refresh_changed_scroll_bars` re-places and re-sizes a live bar when its `Scrollbar` changes; theming touches only the marked thumb, other children of the bar are the app's; the thumb's background is one function of its state, `thumb_background(colors, hovered, pressed)` (`scroll_thumb_background` at rest, `hovered_item` hovered, `selected_hovered_item` pressed or dragged), called by the pointer handler, by the restyle on a `ScrollbarDragState` change (a drag that ends away from the thumb) and by `theme_scroll_area`, so a theme change under the pointer keeps the colour (`scroll_area_thumb_colours.ron`); the thumb requires picking's `Hovered` and gets Bevy's `Pressed` while held; a bar without a `Scrollbar` does nothing) and the thumb, themed here; each bar is also a `bevy_ui_widgets` `Scrollbar` aimed at the contents and its thumb a `ScrollbarThumb` (no `Node`), so the widget sizes and places the thumb, scrolls the contents when it is dragged and pages them on a click on the track, while `show_scroll_bars` hides a bar whose axis fits, looking again when the contents are laid out or the bar's `Scrollbar` changes, so a live orientation change shows or hides the bar by its new axis (`changing_the_headless_orientation_shows_or_hides_the_bar_by_its_new_axis`; a child of a bar that is not the `ScrollThumb` is the app's and is left alone by theming and by inserting `ScrollBar` again, `an_app_owned_child_of_a_scroll_bar_survives_theming_and_reinserting_the_marker`); and the crate-private `update_scroll_position`, which scrolls any `ScrollPosition` under the pointer with the wheel (the targets are found once per frame, not per message, in buffers kept in a `Local`, and each message is then applied and clamped in turn, never summed; a `ScrollPosition` is written only when the clamped result differs, so a wheel at a limit or with no delta along an axis does not mark it changed or close a `Combo` in the area (`a_wheel_that_cannot_move_a_scroll_area_does_not_mark_its_position_changed`, `a_wheel_at_the_limit_leaves_an_open_combo_in_the_scroll_area_open`); a frame with no message returns at once, #682) (an area that scrolls sideways only takes a vertical turn, which is all a mouse wheel gives, for a horizontal one: `scroll_area_wheel_vertical_turn_scrolls_horizontal_area.ron`, the `image_gallery` example) Wheel events reach the nearest scrollable ancestor of a hovered control, stopping at a `FixedNode`; over a nested area only the innermost one scrolls and nothing is passed on at a limit: `scroll_area_wheel_nested_horizontal_in_vertical.ron`. An area clips to a rectangle (Bevy 0.20 has no rounded clip), so one at the bottom of a rounded `Card` shows contents sticking out of the card's corners: insert the opt-in `ClearOfCorners` with the area and it gets a bottom margin of `panel_radius`, kept in step with the `Theme`, and removing it sets the bottom margin back to `Val::ZERO` (an area cannot tell whether it touches the corners, so it is not automatic; `Panel` does the same for its own body, only without a footer; `scroll_area_in_card_stays_clear_of_rounded_corners.ron` on the `ScrollAreas` screen). `PopupScrollBar { list }` (crate-private) is the bar of a list that scrolls itself, a `Combo` or `FilterCombo` popup capped to the window: `popup_scroll_bar(list)` spawns a `ScrollBar` + `Scrollbar` (vertical, aimed at the list) as a child of the list, and `place_popup_scroll_bars` (`PostUpdate`, before layout) keeps it at the visible top and right edge of the list (`top` = the list's `ScrollPosition`, since an absolute child scrolls with its content), as high as the list, hidden while the rows fit; `theme_popup_scroll_bars` themes it like an area's (`theme_bar`, shared). A bar the pointer is on counts as inside the popup for the outside-click rule (`popup_scroll_bar_hovered`; `combo_popup_scroll_bar.ron`). Put a `List` or palette in one to scroll it. `ScrollContents` has 3 px of padding on every side (`FOCUS_RING_REACH`) so a focus ring at the edge is not clipped, and `scroll_into_view` counts it as part of the entity. The area is a `Role::ScrollView`; its bars and thumb have no accessibility node (`scroll_area_accessible_scroll_view.ron`) |
| `list.rs` | Selection changes are collected and lists visited once per frame (#671; `simultaneous_selection_changes_update_only_the_lists_that_follow_them`).  `List` (and, through the shared `watch_items`, `ColorPalette`) starts watching when its `Selection` or the `List` itself is inserted, whichever comes last, because a scene (`List @{Selection::builder()..scene_with_items(..)}`) inserts them in an order of its own and the `List` can arrive after the selection and items (`the_overview_scroll_area_page_has_rows_and_visible_scroll_bars`); it follows the `ItemsChanged` of the items entity its `Selection` draws from with one observer, remembered in the crate-private `ItemsWatch` on the list: pointing the list at other items (a `Combo` does) despawns the old observer, so the old items no longer touch its views, and the record goes with the list (`a_list_pointed_at_other_items_ignores_changes_to_the_old_ones`); `List` with `ListLayout` (`VerticalGrid { columns: 0 }` is laid out as one column and logs an `error!` when the layout is inserted; `List` requires a `Node` that is a grid, except that `ListLayout::Vertical` (one column, the default) is a flex column (`Display::Flex`, `FlexDirection::Column`), because Taffy lays out thousands of rows in a grid several times slower than in a flex column (`list-items-adding` at 4 000 rows: about 530 ms a frame as a grid, 90 ms as a flex column, #729; the other layouts stay grids). Even so a `List` spawns a view per item, and a long list belongs in a `VirtualList`; its hook and an `on_insert` hook of `ListLayout` set only the fields the list owns on the existing `Node` (`display`, `flex_direction`, `position_type`, `grid_auto_flow`, `grid_template_columns`): a `width`, `margin` or `flex_grow` of the app survives, also when `List` is inserted again, and changing `ListLayout` on a live list updates the grid flow and columns; a `NonZeroU16` would be a public API change) and `CornerStyle` (`Separated` is the panel style: 6 px padding, 2 px gaps, rows rounded on all corners, the selected row with a 1 px `selection_ring` border), a plain grid that does not scroll, and has no maximum height: wrap it in a `ScrollArea` whose node has a `max_height` and the area grows with the rows up to it and then scrolls (`a_list_in_a_scroll_area_with_a_maximum_height_grows_with_its_rows_and_then_scrolls`; #505 decided against a `List` field for this): a `ListBox` that renders the items of its `Selection` as `ItemView`s (`ListItem`s) and keeps them in sync, `Selected` included; while focused it keeps its arrow, Home and End keys from bubbling on (`keep_navigation_keys`), and lets those of a focused descendant (a button in a row) past (`pass_first_presses_from_descendants`, see "Directional navigation is the app's") `on_items_changed` reconciles views and items in linear time (one `HashSet` of the wanted items, one of those that already have a view: a view stays when its item is wanted and no earlier view shows it, any other is despawned; `list-items-initial` and `list-items-adding` bench workloads, #676; `a_list_keeps_its_views_and_their_children_when_items_are_added_and_removed`, `a_list_despawns_a_second_view_of_the_same_item`, `a_list_of_thousands_of_items_gets_one_view_each_and_marks_the_selected_one`) `ItemNote { text, tone }` (`NoteTone::{Secondary, Warning}`; `ItemNote::new` / `::warning`) on an `Item<String>` entity adds a note after the label of its default row, in the caption size and `text_secondary` / `warning_text`, or `text_primary` on a row whose background that tone does not reach 4.5:1 on (`note_ink`, kept in step by `note_ink_follows_row` when the row's background changes: hovered, selected, theme): a `Combo`'s popup shows it, its button shows the label only. Renaming an `Item<String>` in place updates the `ItemLabel` text of its default rows (`sync_item_label_text`, through `NoteViews`, so only the rows of that item, also in lists sharing it; custom views are left alone; `renaming_an_item_updates_the_visible_and_accessible_text_of_every_list_that_shows_it`). The row's accessible name is the label followed by the note (`Delft < unreachable >`, kept by `sync_item_labels`); changing, inserting or removing a note, or giving it empty text, updates the row in place (`combo_option_notes.ron`). `sync_item_notes` looks only at the rows a note change concerns: every default `ItemView` gets the crate-private `NoteViewOf(item)` relationship (put on by `ItemView`'s `on_insert`, target `NoteViews` on the item), so a changed or removed `ItemNote` finds exactly the views of its item, also in several lists sharing it, and a new view is looked at once; a `Theme` change looks at all of them (`a_note_change_resyncs_only_the_rows_of_its_item_and_a_theme_change_all_of_them`; the `list-notes-changing` and `list-notes-idle` bench workloads). A custom view (`ItemViewCreator`) does not get it. |
| `virtual_list.rs` | `VirtualList { row_height }`: a list of rows of one height for many thousands of items that renders only those in view. Its node is as tall as all rows, so the `ScrollArea` around it (the nearest ancestor whose `overflow.y` scrolls, else the window) scrolls the whole list; under it is a pool of absolutely placed `VirtualListRow { list, shown: Option<ShownItem> }`s (`ShownItem { index, item }`: the item and its index exist together, `None` for a row that shows nothing; immutable, `ListItem`s): the rows in view, one for a part row and `OVERSCAN_ROWS` (2) beyond each edge. When `ScrollPosition` moves, `show_rows` (`PostUpdate`, before layout) hands rows on to the items now in view (item `i` goes to row `i % pool`, so a scroll by one row re-points one row); a row is never spawned or despawned for a scroll, and the pool only grows, when the view does. A row shows the text of an `Item<String>` in an `ItemLabel`, or with `List`'s `ItemViewCreator` the app fills it: the creator runs again, on the row emptied of its children, when the row is handed to another item (or the app observes the `VirtualListRow` insert). Selection is as for a `List` (`Selected` on the rows of selected items, a primary click or touch toggles as far as the selection allows; other pointer buttons pick nothing and the event goes on to bubble for an app's context menu, `only_a_primary_click_or_touch_picks_a_virtual_list_row`) but it is not a `ListBox`, whose observers would only see the pooled rows: it is one tab stop with its own keyboard observer (arrows move the active row, wrapping, Page Up / Page Down by the rows that fit in the view, stopping at the ends, a first Page Down with no active row landing on the first item as Arrow Down does, Home / End, Enter / Space toggle the active row as a click does, through `toggle_selected`, so one on a selected row deselects it as far as the selection allows and a `One` keeps its item (`enter_or_space_on_a_selected_virtual_list_row_deselects_it_as_far_as_the_selection_allows`); each scrolls the active row into view by setting the area's `ScrollPosition`), and `ActiveDescendant` names the row of the active item while it is in the pool; `show_rows` runs before `focus.rs`'s crate-private `DrawFocusRing` set, so the focus ring is on the row the frame it moves, and `activate_on_focus` runs before it as a system that reads `InputFocus` (set in `PreUpdate`), not as an observer of `FocusGained`, which Bevy sends after the layout, so a list focused by Tab or a click names and rings its active row the same frame (`virtual_list_keyboard_pages.ron`). A `Role::ListBox` whose `size_of_set` is the number of items, each shown row a list item with `position_in_set` (its item's index plus one) and the same `size_of_set`, kept by `sync_accessibility` in `AccessibilitySync`, so a pooled row reads as item 5,001 of 10,000. `RowHeight::new(f32)` returns a validated `RowHeight` or `RowHeightError`: the height must be finite and at least `MIN_ROW_HEIGHT` (10 px), which bounds the pool even at the smallest accepted height; `VirtualList::new(RowHeight)` takes that value, and `VirtualList::separated(RowHeight)` returns an error unless it is above the 22 px of vertical padding and ring. The list fields are private, so callers cannot invalidate a live height; construction errors are covered by `invalid_virtual_list_row_heights_are_rejected_and_the_minimum_keeps_the_pool_bounded` and `separated_virtual_list_rejects_a_row_height_at_or_below_its_chrome`. `List` is untouched. `VirtualList::separated(row_height)` (the `separated` field) gives the rows the look of a `CornerStyle::Separated` `List`: 6 px in from the list's edges, 2 px gaps, rows rounded at the control radius with the separated item padding and a 1 px border that is the `selection_ring` on the selected row, which also gets `hovered_selected_item`. It uses `list.rs`'s own constants, `item_background` and `item_border`, so the two cannot drift, and the rows stay absolutely placed: row `i` is at `6 + i * (row_height + 2)`, the node is `2 * 6 + n * (row_height + 2) - 2` high, and the pool, `show_rows`, Page Up / Page Down (`rows_per_page`), `scroll_into_view` in `navigate` and the scroll limit all use that pitch (`VirtualList::pitch` / `row_top` / `content_height`); `position_in_set` is by index, so the gap does not touch it. `row_height` is then the whole row, padding and ring included (above 22 px). Still only pooled rows are styled each frame (`virtual_list_separated_ring_follows_selection.ron`, `virtual_list_separated_keyboard_pages.ron` on the `SeparatedVirtualList` screen) |
| `color_palette.rs` | `ColorPalette`: a grid of colour swatches (`ListItem`s, `Selected` when in the selection) over a selection of `Item<Color>`; a swatch's accessible name (label of its `AccessibilityNode`, kept up to date) is the name of its colour, `tailwind::color_name`; its `on_items_changed` reconciles linearly like a `List`'s (a set of wanted and a set of shown items, a second swatch of one item is despawned, #679; `palette-items-adding` bench workload): the Tailwind name (`blue-500`) when it equals one of `COLORS`, else its lowercase sRGB hex (`#3b82f6`, alpha appended only when not 1). **Edited in place (#691):** `sync_item_colors` (change-driven, `Changed<Item<Color>>`) rewrites the square, highlight and hover state of only the swatches of the changed items, without rebuilding them; the accessible name already follows (`editing_an_item_colour_in_place_updates_its_swatch_highlight_combo_preview_and_name`). **Dormant while closed (#680):** a palette that a `ColorPaletteCombo` owns carries the crate-private `PaletteDormant` while its popup is closed and has no `ItemView` children (`on_items_changed` returns early); `wake` removes the marker and builds the swatches, `sleep` inserts it and despawns them, so selection and items stay on the combo and a reopened popup shows the current selection and items |
| `color_palette_combo.rs` | `ColorPaletteCombo`: a `ColorPalette` inside a combo-style popup; the popup, a `Float` child of the combo, is centred under the button when it opens, 10 px inside the window, above the button when it would run off the bottom; its palette gets `Selection::Ref` to the combo's selection when it is spawned, and `SubRefs` keeps its entity. Its button is a `Role::ComboBox` like a `Combo`'s, without a label, its value the colour names of the selected items that have an `Item<Color>` joined with ", " in selection order (empty for none, also a pending mandatory selection), `controls` the popup's palette. Over a selection of many (`OneOrMore`, `ZeroOrMore`) the button's square shows the first selected colour in selection order (`Srgba::NONE` when none is usable) and nothing else (no split swatch or counter); a pick or Enter / Space leaves the popup open, Escape, an outside click, focus loss and the button still close it `ColorPaletteComboState` (`Open` / `Closed`, immutable, with `inverse()`; formerly `State`), `ColorPaletteComboRefs` on the combo (private fields, getters `button()`, `popup()` and `palette()`; immutable, reflected, built only by the widget; the routing records on the button and palette stay private), and the public reflected markers `ColorPaletteComboButton` (named `ColorPaletteComboButton`) and `ColorPaletteComboFloat` (the popup, named `<combo name> popup`) It shares the button colours, the open/close side effects, the outside-click rule and the combo box node with `Combo` through `popup.rs`, and keeps what differs: the float always counts as inside, a swatch click closes it, it runs `position_float` on open, and its visible part is the float while its focus target is the palette.. **Edited in place (#691):** `sync_edited_item_colors` (`Changed<Item<Color>>`) refreshes the button square when the first selected item's colour is edited, not only on a `Selection` change. **Lazy popup (#680):** opening wakes the palette before layout and keeps the popup `Visibility::Hidden` until `place_opened_popups` (`PostUpdate`, after `UiSystems::PostLayout`) has placed it, so it appears one frame after the click (scenarios `Wait(1)` after opening by click); closing hides it and a `PendingSleep` marker despawns the swatches in `Last`, after the `ListBox`'s `ValueChange` of the click that closed it was handled. A closed popup has no swatches, so scenarios cannot check swatch state there (`a_closed_color_palette_combo_builds_no_swatches_until_it_opens`, `despawning_color_palette_combos_open_or_dormant_leaves_no_swatches_behind`) |
| `color_push_button.rs` | `ColorPushButton`: a button showing a colour square; its accessible name is the name of that colour (same rule as a swatch), kept by a system because Bevy clears a text-less button's label on add; a `ColorPaletteCombo`'s button is not, its colour is its value instead |
| `numeric_stepper.rs` | `NumericStepper`, its `Value`, `UIntValue<T>`, `FloatValue`, `ValueError`, `ValueChange`; `Value::new_u8` / `new_u16` / `new_u32` / `new_f32` return `Result<Value, ValueError>`: a minimum above the maximum, a zero integer step, a non-finite float initial value or bound and a non-finite or non-positive float step are rejected as given (bounds are never swapped, a step never replaced), a valid initial value is clamped, equal bounds and a step larger than the range are valid (both buttons disabled). `Value::F32` holds a `FloatValue`, and `FloatValue` and `UIntValue<T>` have private fields with `value()` / `min()` / `max()` / `step()` getters and validated `new`; `Value` itself stays mutable but exposes no mutable field access. `increment` / `decrement` use checked integer arithmetic and leave a float unchanged when the result would overflow, cross a bound or not strictly change it (a rounding no-op), and `can_increment` / `can_decrement` agree. To change a float's bounds build a new `FloatValue` from the getters. `ValueRef` is an immutable Bevy relationship and `ValueReferrers` (public, reflected, read-only) is its target on the `Value`, listing the mirror steppers and the parts of every stepper showing it; `on_change` visits only the referrers of a `Changed<Value>` (plus a self-held stepper's own node) and writes label text, fill width and the spin button only when they differ, so an unrelated stepper costs nothing and a touched but equal value dirties no layout (`steppers-*` bench workloads, #663; `an_external_value_change_updates_only_the_steppers_that_show_that_value`). Prepared valid `Value`s are inserted in scenes with `~{value}`. **Children:** the stepper owns only the parts it builds (the crate-private `StepperPart(stepper)` marks the track, buttons and label) and an open `ScrubEditor`; inserting a `NumericStepper` again, or for the first time on an entity that already has children, replaces just those and keeps the app's children, with the parts added after them (`inserting_a_numeric_stepper_keeps_app_children_at_first_and_every_later_insert`). Replacing a stepper while its number editor is open and focused despawns the editor, shows the new value label and leaves the arrow keys and F2 working (`reinserting_a_numeric_stepper_while_its_number_editor_is_open_keeps_app_children_and_leaves_no_stale_editor`). Its "-" and "+" are small `IconButton`s (accessible names `decrease_name` / `increase_name`, "Decrease" / "Increase" by default; icons `decrease_icon` / `increase_icon`, a drawn minus and plus when left at the default handle) that step on `Activate`; `format` (`fn(&Value) -> String`, default `Display`) says how the value is shown, in a label of at least `label_min_width` (`DEFAULT_LABEL_MIN_WIDTH_PX`, 54 px, defined in `slider.rs` and re-exported here, so the two widgets share one constant) so the buttons do not move; fields are read when the component is inserted, and one that cannot step (at the minimum or maximum, or a step short of it) gets `InteractionDisabled`; the arrow keys step it too (up / right increase, down / left decrease) through an observer of `FocusedInput<KeyboardInput>` on the stepper, which a focused button's keys bubble up to and which stops them there (pressed, repeated or released, also at a limit), so only the buttons are tab stops and an app's directional navigation never sees them. The stepper's own node is one spin button: an `AccessibilityNode` with `Role::SpinButton`, `numeric_value` / `min_numeric_value` / `max_numeric_value` / `numeric_value_step` from the `Value` (as `f64`), `value` the text `format` shows (kept in step by `on_change`, also for mirrors through `ValueRef`); `ValueChange` is triggered by the step itself (a button's `Activate` or an arrow key, only when the value moves), never by `on_change`, so spawning a stepper or the app setting its `Value` sends none (the display follows both); a stepper with no `Value` and no `ValueRef`, or a `ValueRef` whose target has no `Value`, logs an `error!` and is not built (no default `Value`) and `name: Option<String>` as its label (none by default); the value text is a plain `Text`, not a `Label`, so it has no node of its own. **Scrubbing and typing (#623):** like a slider, the whole stepper can be dragged. A `ScrubTrack` child (the first child, absolutely placed behind the rest, `Pickable::IGNORE`, inset 30 px (`BUTTON_WIDTH_PX`) on the left and right so it spans the gap between the buttons) holds the `ScrubFill(stepper)` (`Pickable::IGNORE`, `slider_fill` at 25 % alpha, `theme_numeric_stepper` themes it), as wide as `Value::fraction()` (public, 0 to 1) of the track, kept by `on_change`: 0 % is the right edge of the "-" button and 100 % the left edge of the "+" button. The value label has `flex_grow: 1`, so the app can set the stepper's `Node::width` and the buttons stay at the ends. While a scrub drag is on, `on_mouse` sets Bevy's `OverrideCursor` to `ColResize` (cleared by `end_scrub`) and `HoverLock`, so the cursor is the resize one wherever the pointer is and nothing highlights; the "-" and "+" buttons carry the default `EntityCursor`, so hovering them does not change the cursor. A horizontal drag that starts anywhere on the stepper, the "-" and "+" buttons included, moves the value by `distance.x / width` of the range from where the press began, snapped to the step (`Value::set_fraction`, private) and sent as `ValueChange` only when it moves; it needs `DRAG_THRESHOLD` (3 px) first. The private `Scrub { start, dragged }` on the stepper keeps this; while `dragged` is set the buttons' `Activate` does not step (so a drag that ends over a button changes nothing more), and `end_scrub` (`Last`) clears it once the left mouse button is up and no touch is down (a press also resets it). Only the primary button (a touch counts) presses, drags and clicks: `MouseEvent` carries the `button` of the pointer event, and the stepper ignores the others (`a_primary_drag_scrubs_but_a_right_button_drag_does_not`, `scrubbing_a_mirror_stepper_changes_the_shared_value`, `the_scrub_fill_follows_the_value`) A click (not a drag) on the value text opens a number editor: the label gets `Display::None` and a `TextField` marked `ScrubEditor` (as wide as the label's minimum, named "Value", empty with the current value as its placeholder so typing replaces the value, focused by `focus_new_editors`) takes its place in the same child slot; Enter (or a press elsewhere, the field's commit) sets the value to the leading number of the text (`parse_number`; `Value::set_number` clamps to the bounds, rounds an integer and does not snap to the step; text with no number changes nothing), Escape leaves it. The editor is closed in `Last` (`EditorDone`, `finish_editors`) after the commit of the same focus loss was handled, and while it is open the stepper's arrow keys and pointer scrubbing leave it alone. Tests: `numeric_stepper_drag_scrubs.ron`, `numeric_stepper_type_number.ron`, the unit tests of `fraction`, `set_fraction`, `set_number` and `parse_number`. **From the keyboard (#631):** F2 on a focused stepper (a key that bubbles up from a focused button; `on_key_input`, which stops it, ignores a repeat) opens the same editor through the shared `open_editor`, named "Value" for a screen reader like the pointer's. Enter is not the route: on the focused button it is that button's `Activate` and steps. The editor remembers the focused part (`ScrubEditor::restore_focus`) and `finish_editors` gives it the focus back when the editor closes with the focus unclaimed (Enter or Escape; a Tab or a click elsewhere keeps its own target). Test: `numeric_stepper_keyboard_editor.ron`. **Cursors (#644):** the stepper carries Bevy's `EntityCursor` `ColResize` (a double arrow with a bar, for the scrub) and its value label, which now blocks the pointer from the stepper below it so only one cursor is hovered, and the open `ScrubEditor` carry `Text` (the I-beam, where a click edits); the buttons carry the default cursor (hovering them changes nothing) and a drag holds `ColResize` through `OverrideCursor`. `WidgetsPlugin::finish` adds picking's `CursorIconPlugin` unless the app has it, which copies the hovered `EntityCursor` to the window and sets the window's default cursor otherwise (`a_numeric_stepper_shows_a_resize_cursor_and_a_text_cursor_on_its_value`) |
| `slider.rs` | `Slider::new(value, min, max)` and `with_step`, both returning `Result<_, SliderError>` (`SliderError`: `NonFiniteValue`, `NonFiniteBound`, `ReversedBounds`, `RangeTooWide`, `InvalidStep`; bounds must be finite with `min <= max` and a finite f32 `max - min`, a finite out-of-range value is clamped into the range, a step must be positive and finite and may exceed the range; `min == max` is valid and the value then never changes), with `value()`, `min()`, `max()` and `step()` as read-only getters (the fields are private), `with_format` (`SliderFormat`, `fn(f32) -> String`), `with_name` and `with_label_min_width`: a horizontal range control over Bevy's headless `bevy::ui_widgets::Slider` (`SliderValue`, `SliderRange`, `SliderStep`, `TrackClick::Snap`), 200 px wide unless the app gives its `Node` a width, 20 px high: a 4 px track in `slider_track`, a fill up to the value in `slider_fill` and a 16 px thumb of control radius (`card_surface` with a 2 px `slider_fill` border and the small shadow; `accent_subtle` hovered, `accent` pressed or dragged) that stays inside the slider at both ends (it moves in a `SliderTravel` node one thumb narrower, as the headless pointer maths assume). Its colours come from one function of disabled, pressed or dragged, and hovered (`look`); `InteractionDisabled` draws it in `text_disabled`, and the headless slider then ignores the pointer and keys. With a `format` its value is shown in a `SliderLabel` of at least `label_min_width` (54 px) in a right margin the slider reserves (a slider with a `format` owns its `margin.right`, set when the `Slider` is inserted; one without leaves it to the app), absolutely placed so that the track does not move as the text changes, secondary text in rem. A press on the track jumps the thumb there and a drag moves it, both through the headless slider's `ValueChange<f32>`, which a global observer snaps to the `step` (from `min`, clamped; `Slider::snap` computes in f64 so a tiny step cannot overflow, and gives `min` for a non-finite input) before the app's observers on the slider see it, then inserts as the `SliderValue`. One tab stop; while focused left / right move it by `step` (a hundredth of the range without one), Home / End to its ends (Bevy's), and `keep_navigation_keys` stops those keys from bubbling on (pressed, repeated or released); up and down go through. The focus ring is the `TabStop`'s. To a screen reader it is a `Role::Slider` (Bevy's numbers: value, bounds, step; `a_slider_is_a_slider_node_with_its_value_bounds_and_step`) whose `value` is the text `format` shows (none without one) and whose label is `name` (none by default). Fields are read when the component is inserted; an app sets the value by inserting a `SliderValue`. `SliderRefs` names the parts (a reflected component, like `ValueRef` and `UIntValue<T>`, which `tests/reflection.rs` checks). `restyle_sliders` visits only the sliders whose `Hovered`, `SliderValue` or `SliderDragState` changed or that gained `Pressed` or `InteractionDisabled`, plus those that lost either (`RemovedComponents` into a `Local` buffer, sorted and deduplicated, skipping those already styled), so an idle frame walks no slider and a slider that changes several ways at once is styled once (`a_slider_released_away_from_the_control_goes_back_to_rest`, `a_slider_follows_being_disabled_and_enabled_again`, `a_slider_follows_a_drag_and_a_value_change`, `a_slider_pressed_and_enabled_in_the_same_frame_ends_up_resting_and_enabled`, `a_theme_change_restyles_a_pressed_slider_and_idle_frames_leave_it_alone`; `sliders-idle` and `sliders-pressing` bench workloads, #698) |
| `panel.rs` | `Panel`: a `Card` 380 px wide with a header (title `Label`, optional subtitle beside it, optional close `IconButton` that triggers the `PanelClosed` entity event on the panel), a strip, a `Divider`, a body in a `ScrollArea` that grows with its content up to `max_body_height` (620) and then scrolls, and a footer above a divider. The app spawns contents as children of the entities in `PanelRefs` (`strip()`, `body()`, `footer()`); an empty strip or footer is `Display::None`. `Panel` is an immutable component (`&mut Panel` does not compile, guarded by `compile_fail` doctests): insert a new one to change title, subtitle, close icon, `close_name` or maximum height; the parts are updated in place (no `Changed<Panel>` sync), the app's children in strip, body and footer stay, and the close observer is registered once, so `PanelClosed` comes once per activation (`inserting_a_panel_again_keeps_its_parts_content_and_one_close_notification`). `close_name` (`with_close_name`, default "Close", `PanelTemplate::close_name`) is the accessible name of the close `IconButton` (and its tooltip); it does not decide whether the button shows, `close_icon` does (`a_panels_close_button_is_named_close_unless_the_app_names_it`). `PanelRefs` is public, queryable and immutable with private fields and getters `header()`, `title()`, `subtitle()`, `close()`, `strip()`, `scroll_area()`, `body()`, `footer()` (a `compile_fail` doctest guards the private fields); only the widget builds it. Migrating (nopjes): `Query<&mut Panel>` / `get_mut::<Panel>` become an insert of a prepared `Panel`, and `refs.body` and the like become `refs.body()`. `max_body_height` is private (read it with `max_body_height()`) and must be finite and above zero: `with_max_body_height(h)` returns `Result<Panel, PanelHeightError>` and rejects NaN, infinities, zero and negatives without clamping, and `PanelTemplate`'s public `f32` field is checked by the same helper in `build_template`, so a `bsn!` `Panel { max_body_height: .. }` with a bad value is a template error (`spawn_scene` returns `Err`) and builds no panel (`a_panel_scene_with_an_invalid_maximum_body_height_fails_to_build_and_spawns_no_panel`). For `max_body_height`, replace field reads with `max_body_height()` and add `?` / `.unwrap()` handling to `with_max_body_height`. A scroll area clips to a rectangle, which cannot follow the card's rounded corners, so with no footer the body's scroll area has a bottom margin of `panel_radius` and ends above the corners (its `max_height` is `max_body_height` plus that margin; `sync_optional_sections` keeps both, `panel_body_stays_clear_of_rounded_corners.ron`). The body is `should_block_lower: false`, or it would cover the scroll contents and the wheel would never reach them. Grow-then-scroll needs only `max_height` on the `ScrollArea` node (`a_panel_body_grows_with_its_content_up_to_its_maximum_and_then_scrolls`). It is a `Role::Group` (from `Card`) with no label of its own that is `labelled_by` its title `Label` (so a new `Panel` needs no relabelling); the close button keeps its own name (`panel_accessible_group.ron`). `sync_optional_sections` looks only at panels whose `Panel` or `PanelRefs` changed or whose strip or footer `Children` changed (the private `PanelSection(panel)` marker on both names the panel), all of them on a `Theme` change (`a_panel_keeps_its_sections_and_corner_margin_across_theme_changes_and_quiet_frames`; `panels-idle` and `panels-changing` bench workloads, #685) |
| `push_button.rs` | `PushButton` with label and optional icon; **the reference widget to copy**. `ButtonSize::Compact` (insert it with the button or later) makes it 28 px high with 10 px side padding, a 6 px radius, a semibold 12 px label and no shadow (`Theme::compact_button_*`, `font_size_small`); `Regular` is the default. `IconPlacement::{Left, Right (default), Top, Bottom}` (insert it with the button or later) says where the icon sits relative to the label, by the `Node`'s flex direction (children are not respawned). Merging with `IconButton` was investigated (#377) and not done: `PushButton` takes a text glyph, `IconButton` an image `Handle`, a name and `ToggleState`; they stay two widgets. Its accessible label is its text, not the icon (a combo's button is a combo box and has none). Inserting a `PushButton` again changes a live button (`on_insert`, like `IconButton`): the label `Text` is set in place, an icon child is updated, spawned or despawned (`PushButtonRefs` follows), and the observers are registered once (`inserting_a_push_button_again_changes_its_text_in_place_and_keeps_its_label_in_step`, `a_push_button_gains_and_loses_its_icon_when_inserted_again`). `NoDefaultHover` may be inserted or removed at any time: `handle_hover` is always registered and ignores a button that has it, and the button restyles when the marker changes. An optional immutable `ToggleState(bool)` (`icon_button.rs`'s, the same type and its `toggle_on_activate` observer) makes it an on/off toggle: `Activate` inserts the flipped value, an on button has the `accent_subtle` background (`hovered_selected_item` hovered) and `accent_subtle_text` text, and its `AccessibilityNode` is `toggled` with it (none without a `ToggleState`); no new theme slot (`push_button_toggle.ron` on the `Buttons` screen, whose "Toggle" button is spawned last so the tab order stays). `ButtonNames` is the scenario screen for button labels. |
| `label.rs` | `Label` with `Title` (`Label::title`, also `Label::header`), `Regular`, `Secondary`, `Caption` and `Overline` (shown in upper case, no letter spacing), sized and coloured from the theme's type scale and text slots; it requires Bevy's `ui::widget::Label`, so it is a `Role::Label` accessibility node with its text as value, and the hook inserts that marker again so a new `Label` updates the value; immutable (`#[component(immutable)]`, with `Kbd`, `Pill`, `Toast` and `Badge`: build or clone a value and insert it to update, compile-fail doc tests pin it); empty text keeps the node and role styling |
| `focus.rs` | a crate-private module (its `FocusPlugin` is added by `WidgetsPlugin`): the crate-private `TabStop` marker (requires `TabIndex`), the `RovingItem` / `RovingGroup` pair and `sync_roving_groups` of the composite widgets (see "Composite widgets" below), `sync_tab_index` and the focus ring (`draw_focus_ring`, in the crate-private `DrawFocusRing` set in `PostUpdate` before the layout), see "Keyboard focus" below. None of the three walks all candidates per frame: `draw_focus_ring` works out the one entity that should have the ring and touches only it and the one that had it (kept in a `Local`); `collect_hidden_changes` (`PostUpdate`, after visibility propagation, before its consumers) runs once a frame and writes the `HiddenDirty` resource that `sync_tab_index`, `sync_accessibility_hidden`, `sync_roving_groups` and `tab_bar.rs`'s `sync_tab_index` read (each adds its own extras, such as new stops or markers): the entities whose `InheritedVisibility` or parent changed, whose `Display::None` flipped (tracked in a `Local` set), or that lost a `Node` or parent, with the whole subtree of each such root, walked once (overlapping roots, nested flips and a reparent together never revisit a descendant), and `is_hidden` is called only for those (`overlapping_display_flips_and_a_reparent_in_one_frame_reach_every_consumer_of_a_deep_tree`; the `deep-trees-*` bench workloads, #684). Also the crate-private `AccessibilitySync` set (`PostUpdate`, after `UiSystems::PostLayout`, where Bevy's `button_changed` sets `Role::Button` when a `Button` changes, and before `AccessibilitySystems::Update`) with its `sync_role` / `sync_toggled` / `sync_label` / `sync_value` / `clear_label` / `sync_controls` / `sync_labelled_by` helpers (`node_id` turns an entity into the `accesskit::NodeId` Bevy gives it), which write a node only when it differs: widgets set their roles, toggled states and labels there, so Bevy does not undo them. A container group without a title of its own (`Card`, a toolbar or radio group) has no label; `SettingRow` is labelled with its title and `Panel` is `labelled_by` its title `Label`; an app can insert Bevy's `AccessibleLabel` on it |
| `focus.rs` | a crate-private module (its `FocusPlugin` is added by `WidgetsPlugin`): the crate-private `TabStop` marker (requires `TabIndex`), the `RovingItem` / `RovingGroup` pair and `sync_roving_groups` of the composite widgets (see "Composite widgets" below), `sync_tab_index` and the focus ring (`draw_focus_ring`, in the crate-private `DrawFocusRing` set in `PostUpdate` before the layout), see "Keyboard focus" below. Also the crate-private `AccessibilitySync` set (`PostUpdate`, after `UiSystems::PostLayout`, where Bevy's `button_changed` sets `Role::Button` when a `Button` changes, and before `AccessibilitySystems::Update`) with its `sync_role` / `sync_toggled` / `sync_label` / `sync_value` / `clear_label` / `sync_controls` / `sync_labelled_by` helpers (`node_id` turns an entity into the `accesskit::NodeId` Bevy gives it), which write a node only when it differs: widgets set their roles, toggled states and labels there, so Bevy does not undo them. A container group without a title of its own (`Card`, a toolbar or radio group) has no label; `SettingRow` is labelled with its title and `Panel` is `labelled_by` its title `Label`; an app can insert Bevy's `AccessibleLabel` on it |
| `card.rs` | `Card`: a themed background, border, radius and shadow on a node; a `Role::Group` without a label (`themed_card_accessible_group.ron`). A `ScrollArea` at its bottom edge needs `ClearOfCorners` to stay clear of the rounded corners (see `scroll_area.rs`) |
| `indicator.rs` | a crate-private module: the sliding highlight behind the selected item of a track, shared by `SegmentedControl`, `TabBar` and `LightDarkToggle`. The `Indicator` trait (`of(owner)`, `owner()`, `color(theme)`, `segmented_selected` by default), the generic systems `spawn::<I, Track>` (gives a track without one its indicator, looking only at candidates: the tracks whose component changed (`Changed<T>`, a new track included) or, once any indicator of that kind was removed (`RemovedComponents<I>`), every track; a track has one when a child of its own is an indicator that names it as owner, never a scan of all indicators, so an idle scene costs nothing and a re-inserted or despawned indicator leaves exactly one; an indicator reparented away from its track without being removed is a stray that the next pass puts back under the track it names (strays found by owner once per pass, only when a candidate lacks a child indicator; `an_indicator_reparented_away_from_its_track_is_taken_back_not_doubled`, #730) (`every_track_has_exactly_one_indicator_after_reinsertions`, `a_removed_indicator_is_made_again_once_and_other_tracks_keep_theirs`; the `indicators-animating` and `indicators-removed` bench workloads), a child absolutely placed behind the items with `ZIndex(-1)`, `Pickable::IGNORE`, hidden until placed) and `animate::<I, Track, Item>` (slides it to the laid-out box of the track's `Selected` item with an exponential ease, `INDICATOR_RATE` 18 per second, snapping at the first placement and under `INDICATOR_SNAP` 0.5 px, hiding it with no selected item). Each widget has its own indicator component (`SegmentIndicator`, public; `TabIndicator` named `TabIndicator`, `OptionIndicator` named `OptionIndicator`, crate-private), registers the two systems, and makes its items transparent (`Color::NONE`) so the indicator is the selected look; the text and icon colours still follow the state at once (`a_tab_bar_slides_its_indicator_to_a_newly_selected_tab`, `a_light_dark_toggle_slides_its_indicator_to_a_newly_chosen_option`, `a_segmented_control_slides_its_indicator_to_a_newly_selected_segment`) |
| `segmented_control.rs` | `SegmentedControl`: a track of `Segment`s (`Button`s; the control is one tab stop, at the selected segment, with the arrow keys moving and picking, a `RovingGroup`) over a `Selection` of `Item<String>`, own or `Selection::Ref`, normally a `One`; a segment's `Activate` sets the selection to its item (`set_selection`), `Selected` and the text colours follow the selection and hover. The selected look is a `SegmentIndicator` (a `segmented_selected` child of the control, absolutely placed behind the segments with `ZIndex(-1)`, spawned by the first frame after the control), which `indicator.rs`'s `animate` slides to the selected segment's laid-out box (exponential ease, `INDICATOR_RATE`, snapping at the first placement and under 0.5 px); the segments' own backgrounds are transparent (`a_segmented_control_slides_its_indicator_to_a_newly_selected_segment`). A segment shows the item's text, an icon, or both: an `ItemIcon(Handle<Image>)` next to the `Item<String>` on the item entity (like `ItemNote`; `(Item::new("Auto"), ItemIcon(handle))` works in a `bsn!` too) gives its segment a `SegmentIcon` (a plain 16 px `ImageNode` before the text, 6 px gap, `Pickable::IGNORE`, no accessibility node; not an `Icon`, since no `IconColor` role is `segmented_selected_text`) that `theme_segmented_controls` tints with the segment's text colour (`segment_colors(..).1`), so selection, hover and theme change keep it right. `SegmentLabels::{Shown (default), Hidden}` (immutable, required by `SegmentedControl`, like `ToolbarLabels`) hides the text (`Display::None`) of items that have an icon, which then show the icon alone with a `Tooltip` of the text (one the widget adds and marks with the crate-private `OwnedTooltip` of `tooltip.rs` (shared with `Toolbar`) holding the text it gave it, and removes again when the labels are shown; a `Tooltip` with other text, one the app put on a segment or swapped in over the widget's, is the app's and is never overwritten or removed, the marker being dropped when the widget next looks, so a hidden-label segment of the app's keeps its text; the one limit is a `Tooltip` with exactly the same text as the widget's own, which is indistinguishable from it, so it keeps the `OwnedTooltip` marker and is removed when the labels switch to `Shown`); an item without an icon keeps its text. `sync_icons` updates icon, text display and tooltip in place when an `ItemIcon` is added, changed or removed or `SegmentLabels` is inserted again (`changing_or_removing_an_item_icon_updates_its_segment_in_place`); the accessible name is always the item's text, so an icon-only segment is a named radio button (`segmented_control_icons_*.ron`). `sync_icons` finds a segment's icon and text among that segment's own `Children`, so a change costs work per segment, not per segment times every icon and label in the world (`many_icon_segmented_controls_keep_one_icon_and_hidden_label_per_segment`: 30 controls of 40 segments, 20 frames each changing one icon, 4.6 s before, 72 ms after). `selection_change` looks only at the controls whose own `Selection` or the one they refer to (one `Selection::Ref` hop) changed, so a change to one selection restyles no unrelated control (`a_segmented_control_follows_only_the_selection_it_draws_from`; `segmented-controls-selection-changing` bench workload, #709). Changing the text of an `Item<String>` updates its segment's text in place (`sync_label_text`, also the tooltip of an icon-only segment), and inserting or reordering items puts the segments in the items' order without respawning the others (`order_segments`, linear through `select.rs`'s `items_in_order`; `on_items_changed` reconciles with a set of wanted and a set of shown items, a second segment of one item is despawned, #679; the indicator stays after them) (`changing_an_items_text_updates_its_segment_in_place`, `an_item_inserted_in_the_middle_puts_its_segment_there`, `reordering_items_reorders_their_segments`). It is its own widget, not a `List` style, so `List` is untouched. To a screen reader the control is a `Role::RadioGroup` and each segment a `Role::RadioButton`, `toggled` following `Selected` (`segmented_control_accessible_radio_group.ron`); the segments keep `Button`, so `Role(role: Button, ..)` still finds them |
| `light_dark_toggle.rs` | `LightDarkToggle`: a three-way Light / System / Dark switch, a track of three `LightDarkOption`s (`Button`s; the toggle is one tab stop at the option of the current preference, with arrow keys, Home and End that move the focus and pick the option they land on, a `RovingGroup::marked_radio()`; each a drawn 16 px icon, sun, computer and moon, built by the module as images with the label beside it, no app assets), styled from the `segmented_*` slots like a `SegmentedControl`. The preference's look is an `OptionIndicator` (a crate-private child of the toggle from `indicator.rs`, as for a `SegmentedControl`) that slides to the option of the preference when it changes, also from elsewhere; the options' own backgrounds are transparent. Spawning it is all an app does: it inserts the `ThemePreference` resource (`Light`, `System` default, `Dark`) when the app has none, and an option's `Activate` sets it. `System` follows the operating system: `SystemTheme` (`Option<ThemeMode>`, `None` until known) is read from the primary window once (not when the window was spawned with a fixed `window_theme`: the crate-private `PresetWindowTheme`, put on by an observer of `Add<Window>` and, in `LightDarkTogglePlugin::build`, on the windows that already exist (`WindowPlugin` spawns the primary one before), keeps it `None`, since winit reports the forced theme back as the window's) and then from Bevy's `WindowThemeChanged` messages of the primary window only (`track_system_theme`, `PreUpdate`, also while a theme is fixed, as far as the platform reports it): a message whose `window` is not the current primary window is ignored, as are all of them while there is no primary window, and a primary window that is a new entity is read once like the first, the known theme staying until then (`theme_messages_of_a_secondary_window_do_not_change_the_system_theme`, `a_fixed_preference_ignores_theme_messages_of_every_window`, `theme_messages_are_ignored_without_a_primary_window_and_a_replacement_is_followed`). The preference sets `Theme::set_mode` (`apply_preference`; `System` before the system theme is known is light) and the primary window's `window_theme` (`apply_window_theme`: `Some` for a fixed preference, `None`, the system's, for `System`). An app fixes light or dark programmatically by inserting `ThemePreference::Light` / `Dark` (before or after the toggle), and one without a toggle never gets the preference, so nothing changes for it. The systems run only while the `ThemePreference` resource exists. To a screen reader the toggle is a `Role::RadioGroup` labelled "Theme" and each option a `Role::RadioButton` labelled with its text, `toggled` while it is the preference (`light_dark_toggle_click_sets_preference.ron`, `light_dark_toggle_keyboard_selects.ron`) |
| `tab_bar.rs` | `TabBar`: Bevy's `bevy_ui_widgets` `TabList` of text `TabBarTab`s (Bevy `Tab`s) over a `Selection` of `Item<String>`, own or `Selection::Ref`, normally a `One`, styled from the `segmented_*` slots like a `SegmentedControl`. The selected look is a `TabIndicator` (a crate-private child of the bar from `indicator.rs`, as for a `SegmentedControl`) that slides to the selected tab when the selection changes; the tabs' own backgrounds are transparent, and `order_tabs` keeps the indicator after the tabs. `SelectedTab` follows the `Selection`, only of the bars whose own or referenced selection changed (`selection_change` reads `Changed<Selection>` and `SelectionParam::get_ref`, so an unrelated bar is not restyled; `a_tab_bar_follows_only_the_selection_it_draws_from`), which keeps the last word: Bevy's `tablist_self_update` is not registered, and the `ValueChange<Option<Entity>>` the tab list sends for a click, Enter or Space goes through `SelectionParam::update_selection` (`set_selection`). The tabs are not `Button`s (a `Button` stops the click before the tab list sees it) nor `TabStop`s: Bevy's `TabPlugin` (added by `WidgetsPlugin` unless the app has it) gives the selected or focused tab `TabIndex` 0 and the others -1, so the bar is one tab stop and the arrows, Home and End move the focus between its tabs. A hidden bar (itself or an ancestor `Display::None`, as in a hidden `Stack` panel, or hidden visibility) gives all its tabs -1, so Tab skips it, and the roving 0 comes back when it is shown: `sync_tab_index` in `tab_bar.rs` runs after Bevy's and after visibility propagation and keeps both, looking only at bars that changed (new, `SelectedTab`, `Children`, a tab's `InteractionDisabled` or `TabIndex`, a focus move to or from one of its tabs, or the shared `HiddenDirty` of visibility, `Display::None` and parent; `tab-bars-idle` bench workload, #670), with `focus.rs`'s `is_hidden` (`a_hidden_tab_bar_is_not_reached_by_tab_and_is_again_once_shown`, `tab_skips_a_tab_bar_in_a_hidden_stack_panel`). Changing the text of an `Item<String>` updates its tab's text in place (`sync_label_text`), and inserting or reordering items puts the tabs in the items' order without respawning the others (`order_tabs`, linear through `select.rs`'s `items_in_order`; `on_items_changed` reconciles with a set of wanted and a set of shown items, a second tab of one item is despawned, #679) (`changing_an_items_text_updates_its_tab_in_place`, `an_item_inserted_in_the_middle_puts_its_tab_there`, `reordering_items_reorders_their_tabs`). To a screen reader the bar is a `Role::TabList` and each tab a `Role::Tab` labelled with its item's text, `selected` while it is the `SelectedTab` (`tab_bar_accessible_tab_list.ron`) |
| `setting_row.rs` | `SettingRow::new(title, description)` (`on_insert`: insert a new `SettingRow` on a live row to change its title and description; the title `Text`, description `Label` and accessibility node follow, and the children, `SettingRowRefs` and the app's control stay, `inserting_a_setting_row_again_changes_its_text_and_keeps_its_control`): a title (semibold, `font_size_body`) over a secondary description on the left and an empty control slot on the right (12 px gap and vertical padding), then a `Divider`; `SettingRowRefs { title_id, description_id, control_slot, divider_id }` names them, and the app spawns its control (a `SegmentedControl`, say) as a child of `control_slot`. Put the rows in a `SettingRows` column: a system hides the divider of the last row and shows the others'. `sync_dividers` looks only at columns whose `Children` changed (or that are new) and at the parent of a row whose `SettingRowRefs` just arrived, finds the last row by a reverse scan and collects nothing, so a quiet frame does no work; non-row children are skipped and a column with no row is left alone (`setting-rows-idle` and `setting-rows-changing` bench workloads, #689). A row is a `Role::Group` labelled with its title, its description the node's `description` (`setting_row_accessible_group.ron`) |
| `divider.rs` | `Divider::horizontal()` / `Divider::vertical()`: a 1 px rule in `ThemeColors::divider`, stretching along the other axis unless the caller sets a length |
| `icon.rs` | `Icon::bundle(handle, size)` / `Icon::bundle_with_color`: an `ImageNode` from an app-provided white-on-transparent `Handle<Image>`, tinted with the `IconColor` theme role of its `Icon` (primary text by default); a widget changes the role by inserting `Icon` again |
| `icon_button.rs` | `IconButton::new(handle, name, size)` with `IconButtonSize::{Regular, Small, Large}` (44 px / 22 px icon, 30 px / 16 px icon, 54 px / 24 px icon): a borderless `Button` of control radius with an `Icon` child (`IconButtonRefs`); `name` is the label of its `AccessibilityNode`, kept there by a system because Bevy clears a text-less button's label on add; inserting `IconButton` again swaps icon, name or size; small icons are secondary text until hovered; an app observes `Activate`; an optional immutable `ToggleState(bool)` makes it an on/off toggle: `Activate` inserts the flipped value (an app can insert one too), and an on button shows `accent_subtle`, an `AccentSubtleText` icon and a 2 px `accent_subtle_bar` child along the bottom edge, inset by the control radius to stay on the straight part (spawned with the first `ToggleState`). `IconButton::with_tone(Tone)` (`Neutral` default, `Warning`, `Danger`; also the tone of a `Tool`): `Warning` has `warning_background`, a 1 px `warning_border`, a `WarningText` icon and the large shadow, mixes 25 % of the border colour into the background when hovered or on, and when on shows a 3 px `warning_border` bar (a 54 px warning button with a `Badge` is the redesign's warnings button); `Danger` turns the icon `danger_hover_text` when hovered and is filled with `danger` and `DangerText` when on, without a bar. With a `ToggleState` its accessibility node is `toggled` with it; without one it has no `toggled` at all (`icon_button_accessible_toggle.ron`) |
| `kbd.rs` | `Kbd::new("Esc")`: a display-only key cap, a bordered box (`ThemeColors::border`, `text_secondary` text in the caption size; it reuses those slots and has none of its own); a `Role::Label` accessibility node with the key as value; immutable, re-insert to change the key; empty text keeps the empty box |
| `badge.rs` | `Badge { text, tone }` (`BadgeTone::Neutral` or `Warning`; `Badge::count(n)`): a 16 px high pill (min width 16, 4 px side padding, 10 px semibold text; height, min width and radius are `Val::Rem`, so the pill grows with its text when `RemSize` grows, which in pixels clipped the count) spawned as an absolute, pointer-ignoring child in the top-right corner of its host, on an `IconButton` or on the `Item<Tool>` of a `Toolbar` (mirrored onto that item's button). Neutral uses `pill_background` / `pill_text`, warning `warning_text` as fill with `warning_background` text. Immutable: insert a new `Badge` to change it (on an `Item<Tool>` too, `mirror_item_badges` copies it); remove it, or give it empty text, to hide it (no overlay, host description cleared; a nonempty one brings it back); the badge node is not in the accessibility tree, its text is the `description` of the host's `AccessibilityNode` (cleared when the badge is removed or empty). `mirror_item_badges` and `sync_badges` look only at what changed: hosts whose `Badge` changed or was removed, new nodes, changed `AccessibilityNode`s (Bevy rewrites them) and the buttons of a changed or new `Item<Tool>` or new `ToolbarButton`; a `Theme` change looks at all, a quiet frame does no work (`a_badge_follows_its_item_to_the_buttons_made_when_the_toolbar_style_changes`, `a_theme_change_recolours_a_badge_that_did_not_change`; `badges-idle` and `badges-changing` bench workloads, #685) |
| `pill.rs` | `Pill::new(text)` with a `PillTone` (`Neutral`, `Info`, `Warning` = waiting, `Problem`): a small uppercase status tag; the app picks the tone; immutable, re-insert a `Pill` to change tone or text (empty text keeps the empty pill); a `Role::Status` accessibility node labelled with the text as given |
| `filter_chips.rs` | `FilterChips` over a `One` `Selection` (own or `Selection::Ref`) of `Item<Chip>` (`Chip { label, count: Option<usize> }`): a wrapping row (6 px gap) of 30 px pill `ChipView`s, each a `Button` that picks its item on `Activate`, the row one tab stop with arrow keys (a `RovingGroup`); the active chip is filled with `chip_on` / `chip_on_ink`, the others outlined in `chip_border` with `text_secondary` text (`text_primary` on hover) and the count at 0.6 alpha. Changing the `Chip` in an `Item` updates its chip's text in place; adding, removing or reordering items reconciles the chips by item (`select.rs`'s `plan_views` / `order_views`): a chip whose item stays keeps its entity, focus and hover, only missing chips are spawned and stale ones despawned, and the row is reordered once only when the order differs (`a_structural_change_to_chips_keeps_the_views_and_the_focus_of_the_items_that_stay`; `filter-chips-editing` bench workload, #690). The respawn check (`sync_chips`) looks only at the widgets `select.rs`'s crate-private `StructureChanges::affected` names, so a frame without a change, or a change to another widget's source, builds no `Vec` of the items (`filter_chips_do_not_look_at_their_chips_in_a_frame_without_a_change`). Text size is `Theme::font_size_chip`. Styled by `theme_chips` (new chip, `Theme` change, hover, selection or item change), not every frame; `restyle_changed_chips` restyles only the chips of rows whose own or referenced `Selection` changed, the hovered chips and the chips of a changed `Item<Chip>`, never every chip, and a hover or item edit is handled in a frame where a selection changed too (`filter_chips_restyle_only_the_rows_whose_selection_changed`, `a_hover_and_an_item_edit_are_restyled_in_a_frame_where_a_selection_changed`; `filter-chips-selection-changing` bench workload, #709); the arrows move between chips and pick (see "Composite widgets"). To a screen reader the row is a `Role::RadioGroup` and each chip a `Role::RadioButton` labelled with its label and count (`Waiting 3`), toggled while active (`filter_chips_accessible_radio_group.ron`) |
| `gallery.rs` | `Gallery` over a `One` `Selection` (own or `Selection::Ref`) of `Item<GalleryCard>` (`GalleryCard { image, label, group }`): neighbouring items with the same `group` form a group, an overline `Label` over a row of 104 x 92 px `GalleryCardView`s (6 px gap; each a `Button` that picks its item on `Activate` with `toggle_selected`; the gallery is one tab stop with arrow keys through the cards in the order shown, a `RovingGroup`), groups side by side with a vertical `Divider` between them. A card is a `card_surface` with a 1 px `border` and control radius, a 72 x 30 `Icon` image above a 12 px semibold label that wraps; hover is `hovered_item`; the picked card has a 2 px `selection_ring` border and the `hovered_selected_item` background. The gallery does not shrink, so put it in a horizontal `ScrollArea` to scroll it. Changing the items or their groups rebuilds the cheap group scaffolding (dividers, overline labels, rows) but keeps the card of every item that stays, re-parented into its new row, so its entity, focus and hover survive (`plan_views` over the `GalleryOwned` descendants; `a_structural_change_to_a_gallery_keeps_the_cards_of_the_items_that_stay`; `gallery-editing` bench workload, #690); changing a `GalleryCard` updates in place. **Child ownership:** the gallery owns only the groups and dividers it spawns, marked with the crate-private `GalleryOwned`; a rebuild despawns just those, so children the app puts under the gallery entity (before the first build or later) are kept, with their state. Those present before the first build stay before the gallery's own; a child added later is not reordered, and a rebuild appends the new groups after it (`a_gallery_keeps_children_of_the_app_through_the_first_build_and_rebuilds`). `sync_cards` looks only at the widgets `StructureChanges::affected` names, so a quiet frame builds no groups to compare. To a screen reader the gallery is a `Role::RadioGroup` and each card a `Role::RadioButton` labelled with its item's label, toggled while picked |
| `toolbar.rs` | `Toolbar { style }` over a `Selection` (own or `Selection::Ref`) of `Item<Tool>` (`Tool { icon, label, group, key_hint, tone }`): a row of `ToolbarButton`s (`Button`s; the toolbar is one tab stop, left / right / Home / End move the focus only, a `RovingGroup`) that pick their item on `Activate` with `toggle_selected`, so a `One` toolbar keeps its tool and a `ZeroOrOne` one clears when the open one is activated; a vertical `Divider` goes between neighbouring tools of different `group`. `ToolbarStyle::Filled` is 66 x 60 px with a label under a 22 px icon, the active button filled with `accent` and `AccentText` ink; `Underlined` is 44 px icon-only, the active one `accent_subtle` with an `AccentSubtleText` icon and an `accent_subtle_bar` bar. A tool's `tone` (`Tool::with_tone`) recolours it: `Danger` turns icon and label `danger_hover_text` when hovered and fills the active button with `danger` and `DangerText` (a bulldoze tool); `Warning` shows `WarningText` on `warning_background` when hovered or active. The tool's label is the `AccessibilityNode` label in both. `ToolbarLabels` (`Shown` default, `Hidden`; immutable, required by `Toolbar`) hides the labels of a `Filled` toolbar: the label gets `Display::None` and the buttons shrink to 56 x 52 px, without respawning; `Underlined` is unaffected. Changing the items or their groups reconciles the buttons and dividers by key (`Slot`: item and hint, or divider; `plan_views`): kept buttons keep their entity and focus, dividers are regrouped, and a style change respawns the buttons (`a_structural_change_to_a_toolbar_keeps_its_buttons_and_regroups_the_dividers`; `toolbar-editing` bench workload, #690); changing a `Tool` updates in place. `sync_buttons` looks only at the widgets `StructureChanges::affected` names (`a_toolbar_does_not_look_at_its_buttons_in_a_frame_without_a_change`); `update_buttons` and `sync_tooltips` still visit the buttons each frame. A tool with a `key_hint` (`Tool::with_key_hint("1")`) gets an absolutely placed hint text (10 px semibold, 5 px from the top, 7 px from the right) in the button's ink at 60 % opacity, so it follows the active state; the library only shows it, the app binds the key. To a screen reader the toolbar is a `Role::Toolbar`; its buttons stay `Role::Button`, `toggled` true while their tool is active and false otherwise. A `ToolbarButton` has private fields (`item()`, `owner()`) and only the widget builds it; it has no style of its own, the style is its toolbar's, and `sync_buttons` respawns the buttons when the `ToolbarButtonRefs` variant no longer matches `Toolbar::style`. `ToolbarButtonRefs` is an enum, `Filled { icon, label, hint }` or `Underlined { icon, bar, hint }` (`hint` only for a tool with a `key_hint`), with accessors `icon()`, `label()`, `bar()`, `hint()` and `style()` (`toolbar_buttons_always_have_the_children_of_their_toolbars_style`). The buttons of an underlined toolbar get a `Tooltip` with their tool's label from `sync_tooltips`. A widget only touches a tooltip it added itself, marked with the crate-private `OwnedTooltip(text)` of `tooltip.rs` (shared with `SegmentedControl`): a `Tooltip` with other text, put on by the app, is kept, and the widget's own is removed again when the button no longer needs it (a toolbar not underlined). Limit: an app `Tooltip` with exactly the tool's label, put on before the widget's, is not told apart from it |
| `tooltip.rs` | `Tooltip { text }`: after the pointer (`Hovered`) rests on the node for `TOOLTIP_DELAY_SECS` (0.5 s) a `TooltipPopup` `Float` (`card_surface`, `border`, small `text_primary`) is spawned as a child of the node, hidden until laid out, then centred under the node, kept 6 px inside the primary window and put above the node when it would run off the bottom, its `top` never above the 6 px margin; a `Tooltip` with empty text shows no popup, and a showing one goes when the text becomes empty; it never takes the pointer. Leaving the node or a `Pressed` hides it, and a press keeps it hidden until the pointer has left and returned. Only active tooltips are visited: `track_active_tooltips` puts the private `TooltipActive` marker on a node when its `Hovered` or `Tooltip` changes and it is hovered, `update_tooltips` queries only marked nodes and takes the marker off once the pointer is gone and no popup shows, so a tooltip the pointer is not on is not visited (a hovered one with empty text still is, while the pointer stays; the marker arrives through `Commands`, so a hover is first counted in the frame after it starts), guarded by the `tooltip.rs` unit tests `only_a_hovered_or_showing_tooltip_is_visited` and `an_unmarked_tooltip_is_not_visited_in_a_quiet_frame`. While a `NumericStepper` scrub drag holds the crate-private `HoverLock`, no tooltip shows or counts its delay and a showing one goes (`a_tooltip_stays_hidden_while_a_numeric_stepper_is_scrubbed`). `IconButton` inserts one with its name; the buttons of an underlined `Toolbar` (labels hidden) get one with their tool's label. |
| `toast.rs` | `Toast::new(text)` (`with_duration`, 3 s default; negative and NaN mean 0, so it goes after its first tick, and anything above one day, infinity included, is capped to one day: `ToastTimer::for_duration` is the one place that turns the duration into a timer): a `Card` with the text, spawned as a child of the UI root (`ChildOf(root)`), which moves into the `ToastStack` that is a child of its own parent (an absolute, pointer-ignoring column along the bottom, centred, `GlobalZIndex(1100)`, created by the first toast under a parent, so two roots each get their own) where several stack upwards, newest at the bottom; a toast with no parent has no stack to join: it logs an `error!` and is despawned; a new toast with the text of one showing in the same stack, or of one kept earlier in the same run, is despawned and restarts that one's timer with its own duration (the last of several same-frame duplicates wins), and re-inserting a `Toast` (immutable, like the other display widgets) on a live entity updates text and accessibility label and restarts its own timer with the new duration, keeping its entity and its stack position; empty text keeps the card and normal lifetime. `ToastPlacement { bottom }` (resource, 24 px) is the stack's distance from the bottom edge, for an app to lift above its dock. `place_toasts` runs only while a toast has not moved into a stack (`any_fresh_toast`, a `Without<ToastShown>` query, so an idle frame builds nothing), looking parents up in a `HashMap` of the stacks, and `place_stack` runs only when `ToastPlacement` changed or a `ToastStack` was added (`placement_needs_applying`, so a stack made late still gets an unchanged placement); a stack that has emptied stays, one small node per parent that the next toast there reuses (`toasts-idle` and `toasts-bursting` bench workloads, #678; `a_stack_made_after_idle_frames_gets_a_placement_set_before_it_and_left_unchanged`, `bursts_of_toasts_under_many_parents_keep_one_stack_each_across_idle_frames`, `reinserting_a_shown_toast_after_idle_frames_updates_it_in_its_stack`). A system ticks a private timer with `Time` and despawns the toast when it runs out. `Pickable::IGNORE`, no tab stop; a `Role::Status` node (polite live region) labelled with the text |
| `text_field.rs` | `TextField::new(value)` with `with_placeholder`, `with_name`, `with_min_width` / `with_max_width` (`Val`s, `Val::Px(160.0)` / `Val::Px(320.0)` by default: Bevy layout constraints passed unchanged to the field's `Node`, so `Auto`, percentages of the parent, viewport units and the rest work and are resolved by layout, logical pixels only for `Val::Px`; nothing is converted to pixels or compared before layout, and when both resolve with the minimum above the maximum the minimum wins as in Bevy's layout, so a 400 px minimum with the default 320 px maximum is 400 px wide: a documented rule, not an error, neither bound is clamped nor logged; `a_text_field_with_a_minimum_above_its_maximum_is_as_wide_as_the_minimum` and the neighbouring layout tests in `tests/scenarios.rs`; inserting a new `TextField` replaces the widths in place, without respawning the input or a `TextFieldChange`): a single-line text input over Bevy's `TextInput` / `EditableText`. The field is a bordered box (1 px `border`, `card_surface`, control radius, 10 / 7 px padding) holding a `TextInput` child (`TextFieldRefs::input`, a tab stop, body text in `text_primary`, no wrap, so the text scrolls inside the box instead of growing it) and an absolutely placed placeholder (`TextFieldRefs::placeholder`, `text_tertiary`, shown while the text is empty). Bevy's `PointerFocusPlugin` gives the input the focus on a click and takes it on a press elsewhere, and an observer of `AcquireFocus` on the box gives it to the input for a click on the box's border too (`text_field_border_click_focuses.ron`); `Enter` (also `NumpadEnter`) gives it up, `Escape` puts back the text from when it took the focus and gives it up (both by key code, so the tester's `Tap` works; Bevy's own `Escape` reads the logical key); both call `propagate(false)`, so neither key bubbles to the field's ancestors (`enter_and_numpad_enter_commit_the_text_and_stop_there`, `escape_puts_the_text_back_without_a_commit_and_stops_like_enter`, `tab_commits_the_text_even_when_it_did_not_change`, `inserting_a_new_text_field_sets_the_text_without_a_change_event` in `tests/headless.rs`, which adds `TextPlugin` so edits apply). Entity events on the field: `TextFieldChange { value }` on every edit (and an `Escape` restore), `TextFieldCommit { value }` when the focus leaves other than by `Escape` (Enter, Tab, a press elsewhere), changed or not. The focus ring is on the box, after a click too (`RingOn` / `RingTarget` in `focus.rs`). Insert a new `TextField` to set the text (no change event), placeholder, name or widths. To a screen reader the input is a `Role::TextInput` with its text as value and `name` as label (`text_field_accessible_text_input.ron`). It needs `TextInputPlugin` and `PointerFocusPlugin` from `UiWidgetsPlugins` (in `DefaultPlugins` and the tester's `HeadlessPlugins`); `WidgetsPlugin` does not add them |

### Examples

Each is `just example NAME` (`cargo run --example NAME`) and needs a window.

| Example | Shows |
| --- | --- |
| `overview` | the one example of every widget that stands alone, in `examples/overview/` (`main.rs`, and `ui/` with `mod.rs`: the plugin, the tab list, the layout helpers `page` / `groups` / `row` / `variant`, plus a module per group of pages: buttons, text, lists, combos, colours, selection, navigation, values, toolbars, cards, scroll_area, feedback, theme_editor). 21 pages, among them `FilterCombo`, `LightDarkToggle` and the `ThemeEditor` (last, with "Change theme" and "Export", which logs the edited theme as Rust text through the page's `colors_source` / `pair_source` / `numbers_source` / `theme_source`; the library only edits), behind a vertical `TabBar` driving a `Stack` in a vertically scrolling `ScrollArea`; the `LightDarkToggle` above the tabs switches the theme. Every page has the same layout: a `Label::header`, then the variants in a centred, wrapping row, each a `variant(caption, content)` group with a `Label::secondary` naming what differs; the page node is named `Page` and as wide as the area, so pages do not shift when switching tabs. A new widget or variant goes on its page here, not in an example of its own. `Stack` is shown on the "Tabs and trails" page twice: `BreadCrumbs` driving panels with push, pop and replace-the-last-crumb buttons (`StackItem::new`, `new_with_reference` and `new_reference`), and a `TabBar` driving panels of palette combos, a card, a list beside a 100-item scroll area and a 300-card two-way grid; the "Combos" page has a list and combos sharing one `ZeroOrMore` selection with "Add List" / "Add Combo" / "Add PushButton" buttons. Left out on purpose: `Float`, `Icon`, `tailwind`, the crate-private modules and the `hud`. Icons are drawn at startup with `examples/common/mod.rs` (`mod common;`; a directory without a `main.rs` is not an example). `tests/headless.rs` includes `examples/overview/ui/mod.rs` with `#[path]` and shows every page (`the_overview_example_builds_and_every_page_runs`); `tests/scenarios.rs` does too to lay it out in the default 1280 x 720 window (`every_page_of_the_overview_is_centred_in_the_same_area_and_fits_the_window`) |
| `hud` | the redesign's HUD composed from the widgets over sample data, no game: pause button, `Kbd` and a `Slider` for the simulation speed (0.25x to 8x in octaves, written to the example's own `SimulationSpeed`), a panel `Toolbar` with badges driving a `Stack` of `Panel`s (trains with `FilterChips` and `Pill` rows, stops, warnings, preferences with `SettingRow`s, among them a "Text size" `NumericStepper` that sets `RemSize` and a "Company name" `TextField`; the hotkeys are off while a text field has the keys and on the frame it hands them back), and a tool dock whose active tool brings an option bar (`SegmentedControl`s, `NumericStepper`, vehicle `Gallery`). The "Show" buttons of the stops and warnings panels raise a `Toast`. Its icons are drawn at startup. The UI lives in `examples/hud/ui/mod.rs`, which `tests/headless.rs` includes with `#[path]` for a smoke test |
| `bench` | the headless CPU and memory benchmark of the widgets (`just bench`, an optimised build): per-frame time (mean, median, p95, max), startup time (spawn plus the first three frames, #735), entities and RSS for `push-buttons`, `tooltips`, `virtual-list` and the like, each beside a plain-Bevy baseline so the library's overhead is the difference. No allocation counter (it needs `unsafe`, which is forbidden). The `ticks-*` workloads compare `Changed<T>` with Bevy 0.20 `summary_tick` readers; summary ticks were measured and not adopted, and the headless widgets of 0.20 were reviewed (#656, recorded in `docs/benchmarks.md`). The `steppers-*` workloads cover `NumericStepper` value sync (#663). The `deep-trees-*` workloads cover hidden-state change collection (#684). The `indicators-*` workloads cover the sliding-indicator lifecycle (#677). The `palette-combos-idle` and `theme-editors-idle` workloads cover closed palette popups (#680). `wheel-deep` and `wheel-deep-idle` cover the wheel handler over a deep tree (#682). `tooltips-hovering` covers hover and popup cost with a fake pointer, `bevy-text-children` is a closer `push-buttons` baseline (a parent and a child per button), and an unknown `--workload` is an error listing the valid names (#704). `badges-idle`, `badges-changing`, `panels-idle` and `panels-changing` cover badge and panel synchronization (#685). `filter-chips-editing`, `toolbar-editing` and `gallery-editing` cover small structural edits (#690). `setting-rows-idle` and `setting-rows-changing` cover the `SettingRows` divider sync (#689). `sliders-idle` and `sliders-pressing` cover slider restyling (#698). `filter-chips-selection-changing` and `segmented-controls-selection-changing` cover a selection change restyling only its own widgets (#709). Not part of `just ci`; see `docs/benchmarks.md` |
| `gallery` | the tester's gallery: the widgets in one window, and a **Scenarios** window listing every file of `tests/ron/` (one group per widget) to watch play, step, take over or record; see [The gallery](#the-gallery) |

**The examples build their trees with `bsn!`.** Every example but `gallery` (whose few nodes are plain spawns)
spawns its nodes as scenes (`bsn!` / `bsn_list!` and `spawn_scene`), with
`on(..)` for observers; a single entity may still be a plain `spawn`, as may
the `tests/widgets_setup` screens. **Never change a library component for
`bsn!`**: no `Default`, `Clone` or other derive added only so a scene can
patch it. Insert a prepared value instead: `~{PushButton::from_label("Add")}`
or `~{value}` from a variable. A template's output must be a component, which
the scene inserts together with the entity's `ChildOf` and its other
components. For the same reason a scene spawned under a parent outside a
parent scene is `commands.spawn(ChildOf(parent)).apply_scene(..)`, never
`spawn_scene(..)` followed by an insert of `ChildOf`. What does not fit `~{..}`
has a way of its own:

- `~{..}` takes only an `Unpin` value, and a `Handle` is not one, so a widget
  holding one has a `FromTemplate` and is written as a struct in the scene, a
  handle as `icon: {icon.clone()}` (a field value that is a method call or a cast
  goes in braces):
  `IconButton { icon: {icon.clone()}, name: "Pause", size: IconButtonSize::Regular }`
  (derived), `Panel { title: "Trains", close_icon: {HandleTemplate::Handle(close.clone())} }`
  and `NumericStepper { decrease_name: "Lower" }`, written by hand as
  `PanelTemplate` and `NumericStepperTemplate`: their defaults are those of
  `Panel::new("")` and `NumericStepper::default()`, which a derive would not give
  (a `format` function is `{level as ValueFormat}`). A new widget holding a handle
  gets one too.
- `Selection::builder()` and `Divider` have scene constructors, written with
  `@{..}`: `@{Selection::builder().one().scene_with_items(items)}` (the items are
  cloned for each spawn), `scene_with_items_ref(entity)`,
  `@{Divider::vertical_scene()}` and `horizontal_scene()`. The bundle forms stay
  for code that spawns with `Commands`.
- `Stack` derives `FromTemplate`, so a scene writes `Stack { selection_ref: #tabs }`
  with a `#name` of an entity in the same scene (the `tabs` and `breadcrumbs`
  examples put the selection on the `TabBar` / `BreadCrumbs` through `@{Selection::builder()..scene_with_items(..)}`);
  a selection the application also keeps (the `hud`'s resources) is spawned
  first and passed as `Stack { selection_ref: bar }`.

`the_hud_example_builds_and_runs` in `tests/headless.rs` adds `AssetPlugin` and
Bevy's `ScenePlugin`, which spawning a scene needs.

Every example has a way to toggle the light and dark theme (the gallery's
button is on its screens, the `hud`'s is the Appearance setting of its Preferences
panel, the `overview`'s a `LightDarkToggle` above its tabs), and all but `gallery` set `GlobalUiDebugOptions`, which is
why `bevy_ui_debug` is switched on in `[dev-dependencies]`.

### The gallery

`examples/gallery.rs` uses the tester's `gallery` feature (switched on for the
`[dev-dependencies]` entry in `Cargo.toml`, so it also builds with the tests).
It needs a display, and `just gallery-test` (`--run-all`) plays every scenario
once in a real window and exits non-zero naming the failures; it is not part of
`just ci`. Words after `--run-all` keep only the files whose name contains one
of them (`just example gallery -- --run-all stack link`).

- **The tests' enums are shared, not copied.** The example pulls
  `tests/common/mod.rs`, `tests/widgets_setup/mod.rs` and
  `tests/widgets_check/mod.rs` in with `#[path = "../tests/.."] mod ..;`, so the
  scenario files run against the same `WidgetsSetup` and `WidgetsCheck`, and the
  library depends on nothing of the tester.
- **The catalogue is `tests/ron/`**, read when the example starts, grouped by
  the start of the file name (`GROUPS` in the example; a new widget's files need
  a row there or land in "Selection"). A file that does not load is skipped
  with a warning.
- **The reset hook** (`GalleryPlugin::with_reset`) despawns every `UiRoot`, every
  `Camera2d` and every `Selection` and `ItemOf` entity that is not
  `NotUnderTest` (setups spawn selections beside the UI, not under it), and puts
  `Theme` back; the scenario's `WidgetsSetup` builds the screen again. Leave
  the selections out and stale ones make later scenarios fail.
- **The window has the display's scale factor.** The files state logical
  values (see "Scenario files"), so they pass at any scale factor.
- **`AppSetup::capture`** returns the `WidgetsSetup` the app was last built from
  (kept in the `CurrentScreen` resource), so **Record** and **New: current
  state** write `setup: App(Lists)` and the file loads back. What was done to
  the screen before recording started is not captured. There is no `AppView`:
  the widgets have no view to name.

### Tests

| Target | Holds |
| --- | --- |
| `tests/headless.rs` | widget behaviour driven by hand-written triggers in a headless `App` |
| `tests/reflection.rs` | the widget types and `Theme` are in the type registry |
| `tests/scenarios.rs` | Rust scenarios played by `bevy_stagecraft`, a smoke test that every `WidgetsSetup` screen builds, the tests of `WidgetsCheck`, and generated per-file tests that play each `tests/ron/` file with the UI drawn to an image (`image_target_app`) and under shuffled schedules, see [Determinism](#determinism) |
| `tests/ron_scenarios.rs` | Generated libtest cases for every `.ron` file and each of its default, scale 2 and text 1.5x variants, see [Scenario files](#scenario-files) |
| `tests/recorded_scenarios_gpu.rs` | needs `--features record` and a GPU, and is `#[ignore]`d: records every `tests/ron/` file offscreen, see [Recording the scenarios](#recording-the-scenarios) |
| `tests/common/mod.rs` | `widgets_app()` and `spawn_camera_and_root`, shared by the scenario targets |
| `tests/widgets_setup/mod.rs` | `WidgetsSetup`, the screens scenarios start from, one per widget, each started from its example but a fixture of its own |
| `tests/widgets_check/mod.rs` | `WidgetsCheck`, the checks about widget state the tester's built-in ones cannot say (`Selected`) |

## How the pieces fit

- **`UiRoot`.** An observer on `OnAdd, Node` inserts `UiRoot` on a node
  without a parent and without a `UiTargetCamera`, so root nodes aimed at
  another camera (a second window's UI, an overlay) are not the app's and are
  left alone. The camera is read when the `Node` is added, so a
  `UiTargetCamera` inserted later is not seen: spawn it in the same bundle. An
  observer of `Insert, ChildOf` removes `UiRoot` again from a node parented
  after its `Node` was added (a scene writes `ChildOf` last), so the marker
  follows the hierarchy. A `PostStartup` system logs an `error!` (it does not
  panic) unless there is exactly one `UiRoot`; an app should spawn one. Two
  roots both get `UiRoot`, so a widget must not take `Single<.., With<UiRoot>>`;
  a popup is a `Float` child of its widget instead, which needs no root at all.
  The tester's cursor overlay spawns its roots with a `UiTargetCamera`, so they
  get no `UiRoot` (`the_cursor_overlays_roots_get_no_ui_root` in
  `tests/scenarios.rs`).
- **Mouse events.** Global observers (registered by `WidgetsPlugin`, not on
  `UiRoot`) watch Bevy's flat pointer events `PointerOver`, `PointerPress`,
  `PointerRelease`, `PointerClick`, `PointerDrag` and `PointerOut` (one generic
  observer over the `PointerEvent` trait) and re-trigger each as a `MouseEvent`
  on the original target, on the first hop of the event only. `MouseEvent` is
  crate-private: it is how widgets inside this crate react to the pointer. They
  are global because Bevy's `Button` calls `propagate(false)` on `PointerPress`,
  `PointerRelease`, `PointerClick`, `PointerDragEnd` and `PointerCancel`: an
  observer on `UiRoot` would never see those for a button. `PointerOver`,
  `PointerDrag` and `PointerOut` are not stopped. `MouseKind` says what the pointer did:
  `Over`, `Pressed`, `Released`, `Click(button)` and `Drag`, plus `Out`. `Drag` carries the pointer's `distance` from where the press began (logical pixels), which the `NumericStepper` scrubs by.
  `Click` is a press and a release on the same node, which `Released` cannot
  say (it also ends a press that began elsewhere), and `Drag` is a move with a
  button held, which widgets ignore but the `NumericStepper` (the scroll thumb's drag is the
  `bevy_ui_widgets` `Scrollbar`'s). The combo's outside-click handler
  reads `ButtonInput<MouseButton>`, not a pointer event, so a `Button` cannot
  starve it.
- **"Is the pointer on it?" is picking's `Hovered`.** A system that asks where
  the pointer is now, outside a `MouseEvent`, reads `bevy::picking::hover::Hovered`
  (required by the component it asks about): true while the mouse pointer is on
  the entity or on any of its descendants, as picking's `HoverMap` says. The
  outside-click systems of `Combo` and `ColorPaletteCombo` read it on their
  button (`ComboButton`, the palette combo's `ColorPaletteComboButton`) and popup
  (`ComboList`, `ColorPaletteComboFloat`), and the `selection_change` of
  `list.rs` and `color_palette.rs` on each `ItemView`, to keep the hover colour of
  the item under the pointer. It is on the button rather than the combo because a
  `Combo`'s popup list is its descendant, so the combo's own `Hovered` would be
  true over the list as well. Bevy's `RelativeCursorPosition` is not used: it is
  only computed for cameras that render to a window, so UI drawn to an image (the
  recorder, any render-to-texture UI) never had the pointer on it (#100).
  `Hovered` follows the hover map, not the geometry: a node is only hovered when
  nothing above it that blocks the pointer (`should_block_lower`) is in the way,
  so an item under an open popup is not. A node with `should_block_lower: false`
  (a combo's button, an `ItemView`) is hovered and lets what is below be hovered
  too; a `Pickable::IGNORE` node (an item's label) is never in the hover map, so
  only the node it lies on counts. It follows `PointerId::Mouse` only, not touch;
  the outside click reads the mouse button anyway. It is updated in `PreUpdate`,
  so an `Update` system sees this frame's pointer.
- **Buttons are `bevy::ui_widgets::Button`.** Every button
  (`PushButton`, `ColorPushButton`, `Link`, `IconButton` and the rest) is a
  headless `bevy_ui_widgets` button, and nothing uses the deprecated `bevy::ui`
  `Button` or `Interaction`. The scroll thumb is no button: a `bevy_ui_widgets`
  `ScrollbarThumb` without a `Node` of ours, sized and placed by its `Scrollbar`,
  that requires `Pickable::default()` so it blocks the pointer and gets the
  `MouseEvent`s of its hover colour.
  A button: `Pressed` while held, `Activate`
  on release over the button and on Enter / Space for a focused one, never while
  `InteractionDisabled`. `WidgetsPlugin` adds `ButtonPlugin` (and `ListBoxPlugin`, `ScrollbarPlugin`, `SliderPlugin` and `TabPlugin`) in
  its `Plugin::finish`, unless the app has it by then (`DefaultPlugins` and the
  tester's `HeadlessPlugins` add all of `UiWidgetsPlugins`; adding twice panics),
  so the order of `WidgetsPlugin` and `UiWidgetsPlugins` does not matter; an app
  driven by hand with `App::update` must call `App::finish` first. A
  `ColorPushButton` with `InteractionDisabled` has its border in `text_disabled`
  (it has no hover colours to ignore). **`Activate` and `MouseEvent`:**
  "the button was used" is `Activate`, so an action observes `Activate` on the
  button itself: it does not bubble (`bevy_ui_widgets`' `Activate` has no
  `propagate`, `an_activate_does_not_bubble_to_an_observer_on_an_ancestor`), so an
  observer on an ancestor never hears it; how the button *looks* under the
  pointer (hover and press colours) is `MouseEvent`. Never do both for one
  thing. A disabled `PushButton` (insert
  `InteractionDisabled`) shows its text in `ThemeColors::text_disabled` and
  ignores hover. A `Link` is disabled the same way: insert `InteractionDisabled`
  to disable it and remove it to enable it again (there is no `LinkState`); it
  keeps its `Button` and `Role::Link`, shows plain text in `text_primary`, ignores
  hover and press, and `restyle_on_disabled_change` in `link.rs` restyles it when
  the marker is added or removed, also under the pointer.
  The crumbs of `BreadCrumbs` observe `Activate`; the current crumb is a
  `Link` with `InteractionDisabled`.
- **A list can render its own item views.** `ItemViewCreator` (next to the
  `List`, a registered system, like `Stack`'s `StackChildSpawner`) is run once
  per item with an `ItemViewSpawner { item, view }` and spawns the contents
  under the `ItemView`, which the list still owns, spawns and despawns with
  `ItemsChanged` and themes. The items need not be `Item<String>`. Such a view
  is `CustomView`: its children take the pointer, so hover colours come from
  `Hovered` (`custom_view_hover`) and press and release from a global
  `MouseEvent` observer that walks up from the target to the view
  (`custom_view_of`), ignoring a target inside a nested `Button`, `List`, `Combo` or `FilterCombo`
  (which handle their own press). A `Button` stops the click that hits it, but a
  combo's root node and popup let the pointer through, and their click bubbles
  through the row to the outer `ListBox`, which takes any click under a row as a
  pick of that row; so each custom view stops a click whose original target lies
  in a nested `Button`, `List`, `Combo` or `FilterCombo` (`keep_nested_clicks_from_row`,
  `list_combo_in_custom_row_pick_leaves_outer_list.ron` and
  `list_filter_combo_in_custom_row_pick_leaves_outer_list.ron`, the `ComboRows` and
  `FilterComboRows` screens). Selection and `Selected` are as
  for default views.
- **Lists are `ListBox`es, and `Selection` keeps the last word.** A `List` is a
  `bevy::ui_widgets::ListBox` and its `ItemView`s are `ListItem`s (`Selectable`,
  `Role::ListItem`); `WidgetsPlugin` adds `ListBoxPlugin` unless the app has it.
  `Selected` is Bevy's marker for an item in the selection, and `selection_change`
  in `list.rs` keeps it in step with the `Selection` (own or `Selection::Ref`), so
  it is on exactly the items of the selection, and `ListBoxMultiSelect` is on the
  list while the selection allows more than one item (`OneOrMore`, `ZeroOrMore`).
  **The choice:** the `ListBox` observers send `ValueChange<Entity>` for a click
  on a row (except the first selected row, see below), and for Enter / Space on
  the active row, and insert and remove `Selected` only if the app registers `listbox_update_selection`.
  We do not register it: `on_value_change` turns the `ValueChange` into a
  `SelectionParam::update_selection` instead, so `One`, `ZeroOrOne`, `OneOrMore`
  and `ZeroOrMore` (and `None` for a disallowed change) keep deciding, and
  `Selected` can never disagree with the `Selection`. The `ListBox` ignores a click
  only on the first selected row (`listbox_on_row_click` compares the row with the
  current one); on any other selected row it still sends the `ValueChange`, and a click
  on the first selected row, like Enter or Space on an active row that is already
  `Selected`, sends a `ReselectListRow` instead. `on_value_change` and `on_reselect_row`
  (`list.rs`) both go through `toggle_selected`, so a click or a key on a selected row
  deselects it as far as the selection allows (`One` keeps its item) and the `ListBox`'s
  events are the only path: no `MouseEvent::Released` deselects, or a click on the first
  selected row would toggle twice (`list_click_deselects_once.ron`,
  `list_keyboard_deselects.ron`, `color_palette_keyboard_deselects.ron`). Both ignore a row
  whose `ItemView::list_ref` is not the event's source list: a nested list (a combo's popup
  in a custom view) reports the row of the list around it, because `listbox_on_row_click`
  searches above the popup when the click's target is the popup itself. Hover and
  selected colours stay on `MouseEvent`. A `ListBox` also sets an
  `ActiveDescendant` on a click and, once the list has focus (a `List` is one tab stop),
  handles the arrow keys (all four move, wrapping round), Home and End by moving it, and
  Enter / Space by sending the `ValueChange` of the active row, which goes through
  `on_value_change` like a click, so the `Selection` decides, also a `Selection::Ref` and a
  palette's. Moving the active row selects nothing.
  The list's focus ring is drawn on the active row, not on the box (see "Keyboard
  focus"). `ItemView`s keep `should_block_lower: false` so the wheel
  reaches the scroll area, which also lets a click through to what is below: a popup
  that floats over other widgets blocks the pointer itself (`Combo`'s list has
  `Pickable::default()`).
  A `ColorPalette` is a `List` with `NoDefaultItems`, so it is a `ListBox` of
  `ListItem`s as well and follows the same pattern: `color_palette.rs` puts
  `Selected` on its swatches (and `ListBoxMultiSelect` on the palette) from its
  `on_selection_change`, using `list.rs`'s `sync_selected` / `sync_multi_select`,
  `list.rs`'s `on_value_change` turns the `ListBox`'s pick into a selection update
  for palettes too, and `on_reselect_row` deselects a swatch that is `Selected`.
  A swatch has no text, so its accessible name is the name of its colour (see
  `color_palette.rs` in the module table); a `Role(role: ListItem)` locator without a label
  still reaches all of them, so the scenarios use `Nth(..)` or
  `App(Accessible(..))`.
- **Keyboard focus: tab stops and the focus ring.** `WidgetsPlugin` adds Bevy's
  `InputFocusPlugin`, `InputDispatchPlugin` and `TabNavigationPlugin` in
  `Plugin::finish` unless the app has them by then (the tester's `HeadlessPlugins` has the first two, not the third), and
  `UiRoot` requires `TabGroup::new(0)`, so the widget tree is one non-modal tab group.
  `PushButton`, `ColorPushButton`, `Link`, `Slider` and `List` (so a `ColorPalette` as well)
  require the crate-private `TabStop`, which requires `TabIndex`. `sync_tab_index`
  owns the value: 0 for a stop that is not `InteractionDisabled` (a disabled
  `Link` has it), -1 otherwise or while it has the crate-private `Unreachable` marker (the
  list of a closed combo, put on and taken off with `ComboState` / `ColorPaletteComboState`) or while it is
  hidden (its `InheritedVisibility` is hidden, as Bevy's UI renderer reads it, or itself or an
  ancestor is `Display::None`, as in a hidden `Stack` panel; it is Tab-reachable again once
  shown). `sync_tab_index` runs in `PostUpdate` after `VisibilitySystems::VisibilityPropagate`,
  so an app needs Bevy's `VisibilityPlugin` (in `DefaultPlugins`, and the tester's
  `HeadlessPlugins` through `CameraPlugin`), or every stop counts as hidden; **no widget writes `TabIndex` by hand** (the roving groups below are `focus.rs`'s too, and `TabBar` keeps Bevy's). Within the group
  the order is `TabIndex`, then tree order, which is spawn order. The focused `TabStop`
  gets an `Outline` (2 px wide, 1 px offset, `ThemeColors::focus_ring`) while
  `InputFocusVisible` is true (an `Outline` the app or a widget set on it is kept aside and put back when the ring leaves, so the ring owns only its own), except that a list's ring is on its `ActiveDescendant` row
  (a `ListItem`) and the box has none, and a focused `Tab` of a `TabBar` gets the ring too
  (its `TabIndex` is Bevy's `TabPlugin`'s, kept at -1 while the bar is hidden by `tab_bar.rs`, not `focus.rs`'s `sync_tab_index`); the `ListBox` clears the active row when the list
  loses focus, and the ring goes with it, and `ActiveDescendant` is registered in the type
  registry for checks (`App(Active(of: .., is: ..))`, `WidgetsCheck::Active`); a click focuses without showing it (Bevy's
  `TabNavigationPlugin`), Tab shows it. Scenarios check the focus with
  `App(Focused(of: ..))` (`WidgetsCheck::Focused`) and press Tab with `Tap(key: Tab)`
  (`tests/ron/keyboard_tab_order.ron`). A focused entity with the crate-private `RingOn(target)` (a `TextField`'s input) has the ring on its `RingTarget` (the field's box) whether the focus is visible or not, so a clicked field shows where the keys go. `keyboard_focus_ring_in_both_themes.ron` Tabs to "Change theme" and toggles the theme, to record the ring on both. The `*_keyboard_*.ron` files drive lists, palettes and the tab bar (`tabs_keyboard_switches_panel.ron`) with the arrow keys. The lists of closed combos are skipped by Tab, so the palette scenario Tabs 8 times.
- **Composite widgets are one tab stop with arrow keys.** A `SegmentedControl`, `FilterChips`,
  `Gallery`, `Toolbar` and `LightDarkToggle` each require a crate-private `RovingGroup` (`RovingGroup::radio()`
  for the first three, `::toolbar()`, `::marked_radio()` for the toggle) and each item button (`Segment`, `ChipView`,
  `GalleryCardView`, `ToolbarButton`, `LightDarkOption`) carries a `RovingItem { owner, item }`, put on when the
  widget spawns it. `sync_tab_index` leaves those buttons alone; `sync_roving_groups` (same
  module, `PostUpdate` after it) owns their `TabIndex`: 0 for exactly one eligible button, -1 for
  the rest, and the container is no stop. Eligible means not `InteractionDisabled`, not
  `Unreachable` and not hidden (the same `is_hidden` rule); a group with no eligible button, or
  none at all, has no stop. The stop is the focused item (the roving choice follows the focus,
  also after a click), else the group's entry: `RovingEntry::Selected` (the radio widgets) the
  selected eligible item, else the first; `Marked` (the `LightDarkToggle`, whose state is the `ThemePreference` resource, not a `Selection`) the eligible option that has Bevy's `Selected` marker, which `theme_toggles` keeps in step with the preference (a `Selected` added or removed on an item counts as a change of its group, so the entry follows an app that inserts `ThemePreference`), else the first; `Last` (the toolbar) the tool that had the focus last
  while eligible, else the first. Items are found by walking the group's descendants in tree
  order, which is the order shown (a gallery's groups included), so wrapping and Home / End are
  by item order, never by geometry. Only the groups that changed are looked at: an item added,
  disabled, hidden or removed, the group's children, a change of the `Selection` the group reads (its own or the one its
  `Selection::Ref` resolves to; radio groups) or a move of the focus into or out of the group (the group that held it, the
  owner of the newly focused item); nothing walks every frame, and an unrelated group is never looked at
  (`a_selection_or_focus_change_looks_only_at_the_groups_it_concerns`,
  `a_shared_selection_change_looks_at_every_group_that_reads_it_and_no_other`; the `roving-groups-*` bench workloads). `navigate_roving_group`, a global observer of
  `FocusedInput<KeyboardInput>`, acts only when the `InputFocus` is on one of the group's own
  items: the radio widgets use left / up and right / down (previous and next, wrapping) and Home
  / End, the toolbar left / right and Home / End (up and down go on to the app). It stops those
  keys from bubbling (pressed, repeated and released, also with one eligible item), moves the
  focus with `FocusCause::Navigated`, makes it visible, asks `ScrollIntoView` for the new item
  and, for a radio group over a selection of at most one (`One`, `ZeroOrOne`), picks it with
  `set_selection([item])`, never toggling, so wrapping to the same item keeps it; the toggle's `RovingPick::Activate` instead triggers `Activate` on the option landed on, which sets `ThemePreference` exactly as a click does (and changes nothing when it is the current one). With
  `OneOrMore` / `ZeroOrMore` the arrows move the focus only; Enter / Space keep their
  `Activate` behaviour (`toggle_selected`), as does a toolbar's. When the focused item goes
  (despawned, disabled, hidden) the focus moves to the next eligible item, else the previous,
  and is cleared when none is left; focus that went to another widget, or that something cleared,
  is never taken or put back (`holds_focus` remembers that the group had it; a despawned item
  keeps the `InputFocus` until the next key, which is how a stale focus is told from a cleared one). A `Selection::Ref` shared by several groups
  is read per group. Tests: `*_roving_tab_stop.ron` for each widget (Tab, Shift+Tab with
  `Key(ShiftLeft)` held, because the tester's `Tap(.., modifiers: [Shift])` releases Shift in
  the frame it presses it, which Bevy's Tab navigation does not see), and the
  `composite_screen` tests of `tests/scenarios.rs` (held arrows, an app's directional
  navigation, disabled / hidden / empty groups, removal, reorder, a pick that lands on the
  already selected item, shared selections). The keyboard
  behaviour of these four widgets changed in #493; nopjes tabs once into each and uses the
  arrows inside.
- **Directional navigation is the app's.** The library adds none: an app that wants the
  arrow keys (or a gamepad stick) to move the focus between widgets puts Bevy's
  `AutoDirectionalNavigation` on the widgets that take part, adds
  `DirectionalNavigationPlugin`, and calls `AutoDirectionalNavigator::navigate` from an
  observer of `FocusedInput<KeyboardInput>` on the primary window, which gets only the keys
  no focused widget kept (a system reading `ButtonInput<KeyCode>` would see them all); it
  sets `InputFocusVisible` itself, as Bevy's `directional_navigation` example does. What the
  widgets guarantee: a widget that uses the arrows stops them from bubbling on, so they
  never reach that observer. A focused `List` (so a palette and an open combo's
  list too) keeps the arrows, Home and End (the `ListBox` stops a first press,
  `keep_navigation_keys` in `list.rs` the repeats and releases, only while the list itself
  is `InputFocus`), a focused tab of a `TabBar` keeps left, right, Home and End (Bevy's tab
  list stops a first press, `keep_navigation_keys` in `tab_bar.rs` the repeats and
  releases) and lets up and down through, a focused `VirtualList` keeps the arrows, Page Up,
  Page Down, Home, End, Enter and Space (pressed, repeated or released; `navigate` in `virtual_list.rs`,
  only while the list itself is `InputFocus`, so the keys of a focused descendant, a button
  in a creator's row, go on past it), a focused `Slider` keeps left, right, Home and End
  (`keep_navigation_keys` in `slider.rs`, also at a limit) and lets up and down through, and a
  `NumericStepper`
  keeps the arrows that reach it from its buttons, also at a limit. The focus ring follows
  `InputFocus` whatever moved it, while `InputFocusVisible` is true. A `SegmentedControl`, `FilterChips` and
  `Gallery` and `LightDarkToggle` keep the arrows, Home and End while one of their own items has the focus, a `Toolbar`
  left, right, Home and End (up and down go through), all through `navigate_roving_group` (see
  "Composite widgets"). Every other widget lets
  the arrows through. A focusable child of a list row (a button in a `CustomView`) is not
  the list: its arrows, Home and End all go on past the list. Bevy's `ListBox` would take a
  first press of one as its own, because the `focused_entity` it checks is the hop the event
  has reached, not the focus, so `pass_first_presses_from_descendants` in `list.rs` stops
  that press on the list's child it comes through and triggers it again from the list's
  parent (the window, for a list without one); the active row stays. `tests/scenarios.rs`
  turns it on for two buttons, a list and a stepper (`directional_navigation_screen`), for
  buttons in list rows (`button_in_a_list_row_app`), and for a tab bar
  (`a_held_arrow_key_does_not_move_the_focus_out_of_a_tab_bar`).
- **Hidden widgets are hidden from the accessibility tree.** `sync_accessibility_hidden`
  in `focus.rs` runs next to `sync_tab_index` and marks the `AccessibilityNode` of
  anything hidden by the tab stop rule (`is_hidden`) as hidden, clearing it on show.
- **A combo's popup takes the keyboard focus while it is open.** Opening a `Combo` or a
  `ColorPaletteCombo` (the hook of `ComboState`, the observer of `ColorPaletteComboState`) sets
  `InputFocus` to the popup's list (the palette, for the palette combo) with
  `FocusCause::Navigated`, so the `ListBox` arrows and Enter / Space work on it; closing
  it, however it was closed, moves the focus back to the button if it was still on the
  list. The observers sit on that list, not on the combo, because a palette combo's popup
  is a child of its root: `FocusedInput<KeyboardInput>` closes the combo on Escape, and
  on Enter / Space for a selection of one item at most (the `ListBox` picks the active
  row, or leaves a selected one, and the popup is done; a selection of many stays open),
  and `FocusLost` closes it when the focus leaves the list (Tab away, a click on
  something else). A click on the button or in the popup also takes the focus off the
  list (nothing in them is focusable), but that is not leaving: the pointer being on
  either (their `Hovered`) makes the observer ignore it (and put the focus back on the
  list for a click in the popup). On a close by losing the focus the focus goes back to the
  button only if nothing took it. `ComboState` / `ColorPaletteComboState` stay the one source of truth. The
  `combo_keyboard_*` and `color_palette_combo_keyboard_*` files cover it.
- **Combo buttons open on `Activate` and say whether the popup is open.** The
  `ComboButton` and the `ColorPushButton` of a `ColorPaletteCombo` are `Button`s;
  the combo toggles its `ComboState` (the palette combo its `ColorPaletteComboState`) on their
  `Activate`, so on release and on Enter / Space for a focused button, and a
  press alone opens nothing. Hover and press colours stay `MouseEvent`, observed
  on the combo, which gets the pointer because the button has `should_block_lower:
  false`. The outside-click systems read `ButtonInput<MouseButton>` and the
  `Hovered` of the button and the popup, not `Activate`, and a click on the
  button is inside the combo, so they never close what the same click opens. A
  release on a palette combo's popup is inside as well, whatever the selection:
  its scroll bar, or a drag of its thumb, leaves it open. The popup closes on a
  pick by itself, not through the outside click: a global observer of
  `MouseEvent`s acts on a `MouseKind::Click` (a press and a release on the same
  swatch) on a swatch that belongs to the popup's `ColorPalette` and, for a
  selection of one item at most (`One`, `ZeroOrOne`), inserts `ColorPaletteComboState::Closed`,
  whether or not the click changed the selection (the selected colour of a `One`
  combo, a deselecting click of a `ZeroOrOne` one). A selection of many keeps
  the popup open, on a click and on Enter / Space (a palette combo over many
  is supported: `color_palette_combo_many_stays_open.ron`). A `Combo` has not been moved over (#96 left it): a release on
  its list still closes it, unless the selection may hold several items. The
  state is also `expanded` on the button's `AccessibilityNode` (`set_expanded`), set
  where `ComboState` / `ColorPaletteComboState` is inserted (the hook, the observer); the button is a
  `Role::ComboBox` whose value is the selection and which `controls` the popup's list
  (palette), set by each combo's `sync_accessibility` in `AccessibilitySync`. Scenarios
  check `ComboState` / `ColorPaletteComboState`, and `App(Accessible(..))` reads the node's role, value
  and `Expanded` / `Collapsed`; `controls` is checked headless. The popup is still the
  home-made `Float`, not `bevy_ui_widgets`' `MenuButton` / popover: that would
  change positioning, z-index and outside-click behaviour, so it is a ticket of
  its own if wanted.
- **Popups are placed against the UI target of their widget, not the primary window (#669).**
  `float.rs`'s crate-private `UiViewport::size_of(node)` returns the logical size of the render
  target the node is drawn to: Bevy's `ComputedUiRenderTargetInfo` (the camera's viewport in
  physical pixels over its scale factor, `UiScale` included), so a camera with a restricted
  viewport, a second window or an image target clamps against its own space. UI coordinates are
  viewport-local, so no offset is added. Unusable data (no camera, an empty viewport, a scale that
  is not positive and finite) falls back to the primary window's size (`target_size`), and with
  neither the popup is left where it is. `Combo`, `FilterCombo`, `ColorPaletteCombo`, `Tooltip`
  and `VirtualList`'s fallback height (no scroll area around it) all use it
  (`a_combo_list_stays_inside_the_ui_target_of_its_button_and_falls_back_to_the_window`,
  `a_tooltip_popup_stays_inside_a_restricted_viewport_not_the_primary_window`; the headless app has
  no camera, so the tests set the target data by reflection).
- **A `Float` stays in its window (investigation #506, not built).** Putting a popup in a
  window of its own, so it can leave the application window, was looked at against Bevy 0.20
  and not implemented. What the Bevy source says: a UI tree is drawn by the camera of its
  root. `propagate_ui_target_cameras` (`bevy_ui` `update.rs`) reads `UiTargetCamera` (else the
  `IsDefaultUiCamera` camera, else the highest-order camera of the primary window) from root
  nodes only, and propagates the result, `ComputedUiTargetCamera` and the scale factor and
  size in `ComputedUiRenderTargetInfo`, to every descendant; `UiTargetCamera` on a non-root
  node "will have no effect". `FixedNode` only makes a node lay out against its camera's
  viewport rather than its parent, it keeps the parent's camera. So a `Float` that is a
  child of its widget (what every popup is today) is always in the widget's window, and a
  popup in another window needs its own `Window`, a `Camera2d` with
  `RenderTarget::Window(WindowRef::Entity(..))` and a root `Node` with that camera as
  `UiTargetCamera`, a node that has no parent. `UiRoot` already leaves such a root alone
  (a `UiTargetCamera` in the bundle means no `UiRoot`, as the tester's cursor overlay shows).
  What that breaks, and why it is a design of its own rather than a change to `Float`:
  - the popup is no longer a descendant of its widget, so `Hovered` (the outside-click
    systems of `Combo` and `ColorPaletteCombo` rely on "the popup is a descendant"),
    visibility and `Display::None` inheritance (a popup of a hidden `Stack` panel), the
    `Within(..)` reach of the scenarios, the closing of the popup with its widget and event
    bubbling from the popup to the combo would all need an explicit link and cleanup;
  - the popup window takes the OS focus from the application window, so `InputFocus`, the
    `FocusLost` that closes a combo and the keyboard hand-over of "A combo's popup takes the
    keyboard focus" change meaning; a tooltip must never take focus at all;
  - every placement system (`place_open_lists`, the palette combo, `Tooltip`,
    `VirtualList`'s window height, `float.rs`'s `keep_in_window_x` / `below_or_above`) reads
    the UI target's logical size (`UiViewport`, see above), which a popup window would have to replace;
    a popup window needs screen coordinates (`Window::position` plus the widget's offset,
    the scale factor of whichever monitor it is on) and a reflow of the window when the popup
    resizes;
  - Wayland does not let a client place a toplevel window, so the feature cannot be
    uniform across platforms, and web/mobile targets have one window only; the in-window
    `Float` must stay as the fallback;
  - the headless tester has no real windows, so the scenario files, which pass at scale
    factors 1 and 2, `[text 1.5x]` and under shuffled schedules, could not cover it without
    extra support in the tester.
  If wanted, a first step is an opt-in `PopupWindow` mode on `Tooltip` and `Combo`, with the
  in-window `Float` as default and fallback; it is tracked as a follow-up ticket, not here.
- **Selections are immutable components.** Change one by going through
  `SelectionParam::update_selection` / `update_selection_single`, whose
  closures return `Option<SelectionValue>` from methods such as
  `toggle_selected`, `push`, `pop`, `trim_to_length` and `set_selection`
  (`None` means the change is not allowed for that variant). The new value is
  inserted, which fires `OnInsert` observers.
- **Shared selections.** A widget holding `Selection::Ref(entity)` reads and
  writes the selection on `entity`. `BreadCrumbs`, `List` and `Combo` accept
  one; `Stack` takes the selection entity directly in `selection_ref`. **One hop
  is the contract:** a `Ref` must point to an entity holding a `Selection::Value`.
  A target that is missing, has no `Selection`, is the `Ref`'s own entity, or is
  another `Ref` (a chain) resolves to nothing (`SelectionParam::find` returns
  `None`, so `single` is an error and `get`, `get_ref`, `get_with_ref` return
  `None`) and `find` logs an `error!` each time; chains are not followed
  (`a_selection_ref_to_*` in `tests/headless.rs`).
- **Shared items.** `ItemsRef(entity)` lets several selections draw from one
  `ItemValues`.
- **A scroll area leaves room for a focus ring.** `ScrollContents` has 3 px of
  padding on every side (`FOCUS_RING_REACH`: `focus.rs`'s `FOCUS_RING_WIDTH` of 2 px
  plus `FOCUS_RING_OFFSET` of 1 px, which `draw_focus_ring`, `scroll_area.rs` and
  `virtual_list.rs` all use), because the area clips at its padding box and the ring of a row at
  the edge would be cut off (#293). `scroll_into_view` counts the same 3 px as
  part of the entity, so a row scrolled into view shows its ring. The maximum
  scroll of every area is 6 px more than the contents' overflow alone.
- **Scroll area contents keep their natural size.** `ScrollContents` uses
  `align_items: FlexStart` instead of stretching its children. Taffy 0.10
  (Bevy 0.19) reported no overflow for a grid whose items overflow it, so a
  `List` or palette stretched to the area's height left nothing to scroll;
  taffy 0.14 (Bevy 0.20) was tried (#265): with `Stretch` the content still
  scrolls, but `list_wheel_scrolls` and
  `scroll_area_keyboard_scrolls_item_into_view` (both scale factors) fail, the
  maximum scroll 2 px short (296, not 298), so the workaround stays. For the
  same reason, don't give scroll contents `height: 100%`. The
  `scroll_area_*.ron` scenarios guard this on taffy 0.10: put `Stretch` back
  and the wheel and thumb scenarios failed there.
- **A scroll area's scroll position is kept in range.** The wheel handler
  (`update_scroll_position` in `scroll_area.rs`) moves only an axis whose
  `Node::overflow` is `Scroll` and leaves `ScrollPosition` inside `[0, max]`
  per axis, `max` being how far the contents overflow in logical pixels (0 when
  they fit); a node that does not scroll along an axis is not moved along it,
  except that, after the Ctrl swap of the wheel's two deltas, a node that scrolls
  horizontally but not vertically takes an event with no horizontal delta
  (`dx == 0`) as a horizontal scroll by its vertical delta. **Nested areas:** the wheel
  goes to the entities in picking's hover map, and a `ScrollContents` blocks the
  pointer for what is below it, so over a nested area only that one scrolls: the
  area around it is not hovered and gets nothing, also when the inner one is at its
  limit (nothing is passed on). The outer area scrolls where the pointer is on it
  beside the inner one (`scroll_area_wheel_nested_horizontal_in_vertical.ron`, the
  `outer scroll area` of the `ScrollAreas` screen).
  A scenario reads `ScrollPosition.0.y`, the value that was set (logical
  pixels). `ComputedNode.scroll_position.y` is what the layout shows, floored,
  in physical pixels and one layout pass behind, so it doubles at a scale
  factor of 2 and scenario files do not read it. A thumb drag and a click on
  the track are the `bevy_ui_widgets` `Scrollbar`'s, which clamps to the same
  range; a click on the track beside the thumb pages by one visible length
  towards the click (`scroll_area_track_click_pages.ron`). The wheel stays
  ours rather than `bevy_ui_widgets`' `ScrollArea`:
  picking sends a `PointerScroll` to every hovered entity and each bubbles up to
  the area, and an `ItemView`, its `List` and the `ScrollContents` are all
  hovered (`should_block_lower: false`), so one turn would scroll two or three
  times; it also scrolls by `MouseScrollPixelsPerLine` (100 px a line, not our
  21) and has no Ctrl axis swap.

## Conventions

- **A scroll area scrolls a focused row into view.** `scroll_into_view` in
  `scroll_area.rs` observes `bevy_ui_widgets::ScrollIntoView`, which a `ListBox`
  triggers when the arrow keys, Home or End move its active descendant. It finds
  the `ScrollContents` above the entity and moves its `ScrollPosition` by the
  least that shows the entity, per axis only where the contents scroll and within
  `[0, max]`; an entity outside any scroll area is left alone, and so is one in a
  `FixedNode` (a combo's popup) inside one, since scrolling the area would not
  show it; a `FixedNode` that itself scrolls (a `Combo` or `ColorPaletteCombo` popup
  capped to the window height) has its own `ScrollPosition` moved instead, never the
  area's (`combo_taller_than_window_keyboard_scrolls_the_list.ron`). It reads the
  entity's laid-out position, so it is one layout pass behind like the wheel's
  `ComputedNode`. The wheel is untouched, and so is the `Scrollbar`, which
  follows the `ScrollPosition` it sets
  (`scroll_area_keyboard_scrolls_item_into_view.ron`).
- **`sync_accessibility` systems look at what changed.** Bevy resets a node's role and
  label in `UiSystems::PostLayout` through `DerefMut`, so `Changed<AccessibilityNode>` catches
  every reset before `AccessibilitySync` runs. Gallery, filter chips, toolbar, segmented
  control, tab bar, light-dark toggle, push button, icon button and link therefore query only
  `Or<(Changed<AccessibilityNode>, Changed<View>, ..)>` (a `ParamSet` holds the unfiltered
  query for the rest); the selection widgets add the owners `StructureChanges::affected` names,
  a `SegmentedControl` also `Added<Selected>`, removed `Selected` and changed `Item<String>`,
  buttons removed `ToggleState`, the light-dark toggle a changed `ThemePreference`. The `sync_*`
  helpers write only on a difference, so a system never marks its own nodes changed; an idle
  frame writes none (`filter_chips_leave_their_accessibility_nodes_alone_on_idle_frames_and_for_other_chips`).
  New bench workloads `segmented-controls-idle` and `galleries-idle`; numbers in `docs/benchmarks.md`.
- **Nothing that runs every frame walks the whole world.** An app may hand a widget ten
  thousand items, so a system that runs each frame touches what is on
  screen, not every item: `VirtualList` looks at its pooled rows only, and finds an item by
  index in `ItemValues`. Work over all items belongs to a change (`ItemsChanged`, a new
  `Selection`) or a key press, not to the frame. A `List` spawns a view per item, so use a
  `VirtualList` for long lists.
- **Deferred construction decides at run time.** A hook that builds parts on first insert
  (`PushButton`, `Link`, `IconButton`, `Panel`, `SettingRow`) never tests the world for its refs
  while it queues the work: two inserts applied in one flush (`World::insert_batch`) both run
  their hooks before either hook's commands, so both would build. The test for existing parts
  sits inside the queued command or system, so one set of parts and observers is built and the
  last insert's configuration wins (`two_inserts_of_a_panel_in_one_command_batch_*`, and the
  same for `SettingRow` and `IconButton`, in `tests/headless.rs`).
- **One `Plugin` per widget**, registered in `WidgetsPlugin::build` in
  `lib.rs`. Downstream apps add only `WidgetsPlugin`.
- **Widgets observe `MouseEvent`, not raw pointer events (`PointerPress` and
  friends).** The only observers of them in the library are the producer in
  `lib.rs`; do not add more. `MouseKind` says `Over`, `Pressed`, `Released`, `Click` and `Drag`,
  so a widget can tell a click (a press and a release on the same node) from a
  release that ends a press begun elsewhere.
- **Styling comes from `Res<Theme>`.** Each widget has a `theme_*` system
  taking `In<Option<Entity>>`: its hook runs it for the one new entity with
  `run_system_cached_with(theme_x, Some(entity))`, and its plugin runs it for
  all of them with `(|| None::<Entity>).pipe(theme_x).run_if(resource_changed::<Theme>)`.
  `itertools::Either` unifies the two iterators. See `card.rs` for the
  smallest example.
  A widget with an interaction state (hovered, pressed, selected) takes its
  background from one function of that state (`background_for` in
  `push_button.rs`, `item_background` in `list.rs`) that both its pointer
  handler and its `theme_*` system call, so a theme change under a resting
  pointer keeps the hover colour. The state is read from `Hovered` and `Pressed`,
  not tracked by hand; a `PushButton` with `NoDefaultHover` rests at
  `background_primary` for its owner to colour.
- **Corner radius is a `Node` field.** Since Bevy 0.18 `BorderRadius` is
  `Node::border_radius`, not a component; set it on the node. Since Bevy 0.20
  its corners are `CornerRadius`es and its constructors (`BorderRadius::all`,
  `top`, ..) are not `const`, so a function that builds one cannot be either.
- **No hard-coded colours outside `theme.rs` and `tailwind.rs`.** Add a slot
  to `ThemeColors` (both default themes) rather than inlining a colour.
- **Setup lives in component hooks and `#[require(…)]`.** `on_add` for
  one-time setup (`PushButton`, `Combo`, `Link`, `ColorPushButton`),
  `on_insert` where re-inserting should re-run it (`Card`, `Label`, `List`,
  `ScrollArea`, `Float`, `NumericStepper`, `ComboState`). Default `Node`
  styling and a `Name` go in `#[require(…)]`.
- **Public widget components, their state and `Theme` derive `Reflect`**, with
  `#[reflect(Component)]` (`#[reflect(Resource)]` for `Theme`), so the type
  registry knows them by name (Bevy's `reflect_auto_register`). A field that
  cannot be reflected gets `#[reflect(ignore)]` with a comment saying why.
  `tests/reflection.rs` fails when a derive is dropped; add a new widget there.
  The short type paths `Label`, `ScrollArea` and `Slider` are also Bevy's own, so a
  lookup by short path is ambiguous for those three; use the full path
  (`bevy_pluim::slider::Slider`).
- **Insert-time configuration is immutable; refs are read-only.** `PushButton`, `IconButton` and
  `Link` are `#[component(immutable)]`: a live button is changed by building or cloning a value,
  editing it outside the world and inserting it again, which updates the children in place (no
  `Changed<..>` synchronisation). `PushButtonRefs`, `IconButtonRefs` and `ColorSquareRef` are
  immutable, queryable records with private fields and no public constructor; read them through
  `PushButtonRefs::label()` / `icon()`, `IconButtonRefs::icon()` and `ColorSquareRef::entity()`.
  `compile_fail` doctests on the types guard both.
- **Input is owned by the app, derived state by the widget.** `Item<T>::data` is edited in place; `ItemOf` is reparented through Bevy's relationship API; `ItemsRef` and `Stack` are immutable, so changing them means inserting a new value, which refreshes their consumers. Nothing hands out `&mut ItemValues`.
- **Configuration is immutable; live state is not.** `Slider`, `NumericStepper` and
  `TextField` are `#[component(immutable)]`: their fields are read by the insert hook, and
  changing a setting (range, format, names, icons, widths, placeholder, initial text) means
  inserting a new component, which rebuilds the parts and syncs the accessibility node (the
  stepper keeps its `ValueRef`, a `TextField` sets its text without a `TextFieldChange`). The
  live data stays mutable where it already is: the stepper's `Value` / `UIntValue` (through
  its validated operations, observed by `on_change`), Bevy's `SliderValue`, and the
  `EditableText` of `TextFieldRefs::input`. A system never takes the three as `&mut` (a
  `compile_fail` doctest on each guards it) and none syncs them with a per-frame `Changed`
  query. A new configuration component follows the same split.
- **Display widgets are configured by replacement.** `Label`, `Kbd`, `Pill`, `Toast` and `Badge` are immutable components with public fields: construct or clone a value, edit it outside the world and insert it. A mutable query (`Query<&mut Pill>`) does not compile, so migrate such code to `commands.entity(e).insert(pill.clone().with_tone(..))` (read with `&Pill`). Live data such as `Item<T>::data` and `NumericStepper`'s `Value` is not configuration and stays outside this rule.
- **State is an immutable component**; a transition inserts a new value
  (`ComboState`, `Selection`, the palette combo's `ColorPaletteComboState`).
- **A view's relationships are private and read-only.** `Segment`, `TabBarTab`, `ChipView`, `GalleryCardView` and `ToolbarButton` (and `SegmentIndicator`, which has only the owner) are immutable components with private `item` and `owner` fields and public `item()` / `owner()` getters, built only by their widget (a `pub(crate)` `new`), so typed code can neither construct one nor change a live one (`compile_fail` doc tests on each). They stay `Reflect`, so scenarios and `WidgetsCheck` read the fields by the reflected paths `item` and `owner` (`the_item_and_owner_of_a_selection_view_are_read_by_getters_and_by_reflection`). The application-authored item data (`Chip`, `GalleryCard`, `Tool`, `Item<String>`) stays public and the app supplies meaningful accessible labels; the library has no non-empty-label type or validation. The child-refs records (`ChipRefs`, `GalleryCardRefs`, `ToolbarButtonRefs`) are a different thing and unchanged.
- **Composite widgets keep their children in a refs component**
  (`PushButtonRefs`, `ComboRefs`, `ScrollAreaRefs`, `ColorSquareRef`,
  `ValueRef`) instead of searching `Children`.
- **Selections can be shared via `Selection::Ref`**; a widget that displays a
  selection must react when either its own entity or the referenced one
  changes (see `selection_change` in `breadcrumbs.rs` or `list.rs`).
- **Lint groups are `deny`.** `clippy::all`, `pedantic` and `nursery` (and
  `missing_const_for_fn`) fail the build, so `just lint` is the bar, not a
  suggestion. The only allows are the Bevy-shaped ones documented in
  `Cargo.toml`: `too_many_arguments`, `needless_pass_by_value` and
  `type_complexity` (systems take many params, `In<_>` / `Res<_>` / `Trigger<_>`
  by value, and nested `Query` types), plus `too_many_lines` and the numeric
  cast lints. `unused_crate_dependencies` stays off because it misfires per
  example / test target; `cargo machete` (in `just ci`) checks dependencies
  instead. Public functions returning `Result` need a `# Errors` section.
- **No panics in library code.** `unwrap_used`, `expect_used`, `panic`,
  `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr` and
  `indexing_slicing` are denied in `Cargo.toml`, and `clippy.toml` bans the
  panicking Bevy accessors (`Query::single(_mut)`, `World::resource(_mut)`).
  A system or observer that cannot find what it needs uses
  `let Ok(x) = query.get(e) else { return; };` (`continue` in a loop) and does
  nothing; a misconfiguration worth reporting is logged with `error!`. The
  `SelectionValue` methods return `None` for a change the variant does not
  allow. Integration tests carry a file-level `#![expect(clippy::unwrap_used,
  …, reason = "tests may abort loudly")]`.
- **Everything public is documented.** `missing_docs` and rustdoc's
  `missing_crate_level_docs` are `deny`: every public item has a `///`
  comment, every module (and each example) a `//!` header, and
  `RUSTDOCFLAGS="-D warnings"` in `just lint` keeps intra-doc links honest. Say
  what a thing is or does, not how it is implemented.
- **`#[expect(…, reason = "…")]`, never `allow`.** `allow_attributes` and
  `allow_attributes_without_reason` are denied in `Cargo.toml`.
- `#[derive(Debug)]` on every public type (`missing_debug_implementations`
  warns), and `const fn` / `#[must_use]` where clippy asks.

### Adding a widget

1. `src/my_widget.rs` with `MyWidgetPlugin` and the `MyWidget` component.
2. `#[require(…)]` for its default `Node` and `Name`; a hook for setup.
3. A `theme_my_widgets` system in the pattern above.
4. `MouseEvent` observers for hover and press.
5. `pub mod my_widget;` and `add_plugins(MyWidgetPlugin)` in `lib.rs`.
6. An example in `examples/`, and a row in the tables above.
7. A `WidgetsSetup` screen for the scenarios: a variant in
   `tests/widgets_setup/mod.rs`, added to the smoke test in
   `tests/scenarios.rs`, and its notes under [Scenario files](#scenario-files).

## Running things

`rust-toolchain.toml` pins the compiler, and the `justfile` is the one place
commands are defined; every recipe is a plain `cargo` call you can read out
of it. You need `just` and, for `unused-deps`, `cargo-machete`.
`sccache` is optional: when it is on the `PATH` the `justfile` exports it as
`RUSTC_WRAPPER`, so a fresh worktree gets its dependencies from the cache
instead of compiling Bevy again. A `RUSTC_WRAPPER` you set yourself wins, and
`RUSTC_WRAPPER= just test` builds without it. Plain `cargo` calls do not go
through the `justfile` and so do not use it.

| Task | `just` | Plain cargo |
| --- | --- | --- |
| Format | `just fmt` | `cargo fmt --all` |
| Lint | `just lint` | `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` |
| Test | `just test` | `cargo test --all-targets` then `cargo test --doc` |
| Run an example | `just example NAME` | `cargo run --example NAME` |
| Play every scenario in a real window | `just gallery-test` | `cargo run --example gallery -- --run-all` |
| Benchmark CPU and memory | `just bench -- --workload tooltips --size 5000` | `cargo run --release --example bench -- ...`, see `docs/benchmarks.md` |
| Check rendered scenarios | `just record` (`just record --save` to save recordings) | Needs a GPU, see [Recording the scenarios](#recording-the-scenarios) |
| All of CI | `just ci` | lint, test, `cargo machete --with-metadata` |

**The first build is slow.** Bevy takes many minutes to compile from cold;
dependencies are built with `opt-level = 3` even in dev (see `Cargo.toml`) so
the examples are usable, which makes that first build slower still. Later
builds only recompile this crate. Prefer `just test` over running examples in
a loop, and note that examples need a display.

**Dependencies have no debuginfo** (`debug = false` in
`[profile.dev.package."*"]`), so a test or example binary is far smaller and a
slot's `target/debug` fits a cloud container's disk; this crate keeps its
own. The first build after that setting landed rebuilds every dependency, once
per `target/`. To debug into a dependency, turn it back on for one invocation:
`cargo --config 'profile.dev.package."*".debug=true' test ...` (that rebuilds
the dependencies again, and again when you drop the flag).

`just bevy-lint` exists but is deliberately **not** part of `just ci`: it
needs a pinned nightly and trails Bevy releases.

### Bevy features

The library depends on `bevy` with `default-features = false` and the features
`ui_api`, `scene`, `picking`, `bevy_ui_widgets` and `bevy_render` (the icons it
draws use `bevy::render::render_resource` image types), so it forces neither
audio, 3D / PBR, glTF nor a windowing backend on a downstream app (#696). The
application chooses the rest: the platform and renderer features (`bevy_winit`,
`bevy_ui_render`, `default_font`, ..), most simply Bevy's `ui` profile or
`default`. The examples and tests get Bevy's default features, `bevy_ui_debug`
and `debug` from `[dev-dependencies]` only. Adding a `use bevy::..` of a module
behind another feature means adding that feature to `[dependencies]`; check
with `cargo check --lib --locked` (dev dependencies are not unified into it) and
with a downstream fixture that sets `default-features = false`:
`cargo tree -e normal` must show no `bevy_audio`, `bevy_pbr`, `bevy_gltf` or
`bevy_winit` (541 packages with Bevy's defaults before, 427 after).

### The scenario tester dependency

`bevy_stagecraft` (`github.com/rinde/bevy_stagecraft`) is a public
`[dev-dependencies]` git dependency pinned to a revision in `Cargo.toml`; it
drives scripted pointer and keyboard scenarios against the widgets
(`tests/scenarios.rs`, see [Writing a scenario](#writing-a-scenario)).

- **Bumping it** means changing `rev` in `Cargo.toml` and `Cargo.lock`
  together (`cargo update -p bevy_stagecraft`); `--locked` in CI fails if
  they disagree. The repository is public, so no credentials are needed.

### Claude Code on the web

`.claude/hooks/session-start.sh`, registered in `.claude/settings.json` under
`SessionStart`, prepares a fresh cloud container: it installs the pinned
toolchain with `clippy` and `rustfmt`, `just` and
`cargo-machete` at pinned versions, the Bevy system headers
`.github/workflows/ci.yml` also installs,
then fetches the dependencies (including the public scenario tester above) and
warms the build cache with `cargo build --all-targets` so the first real
build isn't a cold Bevy compile. It is a no-op unless `CLAUDE_CODE_REMOTE=true`
or run with `--force`, and is idempotent, with progress on stderr.
Repository hooks run in single-repository cloud sessions; Claude's project
threads and sessions with multiple repositories do not load them (see
[cloud hook limitations](https://code.claude.com/docs/en/cloud-environments#limitations-in-cloud-sessions)).
`.claude/settings.json` also sets the auto-compact window to 200k tokens for
Sonnet 5.5 and Opus 5.5 (`modelSettings.<model>.autoCompactWindow`).

## Testing

- unit tests go in a `#[cfg(test)] mod tests` next to the code; `select.rs`'s
  `SelectionValue` tests are the reference for pure-logic coverage;
- integration tests go in `tests/`, driving a headless `App`;
- name tests as sentences: `a_stack_shows_only_the_selected_child`.

### Writing a headless integration test

`tests/headless.rs` builds an `App` with `MinimalPlugins` plus the plugins
`WidgetsPlugin`'s systems need to run without panicking on a missing
resource, none of which need a window or GPU:

- `WindowPlugin { primary_window: None, ..default() }` — registers
  `WindowEvent` and friends, which `bevy_picking` reads; no window is
  actually opened.
- `InputPlugin` — provides `ButtonInput<MouseButton>`, needed by
  `combo.rs`'s outside-click observer.
- `DefaultPickingPlugins` — provides `HoverMap`, needed by
  `scroll_area.rs`'s scroll handling.
- `VisibilityPlugin` — propagates `InheritedVisibility`, which `focus.rs`'s
  `sync_tab_index` reads; without it every tab stop counts as hidden.
- `ClearColor` (via `init_resource`) — normally inserted by the render
  plugins; `theme.rs`'s `update_theme` writes to it whenever `Theme`
  changes.
- `UiScale` (via `init_resource`) — normally inserted by `UiPlugin`; the
  `bevy_ui_widgets` `Scrollbar` of a scroll area reads it when it is pressed,
  so without it any press panics.

These are all needed unconditionally because `WidgetsPlugin` registers every
widget's systems regardless of which widgets a given test actually spawns.

Spawn entities on `app.world_mut()` directly (component hooks and `OnAdd`/
`OnInsert` observers fire synchronously), then call `app.update()` at least
twice to flush the `Commands` those observers queue and let `Update`-schedule
systems (like `breadcrumbs.rs`'s `selection_change`) react. Register
`StackItem` creator systems with `world.register_system`, matching the
`In<StackChildSpawner>` pattern in `examples/tabs.rs`.

If a widget can't be exercised headlessly because it needs a render or text
plugin, note it in the PR rather than pulling in a window.

### Writing a scenario

A scenario drives the widgets with scripted pointer and keyboard input through
`bevy_stagecraft` (see its README for the `Step`, `Target` and `Check`
vocabulary) and asserts on what they did. Prefer it to hand-written pointer
event triggers (`PointerPress` and the like) whenever the claim is about what a
person would see after using the UI.

- **Where.** Scenarios go in `tests/scenarios.rs`, a test target of its own, so
  `tests/headless.rs` stays as it is. Shared code lives in `tests/common/mod.rs`
  and is pulled in with `mod common;`. Its items are `pub` under a file-level
  `#![expect(unreachable_pub, ..)]`, because clippy's `redundant_pub_crate`
  rejects `pub(crate)` there. Keep it small: a helper that is not used by every
  target that pulls the module in fails `dead_code` in the ones that do not.
- **The header.** If a scenario file unwraps (a `Result` from `from_ron`, say),
  give it the same file-level `#![expect(clippy::unwrap_used,
  clippy::indexing_slicing, reason = "tests may abort loudly")]` as the other
  integration tests; `expect`, never `allow`. Leave it off a file that needs
  none, because an unfulfilled `expect` is itself an error under `-D warnings`.
- **Build the app with `common::widgets_app()`**: the tester's
  `HeadlessPlugins` (a virtual window, fixed 1/60 s frames, picking, UI, no
  GPU) plus `WidgetsPlugin`, with `ClearColor` initialised (`HeadlessPlugins`
  does not insert it and `theme.rs`'s `update_theme` writes it), finished and
  cleaned up. Do not use `Scenario::headless_app()`: its app is already
  finished and refuses `add_plugins`. Play with
  `scenario.run_in(&mut widgets_app())`, which returns a `ScenarioReport`;
  assert `report.passed()` with `report.failures()` as the message.
- **Setup.** The scenario's `.setup(|world| ..)` closure builds the UI and runs
  before the first update. It must spawn a `Camera2d` and exactly one root
  `Node` (`common::spawn_camera_and_root` does both and returns the root), or
  `UiRoot`'s `PostStartup` check finds no root.
- **Targets.** `PushButton` inserts Bevy's `Button`, so
  `Target::role(WidgetRole::Button, "Add")` finds it by its label.
- **Checks are data.** There are no check closures. Read widget state with
  `Check::component::<C>(target, ".field")` and `Check::resource::<R>(".field")`
  (the type must derive `Reflect`, with `#[reflect(Component)]` /
  `#[reflect(Resource)]`), or write the app's own `#[reflect(AppCheck)]` enum (the tests have one,
  `WidgetsCheck`; see [Scenario files](#scenario-files)).
- **Frames.** Time is fixed at 1/60 s a frame. A pointer step (`move_to`,
  `press`, `release`) takes one frame and a `Step::check` none, so a check right
  after the step sees its effect, `MouseEvent` observers included; add
  `Step::wait(n)` only for something that needs more frames.
- **Start from a screen.** `.app_setup(WidgetsSetup::Buttons)` (or `setup:
  App(Buttons)` in a file) instead of a `.setup(..)` closure, when a screen of
  `tests/widgets_setup/mod.rs` has the widgets the scenario needs. `mod
  widgets_setup;` next to `mod common;` pulls it in; it is its own module,
  not part of `common`, because `common` items an including target does not
  use fail `dead_code`.
- **Prove the assertion is real.** When you write a scenario, break the
  expectation once (a different theme slot, say) and see it fail before
  trusting it.

### Scenario files

A scenario can be data: `tests/ron/NAME.ron` is compiled into individual
libtest cases by `build.rs`. Each file gets default, scale 2 and text 1.5x
cases named with its file and variant, and standard Cargo filters select a
single case (for example, `cargo test --test ron_scenarios scenario_push_button_scale_2`).
`just test` runs them with everything else. Each case runs in a fresh
`common::widgets_app()` after `Registry::install`, so a file can check any
`Reflect` type of the app by short type path. `skip` and `fixme` scenarios are
ignored by libtest, variant restrictions leave their out-of-scope cases
unplayed, and `should_fail` passes only when the scenario fails as expected.

- **The format** is the tester's RON script (`format_version: 3`); see its
  README and `tests/ron/*.ron` there. The same file also plays in the tester's
  gallery and recorder. Write or record it there, then drop it in `tests/ron/`.
  A colour in a check is `Srgba((red:..,green:..,blue:..,alpha:..))`; to get
  the numbers of a theme slot, build the scenario in Rust and print
  `to_ron()`.
- **The setup** is `setup: App(Buttons)`: a variant of `WidgetsSetup`, the one
  `#[reflect(AppSetup)]` enum of the tests (`Buttons`, `Combos`, `Lists`,
  `ScrollAreas`, `BreadCrumbs`, `Tabs`, `ColorPalette`, `NumericStepper`, `Slider`, `SegmentedControls`, `LightDarkToggle`, `FilterChips`,
  `Panel`, `SettingRows`, `Badge`, `Gallery`, `Toolbar`, `Tooltip`, `Themed`, `IconButtons`, `TextFields`, `VirtualList`, `SeparatedVirtualList`, `ComboRows`, `FilterComboRows`, `FilterCombos`, `ThemeEditor`), which spawns a `Camera2d`, the one root `Node` and a screen, each
  with a "Change theme" button.
  - **A screen is a fixture of its own.** It starts from the example of the
    widget it is named after, then adds names, widgets and sizes for testing
    (a name to target, a layout that fits the 1280 by 720 window) and does not
    follow later changes to the example. A change to an example needs no
    matching change to its screen, and vice versa. They are not shared because
    the example spawns through `Commands` in a startup system while a setup is
    handed the `World`, and shared code would put test-only names and widgets
    into every example, or take them out of the screens.
  - **`Buttons` and `Themed` have no example of their own.** `Themed` puts one
    of each widget that restyles with the theme on one screen.
  - **A variant** is found by Bevy's automatic registration, so adding one
    needs no registration call; add it to the smoke test in
    `tests/scenarios.rs` as well.
  - **`Buttons`** also keeps the `Clicks` resource, the `Activate`s of "Add",
    and has a "Disabled" button (`InteractionDisabled`), which Tab skips. "Toggle" is a `ToggleState` push button, off at first.
  - **`AppSetup::capture`** (recording a session from the app as it is) is not
    implemented: only the tester's gallery and recorder call it, which these
    tests do not run, so it would go untested.
- **The check** is `App(Selected(of: .., items: [..]))`: `WidgetsCheck`, the
  one `#[reflect(AppCheck)]` enum of the tests, in `tests/widgets_check/mod.rs`
  (Rust: `Check::app(WidgetsCheck::Selected { .. })`; `mod widgets_check;`
  pulls it in, like `widgets_setup`, and it stays out of `common` for the
  same `dead_code` reason). A `Selection` holds entities, which mean nothing
  in a script, so `Selected` names the selected items by what they show, in
  the order they were selected: an `Item<String>` by its text, an `Item<Tool>` or
  `Item<GalleryCard>` by its label, an `Item<Color>` by its Tailwind colour (`blue-500`) when it is one, else its hex (`color_name`). `of` is
  any target that resolves to a widget: the selection is the first on it or
  above it, and a `Selection::Ref` is followed, so the `List`, `Combo`,
  `BreadCrumbs` or `ColorPalette` that shares a selection reports it. It goes
  through `resolve`, which needs the widget laid out (a scenario waits a
  frame or two after its setup) and a `Node`; a bare selection entity
  without one cannot be named, but every widget referring to it can. A
  failure says `<target> has ["b"] selected, not ["a"]`. `facts` proposes a
  `Selected` for each list, combo, trail and palette that one target leads
  back to (the type's `Component(..)` when it is alone, else `Nth(i, ..)`),
  so a recording suggests the selections that changed. Add a variant here
  when a claim about widgets cannot be said with `Component(..)` or
  `Resource(..)`.
- **The accessibility check** is `App(Accessible(of: .., role: Some("Button"),
  label: Some("Add"), value: None, states: [Disabled]))`
  (`WidgetsCheck::Accessible`): `AccessibilityNode` wraps an `accesskit::Node`,
  which reflection cannot read, so `Component(..)` cannot say what a widget
  exposes. `of` resolves like `Selected`'s, but the node is read on that entity,
  not above it, and a failure says `<target> has no AccessibilityNode` when it
  has none. `role` is the `accesskit::Role` by its `Debug` name (`"Button"`,
  `"ListItem"`); `label` and `value` are compared exactly; `None` is not
  checked. `states` are `AccessibleState`s that must all hold, unlisted ones
  are not checked: `Selected` / `NotSelected`, `Toggled` / `NotToggled` and
  `Expanded` / `Collapsed` need the state set (a node that cannot be expanded
  is neither), `Disabled` / `Enabled` and `Hidden` / `Shown` read a flag. A
  failure names every difference: `<target> has role Button, not ComboBox, is
  enabled, not Disabled`. **A `PushButton` has no label yet**: Bevy labels its
  `bevy::ui::Button`s with their text, not the `bevy::ui_widgets::Button`s ours
  are, so `accessible_names_today.ron` checks only the role and the disabled
  state. The tester's `Role(role: Button, label: ..)` still finds the button,
  because it reads the text.
- **Targets on a screen with several of one widget.** The `NumericStepper`
  screen puts a `Name` on every stepper (`default stepper`, `u16 stepper`,
  `u32 stepper`, `f32 stepper`, `stepper by two`, and `shared stepper` with
  the `first mirror stepper` and `second mirror stepper` that point to it
  through `ValueRef`), so a file reaches one stepper's button with
  `Within(Name("u16 stepper"), Name("Increase"))` (the icon buttons have no text for a
  `Role` label to match, so they are named after their accessible name) and reads its
  value with `TextOf(Name("u16 stepper"), Exact("11"))`. Move the pointer to the button before every click: a wider
  value moves the buttons. A button at its limit is `InteractionDisabled`, so
  the tester's click on it does nothing (`numeric_stepper_bounds.ron`).
- **The `Panel`, `SettingRows`, `Badge` and `Tooltip` screens.** `Panel`
  has `closable panel` (title "Trains", subtitle "3 open", three lines; its
  `PanelClosed` observer despawns it) and `tall panel` (100 px maximum body,
  twelve lines, so the wheel scrolls it: read `ScrollPosition.0.y` of its
  `ScrollContents`); the close button is `Component("IconButton")` within the
  panel, as an icon button has no text for a `Role` label to match. Panel
  and `SettingRow` do not share a screen: `SettingRows` is the `setting rows`
  column of `appearance row` (a `SegmentedControl`), `text size row` (a
  stepper) and `last row` (a button), whose dividers are `Nth(i, Within(Name
  ("setting rows"), Name("Divider")))` and read `Node.display`. `Badge` has
  `counted button` (3), `warning button` (12), `plain button` and "Add badge";
  the pill is a `Name("Badge")` child with the count as its text. `Tooltip` has
  `tooltip anchor` (tip "Anchor tip") and `help button` (an icon button, tip
  "Help"); the popup's text exists only while it shows, so `Exists(Text(..))`
  says whether it does, after `Wait(40)` frames (0.5 s at 60 per second).
- **The `Gallery` and `Toolbar` screens.** `gallery` (Steam, Diesel and Passenger cards in
  two groups, a `One` selection) and `filled toolbar` (Select, Build, Bulldoze, a `One`
  selection), each with "Change theme". Their `*_text_follows_rem_size.ron` files check that a
  label is `Rem(0.6)` text and fits inside its fixed px card (104 x 92) or button (66 x 60) at
  `[text 1.5x]`; both pass, so those sizes stay px. The `Toolbar` screen also has three toolbars
  absolutely placed at x 0 (y 150, 300, 450) and spawned last, so nothing above moves and the tab
  order of the first two stays: `optional toolbar` (Pan, Zoom, `ZeroOrOne`, nothing open),
  `icon only toolbar` (`ToolbarLabels::Hidden`, Rail with key hint "1" and Signal with "2", `One`)
  and `underlined toolbar` (Trains, Stops, `One`, icon only, so each button has a `Tooltip`).
  `toolbar_click_picks_tool.ron` and `gallery_click_picks_card.ron` read the active look (the
  button's `BackgroundColor.0` and text colour, the card's `BorderColor.top`) against a `Capture`
  of the resting one; `toolbar_keyboard_picks_tool.ron` and `gallery_keyboard_picks_card.ron` Tab
  in, pick with Enter / Space (toolbar) or the arrows (gallery) and Tab out;
  `toolbar_labels_hidden_and_key_hint.ron` checks the button sizes (66 x 60 shown, 56 x 52
  hidden), that a hint takes the ink of the active state, and the underlined tooltip. A tool with
  a key hint has the hint in its button's text, so reach those buttons with `Nth(i, Within(Name(..),
  Name("ToolbarButton")))`: a `Role` label would read "1 Rail".
- **The `Slider` screen.** A column of `continuous slider` (0 to 1, 320 px wide, name
  "Continuous"), `stepped slider` (0 to 10 in steps of 1, 200 px, name "Stepped"; the
  `SliderChanges` resource keeps the `last` value of its `ValueChange<f32>`s and counts the
  `finals`), `formatted slider` (0 to 100 in steps of 5 from 25, shown as `25 %`, name
  "Speed") and `disabled slider` (`InteractionDisabled`, 3, shown as `3`, no name), then
  "Change theme". The value of a slider is `SliderValue.0`, its parts `Within(Name(..),
  Name("SliderThumb"))` (and `SliderTrack`, `SliderLabel`); a press on the track is a
  `Click(Left)` after `MoveTo(Name(..))` and a drag `Drag(from: <thumb>, to: Screen(..))`.
  The label and the track let the pointer through, so a file cannot `MoveTo` the label.
  A press on the thumb alone does not make the slider `Pressed`, only one on the track or a
  drag does, so `slider_thumb_colours.ron` presses the track for the `accent` thumb.
  `zero range slider` (320 px, 3 to 3 in steps of 1, spawned last) has the `ZeroRangeChanges`
  resource count its `ValueChange<f32>`s (`count`) and those not at 3 (`off_bound`);
  `slider_zero_range_pointer.ron` presses its track and drags the thumb past both ends and
  checks the value stays 3, `off_bound` is 0, the thumb stays at a finite position and the
  accessibility node is a `Slider`.
- **The `TextFields` screen.** `name field` ("Alpha", named "Name"), `seed field`
  (empty, placeholder "Seed", named "Seed"), an "Other" button and "Change theme". The
  `FieldEvents` resource counts the fields' `TextFieldChange`s and `TextFieldCommit`s and keeps
  the last value of each (`changes`, `changed`, `commits`, `committed`). Type into a field
  with `Type(text: "42", into: Within(Name("seed field"), Component("TextInput")))` (the
  tester clicks it first), read its text with `TextOf(..)` of the same target and whether
  the placeholder shows with `Visible(Within(Name("seed field"), Text("Seed")))`. The
  `text_field_*.ron` files cover typing and `Enter`, `Escape` restoring, a click elsewhere,
  a click on the border (at `Screen((20.5, 32.0))`, the left border of `name field`),
  Tab and the screen reader node, and the text following `RemSize`.
- **The `Toast` screen.** "Save" and "Save again" raise a toast "Saved", "Delete" one "Deleted 3 pieces", each shown for 1 s (the file waits for time with `Within`, not a number of frames, and `the_toast_scenario_passes_when_frames_are_shorter_than_a_sixtieth_of_a_second` plays it at 1/240 s), and "Change theme". `Count(with: ["Toast"], ..)` counts the toasts showing (`toast_shows_stacks_and_goes.ron`).
- **The `FilterCombos` screen.** A row of `filter combo` (named "City") and `ordinary combo`, both sharing one `ZeroOrOne` selection (Amsterdam, Delft, Den Haag, Utrecht, Rotterdam) with the `city list` below them, which is never filtered, and "Change theme"; then, absolutely placed and spawned last so the tab order stays: `multi filter combo` ("Cities", `ZeroOrMore`, own items, x 300 y 300), `one filter combo` (`One`, x 550), `long filter combo` (80 options at x 800 y 600, so the popup opens upward, capped and scrolling) and `empty filter combo` (`One`, no items, x 1050). Reach the button with `Within(Name("filter combo"), Component("ComboButton"))`, the search input with `Component("TextInput")`, the options with `Nth(i, Within(Name(..), Component("ItemView")))` and read `Node.display` (`Eq(None)` for a filtered-out row; `Role(role: ListItem, ..)` does not find a hidden row), the active option with `App(Active(of: <input>, is: <row>))` and the zero-results line with `Within(.., Name("FilterComboEmpty"))`. Type with `Type(text: ..)` after the popup opened (it focuses the input); a deleting key cannot be played (see `filter_combo.rs`), and Shift+Tab is `Key(key: ShiftLeft)`, `Tap(key: Tab)`, `KeyUp(key: ShiftLeft)` because the tester's `modifiers` do not reach Bevy's tab navigation. The `filter_combo_*.ron` files cover opening, filtering, the arrows, Enter, continuing to type, Escape, Tab, Shift+Tab, an outside click, no matches, a hidden selection, many, the accessibility tree, a capped popup scrolling to results below the visible part (with a held arrow), the popup's scroll bar showing while the list overflows and a press on it keeping the popup open (`filter_combo_popup_scroll_bar.ron`) and both themes; all run at scale 2 and `[text 1.5x]` too. `filter_combo_popup_scroll_bar_hides_when_rows_fit.ron` checks the bar hides once the filtered rows fit (layout reports a padded list's content taller than its box by the bottom padding, so `place_popup_scroll_bars` subtracts it, #641).
- **The `ComboRows` screen.** A `row list` (a, b, c; `ZeroOrOne`, nothing selected) of
  custom-view rows, each a text and a `combo in row a` (b, c) over Alpha, Beta and Gamma with
  its own selection; a pick in a row's combo leaves the list unselected. `FilterComboRows`
  has the same rows with a `filter combo in row a` (b, c) in each.
- **The `IconButtons` screen.** Six icon buttons with the default (blank) icon image,
  each with a `Name`: `small button`, `regular button`, `large button`,
  `toggle button` (a `ToggleState` that starts off), `warning button` and
  `danger button`, then "Change theme". `Clicks` counts the `Activate`s of `regular
  button`. `icon_button_hover_colour.ron` reads the background of the regular
  button and the icon tint (`Within(Name("small button"), Component("Icon"))`,
  `ImageNode.color`) of the small one; `icon_button_activates.ron` the `Clicks` and the
  `ToggleState.0` of the toggle.
- **The `SegmentedControls` screen.** A row of `own segmented control` (Auto,
  Manual, its own selection, Auto first) and `first shared segmented control` and
  `second shared segmented control` (Dark, Light, one shared selection through
  `Selection::reference`) and "Change theme", then `three-way segmented control` (Dark, System, Light, its own selection; spawned last to keep the tab order), `icon segmented control` (Grid, List, `ItemIcon`s of the default blank image, text shown) and `icon only segmented control` (Map, Table, `SegmentLabels::Hidden`), spawned after it, so the tab order before them is unchanged. A file reaches an icon with `Nth(i, Within(Name("icon segmented control"), Name("SegmentIcon")))` and an icon-only segment's hidden text with `Component(target: Within(Name("icon only segmented control"), Text("Map")), path: "Node.display", ..)` (`Visible` reads `InheritedVisibility`, which `Display::None` does not change). A file reaches a segment with
  `Within(Name("own segmented control"), Role(role: Button, label: "Manual"))`,
  says what a control shows with `App(Selected(of: Name(..), items: [..]))` and
  whether a segment is `Selected(..)`; each control is one tab stop, so Tab walks
  Auto, Dark, Dark (the selected segment of each control) and the arrows move inside one.
- **The `LightDarkToggle` screen.** `light dark toggle` (a `LightDarkToggle`, so the `ThemePreference` resource starts at `System`) and "Change theme". A file reaches an option with `Within(Name("light dark toggle"), Role(role: Button, label: "Dark"))`, reads the preference with `Resource(path: "ThemePreference", is: Eq(Dark))`; the toggle's sliding indicator is `Name("OptionIndicator")` (`light_dark_toggle_indicator_slides.ron` reads its `UiGlobalTransform` centre, `ComputedNode` width and `Node.left.0` mid-slide and on arrival), the toggle is one Tab stop at the option of the preference, the arrows, Home and End move the focus and pick (`light_dark_toggle_roving_tab_stop.ron`), and Enter / Space pick the focused one. The scenarios have no primary window, so the window theme and `WindowThemeChanged` are covered by headless tests (`a_system_preference_follows_the_system_theme_as_it_changes`, `the_primary_window_theme_follows_the_preference`).
- **The `FilterChips` screen.** A row of `status chips` (All 4, Running 2, Waiting 1,
  Problems 1; its own selection, All first), `mirror chips` (the same selection through
  `Selection::reference`), `narrow chips` (Alpha, Beta, Gamma, Delta in 150 pixels, so they
  wrap, with a selection of its own) and "Change theme". `App(Selected(of: Name(..),
  items: [..]))` names an `Item<Chip>` by its label. Each row is one tab stop, so Tab walks
  "All 4" of the status chips, of the mirror chips and "Alpha" of the narrow ones; the arrows
  move inside one and pick (a chip without a count is labelled with a trailing space, `"Alpha "`).
- **The `Lists` screen** names every list after its selection so a file can
  reach it: `one-or-more list` and `one-or-more grid list` (one shared
  `Selection::reference`), `one list` (own items), `zero-or-one list` and
  `zero-or-more list` (own selections over the shared items, `ItemsRef`),
  `zero-or-more grid list` (own items), `separated list` (`CornerStyle::Separated`, `One`, own items NL to BE; `list_separated_ring_follows_selection`), and `scrolling list` (twelve rows
  inside the `list scroll area`, 100 pixels high). "Add country" spawns the
  next of Suriname, Chile and Kenya into the shared items. A file reaches an
  item with `Within(Name("one list"), Role(role: ListItem, label: "CA"))`
  (an `ItemView` is a `ListItem`; target it, not its text, which the pointer
  passes through; `Selected(..)` and `Not(Selected(..))` say whether it is in the
  selection, and a `Tree(..)` shows `list-item "CA" [selected]`). `lists_screen_fits_window` keeps all of it inside the
  window. A list does not scroll itself: the wheel moves the `ScrollContents`
  of the area around it; read where it scrolled to as the scroll area's
  position above describes.
- **The `SeparatedVirtualList` screen.** A `separated virtual list` of ten thousand rows (`VirtualList::separated(RowHeight::new(44.0)?)?`, "Row 1" to "Row 10000", a `One` selection of its own) in the 240 by 200 `separated virtual list scroll area`, and "Change theme". It is a screen of its own because the `VirtualList` screen's files count its single list and its thirteen rows. Four rows with their gaps fit the view (46 px a row), so Page Down moves four.
- **The `VirtualList` screen.** A `virtual list` of ten thousand rows ("Row 1" to
  "Row 10000", 28 px high, a `One` selection of its own, "Row 1" first) in the 240 by 200
  `virtual list scroll area`, and "Change theme". Its pool is thirteen rows (eight in view,
  one for a part row, two beyond each edge), counted with `Count(with:
  ["VirtualListRow"], ..)`; a row is reached by the item it shows, `Within(Name("virtual
  list"), Role(role: ListItem, label: "Row 9999"))`, and an item out of view has no row
  (`Not(Exists(..))`). The list is the screen's first tab stop
  (`virtual_list_scrolls_to_the_bottom`, `virtual_list_keyboard_moves_and_selects`,
  `virtual_list_keyboard_pages`, where Page Down moves seven rows, the 200 px view of 28 px
  rows); `a_virtual_list_of_ten_thousand_rows_keeps_its_entities_bounded_at_any_scroll_position`
  in `tests/scenarios.rs` checks that rows and entities stay the same at any scroll
  position and when an item is added, and
  `a_virtual_list_tells_a_screen_reader_where_each_row_is_among_all_its_items` the rows'
  `position_in_set` and `size_of_set`, which
  `a_virtual_list_row_that_shows_no_item_has_no_position_among_the_items` checks are
  cleared on a row that shows no item;
  `a_first_page_down_on_a_virtual_list_without_an_active_row_lands_on_the_first_item`
  that Page Down starts where Arrow Down does.
- **Targets on the `ScrollAreas` screen.** Four 200 by 200 areas side by side
  (they and "Change theme" fit the default 1280 by 720 window, which every
  scenario runs in), each with a `Name`: `two-way scroll area`, `vertical
  scroll area`, `horizontal scroll area` and `fitting scroll area` (three
  squares that never overflow). `MoveTo(Name("vertical scroll area"))` puts
  the pointer over the squares, so `Wheel(x: 0.0, y: -100.0)` scrolls it down
  (a positive wheel y scrolls up). Reach contents with `Within(Name(".."),
  Component("ScrollContents"))` and a thumb with `Within(Name(".."),
  Name("ScrollThumb"))` (`Nth(0, ..)` is the vertical bar's, `Nth(1, ..)` the
  horizontal one's in the two-way area; a horizontal-only area has just the
  one, `Nth(0, ..)`). The grid places the contents and the bars explicitly
  (contents row 1 column 1, vertical bar row 1 column 2, horizontal bar row 2
  column 1), so a bar sits in its own cell whichever bars the area has. A fifth
  area, `outer scroll area` (vertical, 230 by 200, absolutely placed at x 0, y 300 and
  spawned last so nothing moves), holds a `nested horizontal scroll area` in a column
  400 px taller than it; the outer contents are `Nth(0, Within(Name("outer scroll
  area"), Component("ScrollContents")))` because the nested ones are below it too.
  Two 200 by 200 `Card`s, `card scroll area` (its area has `ClearOfCorners`) and `card scroll area
  without opt-in`, sit at x 400 and 650, y 300, spawned last, each holding a bare vertical area
  of squares (`scroll_area_in_card_stays_clear_of_rounded_corners.ron` reads the area's
  `Node.margin.bottom`).
- **Observers on a screen's entities** are added with `.observe(..)` on the
  spawned entity. `Observer::new(..)` in a bundle is a global observer.
- **Every file runs at scale factors 1 and 2.** `ron_scenarios` plays each file
  as written (`name`) and on a window with the same logical size and a scale
  factor of 2 (`name [scale 2]`), so a file states logical values. Read
  `ScrollPosition` (logical) rather than `ComputedNode.scroll_position`
  (physical). `ComputedNode.size` and `UiGlobalTransform` are physical: check
  them with `unit: Logical`, which divides by the scale factor, or compare with
  a `Captured`. `Screen(..)` is in logical pixels and runs in both variants; a
  file that really needs a screen position and cannot run at scale 2 says
  `variants: ["default"]`, with a comment saying why (none does today).
- **Every file runs with larger text too.** A third variant, `name [text
  1.5x]`, plays each file with Bevy's `RemSize` (the root font size the
  widgets' text is in rem of, see `theme.rs`) at 1.5 times its default of 20
  px; the window keeps its size. A `TextFont.font_size` is the same rem value
  in every variant (`Eq(Rem(0.7))` for 14 px), so a file that the larger text
  should change checks sizes against a `Captured` one rather than a number.
  The `*_text_follows_rem_size.ron` files (`Themed`, `NumericStepper`,
  `FilterChips` and `Badge` screens) check a label's font size, that the text fits its widget and that the screen
  fits the window. A file whose numbers hold only for the default text (the
  scroll positions of `list_wheel_scrolls` and
  `scroll_area_keyboard_scrolls_item_into_view`, whose rows grow with the
  text) says `variants: ["default", "scale 2"]` with a comment saying why.
- **Prove it fails** as with Rust scenarios: change an expected value in the
  file and see the test fail before trusting it.
- **Every file is played more than once.** Generated libtest cases play it as
  three variants, draw it to an image, and run each schedule/seed combination
  (see [Determinism](#determinism)). Standard test filters select each case.
- **Targets on the `Combos` screen.** A row of `first combo`, `own combo` and
  `third combo` (`first` and `third` share one selection through
  `Selection::reference`, `own` has a selection of its own over the same
  items) and "Change theme", then below it an "Underneath" button and the
  `country list`, which shares the first selection. An open combo's list
  floats over "Underneath", the guard that the popup blocks the pointer for what
  lies beneath it: `Not(Hovered(..))` of it with the pointer on `Canada`
  (`combo_opens_list`), and `Not(Pressed(..))` of it while a click picks
  `Canada` (`combo_popup_click_does_not_reach_underneath`). A `List` and its
  `ItemView`s do not block (`should_block_lower: false`, so the wheel reaches
  the scroll area around them); the combo's list does (`Pickable::default()`
  in `combo.rs`), as does the palette combo's popup, which is why a click on
  either does not reach the `country list` or the `palette` under it. The
  button also keeps the Float's `GlobalZIndex` testable: the tester's
  actionability walk skips nodes with `should_block_lower: false`, so only a
  blocking node can cover a list item.
  Reach the button with `Within(Name("first combo"), Component("ComboButton"))`,
  its list with `Component("ComboList")` and an item with
  `Within(Name("first combo"), Role(role: ListItem, label: "Canada"))` (the
  hidden lists hold the same labels, so always narrow by combo). `ComboState`
  is read with `Component(target: Name(..), path: "ComboState", is: Eq(Open))`;
  `Visible` of the list (or one item) says whether it shows. A click on empty
  space is `MoveTo(Screen((900.0, 500.0)))` then `Click(Left)`. The chevron is
  a child `Icon` of the combo's button, `Within(Name("first combo"),
  Name("Chevron"))` (an `Icon` image, `ImageNode.color` is its tint); the chevron is a
  16 px white-on-transparent triangle drawn by the library, tinted `text_primary` like the label.
  `noted combo` (Amsterdam, Delft `< unreachable >`, Utrecht `4 km`; absolute at x 700, y 200, spawned last) shows `ItemNote`s (`combo_option_notes`). `scrolled combo` (its own selection) sits in the 100 pixel high `combo
  scroll area`, absolutely placed at the right (x 1000, y 300) and spawned last
  so nothing moves and the tab order stays; its list reaches below the area and
  "Belgium" there is picked (`combo_in_scroll_area_popup_is_not_clipped`). A 1 by 400
  pixel spacer after the combo makes the area's contents overflow, so the arrow keys
  in the open list would scroll it were `scroll_into_view` not to stop at a `FixedNode`
  (`combo_in_scroll_area_keyboard_does_not_scroll_the_area`). `right edge combo` (x 1200,
  y 100, 100 wide) and `bottom edge combo` (x 500, y 660) and `long combo` (80 options, x 200, y 300) are absolutely placed and spawned
  last too; their lists stay inside the window (`combo_keeps_list_inside_window`).
  `upward combo` (80 options, x 800, y 600, 200 wide; absolute, spawned last) has no room under it, so its list opens above the button, capped to the room there, and its top is the window's 10 px margin: `Node.top.0` is between 10 and 12 and the pointer is on the list at row 12 and not at row 9 (`combo_list_opening_upward_stays_below_the_top_margin`).
- **Targets on the `BreadCrumbs` and `Tabs` screens.** The trail is
  `Name("BreadCrumbs")` and the stack `Name("Stack")`. A crumb before the last
  is a `Button` (`Within(Name("BreadCrumbs"), Role(role: Button, label:
  "Home"))`); the last is a disabled button (`InteractionDisabled`; `[disabled]`
  in a `Tree(..)` snapshot, `Disabled(..)` as a check) with the same role, so
  `Role(role: Button, label: "Panel A")` reaches it too, as does
  `Nth(i, Within(Name("BreadCrumbs"), Name("Link")))`. The tester will not click
  it (`Force(Click(Left))` does, to show that nothing happens), and the text
  inside is `Pickable::IGNORE`, so a `MoveTo` to the text is "covered by
  `Link`". Read its `BackgroundColor.0`; the colour of its label is
  `TextColor.0` of `Text("..")`. A `Stack` panel hidden by `Display::None` still exists
  but is left out of a `Tree(..)` snapshot, which also accepts extra
  siblings, so say that a panel is hidden with `Not(Visible(Within(Name("Stack"),
  Text("Panel A"))))` and that one was never created with `Not(Exists(..))`.
  The panels that were created are counted by `Count` of `PushButton`s
  (`stack_shows_only_the_selected_item`); the `Tabs` screen's number list
  keeps its selection while its panel is hidden
  (`stack_keeps_state_of_hidden_item`). The combos in Panel A open popups that are descendants of the panel, so hiding it hides them and takes the list out of the tab order (`stack_hidden_panel_hides_combo_popups`). The tabs are the `Tab`s of a
  `TabBar` (its sliding indicator is `Name("TabIndicator")`, `tab_bar_indicator_slides.ron`), `Role(role: Tab, label: "Panel A")`, `Panel B` and `Card demo`;
  the bar is one tab stop, the arrows move between its tabs and Enter or
  Space picks the focused one.
- **Targets on the `ColorPalette` screen.** A row of `first palette combo`,
  `second palette combo` (the two share one selection) and `own palette combo`
  (its own, over the same colours) and "Change theme"; `one palette combo`
  is out of the row (absolute, 300 pixels from the left) with a `One` selection
  over the same colours, slate-50 until another is picked, so that clicking
  the selected colour changes nothing; below the row `palette` (all
  242 Tailwind colours, 22 columns so that it, 836 by 418, fits the window; the
  example's 11 columns are taller than it) and `palette scroll area` (300 by
  300, 11 columns, sharing `palette`'s selection through its inner
  `Component("ColorPalette")`). Swatches have no text: they are `ItemView`s,
  reached by position as `Nth(60, Within(Name("palette"),
  Component("ItemView")))` (`Within` first narrows the `ItemView`s to those
  below `palette`, in document order, and `Nth` then counts in that list; 60
  is red-500, the order of `tailwind::COLORS`). A selected
  swatch is `Selected` (`Check(Selected(Nth(60, ..)))`) and has a `BorderColor.top`
  other than `background_primary`. A combo's
  button is its `ColorPushButton` and the colour it shows the
  `BackgroundColor.0` of its `ColorSquare`. The popup is a `Float` child of the
  combo, so `Within(Name("first palette combo"), ..)` reaches into it as well,
  and is named after it when it is spawned (`first palette combo popup`, `second palette combo
  popup`, `own palette combo popup`; a combo with only its default name has
  `ColorPaletteCombo popup`), whatever order the combos are in. Its swatches
  are `Nth(60, Within(Name("first palette combo popup"),
  Component("ItemView")))`; the combo's state is `Component(target:
  Name("first palette combo"), path: "ColorPaletteComboState", is: Eq(Open))` and the popup's
  position the `UiGlobalTransform.0.translation` (the centre) of the float.
  The open popup lies over the first row of `palette`: swatch 0 of the popup
  (centre 314, 79) is over swatch 7 of `palette`, which is not hovered, pressed
  or selected by a click on it
  (`color_palette_combo_popup_click_does_not_reach_palette`).
  The popup is centred under its button and pushed in to 10 pixels from the
  window's left and right edges; the screen has `left edge palette combo` and
  `right edge palette combo`, absolutely placed in the top corners so nothing
  else moves, which open into those margins
  (`color_palette_combo_keeps_margin_at_window_edge`). In a 1280 by 260 window (`window_size`) the popup is capped to 192 high, top 58 and bottom 250, in every variant; the pointer on rows 249 and 250 tells whether it is on the popup, and Arrow Down scrolls the `ScrollContents` inside it, not the popup (`color_palette_combo_taller_than_window_stays_inside_window`, `color_palette_combo_taller_than_window_keyboard_scrolls_the_popup`). The layout rounds the
  popup to whole pixels, so beside the 39 pixel wide button (centre on a half
  pixel) its centre is the button's plus or minus 0.5. The popup's palette refers to
  the combo's selection from the moment it is spawned, so
  `color_palette_combo_pick_as_first_action` opens `own palette combo` and picks a
  swatch as its first steps, with no wait. `scrolled palette combo` (its own
  selection) sits in the 100 pixel high `palette combo scroll area`, absolutely
  placed under `palette scroll area` and spawned last so the tab order stays;
  its popup reaches below the area and a swatch there is picked
  (`color_palette_combo_in_scroll_area_popup_is_not_clipped`).
  `zero or more palette combo` (empty) and `one or more palette combo` (slate-50) are absolutely
  placed at x 1000 and 1100, y 250, spawned last, over the palette's items and a shared
  selection of their own (`color_palette_combo_many_stays_open`).
- **The `Themed` screen and the theme scenarios.** One of each widget that
  follows the theme in a row: `themed button` (a `PushButton` "Other"),
  `themed card`, `themed list` (`One`, `Two`, `Three`, the first selected) and
  "Change theme". The theme scenarios start here because it is the only screen
  with a `Card`. `Theme.colors` is checked whole with
  `Resource(path: "Theme.colors", is: Eq((background_primary: Srgba(..), ..)))`
  (build the scenario in Rust and print `to_ron()` for the numbers),
  `ClearColor` with `Resource(path: "ClearColor.0", ..)`, a shadow with
  `Component(target: .., path: "BoxShadow.0[0].color", ..)` and the colour of
  a label with `TextColor.0` of `Text("..")`. Read a widget the pointer is not
  on: "Change theme" stays hovered after its click, and a hovered item has
  hover colours. The one that reads
  the hovered "Change theme" is `theme_keeps_hover_colour_under_pointer.ron`: a
  button the pointer rests on shows the new theme's `hovered_item`. A button
  let go with the pointer still on it is `hovered_item` too, not
  `selected_item`.

### Determinism

Since Bevy 0.20 many system sets (the UI's in `PostUpdate`) are ordered only
where systems' data access conflicts, so a system that runs at the right
moment only by luck would pass on one order and fail on another.
Generated tests in `tests/scenarios.rs` play every `tests/ron/` file once for
each of `PreUpdate`, `Update`, `PostUpdate` and `Last` and each of the fixed
seeds `[1, 2, 3, 0x5eed_cafe]` (never the clock), with that schedule built with
`ScheduleBuildSettings::shuffle_seed` and the single-threaded executor, so the
order is the seed's alone. The `debug` Bevy feature that provides the seed is
in `[dev-dependencies]` only.

- **Reproduce a failure** with the generated test filter, for example
  `cargo test --test scenarios scenario_combo_opens_list_postupdate_seed_5eedcafe
  -- --exact`. The test runs that file with `shuffle(&mut app, schedule,
  seed)` after `Registry::install`, so its name is enough to replay the failure.
- **Fix it by ordering** the system through the data it touches (`.after(..)`
  or a set). Never widen the seed list or skip the file.

### Recording the scenarios

`just record` plays every file of `tests/ron/` with the tester's offscreen
recorder and checks the results without saving recordings. `just record --save`
writes to `target/recordings/` (`RECORD_DIR` overrides it): per scenario PNGs
at the step boundaries and every tenth frame, `filmstrip.png`, `scenario.gif`,
`trace.html`, and `index.html` under `target/recordings/<scenario>/`. Open the
index and look; no picture is compared with a golden one. Each scenario is an
ignored, independently runnable test named `record_<scenario>` in
`recorded_scenarios_gpu`, with separate output directories for parallel runs.
It needs a GPU, or lavapipe (`mesa-vulkan-drivers libvulkan1` and
`WGPU_BACKEND=vulkan`), so `just test` and `just ci` never build it; lint it
with `cargo clippy --test recorded_scenarios_gpu --features record`.

- **A drawn recording is not the headless one.** The recorder's cameras render
  to an `Image`, and Bevy's UI focus (`Interaction`, `RelativeCursorPosition`)
  only follows cameras that render to a window, so a widget must not read it:
  the widgets read hover from picking, which works for any target (see "Is the
  pointer on it?" above). Combos, palette combos and the hover colours of lists
  and palettes used to read `RelativeCursorPosition` and misbehaved in a
  recording (#100). Generated per-file tests in `tests/scenarios.rs` play each
  file headless with the camera aimed at an image, as the recorder does but
  without a GPU, so this is caught by `just test`. A failure names its file;
  there is no allowlist, so a
  scenario that only fails with the UI drawn to an image is a bug to fix, not a
  name to list.
- **CI.** The `record` job of `ci.yml` sets `RECORD_CHECK_ONLY=1` and runs
  the same scenario tests as `just record` on lavapipe. This uses
  `Recorder::check` to run the rendered checks without generating recordings.
  It is not part of `just ci`.

## Before you call it done

1. `just fmt`
2. `just lint`
3. `just test`
4. New behaviour has a test, where the harness allows it
5. This file is updated if a module, example, convention or command changed
6. The work is on a PR, rebased onto `main` if needed
