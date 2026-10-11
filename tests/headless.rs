//! Headless integration tests for `WidgetsPlugin`: a plain `App` built from
//! `MinimalPlugins`, with no window or GPU.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests may abort loudly"
)]

use bevy::MinimalPlugins;
use bevy::camera::ClearColor;
use bevy::camera::NormalizedRenderTarget;
use bevy::input::InputPlugin;
use bevy::picking::DefaultPickingPlugins;
use bevy::picking::backend::HitData;
use bevy::picking::events::{Click, Pointer, Press, Release};
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed, Selected};
use bevy::ui_widgets::{Activate, Button, ListBox, ListBoxMultiSelect, ListItem, ValueChange};
use bevy::window::WindowPlugin;
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::link::{Link, LinkState};
use rin_bevy_ui::list::{ItemView, List};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::push_button::{PushButton, PushButtonRefs};
use rin_bevy_ui::select::{
    Item, ItemOf, ItemValues, Selection, SelectionBuilderWithValue, SelectionValue,
};
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
fn a_root_node_aimed_at_another_camera_is_not_the_ui_root() {
    let mut app = headless_app();
    let camera = app.world_mut().spawn_empty().id();
    let foreign_root = app
        .world_mut()
        .spawn((Node::default(), UiTargetCamera(camera)))
        .id();
    let root = app.world_mut().spawn(Node::default()).id();

    app.update();

    assert!(!app.world().entity(foreign_root).contains::<UiRoot>());
    assert!(app.world().entity(root).contains::<UiRoot>());
}

#[test]
fn two_app_root_nodes_are_reported_instead_of_panicking() {
    let mut app = headless_app();
    let first = app.world_mut().spawn(Node::default()).id();
    let second = app.world_mut().spawn(Node::default()).id();

    app.update();

    assert!(app.world().entity(first).contains::<UiRoot>());
    assert!(app.world().entity(second).contains::<UiRoot>());
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

#[test]
fn a_press_on_a_link_restyles_it() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let link = app
        .world_mut()
        .spawn((Link::new("link"), ChildOf(root)))
        .id();
    app.update();
    app.update();

    let before = *app.world().entity(link).get::<BackgroundColor>().unwrap();
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        Location {
            target: NormalizedRenderTarget::None {
                width: 1,
                height: 1,
            },
            position: Vec2::ZERO,
        },
        Press {
            count: 1,
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        },
        link,
    ));
    app.update();

    let after = *app.world().entity(link).get::<BackgroundColor>().unwrap();
    let expected = app
        .world()
        .get_resource::<Theme>()
        .unwrap()
        .colors
        .background_secondary;
    assert_ne!(before.0, expected);
    assert_eq!(after.0, expected);
}

#[test]
fn a_disabled_link_keeps_its_button_and_is_interaction_disabled() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let link = app
        .world_mut()
        .spawn((Link::new("link"), ChildOf(root)))
        .id();
    app.update();
    app.update();
    assert!(app.world().entity(link).contains::<Button>());
    assert!(!app.world().entity(link).contains::<InteractionDisabled>());

    app.world_mut().entity_mut(link).insert(LinkState::Disabled);
    app.update();
    assert!(app.world().entity(link).contains::<Button>());
    assert!(app.world().entity(link).contains::<InteractionDisabled>());

    app.world_mut().entity_mut(link).insert(LinkState::Enabled);
    app.update();
    assert!(app.world().entity(link).contains::<Button>());
    assert!(!app.world().entity(link).contains::<InteractionDisabled>());
}

#[test]
fn a_disabled_link_ignores_the_pointer_and_shows_the_text_colour() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let link = app
        .world_mut()
        .spawn((Link::new("link"), LinkState::Disabled, ChildOf(root)))
        .id();
    app.update();
    app.update();

    let background = |app: &App| app.world().entity(link).get::<BackgroundColor>().unwrap().0;
    let before = background(&app);
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        Location {
            target: NormalizedRenderTarget::None {
                width: 1,
                height: 1,
            },
            position: Vec2::ZERO,
        },
        Press {
            count: 1,
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        },
        link,
    ));
    app.update();

    assert_eq!(background(&app), before);
    let text = app.world().entity(link).get::<Children>().unwrap()[0];
    assert_eq!(
        app.world().entity(text).get::<TextColor>().unwrap().0,
        app.world()
            .get_resource::<Theme>()
            .unwrap()
            .colors
            .text_primary
    );
}

/// The `Activate`s of the button that [`spawn_push_button`] spawned.
#[derive(Resource, Default)]
struct Activations(u32);

