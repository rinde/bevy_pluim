//! A `TabBar` driving a `Stack`, with combos, colour palettes and cards inside.

use std::sync::LazyLock;

use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::card::Card;
use bevy_pluim::color_palette_combo::ColorPaletteCombo;
use bevy_pluim::combo::Combo;
use bevy_pluim::label::Label;
use bevy_pluim::list::{List, ListLayout};
use bevy_pluim::push_button::PushButton;
use bevy_pluim::scroll_area::ScrollArea;
use bevy_pluim::select::{Item, ItemValues, Selection};
use bevy_pluim::stack::{Stack, StackChildSpawner, StackItem};
use bevy_pluim::tab_bar::TabBar;
use bevy_pluim::tailwind::{COLORS, TailwindColor, Variant};
use bevy_pluim::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .add_systems(Update, update_theme.run_if(resource_changed::<Theme>))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2d);

    let theme_border_color = theme.colors().border;

    let items = [
        (
            Item::new("Panel A"),
            StackItem::new(commands.register_system(create_a)),
        ),
        (
            Item::new("Panel B"),
            StackItem::new(commands.register_system(create_b)),
        ),
        (
            Item::new("Card demo"),
            StackItem::new(commands.register_system(create_card_demo)),
        ),
    ];
    // Root container
    commands.spawn_scene(bsn! {
        Node {
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::max_content(),
                GridTrack::auto(),
                GridTrack::max_content(),
            ],
            grid_template_rows: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
            width: percent(100.0),
            height: percent(100.0),
        }
        Children [
            TabBar
            #tabs
            @{Selection::builder().one().scene_with_items(items)}
            --
            // filler
            BorderColor::all(theme_border_color)
            Node {
                max_width: percent(100.0),
                border: UiRect::bottom(px(1.0)),
            }
            --
            ~{PushButton::from_label("Toggle theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
            --
            Node {
                grid_column: GridPlacement::span(3),
                display: Display::Flex,
                align_items: AlignItems::FlexStart,
                width: percent(100.0),
                height: percent(100.0),
            }
            Stack { selection_ref: #tabs }
        ]
    });
}

fn create_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let colors = commands
        .spawn(ItemValues::spawn(SpawnIter(color_palette().into_iter())))
        .id();
    let selection_ref = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(colors),
        )
        .id();

    spawner.spawn_scene(
        &mut commands,
        bsn! {
        Children [
            ~{Label::regular("Panel A")}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref)}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            @{Selection::builder().zero_or_one().scene_with_items_ref(colors)}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref)}
            --
            ~{Card}
            Node {
                height: px(200.0),
                margin: UiRect::all(px(5.0)),
                padding: UiRect::all(px(10.0)),
                align_items: AlignItems::FlexStart,
                row_gap: px(10.0),
                column_gap: px(10.0),
            }
            Children [
                ~{Combo}
                @{Selection::builder().zero_or_one().scene_with_items(countries())}
                --
                ~{PushButton::from_label("Button on a card")}
            ]
            --
            ~{PushButton::from_label("hello")}
        ]
        },
    );
}

fn create_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
        Children [
            ~{Label::header("Panel B")}
            --
            @item_scroll_area()
            --
            Node {
                align_self: AlignSelf::Start,
            }
            Children [
                List
                @{Selection::builder().zero_or_one().scene_with_items(countries())}
            ]
        ]
        },
    );
}

static LIGHT_COLORS: LazyLock<Vec<TailwindColor>> = LazyLock::new(|| {
    COLORS
        .iter()
        .filter(|x| x.variant == Variant::V200)
        .copied()
        .collect::<Vec<_>>()
});

fn create_card_demo(spawner: In<StackChildSpawner>, theme: Res<Theme>, mut commands: Commands) {
    let background_color = theme.colors().background_primary;
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Grid,
                grid_auto_columns: vec![GridTrack::percent(100.0)],
                grid_auto_flow: GridAutoFlow::Row,
            }
            Children [
                ~{Label::header("Card demo")}
                --
                Node
                Pickable {
                    should_block_lower: false,
                }
                @card_scroll_area(background_color)
            ]
        },
    );
}

/// A wrapping grid of 300 numbered cards.
fn cards(background_color: Color) -> impl Scene {
    let cards = (0..300)
        .map(|i| {
            let color = LIGHT_COLORS
                .get((i / 10) % LIGHT_COLORS.len())
                .map_or(Color::NONE, |c| c.color);
            bsn! {
                Node {
                    min_width: px(50.0),
                    width: px(100.0),
                    height: px(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                ~{Card}
                Pickable {
                    should_block_lower: false,
                }
                Children [
                    Pickable {
                        should_block_lower: false,
                    }
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(0.0),
                        height: percent(50.0),
                        width: percent(100.0),
                        border_radius: BorderRadius::top(px(10.0)),
                    }
                    BackgroundColor(color)
                    --
                    CardBackground
                    BackgroundColor(background_color)
                    Pickable {
                        should_block_lower: false,
                    }
                    Node {
                        padding: UiRect::px(5.0, 5.0, 3.0, 3.0),
                        border_radius: BorderRadius::top(px(5.0)),
                    }
                    Children [
                        ~{Label::regular(format!("{i}"))}
                    ]
                ]
            }
        })
        .collect::<Vec<_>>();
    bsn! {
        Node {
            row_gap: px(16.0),
            column_gap: px(16.0),
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            margin: UiRect::px(8.0, 8.0, 8.0, 0.0),
        }
        Pickable {
            should_block_lower: false,
        }
        Children [ {cards} ]
    }
}

#[derive(Component, Default, Clone)]
struct CardBackground;

fn color_palette() -> Vec<Item<Color>> {
    COLORS.iter().map(|c| Item { data: c.color }).collect()
}

fn countries() -> [Item<String>; 5] {
    [
        Item::new("Netherlands"),
        Item::new("Canada"),
        Item::new("Indonesia"),
        Item::new("Portugal"),
        Item::new("Belgium"),
    ]
}

fn items() -> impl Iterator<Item = Item<String>> {
    (1..100).map(|i| Item::new(i.to_string()))
}

fn update_theme(
    theme: Res<Theme>,
    mut card_background: Query<&mut BackgroundColor, With<CardBackground>>,
) {
    for mut card_background_color in &mut card_background {
        card_background_color.0 = theme.colors().background_primary;
    }
}

/// A vertical scroll area over a list of the items.
fn item_scroll_area() -> impl Scene {
    ScrollArea::scene(
        || {
            bsn_list! {
                List
                @{Selection::builder().one().scene_with_items(items())}
            }
        },
        true,
        false,
    )
}

/// A scroll area over the cards.
fn card_scroll_area(background_color: Color) -> impl Scene {
    ScrollArea::scene(move || bsn_list! { @cards(background_color) }, true, true)
}
