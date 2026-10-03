//! A showcase: a scroll area, a list and combos sharing one selection via `Selection::reference`, and buttons that add more items.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::color::palettes::tailwind::{
    AMBER_300, BLUE_300, CYAN_300, EMERALD_300, FUCHSIA_300, GREEN_300, INDIGO_300, LIME_300,
    ORANGE_300, PINK_300, PURPLE_300, ROSE_300, SKY_300, YELLOW_300,
};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy_egui::EguiPlugin;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::list::List;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(EguiPlugin::default())
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .add_observer(on_press)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    const COLORS: [Srgba; 14] = [
        YELLOW_300,
        SKY_300,
        ROSE_300,
        PURPLE_300,
        PINK_300,
        ORANGE_300,
        LIME_300,
        INDIGO_300,
        FUCHSIA_300,
        EMERALD_300,
        CYAN_300,
        BLUE_300,
        GREEN_300,
        AMBER_300,
    ];
    commands.spawn(Camera2d);

    let country_selection = commands
        .spawn(Selection::builder().zero_or_more().build_with_items([
            Item::new("Netherlands"),
            Item::new("Canada"),
            Item::new("Indonesia"),
            Item::new("Portugal"),
            Item::new("Belgium"),
        ]))
        .id();

    // Root container
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            children![
                (
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(600.0),
                        ..default()
                    },
                    ScrollArea::bundle(
                        |parent: &mut RelatedSpawner<ChildOf>| {
                            parent
                                .spawn((
                                    Pickable {
                                        should_block_lower: false,
                                        ..default()
                                    },
                                    Node {
                                        display: Display::Flex,
                                        flex_wrap: FlexWrap::Wrap,
                                        flex_direction: FlexDirection::Row,
                                        row_gap: Val::Px(10.0),
                                        column_gap: Val::Px(10.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|parent| {
                                    for i in 0..300 {
                                        parent.spawn((
                                            Node {
                                                width: Val::Px(50.0),
                                                height: Val::Px(50.0),
                                                ..default()
                                            },
                                            Pickable {
                                                should_block_lower: false,
                                                ..default()
                                            },
                                            BackgroundColor(
                                                COLORS
                                                    .get(i % COLORS.len())
                                                    .map_or(Color::NONE, |c| Color::Srgba(*c)),
                                            ),
                                        ));
                                    }
                                });
                        },
                        true,
                        false
                    )
                ),
                (
                    List,
                    Selection::builder().zero_or_more().build_with_items([
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ),
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
                (
                    Combo,
                    Selection::builder().zero_or_more().build_with_items([
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ),
                (PushButton::from_label("Change theme"), ThemeButton),
            ],
        ))
        .id();

    commands.entity(root).with_children(|spawner| {
        spawner.spawn(PushButton::from_label("Add List")).observe(
            move |_: On<Pointer<Press>>, mut commands: Commands| {
                commands
                    .entity(root)
                    .with_child((List, Selection::reference(country_selection)));
            },
        );
        spawner.spawn(PushButton::from_label("Add Combo")).observe(
            move |_: On<Pointer<Press>>, mut commands: Commands| {
                commands
                    .entity(root)
                    .with_child((Combo, Selection::reference(country_selection)));
            },
        );
        spawner
            .spawn(PushButton::from_label("Add PushButton"))
            .observe(move |_: On<Pointer<Press>>, mut commands: Commands| {
                commands
                    .entity(root)
                    .with_child(PushButton::from_label("Hello"));
            });
    });
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
