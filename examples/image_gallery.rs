//! `Gallery`: image cards in two titled groups, in a horizontal scroll area.

use bevy::ecs::template::TemplateContext;
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::gallery::{Gallery, GalleryCard};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::theme::Theme;

/// Inserts `bundle` on the entity of a `bsn!` template once the scene has been applied.
///
/// A template that inserted it at once would do so before the scene gives the entity its
/// parent and its own components: a `Node` without a parent makes the entity a `UiRoot`, and
/// the scene's `Node` would overwrite what the bundle's hooks set on it.
fn insert_after_scene(context: &mut TemplateContext, bundle: impl Bundle) -> EmptySceneEffect {
    let entity = context.entity.id();
    context.entity.world_scope(|world| {
        world.commands().entity(entity).insert(bundle);
    });
    EmptySceneEffect
}

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
    // No image set ships with the crate: a real app passes its own white-on-transparent images.
    let cards = [
        ("Steam", "Locomotives"),
        ("Diesel", "Locomotives"),
        ("Electric", "Locomotives"),
        ("Passenger", "Wagons"),
        ("Box wagon", "Wagons"),
        ("Tank wagon", "Wagons"),
        ("Hopper", "Wagons"),
        ("Flat wagon", "Wagons"),
    ]
    .map(|(label, group)| Item {
        data: GalleryCard::new(Handle::default(), label, group),
    });
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            Node {
                width: px(500.0),
            }
            // `ScrollArea::bundle` is a bundle, not a scene, so `template` inserts it; the
            // gallery is a scene, applied to the entity its contents spawner spawns.
            template(move |context| {
                let cards = cards.clone();
                Ok(insert_after_scene(context, ScrollArea::bundle(
                    move |parent: &mut ChildSpawner| {
                        if let Err(error) = parent.spawn_empty().apply_scene(gallery(cards)) {
                            error!("cannot spawn the gallery: {error:?}");
                        }
                    },
                    false,
                    true,
                )))
            })
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
}

/// A gallery of `cards` with a selection of its own.
fn gallery(cards: [Item<GalleryCard>; 8]) -> impl Scene {
    bsn! {
        Gallery
        // `Selection::builder()` gives a bundle, which `template` inserts.
        template(move |context| {
            context
                .entity
                .insert(Selection::builder().one().build_with_items(cards.clone()));
            Ok(EmptySceneEffect)
        })
    }
}
