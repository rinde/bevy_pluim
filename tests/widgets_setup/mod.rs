//! `WidgetsSetup`: the screens scenario files start from.
//!
//! The tester finds this one `#[reflect(AppSetup)]` enum by itself, so a
//! scenario file writes `setup: App(Buttons)` and a Rust scenario
//! `.app_setup(WidgetsSetup::Buttons)`. Each variant spawns a `Camera2d`, the
//! one root `Node` and a screen of the widget it is named after. A screen
//! starts from the example of that name but is a test fixture of its own: it
//! adds names, widgets and sizes for testing and does not follow later changes
//! to the example. The examples are not built on this module: they spawn
//! through `Commands` in a startup system, while a setup is handed the
//! `World`.
//!
//! Pull it in with `mod widgets_setup;` next to `mod common;`; it uses
//! [`common::spawn_camera_and_root`].

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private test module"
)]

use std::path::Path;

use bevy::color::palettes::tailwind::{
    AMBER_600, BLUE_600, CYAN_600, EMERALD_600, FUCHSIA_600, GREEN_600, INDIGO_600, LIME_600,
    ORANGE_600, PINK_600, PURPLE_600, RED_500, ROSE_600, SKY_600, YELLOW_600,
};
use bevy::ecs::spawn::{SpawnIter, SpawnWith};
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Activate, ValueChange};
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::badge::{Badge, BadgeTone};
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::color_push_button::{ColorPushButton, ColorSquareRef};
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::filter_chips::{Chip, FilterChips};
use rin_bevy_ui::gallery::{Gallery, GalleryCard};
use rin_bevy_ui::icon_button::{IconButton, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::label::Label;
use rin_bevy_ui::link::Link;
use rin_bevy_ui::list::{
    CornerStyle, ItemNote, ItemViewCreator, ItemViewSpawner, List, ListLayout,
};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value, ValueRef};
use rin_bevy_ui::panel::{Panel, PanelClosed, PanelRefs};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::segmented_control::SegmentedControl;
use rin_bevy_ui::select::{
    Item, ItemOf, ItemParam, ItemValues, Selection, SelectionParam, SelectionValue,
};
use rin_bevy_ui::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use rin_bevy_ui::slider::Slider;
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::tab_bar::TabBar;
use rin_bevy_ui::tailwind::COLORS;
use rin_bevy_ui::text_field::{TextField, TextFieldChange, TextFieldCommit};
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui::toast::Toast;
use rin_bevy_ui::toolbar::{Tool, Toolbar, ToolbarStyle};
use rin_bevy_ui::tooltip::Tooltip;
use rin_bevy_ui::virtual_list::VirtualList;
use rin_bevy_ui_tester::{AppSetup, ReflectAppSetup};

use crate::common::spawn_camera_and_root;

