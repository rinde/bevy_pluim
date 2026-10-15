//! A showcase: a scroll area, a list and combos sharing one selection via `Selection::reference`, and buttons that add more items.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::color::palettes::tailwind::{
    AMBER_300, BLUE_300, CYAN_300, EMERALD_300, FUCHSIA_300, GREEN_300, INDIGO_300, LIME_300,
    ORANGE_300, PINK_300, PURPLE_300, ROSE_300, SKY_300, YELLOW_300,
};
use bevy::ecs::system::Commands;
use bevy::ecs::template::TemplateContext;
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::list::List;
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
            // `ScrollArea::bundle` is a bundle, not a scene, so `template` inserts it; the
            // squares are a scene, applied to the entity its contents spawner spawns.
            template(|context| {
                Ok(insert_after_scene(context, ScrollArea::bundle(
                    |parent: &mut ChildSpawner| {
                        if let Err(error) = parent.spawn_empty().apply_scene(squares()) {
                            error!("cannot spawn the squares: {error:?}");
                        }
                    },
                    true,
                    false,
                )))
            })
            --
            // `Selection::builder()` gives a bundle, which `template` inserts.
            List
            template(|context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_more()
                        .build_with_items(countries()),
                );
                Ok(EmptySceneEffect)
            })
            --
            ~{Combo}
            template(|context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items(countries()),
                );
                Ok(EmptySceneEffect)
            })
            --
            ~{Combo}
            template(|context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_more()
                        .build_with_items(countries()),
                );
                Ok(EmptySceneEffect)
            })
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
