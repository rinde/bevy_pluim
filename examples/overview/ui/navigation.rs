//! The navigation page(s) of the overview.

use bevy::ecs::spawn::SpawnIter;
use bevy_pluim::select::{ItemParam, ItemValues, SelectionParam, SelectionValue};

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

/// Marks the selection of the trail of the "Bread crumbs with a stack" variant.
#[derive(Component, Copy, Clone, Debug, Default)]
struct TrailSelection;

/// What a stop of the trail is, to find its item.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
enum TrailStop {
    Home,
    PanelA,
    PanelAVariant,
    PanelB,
    PanelBSub,
}

pub(super) fn navigation(spawner: In<StackChildSpawner>, mut commands: Commands) {
    // The trail's stops: panel A is created once and shown again through the variant, which
    // is a reference to it.
    let trail = [
        (
            TrailStop::Home,
            Item::new("Home"),
            StackItem::new(commands.register_system(trail_home)),
        ),
        (
            TrailStop::PanelA,
            Item::new("Panel A"),
            StackItem::new_with_reference(0, commands.register_system(trail_a)),
        ),
        (
            TrailStop::PanelAVariant,
            Item::new("Panel A variant"),
            StackItem::new_reference(0),
        ),
        (
            TrailStop::PanelB,
            Item::new("Panel B"),
            StackItem::new(commands.register_system(trail_b)),
        ),
        (
            TrailStop::PanelBSub,
            Item::new("Panel B sub"),
            StackItem::new(commands.register_system(trail_b_sub)),
        ),
    ];
    let tabs = [
        (
            Item::new("Palettes"),
            StackItem::new(commands.register_system(tab_palettes)),
        ),
        (
            Item::new("List"),
            StackItem::new(commands.register_system(tab_list)),
        ),
        (
            Item::new("Cards"),
            StackItem::new(commands.register_system(tab_cards)),
        ),
    ];
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tabs and trails")}
                --
                ~{groups()}
                Children [
                    @variant("Tab bar", bsn! {
                        TabBar
                        @{Selection::builder().one().scene_with_items(["Overview", "Trains", "Stops"].map(Item::new))}
                    })
                    --
                    @variant("Bread crumbs", bsn! {
                        ~{BreadCrumbs}
                        @{Selection::builder().one_or_more().scene_with_items(["Home", "Lines", "Line 4"].map(Item::new))}
                    })
                    --
                    @variant("Bread crumbs driving a stack: push, pop and replace the last crumb", bsn! {
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            row_gap: px(12.0),
                            width: px(460.0),
                        }
                        Children [
                            ~{BreadCrumbs}
                            #crumbs
                            TrailSelection
                            @{Selection::builder().one_or_more().scene_with_items(trail)}
                            --
                            Node { min_height: px(130.0) }
                            Children [ Stack { selection_ref: #crumbs } ]
                        ]
                    })
                    --
                    @variant("Tab bar driving a stack", bsn! {
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            row_gap: px(12.0),
                            width: px(460.0),
                        }
                        Children [
                            TabBar
                            #tabs
                            @{Selection::builder().one().scene_with_items(tabs)}
                            --
                            Node { min_height: px(190.0) }
                            Children [ Stack { selection_ref: #tabs } ]
                        ]
                    })
                ]
            ]
        },
    );
}

// ---- The trail's panels ----

/// An observer that pushes `target` onto the trail.
fn go_to(
    target: TrailStop,
) -> impl Fn(On<Activate>, SelectionParam<With<TrailSelection>>, ItemParam<String, &TrailStop>) + Clone
{
    move |_, mut selections, items| {
        let Some(item) = items.find_item_ref(|_, stop| **stop == target) else {
            return;
        };
        selections.update_selection_single(|selection| selection.push(item));
    }
}

/// An observer that replaces the last crumb with `target`.
fn replace_with(
    target: TrailStop,
) -> impl Fn(On<Activate>, SelectionParam<With<TrailSelection>>, ItemParam<String, &TrailStop>) + Clone
{
    move |_, mut selections, items| {
        let Some(item) = items.find_item_ref(|_, stop| **stop == target) else {
            return;
        };
        selections.update_selection_single(|selection| selection.pop()?.push(item));
    }
}

fn pop_trail(_: On<Activate>, mut selections: SelectionParam<With<TrailSelection>>) {
    selections.update_selection_single(SelectionValue::pop);
}

