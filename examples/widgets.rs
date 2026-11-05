//! A showcase: a scroll area, a list and combos sharing one selection via `Selection::reference`, and buttons that add more items.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::color::palettes::tailwind::{
    AMBER_300, BLUE_300, CYAN_300, EMERALD_300, FUCHSIA_300, GREEN_300, INDIGO_300, LIME_300,
    ORANGE_300, PINK_300, PURPLE_300, ROSE_300, SKY_300, YELLOW_300,
};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::combo::Combo;
use bevy_pluim::list::List;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::scroll_area::ScrollArea;
use bevy_pluim::select::{Item, Selection};
use bevy_pluim::theme::Theme;

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

    let country_selection = commands
        .spawn(
            Selection::builder()
                .zero_or_more()
                .build_with_items(countries()),
        )
        .id();

    // Root container
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
        }
        Children [
            Node {
                width: px(400.0),
                height: px(600.0),
            }
            @scroll_squares()
            --
            List
            @{Selection::builder().zero_or_more().scene_with_items(countries())}
            --
            ~{Combo}
            @{Selection::builder().zero_or_one().scene_with_items(countries())}
            --
            ~{Combo}
            @{Selection::builder().zero_or_more().scene_with_items(countries())}
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
            --
            // The buttons add to their parent, the root.
            ~{PushButton::from_label("Add List")}
            on(move |activate: On<Activate>, parents: Query<&ChildOf>, mut commands: Commands| {
                if let Ok(root) = parents.get(activate.event_target()) {
                    commands
                        .entity(root.parent())
                        .with_child((List, Selection::reference(country_selection)));
                }
            })
            --
            ~{PushButton::from_label("Add Combo")}
            on(move |activate: On<Activate>, parents: Query<&ChildOf>, mut commands: Commands| {
                if let Ok(root) = parents.get(activate.event_target()) {
                    commands
                        .entity(root.parent())
                        .with_child((Combo, Selection::reference(country_selection)));
                }
            })
            --
            ~{PushButton::from_label("Add PushButton")}
            on(|activate: On<Activate>, parents: Query<&ChildOf>, mut commands: Commands| {
                if let Ok(root) = parents.get(activate.event_target()) {
                    commands
                        .entity(root.parent())
                        .with_child(PushButton::from_label("Hello"));
                }
            })
        ]
    });
}

fn countries() -> [Item<String>; 5] {
    [
        Item::new("Netherlands"),
        Item::new("Canada"),
        Item::new("Indonesia"),
        Item::new("Portugal"),
        Item::new("Belgium"),
    ]
}

/// A wrapping row of 300 coloured squares.
fn squares() -> impl Scene {
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
    let squares = (0..300)
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
            display: Display::Flex,
            flex_wrap: FlexWrap::Wrap,
            flex_direction: FlexDirection::Row,
            row_gap: px(10.0),
            column_gap: px(10.0),
        }
        Children [ {squares} ]
    }
}

/// A vertical scroll area over the coloured squares.
fn scroll_squares() -> impl Scene {
    ScrollArea::scene(|| bsn_list! { @squares() }, true, false)
}
