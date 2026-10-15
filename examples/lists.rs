//! `List` layouts and corner styles.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::list::{CornerStyle, List, ListLayout};
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
        .add_observer(on_press)
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

    // A list with a selection of its own gets the bundle of `Selection::builder()` through
    // `template`, which inserts any bundle.
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
            template(|context| {
                context.entity.insert(
                    Selection::builder()
                        .one()
                        .build_with_items(["NL", "CA", "ID", "PT", "BE"].map(Item::new)),
                );
                Ok(EmptySceneEffect)
            })
            --
            List ~{ListLayout::VerticalGrid { columns: 3 }} ~{Selection::reference(selection_ref)}
            --
            List
            ~{CornerStyle::Tabs}
            template(move |context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items_ref(selection_ref),
                );
                Ok(EmptySceneEffect)
            })
            --
            List
            ~{ListLayout::Horizontal}
            ~{CornerStyle::Tabs}
            template(move |context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_more()
                        .build_with_items_ref(selection_ref),
                );
                Ok(EmptySceneEffect)
            })
            --
            List
            ~{ListLayout::VerticalGrid { columns: 3 }}
            ~{CornerStyle::Tabs}
            template(move |context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_more()
                        .build_with_items(countries()),
                );
                Ok(EmptySceneEffect)
            })
            --
            List
            ~{CornerStyle::Separated}
            template(move |context| {
                context
                    .entity
                    .insert(Selection::builder().one().build_with_items(countries()));
                Ok(EmptySceneEffect)
            })
            --
            ~{PushButton::from_label("Change theme")} ThemeButton
        ]
    });
}

#[derive(Component, Default, Clone)]
struct ThemeButton;

fn on_press(trigger: On<Activate>, button: Query<(), With<ThemeButton>>, mut theme: ResMut<Theme>) {
    if button.get(trigger.event().event_target()).is_ok() {
        theme.light_dark_toggle();
    }
}
