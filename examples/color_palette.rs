//! `ColorPalette` and `ColorPaletteCombo` over `tailwind::COLORS`.

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::ecs::template::TemplateContext;
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::list::ListLayout;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::tailwind::COLORS;
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

    let selection_ref = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(color_palette()),
        )
        .id();

    let selection_ref2 = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items_ref(selection_ref),
        )
        .id();

    // Root container
    commands.spawn_scene(bsn! {
        Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
        }
        Children [
            ~{Selection::reference(selection_ref)}
            ~{ColorPalette}
            ~{ListLayout::VerticalGrid { columns: 11 }}
            --
            Node {
                width: px(430.0),
                height: px(300.0),
            }
            // `ScrollArea::bundle` is a bundle, not a scene, so `template` inserts it; its
            // contents are scenes, applied to the entities its contents spawner spawns.
            template(move |context| {
                Ok(insert_after_scene(context, ScrollArea::bundle(
                    move |parent: &mut ChildSpawner| {
                        let palette = bsn! {
                            Pickable {
                                should_block_lower: false,
                            }
                            ~{ColorPalette}
                            ~{ListLayout::VerticalGrid { columns: 11 }}
                            ~{Selection::reference(selection_ref)}
                        };
                        apply(parent, palette);
                    },
                    true,
                    true,
                )))
            })
            --
            Node {
                width: px(230.0),
                height: px(600.0),
            }
            template(move |context| {
                Ok(insert_after_scene(context, ScrollArea::bundle(
                    move |parent: &mut ChildSpawner| {
                        let palette = bsn! {
                            Pickable {
                                should_block_lower: false,
                            }
                            ~{ColorPalette}
                            ~{ListLayout::VerticalGrid { columns: 11 }}
                            // `Selection::builder()` gives a bundle, which `template` inserts.
                            template(move |context| {
                                context.entity.insert(
                                    Selection::builder()
                                        .zero_or_one()
                                        .build_with_items_ref(selection_ref),
                                );
                                Ok(EmptySceneEffect)
                            })
                        };
                        apply(parent, palette);
                    },
                    true,
                    true,
                )))
            })
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref2)}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            ~{Selection::reference(selection_ref2)}
            --
            ColorPaletteCombo
            ~{ListLayout::VerticalGrid { columns: 11 }}
            template(move |context| {
                context.entity.insert(
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items_ref(selection_ref),
                );
                Ok(EmptySceneEffect)
            })
            --
            Node {
                width: px(100.0),
                height: px(100.0),
            }
            template(move |context| {
                Ok(insert_after_scene(context, ScrollArea::bundle(
                    move |parent: &mut ChildSpawner| {
                        let combo = bsn! {
                            Pickable {
                                should_block_lower: false,
                            }
                            ColorPaletteCombo
                            ~{ListLayout::VerticalGrid { columns: 11 }}
                            ~{Selection::reference(selection_ref)}
                        };
                        apply(parent, combo);
                        let card = bsn! {
                            Node {
                                min_width: px(50.0),
                                width: px(100.0),
                                height: px(100.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                            }
                            ~{Card}
                            Pickable {
                                should_block_lower: false,
                            }
                        };
                        apply(parent, card);
                    },
                    true,
                    true,
                )))
            })
            --
            Node
            Children [
                ~{PushButton::from_label("Change theme")}
                on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
                --
                ~{PushButton::from_label("+")}
                on(add_combo)
                --
                ~{PushButton::from_label("-")}
                on(remove_combo)
            ]
            --
            Node
            ExtraComponentsArea
        ]
    });
}

/// Spawns `scene` as a child of the entity `parent` spawns for.
fn apply(parent: &mut ChildSpawner, scene: impl Scene) {
    if let Err(error) = parent.spawn_empty().apply_scene(scene) {
        error!("cannot spawn a scroll area's contents: {error:?}");
    }
}

fn color_palette() -> Vec<Item<Color>> {
    COLORS.iter().map(|c| Item { data: c.color }).collect()
}

#[derive(Component, Default, Clone)]
struct ExtraComponentsArea;

fn add_combo(
    _: On<Activate>,
    area: Single<Entity, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    commands.entity(*area).with_children(|parent| {
        parent.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::builder()
                .zero_or_one()
                .build_with_items(color_palette()),
        ));
    });
}

fn remove_combo(
    _: On<Activate>,
    area: Single<&Children, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    if let Some(last) = area.iter().last() {
        commands.entity(last).despawn();
    }
}