/// The screens a scenario can start from, one per widget.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(AppSetup)]
pub enum WidgetsSetup {
    /// "Add", "Other", a "Disabled" one (`InteractionDisabled`) and "Change
    /// theme" push buttons; the clicks on "Add" are counted in [`Clicks`].
    Buttons,
    /// Three combos over one list of countries (`first combo` and `third
    /// combo` share one selection, `own combo` has a selection of its own over
    /// the same items), a button ("Underneath") and a list sharing the first
    /// selection (`country list`), both below the combos, so an open combo
    /// floats over them, and "Change theme".
    Combos,
    /// Lists in several layouts and corner styles, one per kind of selection
    /// (`one list`, `zero-or-one list`, `zero-or-more list`,
    /// `zero-or-more grid list` and the `one-or-more list` that the
    /// `one-or-more grid list` shares its selection with, while the
    /// `zero-or-one list` and `zero-or-more list` choose from the same items),
    /// a `scrolling list` taller than its `list scroll area`, "Add country",
    /// which adds a country to the shared items, and "Change theme".
    Lists,
    /// Four 200 by 200 scroll areas, each named after what it does
    /// (`two-way scroll area`, `vertical scroll area`, `horizontal scroll
    /// area` and `fitting scroll area`, whose three squares fit and so
    /// never scroll), and "Change theme".
    ScrollAreas,
    /// A breadcrumb trail over a stack of four panels with buttons that go
    /// further in or back, and "Change theme".
    BreadCrumbs,
    /// A list as tab bar over a stack of three panels, and "Change theme".
    Tabs,
    /// Three palette combos (`first palette combo` and `second palette
    /// combo` share one selection, `own palette combo` has a selection of its
    /// own) and "Change theme" in a row, then below it a `palette` of all 242
    /// Tailwind colours in 22 columns and a 300 by 300 `palette scroll
    /// area` with 11 columns, both showing one selection. The combos choose
    /// from the same items.
    ColorPalette,
    /// Numeric steppers, each named after its type or role (`default
    /// stepper`, `u16 stepper`, `u32 stepper`, `f32 stepper`, `stepper by
    /// two`), three of them showing one value (`shared stepper` holds it,
    /// `first mirror stepper` and `second mirror stepper` point to it), and
    /// "Change theme".
    NumericStepper,
    /// Sliders, one per row, each with a `Name`: `continuous slider` (0 to 1,
    /// 320 pixels wide, accessible name "Continuous"), `stepped slider` (0 to
    /// 10 in steps of 1, at 0, "Stepped"), `formatted slider` (0 to 100 in
    /// steps of 5, at 25, its value shown as "25 %", "Speed") and `disabled
    /// slider` (0 to 10 at 3, shown as "3", `InteractionDisabled`), then
    /// "Change theme". [`SliderChanges`] keeps the `ValueChange`s of `stepped
    /// slider`.
    Slider,
    /// Segmented controls, each with a `Name`: `own segmented control`
    /// (Auto, Manual, with a selection of its own) and `first shared
    /// segmented control` and `second shared segmented control` (Dark, Light,
    /// sharing one selection), then "Change theme".
    SegmentedControls,
    /// Rows of filter chips, each with a `Name`: `status chips` (All 4, Running 2,
    /// Waiting 1, Problems 1, with a selection of its own, All first), `mirror chips`
    /// (the same selection through `Selection::reference`) and `narrow chips` (150
    /// pixels wide, so that the chips wrap, with a selection of its own), then
    /// "Change theme".
    FilterChips,
    /// Two panels, each with a title, a subtitle and a close button, which
    /// closes the panel by despawning it: `closable panel` (a few lines in
    /// its body) and `tall panel`, whose 100 pixel high body is shorter than
    /// its twelve lines and so scrolls. Then "Change theme".
    Panel,
    /// A `setting rows` column of three `SettingRow`s (`appearance row` with
    /// a segmented control, `text size row` with a numeric stepper and `last
    /// row` with a button) and "Change theme".
    SettingRows,
    /// An icon button named `counted button` with a neutral badge saying 3, a
    /// `warning button` with a warning badge saying 12, an icon button with no
    /// badge (`plain button`), "Add badge", which raises the count of the first
    /// by one, and "Change theme".
    Badge,
    /// Icon buttons, each with a `Name`: `small button`, `regular button`,
    /// `large button`, `toggle button` (a `ToggleState` that starts off),
    /// `warning button` and `danger button`, and "Change theme". `Clicks`
    /// counts the `Activate`s of `regular button`.
    IconButtons,
    /// A `gallery` of three cards (Steam, Diesel in "Locomotives", Passenger in
    /// "Wagons"), a `One` selection of its own, and "Change theme".
    Gallery,
    /// A `filled toolbar` of three tools (Select, Build, Bulldoze) with a selection of its
    /// own, and "Change theme".
    Toolbar,
    /// A `tooltip anchor` push button with the tooltip "Anchor tip", an icon
    /// button named "Help" (which has a tooltip of its own, "Help") and
    /// "Change theme".
    Tooltip,
    /// Buttons that raise toasts: "Save" ("Saved", 1 s), "Save again" (the same text),
    /// "Delete" ("Deleted 3 pieces", 1 s) and "Change theme".
    Toast,
    /// Two text fields, each with a `Name`: `name field` (holding "Alpha", accessible name
    /// "Name") and `seed field` (empty, placeholder "Seed", accessible name "Seed"), then
    /// "Other" and "Change theme". [`FieldEvents`] counts what the fields triggered.
    TextFields,
    /// Buttons whose accessible names are checked, each with a `Name`:
    /// `plain button` (a `PushButton` "Add"), `icon push button` (a
    /// `PushButton` "Save" with the icon text "+"), `colour button` (a
    /// `ColorPushButton` showing red-500) and `link` (a `Link` "Docs"), then
    /// "Change theme".
    ButtonNames,
    /// One of each widget whose colours follow the theme, side by side: a
    /// `themed button` ("Other"), a `themed card` and a `themed list` of
    /// three items (the first selected), then "Change theme".
    Themed,
    /// A `virtual list` of ten thousand rows ("Row 1" to "Row 10000", 28
    /// pixels high, a `One` selection of its own, "Row 1" first) in the 240
    /// by 200 `virtual list scroll area`, and "Change theme".
    VirtualList,
    /// A `separated virtual list` of ten thousand rows ("Row 1" to "Row 10000",
    /// 44 pixels high with the separated look, a `One` selection of its own,
    /// "Row 1" first) in the 240 by 200 `separated virtual list scroll area`,
    /// and "Change theme".
    SeparatedVirtualList,
    /// A `row list` of custom-view rows (a, b, c; a `One` selection, "a" first), each holding
    /// a `Combo` named `combo in row a` (and b, c) with a selection of its own over the
    /// countries, then "Change theme".
    ListWithCombos,
}

impl AppSetup for WidgetsSetup {
    fn build(&self, world: &mut World) -> Result<(), String> {
        world.insert_resource(CurrentScreen(*self));
        match self {
            Self::Buttons => buttons(world),
            Self::Combos => combos(world),
            Self::Lists => lists(world),
            Self::ScrollAreas => scroll_areas(world),
            Self::BreadCrumbs => bread_crumbs(world),
            Self::Tabs => tabs(world),
            Self::ColorPalette => color_palette(world),
            Self::NumericStepper => numeric_stepper(world),
            Self::Slider => slider(world),
            Self::SegmentedControls => segmented_controls(world),
            Self::FilterChips => filter_chips(world),
            Self::Panel => panel(world),
            Self::SettingRows => setting_rows(world),
            Self::Badge => badge(world),
            Self::IconButtons => icon_buttons(world),
            Self::Gallery => gallery(world),
            Self::Toolbar => toolbar(world),
            Self::Tooltip => tooltip(world),
            Self::Toast => toast(world),
            Self::TextFields => text_fields(world),
            Self::ButtonNames => button_names(world),
            Self::Themed => themed(world),
            Self::VirtualList => virtual_list(world, false),
            Self::SeparatedVirtualList => virtual_list(world, true),
            Self::ListWithCombos => list_with_combos(world),
        }
        Ok(())
    }

    /// The screen the app was last built from, so a recording started from the
    /// app as it is loads back as that screen. What was done to it since (a
    /// selection, an open combo) is not captured: a script starts from the
    /// screen as built.
    fn capture(world: &mut World, _dir: &Path, _stem: &str) -> Result<Option<Self>, String> {
        Ok(world.get_resource::<CurrentScreen>().map(|screen| screen.0))
    }
}

/// The [`WidgetsSetup`] that built the screen now on show, kept by
/// [`WidgetsSetup::build`] for [`AppSetup::capture`].
#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentScreen(pub WidgetsSetup);

/// How many times the "Add" button of the [`WidgetsSetup::Buttons`] screen
/// was clicked.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Clicks(pub u32);

/// The `ValueChange<f32>`s the `stepped slider` of the [`WidgetsSetup::Slider`]
/// screen sent: the value of the last one and how many were final (the end of
/// a drag or a key).
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SliderChanges {
    /// The value of the last change.
    pub last: f32,
    /// How many changes were final.
    pub finals: u32,
}

/// The root a screen of several widgets goes in: the camera and root of
/// [`spawn_camera_and_root`], laid out as a wrapping row.
fn row_root(world: &mut World) -> Entity {
    let root = spawn_camera_and_root(world);
    if let Some(mut node) = world.entity_mut(root).get_mut::<Node>() {
        node.flex_direction = FlexDirection::Row;
        node.flex_wrap = FlexWrap::Wrap;
        node.column_gap = px(20);
    }
    root
}