/// A push button in a root node, counting its [`Activate`]s in [`Activations`].
fn spawn_push_button(app: &mut App) -> Entity {
    app.init_resource::<Activations>();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((PushButton::from_label("Go"), ChildOf(root)))
        .observe(|_: On<Activate>, mut activations: ResMut<Activations>| activations.0 += 1)
        .id();
    app.update();
    app.update();
    button
}

const fn pointer_location() -> Location {
    Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    }
}

fn press(app: &mut App, target: Entity) {
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Press {
            count: 1,
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        },
        target,
    ));
    app.update();
}

/// Bevy's picking emits `Click` before `Release` when the button comes up.
fn click_and_release(app: &mut App, target: Entity) {
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Click {
            count: 1,
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: std::time::Duration::ZERO,
        },
        target,
    ));
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Release {
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        },
        target,
    ));
    app.update();
}

fn background(app: &App, entity: Entity) -> Color {
    app.world()
        .entity(entity)
        .get::<BackgroundColor>()
        .unwrap()
        .0
}

#[test]
fn a_held_push_button_is_pressed_and_coloured_through_the_root() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);

    press(&mut app, button);

    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert!(app.world().entity(button).contains::<Pressed>());
    assert_eq!(
        background(&app, button),
        colors.selected_hovered_item,
        "Bevy's Button stops Press from bubbling, yet the colour must still follow it"
    );

    click_and_release(&mut app, button);

    assert!(!app.world().entity(button).contains::<Pressed>());
    assert_eq!(background(&app, button), colors.selected_item);
}

#[test]
fn releasing_a_push_button_activates_it_once() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);

    press(&mut app, button);
    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 0);
    click_and_release(&mut app, button);

    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 1);
}

#[test]
fn a_disabled_push_button_is_dimmed_and_does_not_activate() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    app.update();

    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let label = app
        .world()
        .entity(button)
        .get::<PushButtonRefs>()
        .unwrap()
        .label_id;
    let label_color = |app: &App| app.world().entity(label).get::<TextColor>().unwrap().0;
    assert_eq!(label_color(&app), colors.text_disabled);

    press(&mut app, button);
    click_and_release(&mut app, button);
    assert!(!app.world().entity(button).contains::<Pressed>());
    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 0);
    assert_eq!(background(&app, button), colors.background_primary);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    app.update();
    assert_eq!(label_color(&app), colors.text_primary);
}

