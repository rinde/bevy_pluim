//! `ScrollArea`s scrolling vertically, horizontally or both.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::color::palettes::tailwind::{
    AMBER_600, BLUE_600, CYAN_600, EMERALD_600, FUCHSIA_600, GREEN_600, INDIGO_600, LIME_600,
    ORANGE_600, PINK_600, PURPLE_600, ROSE_600, SKY_600, YELLOW_600,
};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(10.0),
            column_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        children![
            (
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    ..default()
                },
                ScrollArea::bundle(create_squares(300, GridAutoFlow::Row), true, true)
            ),
            ScrollArea::bundle(create_squares(50, GridAutoFlow::Row), true, false),
            // ScrollArea::new(create_squares(1000), true, false),
            ScrollArea::bundle(create_squares(200, GridAutoFlow::Column), true, true),
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
                        BackgroundColor(
                            COLORS
                                .get(i % COLORS.len())
                                .map_or(Color::NONE, |c| Color::Srgba(*c)),
                        ),
                    ));
                }
            });
    }
}

#[derive(Component)]
struct ThemeButton;

fn on_press(trigger: On<Activate>, button: Query<(), With<ThemeButton>>, mut theme: ResMut<Theme>) {
    if button.get(trigger.event().event_target()).is_ok() {
        theme.light_dark_toggle();
    }
}