/// Spawns a "Change theme" button that toggles between the light and dark
/// theme.
fn spawn_theme_button(parent: &mut ChildSpawner) {
    parent
        .spawn(PushButton::from_label("Change theme"))
        .observe(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle());
}

/// The items the country selections draw from.
fn countries() -> [Item<String>; 5] {
    ["Netherlands", "Canada", "Indonesia", "Portugal", "Belgium"].map(Item::new)
}

/// The Tailwind colours as items.
fn color_items() -> Vec<Item<Color>> {
    COLORS.iter().map(|c| Item { data: c.color }).collect()
}

/// A bundle that lets the pointer reach what lies below it, for the contents
/// of a scroll area.
const fn see_through() -> Pickable {
    Pickable {
        should_block_lower: false,
        is_hoverable: true,
    }
}

fn buttons(world: &mut World) {
    world.init_resource::<Clicks>();
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn(PushButton::from_label("Add"))
            .observe(|_: On<Activate>, mut clicks: ResMut<Clicks>| clicks.0 += 1);
        root.spawn(PushButton::from_label("Other"));
        root.spawn((PushButton::from_label("Disabled"), InteractionDisabled));
        spawn_theme_button(root);
    });
}

fn combos(world: &mut World) {
    let root = spawn_camera_and_root(world);
    let selection_ref = world
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        )
        .id();
    world.entity_mut(root).with_children(|root| {
        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: px(20),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Combo,
                Name::new("first combo"),
                Selection::reference(selection_ref),
            ));
            row.spawn((
                Combo,
                Name::new("own combo"),
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(selection_ref),
            ));
            row.spawn((
                Combo,
                Name::new("third combo"),
                Selection::reference(selection_ref),
            ));
            spawn_theme_button(row);
        });
        // Below the combos, so the lists they open float over them. A button
        // takes the pointer from what is under it, so a list that is not above
        // it cannot be clicked.
        root.spawn(PushButton::from_label("Underneath"));
        root.spawn((
            List,
            Name::new("country list"),
            Selection::reference(selection_ref),
        ));
        // A combo in a scroll area lower than its open list, out of the flow
        // so that nothing else moves and last so that the tab order stays: the
        // list is not clipped by the area and takes clicks below it.
        root.spawn((
            Name::new("combo scroll area"),
            Node {
                position_type: PositionType::Absolute,
                top: px(300),
                left: px(1000),
                width: px(220),
                height: px(SMALL_SCROLL_AREA_HEIGHT),
                ..default()
            },
            ScrollArea::bundle(
                |contents| {
                    contents.spawn((
                        Combo,
                        Name::new("scrolled combo"),
                        Selection::builder()
                            .zero_or_one()
                            .build_with_items(countries()),
                    ));
                    // Makes the contents overflow the area, so it can scroll.
                    contents.spawn(Node {
                        width: px(1),
                        height: px(400),
                        flex_shrink: 0.0,
                        ..default()
                    });
                },
                true,
                false,
            ),
        ));
        // Combos at the right and bottom window edges, out of the flow and spawned last
        // for the same reason: their lists are kept inside the window.
        root.spawn((
            Combo,
            Name::new("right edge combo"),
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                top: px(100),
                left: px(1200),
                width: px(100),
                ..default()
            },
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        ));
        // A combo whose options carry a note, out of the flow and last like the others:
        // Delft is unreachable, Utrecht has a plain note.
        root.spawn((
            Combo,
            Name::new("noted combo"),
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                width: px(260),
                top: px(200),
                left: px(700),
                ..default()
            },
            Selection::builder().zero_or_one().build_with_items([
                // An empty note shows nothing.
                (Item::new("Amsterdam"), ItemNote::new("")),
                (Item::new("Delft"), ItemNote::warning("< unreachable >")),
                (Item::new("Utrecht"), ItemNote::new("4 km")),
            ]),
        ));
        root.spawn((
            Combo,
            Name::new("bottom edge combo"),
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                width: px(200),
                top: px(660),
                left: px(500),
                ..default()
            },
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        ));
    });
}

/// The height of the scroll areas that hold a combo on the [`WidgetsSetup::Combos`] and
/// [`WidgetsSetup::ColorPalette`] screens: lower than the combo's open popup.
const SMALL_SCROLL_AREA_HEIGHT: f32 = 100.0;

/// The countries the "Add country" button of the [`WidgetsSetup::Lists`]
/// screen adds, one per click, in this order and then again.
const MORE_COUNTRIES: [&str; 3] = ["Suriname", "Chile", "Kenya"];

/// The rows of the list in a scroll area on the [`WidgetsSetup::Lists`] screen.
const SCROLLING_LIST_ROWS: usize = 12;

fn lists(world: &mut World) {
    let root = row_root(world);
    let selection_ref = world
        .spawn(
            Selection::builder()
                .one_or_more()
                .build_with_items(countries()),
        )
        .id();
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new("one-or-more list"),
            List,
            Selection::reference(selection_ref),
        ));
        root.spawn((
            Name::new("one list"),
            List,
            ListLayout::Horizontal,
            Selection::builder()
                .one()
                .build_with_items(["NL", "CA", "ID", "PT", "BE"].map(Item::new)),
        ));
        root.spawn((
            Name::new("one-or-more grid list"),
            List,
            ListLayout::VerticalGrid { columns: 3 },
            Selection::reference(selection_ref),
        ));
        root.spawn((
            Name::new("zero-or-one list"),
            List,
            CornerStyle::Tabs,
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        ));
        root.spawn((
            Name::new("zero-or-more list"),
            List,
            ListLayout::Horizontal,
            CornerStyle::Tabs,
            Selection::builder()
                .zero_or_more()
                .build_with_items_ref(selection_ref),
        ));
        root.spawn((
            Name::new("zero-or-more grid list"),
            List,
            ListLayout::VerticalGrid { columns: 3 },
            CornerStyle::Tabs,
            Selection::builder()
                .zero_or_more()
                .build_with_items(countries()),
        ));
        root.spawn((
            Name::new("separated list"),
            List,
            CornerStyle::Separated,
            Selection::builder()
                .one()
                .build_with_items(["NL", "CA", "ID", "PT", "BE"].map(Item::new)),
        ));
        // A list taller than its scroll area, which the wheel scrolls: the
        // list itself does not scroll, the area around it does.
        root.spawn((
            Name::new("list scroll area"),
            Node {
                width: px(200),
                height: px(100),
                ..default()
            },
            ScrollArea::bundle(
                |parent: &mut ChildSpawner| {
                    parent.spawn((
                        Name::new("scrolling list"),
                        List,
                        Selection::builder().one().build_with_items(
                            (1..=SCROLLING_LIST_ROWS).map(|row| Item::new(format!("Row {row}"))),
                        ),
                    ));
                },
                true,
                false,
            ),
        ));
        root.spawn(PushButton::from_label("Add country")).observe(
            move |_: On<Activate>, mut commands: Commands, mut added: Local<usize>| {
                let Some(country) = MORE_COUNTRIES.get(*added % MORE_COUNTRIES.len()) else {
                    return;
                };
                *added += 1;
                // The lists drawing from the shared items show it.
                commands.spawn((Item::new(*country), ItemOf(selection_ref)));
            },
        );
        spawn_theme_button(root);
    });
}

