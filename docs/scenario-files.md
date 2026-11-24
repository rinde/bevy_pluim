# Scenario files

The `tests/ron/` scenario format, its checks and variants, and the `WidgetsSetup` screens.

When the behaviour described here changes, update this file in the same PR.

A scenario can be data: `tests/ron/NAME.ron` is compiled into individual
libtest cases by `build.rs`. Each file gets default, scale 2 and text 1.5x
cases named with its file and variant, and standard Cargo filters select a
single case (for example, `cargo test --test ron_scenarios scenario_push_button_scale_2`).
`just test` runs them with everything else. Each case runs in a fresh
`common::widgets_app()` after `Registry::install`, so a file can check any
`Reflect` type of the app by short type path. `skip` and `fixme` scenarios are
ignored by libtest, variant restrictions leave their out-of-scope cases
unplayed, and `should_fail` passes only when the scenario fails as expected.

## The format

is the tester's RON script (`format_version: 3`); see its
README and `tests/ron/*.ron` there. The same file also plays in the tester's
gallery and recorder. Write or record it there, then drop it in `tests/ron/`.
A colour in a check is `Srgba((red:..,green:..,blue:..,alpha:..))`; to get
the numbers of a theme slot, build the scenario in Rust and print
`to_ron()`.

## The setup

is `setup: App(Buttons)`: a variant of `WidgetsSetup`, the one
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

## The check

is `App(Selected(of: .., items: [..]))`: `WidgetsCheck`, the
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

## The accessibility check