/// A stepper over a `u8` from 0 to 3, starting at `initial`, returning it and its
/// decrease and increase buttons.
fn spawn_stepper(app: &mut App, initial: u8) -> (Entity, Entity, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let stepper = app
        .world_mut()
        .spawn((
            NumericStepper,
            Value::new_u8(initial, 0, 3, 1),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    let children = app.world().entity(stepper).get::<Children>().unwrap();
    (stepper, children[0], children[2])
}

fn stepper_value(app: &App, stepper: Entity) -> u8 {
    app.world()
        .entity(stepper)
        .get::<Value>()
        .unwrap()
        .as_u8()
        .unwrap()
}

fn activate(app: &mut App, button: Entity) {
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    app.update();
}

fn is_disabled(app: &App, button: Entity) -> bool {
    app.world().entity(button).contains::<InteractionDisabled>()
}

#[test]
fn activating_the_buttons_of_a_numeric_stepper_steps_its_value() {
    let mut app = headless_app();
    let (stepper, decrease, increase) = spawn_stepper(&mut app, 1);

    activate(&mut app, increase);
    assert_eq!(stepper_value(&app, stepper), 2);
    activate(&mut app, decrease);
    activate(&mut app, decrease);
    assert_eq!(stepper_value(&app, stepper), 0);
}

#[test]
fn a_numeric_stepper_button_at_its_bound_is_disabled_until_the_value_moves() {
    let mut app = headless_app();
    let (stepper, decrease, increase) = spawn_stepper(&mut app, 0);

    assert!(is_disabled(&app, decrease), "at the minimum");
    assert!(!is_disabled(&app, increase));

    activate(&mut app, increase);
    assert!(!is_disabled(&app, decrease));
    activate(&mut app, increase);
    activate(&mut app, increase);
    assert_eq!(stepper_value(&app, stepper), 3);
    assert!(is_disabled(&app, increase), "at the maximum");
    assert!(!is_disabled(&app, decrease));

    press(&mut app, increase);
    click_and_release(&mut app, increase);
    assert_eq!(
        stepper_value(&app, stepper),
        3,
        "a disabled button does not step"
    );

    activate(&mut app, decrease);
    assert!(!is_disabled(&app, increase));
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

// ---- List ----

/// A list over a selection of its own on `labels`, in a root node: the list, its
/// item views in the order of the labels, and the selection.
fn spawn_list(
    app: &mut App,
    selection: SelectionBuilderWithValue,
    labels: &[&str],
) -> (Entity, Vec<Entity>, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let items: Vec<_> = labels.iter().map(|label| Item::new(*label)).collect();
    let selection_ref = app
        .world_mut()
        .spawn(selection.build_with_items(items))
        .id();
    let list = spawn_list_on(app, root, selection_ref);
    let views = item_views(app, list);
    (list, views, selection_ref)
}

/// A list sharing the selection of `selection_ref`.
fn spawn_list_on(app: &mut App, root: Entity, selection_ref: Entity) -> Entity {
    let list = app
        .world_mut()
        .spawn((List, Selection::reference(selection_ref), ChildOf(root)))
        .id();
    app.update();
    app.update();
    list
}

fn item_views(app: &App, list: Entity) -> Vec<Entity> {
    app.world()
        .entity(list)
        .get::<Children>()
        .map(|children| {
            children
                .iter()
                .filter(|child| app.world().entity(*child).contains::<ItemView>())
                .collect()
        })
        .unwrap_or_default()
}

fn item_label(app: &App, item: Entity) -> String {
    app.world()
        .entity(item)
        .get::<Item<String>>()
        .unwrap()
        .data
        .clone()
}

/// What the views of `list` that carry [`Selected`] show, in the order of the list.
fn selected_labels(app: &App, list: Entity) -> Vec<String> {
    item_views(app, list)
        .into_iter()
        .filter(|view| app.world().entity(*view).contains::<Selected>())
        .map(|view| {
            let item_ref = app.world().entity(view).get::<ItemView>().unwrap().item_ref;
            item_label(app, item_ref)
        })
        .collect()
}

/// The items that `selection_ref` selects, as their labels.
fn selection_labels(app: &App, selection_ref: Entity) -> Vec<String> {
    let Selection::Value(selection) = app
        .world()
        .entity(selection_ref)
        .get::<Selection>()
        .unwrap()
    else {
        return Vec::new();
    };
    selection.iter().map(|item| item_label(app, item)).collect()
}

/// What the `ListBox` sends when it picks `row`: a click on a row that is not
/// `Selected`, or Enter on the active row.
fn list_box_picks(app: &mut App, list: Entity, row: Entity) {
    app.world_mut().trigger(ValueChange::<Entity> {
        source: list,
        value: row,
        is_final: true,
    });
    app.update();
}

/// A click on `row` as the picking sends it: `Click`, then `Release`.
fn click_row(app: &mut App, row: Entity) {
    click_and_release(app, row);
    app.update();
}

#[test]
fn a_list_is_a_list_box_and_its_items_are_list_items() {
    let mut app = headless_app();
    let (list, views, _) = spawn_list(&mut app, Selection::builder().one(), &["a", "b"]);

    assert!(app.world().entity(list).contains::<ListBox>());
    assert_eq!(views.len(), 2);
    for view in views {
        assert!(app.world().entity(view).contains::<ListItem>());
    }
}

#[test]
fn the_items_of_the_selection_are_selected_and_the_others_are_not() {
    let mut app = headless_app();
    let (list, views, selection_ref) =
        spawn_list(&mut app, Selection::builder().one(), &["a", "b", "c"]);
    assert_eq!(selected_labels(&app, list), ["a"]);

    // Selected follows the selection when something else changes it as well.
    let c = app
        .world()
        .entity(views[2])
        .get::<ItemView>()
        .unwrap()
        .item_ref;
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::One(c)));
    app.update();

    assert_eq!(selected_labels(&app, list), ["c"]);
}

#[test]
fn lists_sharing_a_selection_all_show_it_as_selected() {
    let mut app = headless_app();
    let (first, _, selection_ref) =
        spawn_list(&mut app, Selection::builder().zero_or_more(), &["a", "b"]);
    let root = app.world_mut().spawn(Node::default()).id();
    let second = spawn_list_on(&mut app, root, selection_ref);
    assert!(selected_labels(&app, second).is_empty());

    let views = item_views(&app, first);
    list_box_picks(&mut app, first, views[1]);

    assert_eq!(selected_labels(&app, first), ["b"]);
    assert_eq!(selected_labels(&app, second), ["b"]);
}

#[test]
fn a_list_box_multi_select_is_on_a_list_over_a_selection_of_many() {
    let mut app = headless_app();
    for (selection, multiple) in [
        (Selection::builder().one(), false),
        (Selection::builder().zero_or_one(), false),
        (Selection::builder().one_or_more(), true),
        (Selection::builder().zero_or_more(), true),
    ] {
        let (list, _, _) = spawn_list(&mut app, selection, &["a", "b"]);

        assert_eq!(
            app.world().entity(list).contains::<ListBoxMultiSelect>(),
            multiple
        );
    }
}

#[test]
fn clicking_an_item_selects_it_and_replaces_the_selection_of_a_one_list() {
    let mut app = headless_app();
    let (list, views, selection_ref) =
        spawn_list(&mut app, Selection::builder().one(), &["a", "b"]);

    click_row(&mut app, views[1]);

    assert_eq!(selection_labels(&app, selection_ref), ["b"]);
    assert_eq!(selected_labels(&app, list), ["b"]);

    // The selected item cannot be deselected.
    click_row(&mut app, views[1]);

    assert_eq!(selection_labels(&app, selection_ref), ["b"]);
    assert_eq!(selected_labels(&app, list), ["b"]);
}

#[test]
fn clicking_the_selected_item_of_a_zero_or_one_list_deselects_it() {
    let mut app = headless_app();
    let (list, views, selection_ref) =
        spawn_list(&mut app, Selection::builder().zero_or_one(), &["a", "b"]);

    click_row(&mut app, views[0]);
    assert_eq!(selected_labels(&app, list), ["a"]);

    click_row(&mut app, views[1]);
    assert_eq!(selected_labels(&app, list), ["b"]);

    click_row(&mut app, views[1]);

    assert!(selection_labels(&app, selection_ref).is_empty());
    assert!(selected_labels(&app, list).is_empty());
}

#[test]
fn clicking_a_selected_item_that_is_not_the_first_deselects_it() {
    // The `ListBox` sends a change for every selected row but the first one, and
    // the release of the same click deselects it: the click must not undo itself.
    let mut app = headless_app();
    let (list, views, selection_ref) = spawn_list(
        &mut app,
        Selection::builder().zero_or_more(),
        &["a", "b", "c"],
    );
    for view in &views[..2] {
        click_row(&mut app, *view);
    }
    assert_eq!(selected_labels(&app, list), ["a", "b"]);

    click_row(&mut app, views[1]);

    assert_eq!(selection_labels(&app, selection_ref), ["a"]);
    assert_eq!(selected_labels(&app, list), ["a"]);
}

#[test]
fn the_last_selected_item_of_a_one_or_more_list_stays_selected() {
    let mut app = headless_app();
    let (list, views, selection_ref) =
        spawn_list(&mut app, Selection::builder().one_or_more(), &["a", "b"]);
    assert_eq!(selected_labels(&app, list), ["a"]);

    click_row(&mut app, views[0]);

    assert_eq!(selection_labels(&app, selection_ref), ["a"]);
    assert_eq!(selected_labels(&app, list), ["a"]);

    click_row(&mut app, views[1]);
    click_row(&mut app, views[0]);

    assert_eq!(selection_labels(&app, selection_ref), ["b"]);
    assert_eq!(selected_labels(&app, list), ["b"]);
}

#[test]
fn what_the_list_box_picks_is_selected_as_far_as_the_selection_allows() {
    // Enter or Space on the active row sends the same change as a click.
    let mut app = headless_app();
    let (list, views, selection_ref) =
        spawn_list(&mut app, Selection::builder().one_or_more(), &["a", "b"]);

    list_box_picks(&mut app, list, views[1]);

    assert_eq!(selection_labels(&app, selection_ref), ["a", "b"]);
    assert_eq!(selected_labels(&app, list), ["a", "b"]);

    // A change for a row that is not an item of the list is ignored.
    let stranger = app.world_mut().spawn_empty().id();
    list_box_picks(&mut app, list, stranger);

    assert_eq!(selection_labels(&app, selection_ref), ["a", "b"]);
}

#[test]
fn an_item_added_to_the_items_of_a_selected_list_is_not_selected() {
    let mut app = headless_app();
    let (list, _, selection_ref) =
        spawn_list(&mut app, Selection::builder().zero_or_more(), &["a"]);
    assert_eq!(item_views(&app, list).len(), 1);

    app.world_mut()
        .spawn((Item::new("b"), ItemOf(selection_ref)));
    app.update();
    app.update();

    assert_eq!(item_views(&app, list).len(), 2);
    assert!(selected_labels(&app, list).is_empty());
}