fn trail_home(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexStart, row_gap: px(6.0) }
            Children [
                ~{Label::regular("Home")}
                --
                ~{PushButton::from_label("Go to Panel A")}
                on(go_to(TrailStop::PanelA))
                --
                ~{PushButton::from_label("Go to Panel A variant")}
                on(go_to(TrailStop::PanelAVariant))
                --
                ~{PushButton::from_label("Go to Panel B")}
                on(go_to(TrailStop::PanelB))
            ]
        },
    );
}

fn trail_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexStart, row_gap: px(6.0) }
            Children [
                ~{Label::regular("Panel A")}
                --
                ~{PushButton::from_label("Go back")}
                on(pop_trail)
                --
                ~{PushButton::from_label("Replace this crumb with Panel A variant")}
                on(replace_with(TrailStop::PanelAVariant))
            ]
        },
    );
}

fn trail_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexStart, row_gap: px(6.0) }
            Children [
                ~{Label::regular("Panel B")}
                --
                ~{PushButton::from_label("Go back")}
                on(pop_trail)
                --
                ~{PushButton::from_label("Go to Panel B sub")}
                on(go_to(TrailStop::PanelBSub))
            ]
        },
    );
}

fn trail_b_sub(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexStart, row_gap: px(6.0) }
            Children [
                ~{Label::regular("Panel B sub")}
                --
                ~{PushButton::from_label("Go back to Panel B")}
                on(pop_trail)
            ]
        },
    );
}

// ---- The tabs' panels ----

/// Three colour palette combos sharing one selection, one with its own, and a combo on a card.
fn tab_palettes(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let colors = commands
        .spawn(ItemValues::spawn(SpawnIter(
            COLORS
                .iter()
                .map(|c| Item { data: c.color })
                .collect::<Vec<_>>()
                .into_iter(),
        )))
        .id();
    let shared = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(colors),
        )
        .id();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, align_items: AlignItems::FlexStart, column_gap: px(10.0), row_gap: px(10.0) }
            Children [
                ColorPaletteCombo
                ~{ListLayout::VerticalGrid { columns: 11 }}
                ~{Selection::reference(shared)}
                --
                ColorPaletteCombo
                ~{ListLayout::VerticalGrid { columns: 11 }}
                ~{Selection::reference(shared)}
                --
                ColorPaletteCombo
                ~{ListLayout::VerticalGrid { columns: 11 }}
                ~{Selection::reference(shared)}
                --
                ColorPaletteCombo
                ~{ListLayout::VerticalGrid { columns: 11 }}
                @{Selection::builder().zero_or_one().scene_with_items_ref(colors)}
                --
                ~{Card}
                Node {
                    padding: UiRect::all(px(10.0)),
                    align_items: AlignItems::FlexStart,
                    column_gap: px(10.0),
                }
                Children [
                    ~{Combo}
                    @{Selection::builder().zero_or_one().scene_with_items(countries())}
                    --
                    ~{PushButton::from_label("Button on a card")}
                ]
            ]
        },
    );
}

/// A scroll area of 100 items beside a list.
fn tab_list(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { flex_direction: FlexDirection::Row, align_items: AlignItems::FlexStart, column_gap: px(20.0) }
            Children [
                Node { width: px(200.0), height: px(170.0) }
                @ScrollArea::scene(
                    || bsn_list! {
                        List
                        @{Selection::builder().one().scene_with_items((1..100).map(|n| Item::new(n.to_string())))}
                    },
                    true,
                    false,
                )
                --
                List
                @{Selection::builder().zero_or_one().scene_with_items(countries())}
            ]
        },
    );
}

/// A wrapping grid of 300 numbered cards in a scroll area that scrolls both ways.
fn tab_cards(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            Node { width: px(440.0), height: px(170.0) }
            @ScrollArea::scene(|| bsn_list! { @card_grid() }, true, true)
        },
    );
}

fn card_grid() -> impl Scene {
    let cards = (0..300)
        .map(|n| {
            let color = COLORS
                .get(((n / 10) * 10 + 5) % COLORS.len())
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
                Pickable { should_block_lower: false }
                BackgroundColor(color)
                Children [ ~{Label::regular(n.to_string())} ]
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
        Pickable { should_block_lower: false }
        Children [ {cards} ]
    }
}
