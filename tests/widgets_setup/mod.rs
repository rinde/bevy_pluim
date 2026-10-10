//! `WidgetsSetup`: the screens scenario files start from.
//!
//! The tester finds this one `#[reflect(AppSetup)]` enum by itself, so a
//! scenario file writes `setup: App(Buttons)` and a Rust scenario
//! `.app_setup(WidgetsSetup::Buttons)`. Each variant spawns a `Camera2d`, the
//! one root `Node` and a screen of the widget it is named after, following the
//! example of that name. The examples are not built on this module: they spawn
//! through `Commands` in a startup system, while a setup is handed the
//! `World`.
//!
//! Pull it in with `mod widgets_setup;` next to `mod common;`; it uses
//! [`common::spawn_camera_and_root`].

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private test module"
)]

use bevy::color::palettes::tailwind::{
    AMBER_600, BLUE_600, CYAN_600, EMERALD_600, FUCHSIA_600, GREEN_600, INDIGO_600, LIME_600,
    ORANGE_600, PINK_600, PURPLE_600, ROSE_600, SKY_600, YELLOW_600,
};
use bevy::ecs::spawn::{SpawnIter, SpawnWith};
use bevy::prelude::*;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::list::{CornerStyle, List, ListLayout};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value, ValueRef};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, ItemParam, ItemValues, Selection, SelectionParam, SelectionValue};
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::tailwind::COLORS;
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui_tester::{AppSetup, ReflectAppSetup};

use crate::common::spawn_camera_and_root;

/// The screens a scenario can start from, one per widget.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(AppSetup)]
pub enum WidgetsSetup {
    /// "Add", "Other" and "Change theme" push buttons; the clicks on "Add"
    /// are counted in [`Clicks`].
    Buttons,
    /// Three combos over one list of countries, two of them sharing one
    /// selection, and "Change theme".
    Combos,
    /// Lists in several layouts and corner styles, some sharing a selection,
    /// and "Change theme".
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
    /// Colour palettes and palette combos over the Tailwind colours, and
    /// "Change theme".
    ColorPalette,
    /// Numeric steppers, each named after its type or role (`default
    /// stepper`, `u16 stepper`, `u32 stepper`, `f32 stepper`, `stepper by
    /// two`), three of them showing one value (`shared stepper` holds it,
    /// `first mirror stepper` and `second mirror stepper` point to it), and
    /// "Change theme".
    NumericStepper,
}

impl AppSetup for WidgetsSetup {
    fn build(&self, world: &mut World) -> Result<(), String> {
        match self {
            Self::Buttons => buttons(world),
            Self::Combos => combos(world),
            Self::Lists => lists(world),
            Self::ScrollAreas => scroll_areas(world),
            Self::BreadCrumbs => bread_crumbs(world),
            Self::Tabs => tabs(world),
            Self::ColorPalette => color_palette(world),
            Self::NumericStepper => numeric_stepper(world),
        }
        Ok(())
    }
}

/// How many times the "Add" button of the [`WidgetsSetup::Buttons`] screen
/// was clicked.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Clicks(pub u32);

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
        .observe(|_: On<Pointer<Release>>, mut theme: ResMut<Theme>| theme.light_dark_toggle());
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
            .observe(|_: On<Pointer<Click>>, mut clicks: ResMut<Clicks>| clicks.0 += 1);
        root.spawn(PushButton::from_label("Other"));
        spawn_theme_button(root);
    });
}

fn combos(world: &mut World) {
    let root = row_root(world);
    let selection_ref = world
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        )
        .id();
    world.entity_mut(root).with_children(|root| {
        root.spawn((Combo, Selection::reference(selection_ref)));
        root.spawn((
            Combo,
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        ));
        root.spawn((Combo, Selection::reference(selection_ref)));
        spawn_theme_button(root);
    });
}

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
        root.spawn((List, Selection::reference(selection_ref)));
        root.spawn((
            List,
            ListLayout::Horizontal,
            Selection::builder()
                .one()
                .build_with_items(["NL", "CA", "ID", "PT", "BE"].map(Item::new)),
        ));
        root.spawn((
            List,
            ListLayout::VerticalGrid { columns: 3 },
            Selection::reference(selection_ref),
        ));
        root.spawn((
            List,
            CornerStyle::Tabs,
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        ));
        root.spawn((
            List,
            ListLayout::Horizontal,
            CornerStyle::Tabs,
            Selection::builder()
                .zero_or_more()
                .build_with_items_ref(selection_ref),
        ));
        root.spawn((
            List,
            ListLayout::VerticalGrid { columns: 3 },
            CornerStyle::Tabs,
            Selection::builder()
                .zero_or_more()
                .build_with_items(countries()),
        ));
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
fn pop_on_pointer_release(
    _: On<Pointer<Release>>,
    mut selections: SelectionParam<With<BreadCrumbsSelection>>,
) {
    selections.update_selection_single(SelectionValue::pop);
}

/// Spawns a button that adds the panel `target` to the trail when released.
fn spawn_go_to(parent: &mut ChildSpawner, label: &'static str, target: BreadCrumbsAction) {
    parent.spawn(PushButton::from_label(label)).observe(
        move |_: On<Pointer<Release>>,
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
        .observe(pop_on_pointer_release);
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
                        |_: On<Pointer<Release>>,
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
            .spawn((
                List,
                CornerStyle::Tabs,
                ListLayout::Horizontal,
                Selection::builder().one().build_with_items(items),
            ))
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
        root.spawn((
            ColorPalette,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::reference(selection_ref),
        ));
        root.spawn((
            Node {
                width: px(430),
                height: px(300),
                ..default()
            },
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
        root.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::reference(combo_selection_ref),
        ));
        root.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        ));
        spawn_theme_button(root);
    });
}

fn numeric_stepper(world: &mut World) {
    let root = row_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((NumericStepper, Name::new("default stepper")));
        root.spawn((
            NumericStepper,
            Value::new_u16(10, 5, 15, 1),
            Name::new("u16 stepper"),
        ));
        root.spawn((
            NumericStepper,
            Value::new_u32(10, 5, 15, 1),
            Name::new("u32 stepper"),
        ));
        root.spawn((
            NumericStepper,
            Value::new_f32(10.0, 5.0, 15.0, 0.5),
            Name::new("f32 stepper"),
        ));
        // A step that does not divide the range: from 7 one more step would
        // pass the maximum of 8, and from 1 one fewer the minimum of 0.
        root.spawn((
            NumericStepper,
            Value::new_u8(5, 0, 8, 2),
            Name::new("stepper by two"),
        ));
        // One value shown by three steppers: the first holds it, the other two
        // point to it through `ValueRef`.
        let shared = root
            .spawn((
                NumericStepper,
                Value::new_u32(3, 0, 6, 1),
                Name::new("shared stepper"),
            ))
            .id();
        root.spawn((
            NumericStepper,
            ValueRef(shared),
            Name::new("first mirror stepper"),
        ));
        root.spawn((
            NumericStepper,
            ValueRef(shared),
            Name::new("second mirror stepper"),
        ));
        spawn_theme_button(root);
    });
}
