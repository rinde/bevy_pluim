//! `List` layouts and corner styles, and a `VirtualList` of ten thousand rows.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::list::{CornerStyle, List, ListLayout};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui::virtual_list::{RowHeight, VirtualList};

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

    let selection_ref = commands
        .spawn(Selection::builder().one_or_more().build_with_items([
            Item::new("Netherlands"),
            Item::new("Canada"),
            Item::new("Indonesia"),
            Item::new("Portugal"),
            Item::new("Belgium"),
        ]))
        .id();

    // A list with a selection of its own gets the scene of `Selection::builder()`.
    let countries = || {
        [
            Item::new("Netherlands"),
            Item::new("Canada"),
            Item::new("Indonesia"),
            Item::new("Portugal"),
            Item::new("Belgium"),
        ]
    };

    // Root container
    commands.spawn_scene(bsn! {
        Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
        }
        Children [
            List ~{Selection::reference(selection_ref)}
            --
            List
            ~{ListLayout::Horizontal}
            @{Selection::builder().one().scene_with_items(["NL", "CA", "ID", "PT", "BE"].map(Item::new))}
            --
            List ~{ListLayout::VerticalGrid { columns: 3 }} ~{Selection::reference(selection_ref)}
            --
            List
            ~{CornerStyle::Tabs}
            @{Selection::builder().zero_or_one().scene_with_items_ref(selection_ref)}
            --
            List
            ~{ListLayout::Horizontal}
            ~{CornerStyle::Tabs}
            @{Selection::builder().zero_or_more().scene_with_items_ref(selection_ref)}
            --
            List
            ~{ListLayout::VerticalGrid { columns: 3 }}
            ~{CornerStyle::Tabs}
            @{Selection::builder().zero_or_more().scene_with_items(countries())}
            --
            List
            ~{CornerStyle::Separated}
            @{Selection::builder().one().scene_with_items(countries())}
            --
            Node {
                width: px(240.0),
                height: px(300.0),
            }
            @virtual_list_scroll_area(VirtualList::new(row_height(28.0)))
            --
            Node {
                width: px(240.0),
                height: px(300.0),
            }
            @virtual_list_scroll_area(VirtualList::separated(row_height(44.0)).unwrap_or_else(|_| std::process::abort()))
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}

fn row_height(height: f32) -> RowHeight {
    RowHeight::new(height).unwrap_or_else(|_| std::process::abort())
}

/// A scroll area over a `VirtualList` of ten thousand rows, which spawns only the rows in view.
/// The second has the separated look of a `List` with `CornerStyle::Separated`.
fn virtual_list_scroll_area(list: VirtualList) -> impl Scene {
    ScrollArea::scene(
        bsn_list! {
            ~{list}
            @{Selection::builder().one().scene_with_items((1..=10_000).map(|row| Item::new(format!("Row {row}"))))}
        },
        true,
        false,
    )
}
