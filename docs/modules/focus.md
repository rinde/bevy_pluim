# `focus.rs`

Crate-private keyboard focus: tab stops, roving groups, the focus ring, hidden-state tracking and the accessibility sync helpers.

Read this before changing `src/focus.rs`. When the module's behaviour changes, update this file in the same PR.

Cross-cutting design: [keyboard focus and navigation](../design/keyboard-focus.md).

## Details

a crate-private module (its `FocusPlugin` is added by `WidgetsPlugin`): the crate-private `TabStop` marker (requires `TabIndex`), the `RovingItem` / `RovingGroup` pair and `sync_roving_groups` of the composite widgets (see [composite widgets](../design/keyboard-focus.md#composite-widgets-are-one-tab-stop-with-arrow-keys)), `sync_tab_index` and the focus ring (`draw_focus_ring`, in the crate-private `DrawFocusRing` set in `PostUpdate` before the layout), see [keyboard focus](../design/keyboard-focus.md). None of the three walks all candidates per frame: `draw_focus_ring` works out the one entity that should have the ring and touches only it and the one that had it (kept in a `Local`); `collect_hidden_changes` (`PostUpdate`, after visibility propagation, before its consumers) runs once a frame and writes the `HiddenDirty` resource that `sync_tab_index`, `sync_accessibility_hidden`, `sync_roving_groups` and `tab_bar.rs`'s `sync_tab_index` read (each adds its own extras, such as new stops or markers): the entities whose `InheritedVisibility` or parent changed, whose `Display::None` flipped (tracked in a `Local` set), or that lost a `Node` or parent, with the whole subtree of each such root, walked once (overlapping roots, nested flips and a reparent together never revisit a descendant), and `is_hidden` is called only for those (`overlapping_display_flips_and_a_reparent_in_one_frame_reach_every_consumer_of_a_deep_tree`; the `deep-trees-*` bench workloads, #684). Also the crate-private `AccessibilitySync` set (`PostUpdate`, after `UiSystems::PostLayout`, where Bevy's `button_changed` sets `Role::Button` when a `Button` changes, and before `AccessibilitySystems::Update`) with its `sync_role` / `sync_toggled` / `sync_label` / `sync_value` / `clear_label` / `sync_controls` / `sync_labelled_by` helpers (`node_id` turns an entity into the `accesskit::NodeId` Bevy gives it), which write a node only when it differs: widgets set their roles, toggled states and labels there, so Bevy does not undo them. A container group without a title of its own (`Card`, a toolbar or radio group) has no label; `SettingRow` is labelled with its title and `Panel` is `labelled_by` its title `Label`; an app can insert Bevy's `AccessibleLabel` on it

## Accessibility sync looks at what changed

The convention is in AGENTS.md ("`sync_accessibility` systems look at what changed"). Guarded by `filter_chips_leave_their_accessibility_nodes_alone_on_idle_frames_and_for_other_chips`; bench workloads `segmented-controls-idle` and `galleries-idle`, numbers in `docs/benchmarks.md`.
