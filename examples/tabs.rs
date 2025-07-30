use std::sync::LazyLock;

use bevy::DefaultPlugins;
use bevy::app::{App, PostStartup, Startup, Update};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::{ChildSpawner, Children};
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::{With, Without};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::spawn::{SpawnIter, SpawnRelated, SpawnWith};
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::ecs::{children, related};
use bevy::picking::Pickable;
use bevy::picking::events::{Pointer, Released};
use bevy::text::TextColor;
use bevy::ui::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, Display, FlexWrap,
    GridAutoFlow, GridPlacement, GridTrack, JustifyContent, Node, PositionType, UiDebugOptions,
    UiRect, Val,
};
use bevy::utils::default;
use bevy_egui::EguiPlugin;
use ui::WidgetsPlugin;
use ui::card::Card;
use ui::color_palette_combo::ColorPaletteCombo;
use ui::combo::Combo;
use ui::label::Label;
use ui::list::{CornerStyle, List, ListLayout};
use ui::push_button::PushButton;
use ui::scroll_area::ScrollArea;
use ui::select::{Item, ItemValues, ItemsRef, Selection};
use ui::stack::{Stack, StackChildSpawner, StackItem};
use ui::tailwind::{COLORS, TailwindColor, Variant};
use ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins((EguiPlugin {
            enable_multipass_for_primary_context: false,
        },))
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_systems(Update, update_theme.run_if(resource_changed::<Theme>))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, post_setup)
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2d);

    let theme_border_color = theme.colors.border;

    // Root container
    commands.spawn((
        Node {
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::max_content(),
                GridTrack::auto(),
                GridTrack::max_content(),
            ],
            grid_template_rows: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(theme.colors.background_primary),
        Root,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let id = parent
                .spawn((
                    List,
                    CornerStyle::Tabs,
                    ListLayout::Horizontal,
                    Selection::default(),
                    TabItemsAndSelection,
                    related!(ItemValues[
                        (Item::new("Panel A"), StackItem::new(&create_a)),
                        (Item::new("Panel B"), StackItem::new(&create_b)),
                        (Item::new("Card demo"), StackItem::new(&create_card_demo)),
                    ]),
                ))
                .id();
            // filler
            parent.spawn((
                BorderColor(theme_border_color),
                Node {
                    max_width: Val::Percent(100.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
            ));

            parent.spawn((PushButton::from_label("Toggle theme"), ThemeButton));

            parent.spawn((
                Node {
                    grid_column: GridPlacement::span(3),
                    display: Display::Flex,
                    align_items: AlignItems::FlexStart,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Stack { selection_ref: id },
            ));
        })),
    ));
}

#[derive(Component)]
struct TabItemsAndSelection;

fn post_setup(
    query: Query<(Entity, &Selection, &ItemValues), With<TabItemsAndSelection>>,
    mut commands: Commands,
) {
    let (sref, selection, item_values) = query.single().unwrap();
    let s = selection
        .as_selection()
        .unwrap()
        .toggle_selected(item_values.iter().copied().next().unwrap())
        .unwrap();
    commands.entity(sref).insert(s.into_component());
}

fn create_a(spawner: StackChildSpawner, _theme: &Theme) -> Entity {
    spawner.spawn(Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
        parent.spawn(Label::regular("Panel A"));

        let colors = parent
            .spawn(ItemValues::spawn(SpawnIter(color_palette().into_iter())))
            .id();

        let selection_ref = parent.spawn(Selection::default()).id();

        parent.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::reference(selection_ref),
            ItemsRef(colors),
        ));
        parent.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            ItemsRef(colors),
        ));
        parent.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::reference(selection_ref),
            ItemsRef(colors),
        ));
        parent.spawn((
            Card,
            Node {
                height: Val::Px(200.0),
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(10.0),
                column_gap: Val::Px(10.0),
                ..default()
            },
            children![
                (
                    Combo,
                    related!(ItemValues[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ])
                ),
                PushButton::from_label("Button on a card")
            ],
        ));
        parent.spawn(PushButton::from_label("hello"));
    })))
}