/// The colours the squares in a scroll area cycle through.
const SQUARE_COLORS: [Srgba; 14] = [
    YELLOW_600,
    SKY_600,
    ROSE_600,
    PURPLE_600,
    PINK_600,
    ORANGE_600,
    LIME_600,
    INDIGO_600,
    FUCHSIA_600,
    EMERALD_600,
    CYAN_600,
    BLUE_600,
    GREEN_600,
    AMBER_600,
];

/// The contents of a scroll area: `num` coloured squares in a grid that fills
/// in the direction of `grid_auto_flow`.
fn squares(num: usize, grid_auto_flow: GridAutoFlow) -> impl FnOnce(&mut ChildSpawner) {
    move |parent| {
        parent
            .spawn((
                see_through(),
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![RepeatedGridTrack::auto(7)],
                    grid_template_rows: vec![RepeatedGridTrack::auto(7)],
                    grid_auto_flow,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::FlexStart,
                    row_gap: px(2),
                    column_gap: px(2),
                    ..default()
                },
            ))
            .with_children(|parent| {
                for i in 0..num {
                    parent.spawn((
                        Node {
                            width: px(50),
                            height: px(50),
                            ..default()
                        },
                        see_through(),
                        BackgroundColor(
                            SQUARE_COLORS
                                .get(i % SQUARE_COLORS.len())
                                .map_or(Color::NONE, |c| Color::Srgba(*c)),
                        ),
                    ));
                }
            });
    }
}

/// The size of every scroll area of the [`WidgetsSetup::ScrollAreas`] screen.
///
/// Four of them and "Change theme" fit side by side in the default 1280 by 720
/// window, so a scenario can reach each one without scrolling the page.
const SCROLL_AREA_SIZE: f32 = 200.0;

/// A scroll area of the [`SCROLL_AREA_SIZE`] named `name`.
fn sized_scroll_area(name: &'static str, area: impl Bundle) -> impl Bundle {
    (
        Name::new(name),
        Node {
            width: px(SCROLL_AREA_SIZE),
            height: px(SCROLL_AREA_SIZE),
            ..default()
        },
        area,
    )
}

fn scroll_areas(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn(sized_scroll_area(
            "two-way scroll area",
            ScrollArea::bundle(squares(300, GridAutoFlow::Row), true, true),
        ));
        root.spawn(sized_scroll_area(
            "vertical scroll area",
            ScrollArea::bundle(squares(50, GridAutoFlow::Row), true, false),
        ));
        root.spawn(sized_scroll_area(
            "horizontal scroll area",
            ScrollArea::bundle(squares(200, GridAutoFlow::Column), false, true),
        ));
        // Three squares in a row are smaller than the area, so nothing scrolls.
        root.spawn(sized_scroll_area(
            "fitting scroll area",
            ScrollArea::bundle(squares(3, GridAutoFlow::Row), true, true),
        ));
        spawn_theme_button(root);
    });
}

/// Marks the entity holding the breadcrumb trail's selection.
#[derive(Component, Copy, Clone, Debug)]
struct BreadCrumbsSelection;

/// Which panel an item of the breadcrumb trail stands for.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
enum BreadCrumbsAction {
    Home,
    PanelA,
    PanelAvariant,
    PanelB,
    PanelBsub,
}

/// Adds the panel `target` to the end of the trail.
fn push_to_selection(
    target: BreadCrumbsAction,
    selections: &mut SelectionParam<With<BreadCrumbsSelection>>,
    items: &ItemParam<String, &BreadCrumbsAction>,
) {
    let Some(item_ref) = items.find_item_ref(|_, action| **action == target) else {
        return;
    };
    selections.update_selection_single(|selection| selection.push(item_ref));
}

/// Takes the last panel off the trail.
fn pop_on_activate(_: On<Activate>, mut selections: SelectionParam<With<BreadCrumbsSelection>>) {
    selections.update_selection_single(SelectionValue::pop);
}

/// Spawns a button that adds the panel `target` to the trail when released.
fn spawn_go_to(parent: &mut ChildSpawner, label: &'static str, target: BreadCrumbsAction) {
    parent.spawn(PushButton::from_label(label)).observe(
        move |_: On<Activate>,
              mut selections: SelectionParam<With<BreadCrumbsSelection>>,
              items: ItemParam<String, &BreadCrumbsAction>| {
            push_to_selection(target, &mut selections, &items);
        },
    );
}

/// Spawns a button that takes the last panel off the trail when released.
fn spawn_go_back(parent: &mut ChildSpawner, label: &'static str) {
    parent
        .spawn(PushButton::from_label(label))
        .observe(pop_on_activate);
}

fn create_home(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Home"));
                spawn_go_to(parent, "Go to Panel A", BreadCrumbsAction::PanelA);
                spawn_go_to(
                    parent,
                    "Go to Panel A variant",
                    BreadCrumbsAction::PanelAvariant,
                );
                spawn_go_to(parent, "Go to Panel B", BreadCrumbsAction::PanelB);
            })),
        ),
    );
}

