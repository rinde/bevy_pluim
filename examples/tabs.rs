//! A `List` as tab bar driving a `Stack`, with combos, colour palettes and cards inside.

use std::sync::LazyLock;

use bevy::ecs::spawn::{SpawnIter, SpawnWith};
use bevy::prelude::*;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::list::{CornerStyle, List, ListLayout};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, ItemValues, Selection};
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::tailwind::{COLORS, TailwindColor, Variant};
use rin_bevy_ui::theme::Theme;

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
        .add_observer(on_press)
        .add_systems(Update, update_theme.run_if(resource_changed::<Theme>))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2d);

    let theme_border_color = theme.colors.border;

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
        Root,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let selection_ref = parent
                .spawn((
                    List,
                    CornerStyle::Tabs,
                    ListLayout::Horizontal,
                    Selection::builder().one().build_with_items(items),
                ))
                .id();

            // filler
            parent.spawn((
                BorderColor::all(theme_border_color),
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
                Stack { selection_ref },
            ));
        })),
    ));
}

fn create_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent.spawn(Label::regular("Panel A"));

            let colors = parent
                .spawn(ItemValues::spawn(SpawnIter(color_palette().into_iter())))
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
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::reference(selection_ref),
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
                        Selection::builder().zero_or_one().build_with_items([
                            Item::new("Netherlands"),
                            Item::new("Canada"),
                            Item::new("Indonesia"),
                            Item::new("Portugal"),
                            Item::new("Belgium")
                        ]),
                    ),
                    PushButton::from_label("Button on a card")
                ],
            ));
            parent.spawn(PushButton::from_label("hello"));
        })),
    );
}

fn create_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (children![
            Label::header("Panel B"),
            ScrollArea::bundle(
                |parent| {
                    parent.spawn((List, Selection::builder().one().build_with_items(items())));
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
                    Selection::builder().zero_or_one().build_with_items([
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                )]
            ),
        ],),
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
    let background_color = theme.colors.background_primary;
    spawner.spawn(
        &mut commands,
        (
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
                    Node::default(),
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
                                                    border_radius: BorderRadius::top(Val::Px(10.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(
                                                    LIGHT_COLORS
                                                        .get((i / 10) % LIGHT_COLORS.len())
                                                        .map_or(Color::NONE, |c| c.color)
                                                )
                                            ),
                                            (
                                                CardBackground,
                                                BackgroundColor(background_color),
                                                Pickable {
                                                    should_block_lower: false,
                                                    ..default()
                                                },
                                                Node {
                                                    padding: UiRect::px(5.0, 5.0, 3.0, 3.0),
                                                    border_radius: BorderRadius::top(Val::Px(5.0)),
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
        ),
    );
}

#[derive(Component)]
struct CardBackground;

fn color_palette() -> Vec<impl Bundle> {
    COLORS.iter().map(|c| Item { data: c.color }).collect()
}

fn items() -> impl Iterator<Item = Item<String>> {
    (1..100).map(|i| Item::new(i.to_string()))
}

#[derive(Component)]
struct ThemeButton;

fn on_press(
    trigger: On<Pointer<Release>>,
    button: Query<(), With<ThemeButton>>,
    mut theme: ResMut<Theme>,
) {
    if button.get(trigger.event().event_target()).is_ok() {
        theme.light_dark_toggle();
    }
}

#[derive(Component)]
struct Root;

fn update_theme(
    theme: Res<Theme>,
    mut card_background: Query<&mut BackgroundColor, (With<CardBackground>, Without<Root>)>,
) {
    for mut card_background_color in &mut card_background {
        card_background_color.0 = theme.colors.background_primary;
    }
}
