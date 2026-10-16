//! `FilterChips`: chips with counts that wrap onto further lines, and a second row of chips
//! that shares its selection with the first.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::filter_chips::{Chip, FilterChips};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::select::{Item, Selection};
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
    let chips = commands
        .spawn(Selection::builder().one().build_with_items([
            Item {
                data: Chip::with_count("All", 24),
            },
            Item {
                data: Chip::with_count("Running", 12),
            },
            Item {
                data: Chip::with_count("Waiting", 7),
            },
            Item {
                data: Chip::with_count("Problems", 3),
            },
            Item {
                data: Chip::with_count("Freight", 9),
            },
            Item {
                data: Chip::with_count("Passenger", 11),
            },
            Item {
                data: Chip::new("Depot"),
            },
        ]))
        .id();
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            // Narrow, so that the chips wrap.
            FilterChips
            Node {
                width: px(320.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(6.0),
                row_gap: px(6.0),
            }
            ~{Selection::reference(chips)}
            --
            // The same selection in a wide row.
            FilterChips
            ~{Selection::reference(chips)}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}