fn create_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Panel A"));
                spawn_go_back(parent, "Go back to home");
                // Replaces Panel A by its variant instead of adding to it.
                parent
                    .spawn(PushButton::from_label("Go to panel A variant"))
                    .observe(
                        |_: On<Activate>,
                         mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                         items: ItemParam<String, &BreadCrumbsAction>| {
                            let Some(variant) = items.find_item_ref(|_, action| {
                                **action == BreadCrumbsAction::PanelAvariant
                            }) else {
                                return;
                            };
                            selections.update_selection_single(|x| x.pop()?.push(variant));
                        },
                    );
            })),
        ),
    );
}

fn create_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Panel B"));
                spawn_go_back(parent, "Go back to home");
                spawn_go_to(parent, "Go to panel B sub", BreadCrumbsAction::PanelBsub);
            })),
        ),
    );
}

fn create_b_sub(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Panel B sub"));
                spawn_go_back(parent, "Go back to panel B");
            })),
        ),
    );
}

fn bread_crumbs(world: &mut World) {
    let items = [
        (
            BreadCrumbsAction::Home,
            Item::new("Home"),
            StackItem::new(world.register_system(create_home)),
        ),
        (
            BreadCrumbsAction::PanelA,
            Item::new("Panel A"),
            StackItem::new_with_reference(0, world.register_system(create_a)),
        ),
        (
            BreadCrumbsAction::PanelAvariant,
            Item::new("Panel A variant"),
            StackItem::new_reference(0),
        ),
        (
            BreadCrumbsAction::PanelB,
            Item::new("Panel B"),
            StackItem::new(world.register_system(create_b)),
        ),
        (
            BreadCrumbsAction::PanelBsub,
            Item::new("Panel B sub"),
            StackItem::new(world.register_system(create_b_sub)),
        ),
    ];
    let selection_ref = world
        .spawn((
            BreadCrumbsSelection,
            Selection::builder().one_or_more().build_with_items(items),
        ))
        .id();
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Node {
                width: percent(100),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                parent.spawn((BreadCrumbs, Selection::reference(selection_ref)));
                spawn_theme_button(parent);
            })),
        ));
        root.spawn((
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                ..default()
            },
            children![Stack { selection_ref }],
        ));
    });
}

fn create_tab_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent.spawn(Label::regular("Panel A"));
            let colors = parent
                .spawn(ItemValues::spawn(SpawnIter(color_items().into_iter())))
                .id();
            let selection_ref = parent
                .spawn(
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items_ref(colors),
                )
                .id();
            parent.spawn((
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(selection_ref),
            ));
            parent.spawn((
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(colors),
            ));
            parent.spawn((
                Card,
                Node {
                    height: px(200),
                    margin: UiRect::all(px(5)),
                    padding: UiRect::all(px(10)),
                    align_items: AlignItems::FlexStart,
                    row_gap: px(10),
                    column_gap: px(10),
                    ..default()
                },
                children![
                    (
                        Combo,
                        Selection::builder()
                            .zero_or_one()
                            .build_with_items(countries()),
                    ),
                    PushButton::from_label("Button on a card")
                ],
            ));
            parent.spawn(PushButton::from_label("hello"));
        })),
    );
}

fn create_tab_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        children![
            Label::header("Panel B"),
            ScrollArea::bundle(
                |parent| {
                    parent.spawn((
                        List,
                        Selection::builder()
                            .one()
                            .build_with_items((1..100).map(|i| Item::new(i.to_string()))),
                    ));
                },
                true,
                false,
            ),
            (
                Node {
                    align_self: AlignSelf::Start,
                    ..default()
                },
                children![(
                    List,
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items(countries()),
                )]
            ),
        ],
    );
}

fn create_tab_cards(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children![
                Label::header("Card demo"),
                (
                    Node {
                        column_gap: px(16),
                        row_gap: px(16),
                        flex_wrap: FlexWrap::Wrap,
                        margin: UiRect::all(px(8)),
                        ..default()
                    },
                    Children::spawn(SpawnIter((0..12).map(|i| {
                        (
                            Node {
                                width: px(100),
                                height: px(100),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Card,
                            children![Label::regular(format!("{i}"))],
                        )
                    })))
                ),
            ],
        ),
    );
}

