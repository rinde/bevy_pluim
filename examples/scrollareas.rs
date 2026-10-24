//! `ScrollArea`s scrolling vertically, horizontally or both.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::color::palettes::tailwind::{
    AMBER_600, BLUE_600, CYAN_600, EMERALD_600, FUCHSIA_600, GREEN_600, INDIGO_600, LIME_600,
    ORANGE_600, PINK_600, PURPLE_600, ROSE_600, SKY_600, YELLOW_600,
};
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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(10.0),
            column_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            Node {
                width: px(200.0),
                height: px(200.0),
            }
            @scroll_area(300, GridAutoFlow::Row, true, true)
            --
            @scroll_area(50, GridAutoFlow::Row, true, false)
            --
            @scroll_area(200, GridAutoFlow::Column, true, true)
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}

/// A scroll area over a grid of `num` coloured squares.
///
fn scroll_area(num: usize, grid_auto_flow: GridAutoFlow, v: bool, h: bool) -> impl Scene {
    ScrollArea::scene(move || bsn_list! { @squares(num, grid_auto_flow) }, v, h)
}

/// A grid of `num` coloured squares, laid out along `grid_auto_flow`.
fn squares(num: usize, grid_auto_flow: GridAutoFlow) -> impl Scene {
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
    let squares = (0..num)
        .map(|i| {
            let color = COLORS
                .get(i % COLORS.len())
                .map_or(Color::NONE, |c| Color::Srgba(*c));
            bsn! {
                Node {
                    width: px(50.0),
                    height: px(50.0),
                }
                Pickable {
                    should_block_lower: false,
                }
                BackgroundColor(color)
            }
        })
        .collect::<Vec<_>>();
    bsn! {
        Pickable {
            should_block_lower: false,
        }
        Node {
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::auto(7)],
            grid_template_rows: vec![RepeatedGridTrack::auto(7)],
            grid_auto_flow,
            justify_content: JustifyContent::SpaceBetween,
            align_content: AlignContent::FlexStart,
            row_gap: px(2.0),
            column_gap: px(2.0),
        }
        Children [ {squares} ]
    }
}
