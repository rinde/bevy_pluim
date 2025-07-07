use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::color::palettes::tailwind::AMBER_600;
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::color::palettes::tailwind::CYAN_600;
use bevy::color::palettes::tailwind::EMERALD_600;
use bevy::color::palettes::tailwind::FUCHSIA_600;
use bevy::color::palettes::tailwind::GREEN_600;
use bevy::color::palettes::tailwind::INDIGO_600;
use bevy::color::palettes::tailwind::LIME_600;
use bevy::color::palettes::tailwind::ORANGE_600;
use bevy::color::palettes::tailwind::PINK_600;
use bevy::color::palettes::tailwind::PURPLE_600;
use bevy::color::palettes::tailwind::ROSE_600;
use bevy::color::palettes::tailwind::SKY_600;
use bevy::color::palettes::tailwind::YELLOW_600;
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
use ui::select::Items;
use ui::select::Selection;
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
    println!("Setup in app");
    commands.spawn(Camera2d);

    const COLORS: [Srgba; 14] = [
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
                // (Select::new(["hello", "who", "is", "there"]), List),
                // (
                //     Node {
                //         width: Val::Px(200.0),
                //         left: Val::Px(-300.0),
                //         ..default()
                //     },
                //     children![(
                //         // Selection(Some(1)),
                //         List,
                //         related!(Items[
                //             Item::new("Netherlands"),
                //             Item::new("Canada"),
                //             Item::new("Indonesia"),
                //             Item::new("Portugal"),
                //             Item::new("Belgium")
                //         ]),
                //         Float
                //     )]
                // ),
                ScrollArea::new(
                    |parent: &mut RelatedSpawner<ChildOf>| {
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
                    },
                    true,
                    false
                ),
                (
                    List,
                    Selection {
                        items: smallvec![],
                        multi: true,
                    },
                    related!(Items[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ),
                // (
                //     Text::new("Hello, World!"),
                //     TextColor(theme.colors.text_primary),
                //     HelloWorldLabel
                // ),
                // (Combo, Select::new(["hello", "who", "is", "there"])),
                (
                    Combo,
                    related!(Items[
                        Item::new("Netherlands"),
                        Item::new("Canada"),
                        Item::new("Indonesia"),
                        Item::new("Portugal"),
                        Item::new("Belgium")
                    ]),
                ),
                (
                    Combo,
                    Selection {
                        items: smallvec![],
                        multi: true,
                    },
                    related!(Items[
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
                    related!(Items[
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
                    related!(Items[
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
