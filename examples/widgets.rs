use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::color::palettes::tailwind::AMBER_300;
use bevy::color::palettes::tailwind::BLUE_300;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::color::palettes::tailwind::EMERALD_300;
use bevy::color::palettes::tailwind::FUCHSIA_300;
use bevy::color::palettes::tailwind::GREEN_300;
use bevy::color::palettes::tailwind::INDIGO_300;
use bevy::color::palettes::tailwind::LIME_300;
use bevy::color::palettes::tailwind::ORANGE_300;
use bevy::color::palettes::tailwind::PINK_300;
use bevy::color::palettes::tailwind::PURPLE_300;
use bevy::color::palettes::tailwind::ROSE_300;
use bevy::color::palettes::tailwind::SKY_300;
use bevy::color::palettes::tailwind::YELLOW_300;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use smallvec::smallvec;
use ui::WidgetsPlugin;
use ui::combo::Combo;
use ui::list::List;
use ui::push_button::PushButton;
use ui::scroll_area::ScrollArea;
use ui::select::Item;
use ui::select::ItemValues;
use ui::select::SelectionValue;
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
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
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
            BackgroundColor(theme.colors.background_primary),
            Root,
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
                                            BackgroundColor(Color::Srgba(COLORS[i % COLORS.len()])),
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
                    SelectionValue {
                        items: smallvec![],
                        multi: true,
                    }
                    .into_component(),
                    related!(ItemValues[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ),
                (
                    Combo,
                    related!(ItemValues[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ),
                (
                    Combo,
                    SelectionValue {
                        items: smallvec![],
                        multi: true,
                    }
                    .into_component(),
                    related!(ItemValues[
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
            move |_: Trigger<Pointer<Pressed>>, mut commands: Commands| {
                commands.entity(root).with_child((
                    List,
                    related!(ItemValues[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ));
            },
        );
        spawner.spawn(PushButton::from_label("Add Combo")).observe(
            move |_: Trigger<Pointer<Pressed>>, mut commands: Commands| {
                commands.entity(root).with_child((
                    Combo,
                    related!(ItemValues[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ));
            },
        );
        spawner
            .spawn(PushButton::from_label("Add PushButton"))
            .observe(
                move |_: Trigger<Pointer<Pressed>>, mut commands: Commands| {
                    commands
                        .entity(root)
                        .with_child(PushButton::from_label("Hello"));
                },
            );
    });
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
) {
    for mut background_color in &mut background {
        background_color.0 = theme.colors.background_primary;
    }
    for mut text_color in &mut hello_world {
        text_color.0 = theme.colors.text_primary;
    }
}