fn create_b(spawner: StackChildSpawner, _theme: &Theme) -> Entity {
    spawner.spawn((children![
        Label::header("Panel B"),
        ScrollArea::bundle(
            |parent| {
                parent.spawn((
                    List,
                    Selection::default(),
                    ItemValues::spawn(SpawnIter(items())),
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
                Selection::default(),
                related!(ItemValues[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            )]
        ),
    ],))
}

static LIGHT_COLORS: LazyLock<Vec<TailwindColor>> = LazyLock::new(|| {
    COLORS
        .iter()
        .filter(|x| x.variant == Variant::V200)
        .copied()
        .collect::<Vec<_>>()
});

fn create_card_demo(spawner: StackChildSpawner, theme: &Theme) -> Entity {
    let background_color = theme.colors.background_primary;
    spawner.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Grid,
            grid_auto_columns: vec![GridTrack::percent(100.0)],
            grid_auto_flow: GridAutoFlow::Row,
            ..default()
        },
        children![
            Label::header("Card demo"),
            (
                Node { ..default() },
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
                ScrollArea::bundle(
                    move |parent| {
                        parent.spawn((
                            Node {
                                row_gap: Val::Px(16.0),
                                column_gap: Val::Px(16.0),
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::px(8.0, 8.0, 8.0, 0.0),
                                ..default()
                            },
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                            Children::spawn(SpawnIter((0..300).map(move |i| {
                                (
                                    Node {
                                        min_width: Val::Px(50.0),
                                        width: Val::Px(100.0),
                                        height: Val::Px(100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    Card,
                                    Pickable {
                                        should_block_lower: false,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Pickable {
                                                should_block_lower: false,
                                                ..default()
                                            },
                                            Node {
                                                position_type: PositionType::Absolute,
                                                top: Val::Px(0.0),
                                                height: Val::Percent(50.0),
                                                width: Val::Percent(100.0),
                                                ..default()
                                            },
                                            BorderRadius::top(Val::Px(10.0)),
                                            BackgroundColor(
                                                LIGHT_COLORS[(i / 10) % LIGHT_COLORS.len()].color
                                            )
                                        ),
                                        (
                                            CardBackground,
                                            BorderRadius::top(Val::Px(5.0)),
                                            BackgroundColor(background_color),
                                            Pickable {
                                                should_block_lower: false,
                                                ..default()
                                            },
                                            Node {
                                                padding: UiRect::px(5.0, 5.0, 3.0, 3.0),
                                                ..default()
                                            },
                                            children![Label::regular(format!("{i}")),]
                                        ),
                                    ],
                                )
                            }))),
                        ));
                    },
                    true,
                    true
                )
            )
        ],
    ))
}

#[derive(Component)]
struct CardBackground;

fn color_palette() -> Vec<impl Bundle> {
    COLORS.iter().map(|c| (Item { data: c.color })).collect()
}

fn items() -> impl Iterator<Item = Item<String>> {
    (1..100).map(|i| Item::new(i.to_string()))
}

#[derive(Component)]
struct ThemeButton;

fn on_press(
    trigger: Trigger<Pointer<Released>>,
    button: Query<(), With<ThemeButton>>,
    mut theme: ResMut<Theme>,
) {
    if button.get(trigger.target()).is_ok() {
        theme.light_dark_toggle();
    }
}

#[derive(Component)]
struct Root;

#[derive(Component)]
struct HelloWorldLabel;

fn update_theme(
    theme: Res<Theme>,
    mut background: Query<&mut BackgroundColor, With<Root>>,
    mut hello_world: Query<&mut TextColor, With<HelloWorldLabel>>,
    mut card_background: Query<&mut BackgroundColor, (With<CardBackground>, Without<Root>)>,
) {
    for mut background_color in &mut background {
        background_color.0 = theme.colors.background_primary;
    }
    for mut text_color in &mut hello_world {
        text_color.0 = theme.colors.text_primary;
    }
    for mut card_background_color in &mut card_background {
        card_background_color.0 = theme.colors.background_primary;
    }
}