is `App(Accessible(of: .., role: Some("Button"),
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

## Observers on a screen's entities

are added with `.observe(..)` on the
spawned entity. `Observer::new(..)` in a bundle is a global observer.

## Every file runs at scale factors 1 and 2

`ron_scenarios` plays each file
as written (`name`) and on a window with the same logical size and a scale
factor of 2 (`name [scale 2]`), so a file states logical values. Read
`ScrollPosition` (logical) rather than `ComputedNode.scroll_position`
(physical). `ComputedNode.size` and `UiGlobalTransform` are physical: check
them with `unit: Logical`, which divides by the scale factor, or compare with
a `Captured`. `Screen(..)` is in logical pixels and runs in both variants; a
file that really needs a screen position and cannot run at scale 2 says
`variants: ["default"]`, with a comment saying why (none does today).

## Every file runs with larger text too

A third variant, `name [text
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

## Prove it fails

as with Rust scenarios: change an expected value in the
file and see the test fail before trusting it.

## Every file is played more than once

Generated libtest cases play it as
three variants, draw it to an image, and run each schedule/seed combination
(see [Determinism](testing.md#determinism)). Standard test filters select each case.

## Screens

What each `WidgetsSetup` screen holds and how a file reaches its widgets.

### Targets on a screen with several of one widget

The `NumericStepper`
screen puts a `Name` on every stepper (`default stepper`, `u16 stepper`,
`u32 stepper`, `f32 stepper`, `stepper by two`, and `shared stepper` with
the `first mirror stepper` and `second mirror stepper` that point to it
through `ValueRef`), so a file reaches one stepper's button with
`Within(Name("u16 stepper"), Name("Increase"))` (the icon buttons have no text for a
`Role` label to match, so they are named after their accessible name) and reads its
value with `TextOf(Name("u16 stepper"), Exact("11"))`. Move the pointer to the button before every click: a wider
value moves the buttons. A button at its limit is `InteractionDisabled`, so
the tester's click on it does nothing (`numeric_stepper_bounds.ron`).

### The `Panel`, `SettingRows`, `Badge` and `Tooltip` screens

`Panel`
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

### The `Gallery` and `Toolbar` screens

`gallery` (Steam, Diesel and Passenger cards in
two groups, a `One` selection) and `filled toolbar` (Select, Build, Bulldoze, a `One`
selection), each with "Change theme". Their `*_text_follows_rem_size.ron` files check that a
label is `Rem(0.6)` text and fits inside its fixed px card (104 x 92) or button (66 x 60) at
`[text 1.5x]`; both pass, so those sizes stay px. The `Toolbar` screen also has three toolbars
absolutely placed at x 0 (y 150, 300, 450) and spawned last, so nothing above moves and the tab
order of the first two stays: `optional toolbar` (Pan, Zoom (`Tone::Warning`), `ZeroOrOne`, nothing open),
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

### The `Slider` screen

A column of `continuous slider` (0 to 1, 320 px wide, name
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

### The `TextFields` screen

`name field` ("Alpha", named "Name"), `seed field`
(empty, placeholder "Seed", named "Seed"), an "Other" button and "Change theme". The
`FieldEvents` resource counts the fields' `TextFieldChange`s and `TextFieldCommit`s and keeps
the last value of each (`changes`, `changed`, `commits`, `committed`). Type into a field
with `Type(text: "42", into: Within(Name("seed field"), Component("TextInput")))` (the
tester clicks it first), read its text with `TextOf(..)` of the same target and whether
the placeholder shows with `Visible(Within(Name("seed field"), Text("Seed")))`. The
`text_field_*.ron` files cover typing and `Enter`, `Escape` restoring, a click elsewhere,
a click on the border (at `Screen((20.5, 32.0))`, the left border of `name field`),
Tab and the screen reader node, and the text following `RemSize`.

### The `Toast` screen

"Save" and "Save again" raise a toast "Saved", "Delete" one "Deleted 3 pieces", each shown for 1 s (the file waits for time with `Within`, not a number of frames, and `the_toast_scenario_passes_when_frames_are_shorter_than_a_sixtieth_of_a_second` plays it at 1/240 s), and "Change theme". `Count(with: ["Toast"], ..)` counts the toasts showing (`toast_shows_stacks_and_goes.ron`).

### The `FilterCombos` screen

A row of `filter combo` (named "City") and `ordinary combo`, both sharing one `ZeroOrOne` selection (Amsterdam, Delft, Den Haag, Utrecht, Rotterdam) with the `city list` below them, which is never filtered, and "Change theme"; then, absolutely placed and spawned last so the tab order stays: `multi filter combo` ("Cities", `ZeroOrMore`, own items, x 300 y 300), `one filter combo` (`One`, x 550), `long filter combo` (80 options at x 800 y 600, so the popup opens upward, capped and scrolling) and `empty filter combo` (`One`, no items, x 1050). Reach the button with `Within(Name("filter combo"), Component("ComboButton"))`, the search input with `Component("TextInput")`, the options with `Nth(i, Within(Name(..), Component("ItemView")))` and read `Node.display` (`Eq(None)` for a filtered-out row; `Role(role: ListItem, ..)` does not find a hidden row), the active option with `App(Active(of: <input>, is: <row>))` and the zero-results line with `Within(.., Name("FilterComboEmpty"))`. Type with `Type(text: ..)` after the popup opened (it focuses the input); a deleting key cannot be played (see `filter_combo.rs`), and Shift+Tab is `Key(key: ShiftLeft)`, `Tap(key: Tab)`, `KeyUp(key: ShiftLeft)` because the tester's `modifiers` do not reach Bevy's tab navigation. The `filter_combo_*.ron` files cover opening, filtering, the arrows, Enter, continuing to type, Escape, Tab, Shift+Tab, an outside click, no matches, a hidden selection, many, the accessibility tree, a capped popup scrolling to results below the visible part (with a held arrow), the popup's scroll bar showing while the list overflows and a press on it keeping the popup open (`filter_combo_popup_scroll_bar.ron`) and both themes; all run at scale 2 and `[text 1.5x]` too. `filter_combo_popup_scroll_bar_hides_when_rows_fit.ron` checks the bar hides once the filtered rows fit (layout reports a padded list's content taller than its box by the bottom padding, so `place_popup_scroll_bars` subtracts it, #641).

### The `ComboRows` screen

A `row list` (a, b, c; `ZeroOrOne`, nothing selected) of
custom-view rows, each a text and a `combo in row a` (b, c) over Alpha, Beta and Gamma with
its own selection; a pick in a row's combo leaves the list unselected. `FilterComboRows`
has the same rows with a `filter combo in row a` (b, c) in each.

### The `IconButtons` screen

Six icon buttons with the default (blank) icon image,
each with a `Name`: `small button`, `regular button`, `large button`,
`toggle button` (a `ToggleState` that starts off), `warning button` and
`danger button`, then "Change theme". `Clicks` counts the `Activate`s of `regular
button`. `icon_button_hover_colour.ron` reads the background of the regular
button and the icon tint (`Within(Name("small button"), Component("Icon"))`,
`ImageNode.color`) of the small one; `icon_button_activates.ron` the `Clicks` and the
`ToggleState.0` of the toggle.

### The `SegmentedControls` screen

A row of `own segmented control` (Auto,
Manual, its own selection, Auto first) and `first shared segmented control` and
`second shared segmented control` (Dark, Light, one shared selection through
`Selection::reference`) and "Change theme", then `three-way segmented control` (Dark, System, Light, its own selection; spawned last to keep the tab order), `icon segmented control` (Grid, List, `ItemIcon`s of the default blank image, text shown) and `icon only segmented control` (Map, Table, `SegmentLabels::Hidden`), spawned after it, so the tab order before them is unchanged. A file reaches an icon with `Nth(i, Within(Name("icon segmented control"), Name("SegmentIcon")))` and an icon-only segment's hidden text with `Component(target: Within(Name("icon only segmented control"), Text("Map")), path: "Node.display", ..)` (`Visible` reads `InheritedVisibility`, which `Display::None` does not change). A file reaches a segment with
`Within(Name("own segmented control"), Role(role: Button, label: "Manual"))`,
says what a control shows with `App(Selected(of: Name(..), items: [..]))` and
whether a segment is `Selected(..)`; each control is one tab stop, so Tab walks
Auto, Dark, Dark (the selected segment of each control) and the arrows move inside one.

### The `LightDarkToggle` screen

`light dark toggle` (a `LightDarkToggle`, so the `ThemePreference` resource starts at `System`) and "Change theme". A file reaches an option with `Within(Name("light dark toggle"), Role(role: Button, label: "Dark"))`, reads the preference with `Resource(path: "ThemePreference", is: Eq(Dark))`; the toggle's sliding indicator is `Name("OptionIndicator")` (`light_dark_toggle_indicator_slides.ron` reads its `UiGlobalTransform` centre, `ComputedNode` width and `Node.left.0` mid-slide and on arrival), the toggle is one Tab stop at the option of the preference, the arrows, Home and End move the focus and pick (`light_dark_toggle_roving_tab_stop.ron`), and Enter / Space pick the focused one. The scenarios have no primary window, so the window theme and `WindowThemeChanged` are covered by headless tests (`a_system_preference_follows_the_system_theme_as_it_changes`, `the_primary_window_theme_follows_the_preference`).

### The `FilterChips` screen

A row of `status chips` (All 4, Running 2, Waiting 1,
Problems 1; its own selection, All first), `mirror chips` (the same selection through
`Selection::reference`), `narrow chips` (Alpha, Beta, Gamma, Delta in 150 pixels, so they
wrap, with a selection of its own) and "Change theme". `App(Selected(of: Name(..),
items: [..]))` names an `Item<Chip>` by its label. Each row is one tab stop, so Tab walks
"All 4" of the status chips, of the mirror chips and "Alpha" of the narrow ones; the arrows
move inside one and pick (a chip without a count is labelled with a trailing space, `"Alpha "`).

### The `Lists` screen

names every list after its selection so a file can
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
of the area around it; read where it scrolled to as
[scrolling](design/scrolling.md#a-scroll-areas-scroll-position-is-kept-in-range) describes.

### The `SeparatedVirtualList` screen

A `separated virtual list` of ten thousand rows (`VirtualList::separated(RowHeight::new(44.0)?)?`, "Row 1" to "Row 10000", a `One` selection of its own) in the 240 by 200 `separated virtual list scroll area`, and "Change theme". It is a screen of its own because the `VirtualList` screen's files count its single list and its thirteen rows. Four rows with their gaps fit the view (46 px a row), so Page Down moves four.

### The `VirtualList` screen

A `virtual list` of ten thousand rows ("Row 1" to
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

### Targets on the `ScrollAreas` screen

Four 200 by 200 areas side by side
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

### Targets on the `Combos` screen

A row of `first combo`, `own combo` and
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

### Targets on the `BreadCrumbs` and `Tabs` screens

The trail is
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

### Targets on the `ColorPalette` screen

A row of `first palette combo`,
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

### The `Themed` screen and the theme scenarios

One of each widget that
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