fn tabs(world: &mut World) {
    let items = [
        (
            Item::new("Panel A"),
            StackItem::new(world.register_system(create_tab_a)),
        ),
        (
            Item::new("Panel B"),
            StackItem::new(world.register_system(create_tab_b)),
        ),
        (
            Item::new("Card demo"),
            StackItem::new(world.register_system(create_tab_cards)),
        ),
    ];
    let border = world
        .get_resource::<Theme>()
        .map_or(Color::NONE, |theme| theme.colors.border);
    let root = spawn_camera_and_root(world);
    // The tab bar, a filler and "Change theme" on top, the stack below.
    world.entity_mut(root).insert(Node {
        display: Display::Grid,
        grid_template_columns: vec![
            GridTrack::max_content(),
            GridTrack::auto(),
            GridTrack::max_content(),
        ],
        grid_template_rows: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
        width: percent(100),
        height: percent(100),
        ..default()
    });
    world.entity_mut(root).with_children(|root| {
        let selection_ref = root
            .spawn((TabBar, Selection::builder().one().build_with_items(items)))
            .id();
        root.spawn((
            BorderColor::all(border),
            Node {
                max_width: percent(100),
                border: UiRect::bottom(px(1)),
                ..default()
            },
        ));
        spawn_theme_button(root);
        root.spawn((
            Node {
                grid_column: GridPlacement::span(3),
                align_items: AlignItems::FlexStart,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Stack { selection_ref },
        ));
    });
}

fn color_palette(world: &mut World) {
    let root = row_root(world);
    if let Some(mut node) = world.entity_mut(root).get_mut::<Node>() {
        // Keep the rows at the top instead of stretching them over the window,
        // so where a popup opens does not depend on how much room is left.
        node.align_content = AlignContent::FlexStart;
    }
    let selection_ref = world
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(color_items()),
        )
        .id();
    let combo_selection_ref = world
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        )
        .id();
    world.entity_mut(root).with_children(|root| {
        // The combos come first so that their popups open below them, over the
        // palettes. The row is as wide as the window, which puts the palettes
        // on the next line.
        root.spawn(Node {
            width: percent(100),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::Center,
            column_gap: px(20),
            ..default()
        })
        .with_children(|controls| {
            controls.spawn((
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(combo_selection_ref),
                Name::new("first palette combo"),
            ));
            controls.spawn((
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(combo_selection_ref),
                Name::new("second palette combo"),
            ));
            controls.spawn((
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(selection_ref),
                Name::new("own palette combo"),
            ));
            spawn_theme_button(controls);
        });
        // A combo whose selection is always one colour (the first until another is
        // picked), so that picking the colour it already has changes nothing. Out of
        // the flow, left of the controls, so that nothing else on the screen moves.
        root.spawn((
            ColorPaletteCombo,
            Node {
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                top: px(20),
                left: px(300),
                ..default()
            },
            ListLayout::VerticalGrid { columns: 11 },
            Selection::builder()
                .one()
                .build_with_items_ref(selection_ref),
            Name::new("one palette combo"),
        ));
        // One combo in each top corner, placed out of the flow so that nothing
        // else on the screen moves: the popups are wider than the room these
        // have to the window edge, so the window margin is what places them.
        for (name, left, right) in [
            ("left edge palette combo", Some(px(0)), None),
            ("right edge palette combo", None, Some(px(0))),
        ] {
            root.spawn((
                ColorPaletteCombo,
                Node {
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    top: px(20),
                    left: left.unwrap_or(Val::Auto),
                    right: right.unwrap_or(Val::Auto),
                    ..default()
                },
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(combo_selection_ref),
                Name::new(name),
            ));
        }
        // 22 columns of 11 rows keep the whole palette (836 by 418) inside the
        // window; 11 columns of 22 rows, as in the example, are taller than it.
        root.spawn((
            ColorPalette,
            ListLayout::VerticalGrid { columns: 22 },
            Selection::reference(selection_ref),
            Name::new("palette"),
        ));
        root.spawn((
            Node {
                width: px(300),
                height: px(300),
                ..default()
            },
            Name::new("palette scroll area"),
            ScrollArea::bundle(
                move |parent| {
                    parent.spawn((
                        see_through(),
                        ColorPalette,
                        ListLayout::VerticalGrid { columns: 11 },
                        Selection::reference(selection_ref),
                    ));
                },
                true,
                true,
            ),
        ));
        // A combo in a scroll area lower than its open popup, under the palette
        // scroll area, out of the flow so that nothing else moves and last so
        // that the tab order stays: the popup is not clipped by the area and
        // takes clicks below it.
        root.spawn((
            Name::new("palette combo scroll area"),
            Node {
                position_type: PositionType::Absolute,
                top: px(400),
                left: px(900),
                width: px(200),
                height: px(SMALL_SCROLL_AREA_HEIGHT),
                ..default()
            },
            ScrollArea::bundle(
                move |contents| {
                    contents.spawn((
                        ColorPaletteCombo,
                        ListLayout::VerticalGrid { columns: 11 },
                        Selection::builder()
                            .zero_or_one()
                            .build_with_items_ref(selection_ref),
                        Name::new("scrolled palette combo"),
                    ));
                },
                true,
                false,
            ),
        ));
    });
}

fn numeric_stepper(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((NumericStepper::default(), Name::new("default stepper")));
        root.spawn((
            NumericStepper {
                name: Some("Cars".into()),
                ..default()
            },
            Value::new_u16(10, 5, 15, 1),
            Name::new("u16 stepper"),
        ));
        root.spawn((
            NumericStepper::default(),
            Value::new_u32(10, 5, 15, 1),
            Name::new("u32 stepper"),
        ));
        root.spawn((
            NumericStepper::default(),
            Value::new_f32(10.0, 5.0, 15.0, 0.5),
            Name::new("f32 stepper"),
        ));
        // A step that does not divide the range: from 7 one more step would
        // pass the maximum of 8, and from 1 one fewer the minimum of 0.
        root.spawn((
            NumericStepper::default(),
            Value::new_u8(5, 0, 8, 2),
            Name::new("stepper by two"),
        ));
        // One value shown by three steppers: the first holds it, the other two
        // point to it through `ValueRef`.
        let shared = root
            .spawn((
                NumericStepper::default(),
                Value::new_u32(3, 0, 6, 1),
                Name::new("shared stepper"),
            ))
            .id();
        root.spawn((
            NumericStepper::default(),
            ValueRef(shared),
            Name::new("first mirror stepper"),
        ));
        root.spawn((
            NumericStepper::default(),
            ValueRef(shared),
            Name::new("second mirror stepper"),
        ));
        spawn_theme_button(root);
    });
}

fn slider(world: &mut World) {
    world.init_resource::<SliderChanges>();
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Slider::new(0.0, 0.0, 1.0).with_name("Continuous"),
            Node {
                width: px(320),
                ..default()
            },
            Name::new("continuous slider"),
        ));
        root.spawn((
            Slider::new(0.0, 0.0, 10.0)
                .with_step(1.0)
                .with_name("Stepped"),
            Name::new("stepped slider"),
        ))
        .observe(
            |change: On<ValueChange<f32>>, mut changes: ResMut<SliderChanges>| {
                changes.last = change.value;
                changes.finals += u32::from(change.is_final);
            },
        );
        root.spawn((
            Slider::new(25.0, 0.0, 100.0)
                .with_step(5.0)
                .with_format(|value| format!("{value:.0} %"))
                .with_name("Speed"),
            Name::new("formatted slider"),
        ));
        root.spawn((
            Slider::new(3.0, 0.0, 10.0)
                .with_step(1.0)
                .with_format(|value| format!("{value:.0}")),
            InteractionDisabled,
            Name::new("disabled slider"),
        ));
        spawn_theme_button(root);
    });
}

