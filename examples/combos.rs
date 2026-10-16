//! `Combo` variants.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::combo::Combo;
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
        .spawn(Selection::builder().zero_or_one().build_with_items([
            Item::new("Netherlands"),
            Item::new("Canada"),
            Item::new("Indonesia"),
            Item::new("Portugal"),
            Item::new("Belgium"),
        ]))
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
            ~{Combo} ~{Selection::reference(selection_ref)}
            --
            ~{Combo}
            // `Selection::builder()` gives a bundle, which `template` inserts.
            template(move |context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items_ref(selection_ref),
                );
                Ok(EmptySceneEffect)
            })
            --
            ~{Combo} ~{Selection::reference(selection_ref)}
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
