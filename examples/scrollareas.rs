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
use ui::WidgetsPlugin;
use ui::push_button::PushButton;
use ui::scroll_area::ScrollArea;
use ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
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
    commands.spawn(Camera2d);
    commands.spawn((
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
            ScrollArea::new(create_squares(300, GridAutoFlow::Row), true, false),
            ScrollArea::new(create_squares(50, GridAutoFlow::Row), true, false),
            // ScrollArea::new(create_squares(1000), true, false),
            ScrollArea::new(create_squares(200, GridAutoFlow::Column), true, true),
            (PushButton::from_label("Change theme"), ThemeButton),
        ],
    ));
}

fn create_squares(
    num: usize,
    grid_auto_flow: GridAutoFlow,
) -> impl FnOnce(&mut RelatedSpawner<ChildOf>) + Send + Sync {
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
    move |parent: &mut RelatedSpawner<ChildOf>| {
        parent
            .spawn((
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![RepeatedGridTrack::auto(7)],
                    grid_template_rows: vec![RepeatedGridTrack::auto(7)],
                    grid_auto_flow,
                    // flex_direction: FlexDirection::Row,
                    // flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::FlexStart,
                    row_gap: Val::Px(2.0),
                    column_gap: Val::Px(2.0),
                    // padding: UiRect::all(Val::Px(4.0)),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    // overflow: Overflow::scroll(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                for i in 0..num {
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
    }
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
