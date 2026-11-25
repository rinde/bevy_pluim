# `setting_row.rs`

`SettingRow` and `SettingRows`: a title, description and control slot, divided rows.

Read this before changing `src/setting_row.rs`. When the module's behavior changes, update this file in the same PR.

## Details

`SettingRow::new(title, description)` (`on_insert`: insert a new `SettingRow` on a live row to change its title and description; the title `Text`, description `Label` and accessibility node follow, and the children, `SettingRowRefs` and the app's control stay, `inserting_a_setting_row_again_changes_its_text_and_keeps_its_control`): a title (semibold, `font_size_body`) over a secondary description on the left and an empty control slot on the right (12 px gap and vertical padding), then a `Divider`; `SettingRowRefs { title_id, description_id, control_slot, divider_id }` names them, and the app spawns its control (a `SegmentedControl`, say) as a child of `control_slot`. Put the rows in a `SettingRows` column: a system hides the divider of the last row and shows the others'. `sync_dividers` looks only at columns whose `Children` changed (or that are new) and at the parent of a row whose `SettingRowRefs` just arrived, finds the last row by a reverse scan and collects nothing, so a quiet frame does no work; non-row children are skipped and a column with no row is left alone (`setting-rows-idle` and `setting-rows-changing` bench workloads, #689). A row is a `Role::Group` labeled with its title, its description the node's `description` (`setting_row_accessible_group.ron`)

## Deferred construction

Two inserts applied in one command flush build one set of parts, the last insert's configuration winning (AGENTS.md, "Deferred construction decides at run time"; the `two_inserts_of_*_in_one_command_batch_*` tests in `tests/headless.rs`).
