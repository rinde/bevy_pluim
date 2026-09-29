//! Headless integration tests for `WidgetsPlugin`: a plain `App` built from
//! `MinimalPlugins`, with no window or GPU.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests may abort loudly"
)]

use bevy::MinimalPlugins;
use bevy::input::InputPlugin;
use bevy::picking::DefaultPickingPlugins;
use bevy::prelude::*;
use bevy::render::camera::ClearColor;
use bevy::window::WindowPlugin;
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::select::{Item, ItemValues, Selection, SelectionValue};
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme};

/// `Theme`'s `update_theme` system needs a `ClearColor` resource, and
/// `combo.rs`/`list.rs` need pointer input and picking resources; all
/// normally inserted by the render/window plugins that this headless setup
/// skips.
fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app
}

#[derive(Component)]
struct PanelA;

#[derive(Component)]
struct PanelB;

fn spawn_panel_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(&mut commands, PanelA);
}

fn spawn_panel_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(&mut commands, PanelB);
}

#[test]
fn add_ui_root_inserts_ui_root_on_the_single_root_node() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();

    app.update();

    assert!(app.world().entity(root).contains::<UiRoot>());
}

#[test]
fn a_stack_shows_only_the_selected_child_and_switches_on_selection_change() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);

    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("a"), StackItem::new(create_a)),
            (Item::new("b"), StackItem::new(create_b)),
        ]))
        .id();
    let items = app
        .world()
        .get::<ItemValues>(selection_ref)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    let (item_a, item_b) = (items[0], items[1]);

    let root = app.world_mut().spawn(Node::default()).id();
    let stack = app
        .world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)))
        .id();

    app.update();
    app.update();

    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert!(no_entity_with::<PanelB>(&mut app));
    assert!(
        app.world()
            .entity(stack)
            .get::<Children>()
            .unwrap()
            .contains(&panel_a)
    );
    assert_eq!(node_display(&app, panel_a), Display::Flex);

    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::One(item_b)));

    app.update();
    app.update();

    let panel_b = single_entity_with::<PanelB>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::None);
    assert_eq!(node_display(&app, panel_b), Display::Flex);

    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::One(item_a)));

    app.update();
    app.update();

    assert_eq!(node_display(&app, panel_a), Display::Flex);
    assert_eq!(node_display(&app, panel_b), Display::None);
}

#[test]
fn a_selection_ref_shared_by_bread_crumbs_and_a_stack_stays_in_sync() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);

    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one_or_more().build_with_items([
            (Item::new("a"), StackItem::new(create_a)),
            (Item::new("b"), StackItem::new(create_b)),
        ]))
        .id();
    let items = app
        .world()
        .get::<ItemValues>(selection_ref)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    let (item_a, item_b) = (items[0], items[1]);

    let root = app.world_mut().spawn(Node::default()).id();
    let bread_crumbs = app
        .world_mut()
        .spawn((
            Node::default(),
            BreadCrumbs,
            Selection::reference(selection_ref),
            ChildOf(root),
        ))
        .id();
    let stack = app
        .world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)))
        .id();

    app.update();
    app.update();

    // The selection starts out with only "a" selected: one crumb, and the
    // stack shows the matching panel.
    assert_eq!(
        app.world()
            .entity(bread_crumbs)
            .get::<Children>()
            .unwrap()
            .len(),
        1
    );
    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::Flex);

    // Pushing "b" onto the shared selection grows the crumb trail and
    // switches the stack to show "b" as the most recently selected item.
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::OneOrMore(
            smallvec::smallvec![item_a, item_b],
        )));

    app.update();
    app.update();

    assert_eq!(
        app.world()
            .entity(bread_crumbs)
            .get::<Children>()
            .unwrap()
            .len(),
        2
    );
    let panel_b = single_entity_with::<PanelB>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::None);
    assert_eq!(node_display(&app, panel_b), Display::Flex);

    // Trimming the crumb trail back to "a" (as clicking that crumb would)
    // shrinks the trail and switches the stack back.
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::OneOrMore(
            smallvec::smallvec![item_a],
        )));

    app.update();
    app.update();

    assert_eq!(
        app.world()
            .entity(bread_crumbs)
            .get::<Children>()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(node_display(&app, panel_a), Display::Flex);
    assert_eq!(node_display(&app, panel_b), Display::None);

    let _ = stack;
}

#[test]
fn toggling_theme_light_dark_reaches_a_themed_widget() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let card = app
        .world_mut()
        .spawn((Node::default(), Card, ChildOf(root)))
        .id();

    app.update();

    assert_eq!(
        card_background(&app, card),
        DEFAULT_LIGHT_THEME_COLORS.background_primary
    );

    toggle_theme(&mut app);
    app.update();

    assert_eq!(
        card_background(&app, card),
        DEFAULT_DARK_THEME_COLORS.background_primary
    );

    toggle_theme(&mut app);
    app.update();

    assert_eq!(
        card_background(&app, card),
        DEFAULT_LIGHT_THEME_COLORS.background_primary
    );
}

fn toggle_theme(app: &mut App) {
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .light_dark_toggle();
}

fn node_display(app: &App, entity: Entity) -> Display {
    app.world().entity(entity).get::<Node>().unwrap().display
}

fn card_background(app: &App, entity: Entity) -> Color {
    app.world()
        .entity(entity)
        .get::<BackgroundColor>()
        .unwrap()
        .0
}

fn single_entity_with<T: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<T>>();
    query.single(world).unwrap()
}

fn no_entity_with<T: Component>(app: &mut App) -> bool {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<T>>();
    query.iter(world).next().is_none()
}