fn segmented_controls(world: &mut World) {
    let root = row_root(world);
    let shared = world
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["Dark", "Light"].map(Item::new)),
        )
        .id();
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["Auto", "Manual"].map(Item::new)),
            Name::new("own segmented control"),
        ));
        root.spawn((
            SegmentedControl,
            Selection::reference(shared),
            Name::new("first shared segmented control"),
        ));
        root.spawn((
            SegmentedControl,
            Selection::reference(shared),
            Name::new("second shared segmented control"),
        ));
        spawn_theme_button(root);
    });
}

fn filter_chips(world: &mut World) {
    let root = row_root(world);
    let chips = [
        Chip::with_count("All", 4),
        Chip::with_count("Running", 2),
        Chip::with_count("Waiting", 1),
        Chip::with_count("Problems", 1),
    ];
    let status = world
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(chips.map(|data| Item { data })),
        )
        .id();
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            FilterChips,
            Selection::reference(status),
            Name::new("status chips"),
        ));
        root.spawn((
            FilterChips,
            Selection::reference(status),
            Name::new("mirror chips"),
        ));
        root.spawn((
            FilterChips,
            Node {
                width: Val::Px(150.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                ..default()
            },
            Selection::builder()
                .one()
                .build_with_items(["Alpha", "Beta", "Gamma", "Delta"].map(|label| Item {
                    data: Chip::new(label),
                })),
            Name::new("narrow chips"),
        ));
        spawn_theme_button(root);
    });
}

fn gallery(world: &mut World) {
    let root = row_root(world);
    let cards = [
        ("Steam", "Locomotives"),
        ("Diesel", "Locomotives"),
        ("Passenger", "Wagons"),
    ]
    .map(|(label, group)| Item {
        data: GalleryCard::new(Handle::default(), label, group),
    });
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Gallery,
            Selection::builder().one().build_with_items(cards),
            Name::new("gallery"),
        ));
        spawn_theme_button(root);
    });
}

fn toolbar(world: &mut World) {
    let root = row_root(world);
    let tools = ["Select", "Build", "Bulldoze"].map(|label| Item {
        data: Tool::new(Handle::default(), label),
    });
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Toolbar::new(ToolbarStyle::Filled),
            Selection::builder().one().build_with_items(tools),
            Name::new("filled toolbar"),
        ));
        spawn_theme_button(root);
    });
}

fn panel(world: &mut World) {
    let root = row_root(world);
    let top = Node {
        align_self: AlignSelf::FlexStart,
        ..default()
    };
    let closable = world
        .spawn((
            Panel::new("Trains")
                .with_subtitle("3 open")
                .with_close_icon(Handle::default()),
            Name::new("closable panel"),
            top.clone(),
            ChildOf(root),
        ))
        .observe(|closed: On<PanelClosed>, mut commands: Commands| {
            commands.entity(closed.entity).despawn();
        })
        .id();
    let tall = world
        .spawn((
            Panel::new("Stops")
                .with_close_icon(Handle::default())
                .with_max_body_height(100.0),
            Name::new("tall panel"),
            top,
            ChildOf(root),
        ))
        .id();
    world.entity_mut(root).with_children(spawn_theme_button);
    world.flush();
    for (panel, lines) in [(closable, 3), (tall, 12)] {
        let Some(body) = world.get::<PanelRefs>(panel).map(|refs| refs.body) else {
            continue;
        };
        for line in 1..=lines {
            world.spawn((Label::regular(format!("Line {line}")), ChildOf(body)));
        }
    }
}

fn setting_rows(world: &mut World) {
    let root = row_root(world);
    let rows = world
        .spawn((
            SettingRows,
            Name::new("setting rows"),
            Node {
                width: px(400),
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    let mut slots = Vec::new();
    for (name, title, description) in [
        ("appearance row", "Appearance", "Light or dark"),
        ("text size row", "Text size", "Points"),
        ("last row", "Reset", "Back to the defaults"),
    ] {
        let row = world
            .spawn((
                SettingRow::new(title, description),
                Name::new(name),
                ChildOf(rows),
            ))
            .id();
        slots.push(row);
    }
    world.flush();
    let slot =
        |world: &World, row: Entity| world.get::<SettingRowRefs>(row).map(|r| r.control_slot);
    if let Some(slot) = slots.first().and_then(|row| slot(world, *row)) {
        world.spawn((
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["Light", "Dark"].map(Item::new)),
            ChildOf(slot),
        ));
    }
    if let Some(slot) = slots.get(1).and_then(|row| slot(world, *row)) {
        world.spawn((
            NumericStepper::default(),
            Value::new_u8(12, 8, 20, 1),
            ChildOf(slot),
        ));
    }
    if let Some(slot) = slots.get(2).and_then(|row| slot(world, *row)) {
        world.spawn((PushButton::from_label("Reset"), ChildOf(slot)));
    }
    world.entity_mut(root).with_children(spawn_theme_button);
}

/// Marks the button whose badge "Add badge" raises.
#[derive(Component)]
struct CountedButton;

fn badge(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            IconButton::new(Handle::default(), "Trains", IconButtonSize::Large),
            Badge::count(3),
            CountedButton,
            Name::new("counted button"),
        ));
        root.spawn((
            IconButton::new(Handle::default(), "Warnings", IconButtonSize::Large),
            Badge::count(12).with_tone(BadgeTone::Warning),
            Name::new("warning button"),
        ));
        root.spawn((
            IconButton::new(Handle::default(), "Stops", IconButtonSize::Large),
            Name::new("plain button"),
        ));
        root.spawn(PushButton::from_label("Add badge")).observe(
            |_: On<Activate>,
             mut commands: Commands,
             badges: Query<(Entity, &Badge), With<CountedButton>>| {
                for (entity, badge) in &badges {
                    let count = badge.text.parse::<usize>().unwrap_or(0);
                    commands.entity(entity).insert(Badge {
                        text: (count + 1).to_string(),
                        ..badge.clone()
                    });
                }
            },
        );
        spawn_theme_button(root);
    });
}

fn icon_buttons(world: &mut World) {
    world.init_resource::<Clicks>();
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            IconButton::new(Handle::default(), "Small", IconButtonSize::Small),
            Name::new("small button"),
        ));
        root.spawn((
            IconButton::new(Handle::default(), "Regular", IconButtonSize::Regular),
            Name::new("regular button"),
        ))
        .observe(|_: On<Activate>, mut count: ResMut<Clicks>| count.0 += 1);
        root.spawn((
            IconButton::new(Handle::default(), "Large", IconButtonSize::Large),
            Name::new("large button"),
        ));
        root.spawn((
            IconButton::new(Handle::default(), "Toggle", IconButtonSize::Regular),
            ToggleState(false),
            Name::new("toggle button"),
        ));
        root.spawn((
            IconButton::new(Handle::default(), "Warning", IconButtonSize::Large)
                .with_tone(Tone::Warning),
            Name::new("warning button"),
        ));
        root.spawn((
            IconButton::new(Handle::default(), "Danger", IconButtonSize::Regular)
                .with_tone(Tone::Danger),
            Name::new("danger button"),
        ));
        spawn_theme_button(root);
    });
}

fn tooltip(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            PushButton::from_label("Anchor"),
            Tooltip::new("Anchor tip"),
            Name::new("tooltip anchor"),
        ));
        root.spawn(IconButton::new(
            Handle::default(),
            "Help",
            IconButtonSize::Large,
        ))
        .insert(Name::new("help button"));
        spawn_theme_button(root);
    });
}

fn toast(world: &mut World) {
    let root = row_root(world);
    let raise = |text: &'static str| {
        move |_: On<Activate>, mut commands: Commands, roots: Query<Entity, With<UiRoot>>| {
            for root in &roots {
                commands.spawn((Toast::new(text).with_duration(1.0), ChildOf(root)));
            }
        }
    };
    world.entity_mut(root).with_children(|root| {
        root.spawn(PushButton::from_label("Save"))
            .observe(raise("Saved"));
        root.spawn(PushButton::from_label("Save again"))
            .observe(raise("Saved"));
        root.spawn(PushButton::from_label("Delete"))
            .observe(raise("Deleted 3 pieces"));
        spawn_theme_button(root);
    });
}

/// What the fields of the [`WidgetsSetup::TextFields`] screen triggered: how many
/// `TextFieldChange`s and `TextFieldCommit`s, and the text of the last of each.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct FieldEvents {
    /// The number of `TextFieldChange`s.
    pub changes: u32,
    /// The text of the last `TextFieldChange`.
    pub changed: String,
    /// The number of `TextFieldCommit`s.
    pub commits: u32,
    /// The text of the last `TextFieldCommit`.
    pub committed: String,
}

fn count_change(change: On<TextFieldChange>, mut events: ResMut<FieldEvents>) {
    events.changes += 1;
    events.changed.clone_from(&change.value);
}

fn count_commit(commit: On<TextFieldCommit>, mut events: ResMut<FieldEvents>) {
    events.commits += 1;
    events.committed.clone_from(&commit.value);
}

fn text_fields(world: &mut World) {
    world.insert_resource(FieldEvents::default());
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            TextField::new("Alpha").with_name("Name"),
            Name::new("name field"),
        ))
        .observe(count_change)
        .observe(count_commit);
        root.spawn((
            TextField::new("")
                .with_placeholder("Seed")
                .with_name("Seed"),
            Name::new("seed field"),
        ))
        .observe(count_change)
        .observe(count_commit);
        root.spawn(PushButton::from_label("Other"));
        spawn_theme_button(root);
    });
}

fn button_names(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((Name::new("plain button"), PushButton::from_label("Add")));
        root.spawn((Name::new("icon push button"), PushButton::new("Save", "+")));
        root.spawn((Name::new("colour button"), ColorPushButton));
        root.spawn((Name::new("link"), Link::new("Docs")));
        spawn_theme_button(root);
    });
    world.flush();
    let button = world
        .query_filtered::<&ColorSquareRef, With<ColorPushButton>>()
        .iter(world)
        .next()
        .map(|square| square.0);
    if let Some(mut color) = button.and_then(|square| world.get_mut::<BackgroundColor>(square)) {
        color.0 = RED_500.into();
    }
}

fn themed(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((Name::new("themed button"), PushButton::from_label("Other")));
        root.spawn((
            Name::new("themed card"),
            Card,
            Node {
                width: px(200),
                height: px(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![Label::regular("A card")],
        ));
        root.spawn((
            Name::new("themed list"),
            List,
            Selection::builder()
                .one()
                .build_with_items(["One", "Two", "Three"].map(Item::new)),
        ));
        spawn_theme_button(root);
    });
}

/// How many rows the `virtual list` of the [`WidgetsSetup::VirtualList`] screen has.
const VIRTUAL_LIST_ROWS: usize = 10_000;

fn virtual_list(world: &mut World, separated: bool) {
    let root = row_root(world);
    let (prefix, list) = if separated {
        ("separated ", VirtualList::separated(44.0))
    } else {
        ("", VirtualList::new(28.0))
    };
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new(format!("{prefix}virtual list scroll area")),
            Node {
                width: px(240),
                height: px(200),
                ..default()
            },
            ScrollArea::bundle(
                move |parent: &mut ChildSpawner| {
                    parent.spawn((
                        Name::new(format!("{prefix}virtual list")),
                        list,
                        Selection::builder().one().build_with_items(
                            (1..=VIRTUAL_LIST_ROWS).map(|row| Item::new(format!("Row {row}"))),
                        ),
                    ));
                },
                true,
                false,
            ),
        ));
        spawn_theme_button(root);
    });
}

/// The creator of the rows of [`WidgetsSetup::ListWithCombos`]: a combo named after its row.
fn row_with_combo(
    spawner: In<ItemViewSpawner>,
    items: Query<&Item<String>>,
    mut commands: Commands,
) {
    let Ok(item) = items.get(spawner.item) else {
        return;
    };
    let name = format!("combo in row {}", item.data);
    commands.entity(spawner.view).with_children(|view| {
        view.spawn((
            Combo,
            Name::new(name),
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        ));
    });
}

fn list_with_combos(world: &mut World) {
    let create = world.register_system(row_with_combo);
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new("row list"),
            List,
            ItemViewCreator::new(create),
            Selection::builder()
                .one()
                .build_with_items(["a", "b", "c"].map(Item::new)),
        ));
        spawn_theme_button(root);
    });
}
