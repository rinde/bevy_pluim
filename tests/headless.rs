//! Headless integration tests for `WidgetsPlugin`: a plain `App` built from
//! `MinimalPlugins`, with no window or GPU.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests may abort loudly"
)]

use bevy::MinimalPlugins;
use bevy::a11y::AccessibilityNode;
use bevy::asset::{AssetPlugin, HandleTemplate};
use bevy::camera::ClearColor;
use bevy::camera::NormalizedRenderTarget;
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::system::RunSystemOnce;
use bevy::input::InputPlugin;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::mesh::MeshPlugin;
use bevy::picking::DefaultPickingPlugins;
use bevy::picking::backend::HitData;
use bevy::picking::events::{
    Pointer, PointerClick, PointerOut, PointerOver, PointerPress, PointerRelease,
};
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::prelude::*;
use bevy::text::{DEFAULT_REM_SIZE_PX, LetterSpacing};
use bevy::ui::{
    ComputedNode, FixedNode, InteractionDisabled, Pressed, Selected, UiGlobalTransform,
};
use bevy::ui_widgets::{
    Activate, ActiveDescendant, Button, ControlOrientation, ListBox, ListBoxMultiSelect, ListItem,
    Scrollbar, ScrollbarThumb, SelectedTab, SetSliderValue, SliderValue, SliderValueChange, Tab,
    TabList, ValueChange,
};
use bevy::window::{PrimaryWindow, Window, WindowPlugin, WindowTheme, WindowThemeChanged};
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::badge::{Badge, BadgeTone};
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::{ColorPaletteCombo, State as PaletteComboState};
use rin_bevy_ui::color_push_button::{ColorPushButton, ColorSquareRef};
use rin_bevy_ui::combo::{Combo, ComboState, ComboSubRefs};
use rin_bevy_ui::divider::Divider;
use rin_bevy_ui::filter_chips::{Chip, ChipRefs, ChipView, FilterChips};
use rin_bevy_ui::float::Float;
use rin_bevy_ui::gallery::{Gallery, GalleryCard, GalleryCardRefs, GalleryCardView};
use rin_bevy_ui::icon::{Icon, IconColor};
use rin_bevy_ui::icon_button::{IconButton, IconButtonRefs, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::kbd::Kbd;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::light_dark_toggle::{
    LightDarkOption, LightDarkToggle, SystemTheme, ThemePreference,
};
use rin_bevy_ui::link::{Link, LinkState};
use rin_bevy_ui::list::{
    CornerStyle, ItemNote, ItemView, ItemViewCreator, ItemViewSpawner, List, ListLayout,
};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value, ValueRef};
use rin_bevy_ui::panel::{Panel, PanelClosed, PanelRefs};
use rin_bevy_ui::pill::{Pill, PillTone};
use rin_bevy_ui::push_button::{
    ButtonSize, IconPlacement, NoDefaultHover, PushButton, PushButtonRefs,
};
use rin_bevy_ui::scroll_area::{ClearOfCorners, ScrollArea, ScrollBar, ScrollContents};
use rin_bevy_ui::segmented_control::{
    ItemIcon, Segment, SegmentIcon, SegmentIndicator, SegmentLabels, SegmentedControl,
};
use rin_bevy_ui::select::{
    Item, ItemOf, ItemValues, ItemsRef, Selection, SelectionBuilderWithValue, SelectionParam,
    SelectionValue,
};
use rin_bevy_ui::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use rin_bevy_ui::slider::{Slider, SliderRefs};
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::tab_bar::{TabBar, TabBarTab};
use rin_bevy_ui::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme, ThemeMode};
use rin_bevy_ui::toolbar::{
    Tool, Toolbar, ToolbarButton, ToolbarButtonRefs, ToolbarLabels, ToolbarStyle,
};
use rin_bevy_ui::tooltip::{Tooltip, TooltipPopup};

#[path = "../examples/hud/ui/mod.rs"]
mod hud_ui;

#[path = "../examples/overview.rs"]
#[expect(
    dead_code,
    unreachable_pub,
    reason = "the example's own `main` is not run by the test, and its plugin is `pub` for it"
)]
mod overview;

/// `Theme`'s `update_theme` system needs a `ClearColor` resource, and
/// `combo.rs`/`list.rs` need pointer input and picking resources; all
/// normally inserted by the render/window plugins that this headless setup
/// skips. `focus.rs` reads `InheritedVisibility`, which `VisibilityPlugin`
/// propagates; its bounds systems need the mesh assets. The `bevy_ui_widgets`
/// `Scrollbar` of `scroll_area.rs` reads `UiScale` on every press, which `UiPlugin`
/// would insert.
fn headless_app() -> App {
    headless_app_with(|_| {})
}

/// [`headless_app`] with `extra` run on the app before it is finished.
fn headless_app_with(extra: impl FnOnce(&mut App)) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins((AssetPlugin::default(), MeshPlugin, VisibilityPlugin));
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.init_resource::<ClearColor>();
    app.init_resource::<UiScale>();
    extra(&mut app);
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    app
}

#[derive(Component, Clone, Default)]
struct PanelA;

#[derive(Component, Clone, Default)]
struct PanelB;

fn spawn_panel_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(&mut commands, PanelA);
}

fn spawn_panel_scene(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            PanelA
            Children [ PanelB ]
        },
    );
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
fn a_node_parented_after_its_node_was_added_is_not_a_ui_root() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let child = app.world_mut().spawn(Node::default()).id();
    assert!(app.world().entity(child).contains::<UiRoot>());

    app.world_mut().entity_mut(child).insert(ChildOf(root));
    app.update();

    assert!(!app.world().entity(child).contains::<UiRoot>());
    assert!(app.world().entity(root).contains::<UiRoot>());
    let roots = app
        .world_mut()
        .query_filtered::<(), With<UiRoot>>()
        .iter(app.world())
        .count();
    assert_eq!(roots, 1);
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
fn a_stack_child_can_be_spawned_from_a_scene() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.add_plugins((AssetPlugin::default(), bevy::scene::ScenePlugin));
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    let create = app.world_mut().register_system(spawn_panel_scene);

    let selection_ref = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items([(Item::new("a"), StackItem::new(create))]),
        )
        .id();
    let root = app.world_mut().spawn(Node::default()).id();
    let stack = app
        .world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)))
        .id();

    for _ in 0..4 {
        app.update();
    }

    let panel = single_entity_with::<PanelA>(&mut app);
    let stack_children = app.world().entity(stack).get::<Children>().unwrap();
    assert!(stack_children.contains(&panel));
    assert_eq!(node_display(&app, panel), Display::Flex);
    // The scene's own children are below the stack child.
    let panel_b = single_entity_with::<PanelB>(&mut app);
    assert!(
        app.world()
            .entity(panel)
            .get::<Children>()
            .unwrap()
            .contains(&panel_b)
    );
    assert_eq!(ui_root_count(&mut app), 1);
}

fn spawn_real_panel(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(&mut commands, Panel::new("Trains"));
}

fn app_with_stack_child(create: impl FnOnce(&mut World) -> StackItem) -> App {
    let mut app = headless_app_with(|app| {
        app.add_plugins(bevy::scene::ScenePlugin);
    });
    let item = create(app.world_mut());
    let selection_ref = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items([(Item::new("a"), item)]),
        )
        .id();
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)));
    for _ in 0..4 {
        app.update();
    }
    app
}

#[test]
fn a_panel_spawned_through_a_stack_keeps_its_own_width() {
    let mut app =
        app_with_stack_child(|world| StackItem::new(world.register_system(spawn_real_panel)));
    let panel = single_entity_with::<Panel>(&mut app);
    let width = app.world().entity(panel).get::<Node>().unwrap().width;
    assert_eq!(width, Val::Px(rin_bevy_ui::panel::PANEL_WIDTH_PX));
}

#[test]
fn a_panel_scene_with_an_invalid_maximum_body_height_fails_to_build_and_spawns_no_panel() {
    for height in [f32::NAN, f32::INFINITY, 0.0, -5.0] {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(WindowPlugin {
            primary_window: None,
            ..default()
        });
        app.add_plugins(InputPlugin);
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins((AssetPlugin::default(), bevy::scene::ScenePlugin));
        app.init_resource::<ClearColor>();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(WidgetsPlugin);
        app.finish();
        app.cleanup();
        let result = app
            .world_mut()
            .spawn_scene(bsn! { Panel { title: "Bad", max_body_height: {height} } });
        assert!(result.is_err(), "height {height} should not build");
        for _ in 0..3 {
            app.update();
        }
        let world = app.world_mut();
        assert_eq!(world.query::<&Panel>().iter(world).count(), 0);
        assert_eq!(world.query::<&PanelRefs>().iter(world).count(), 0);
    }
}

fn spawn_real_panel_scene(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(&mut commands, bsn! { Panel { title: "Trains" } });
}

#[test]
fn a_panel_scene_spawned_through_a_stack_keeps_its_own_width() {
    let mut app =
        app_with_stack_child(|world| StackItem::new(world.register_system(spawn_real_panel_scene)));
    let panel = single_entity_with::<Panel>(&mut app);
    let width = app.world().entity(panel).get::<Node>().unwrap().width;
    assert_eq!(width, Val::Px(rin_bevy_ui::panel::PANEL_WIDTH_PX));
}

#[test]
fn a_stack_child_without_a_node_of_its_own_gets_a_default_one() {
    let mut app =
        app_with_stack_child(|world| StackItem::new(world.register_system(spawn_panel_a)));
    let child = single_entity_with::<PanelA>(&mut app);
    assert_eq!(
        app.world().entity(child).get::<Node>().unwrap().width,
        Node::default().width
    );
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

fn item_entities_of(app: &App, owner: Entity) -> Vec<Entity> {
    app.world()
        .get::<ItemValues>(owner)
        .unwrap()
        .iter()
        .collect()
}

fn count_entities_with<C: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<(), With<C>>().iter(world).count()
}

fn select_one(app: &mut App, selection_ref: Entity, item: Entity) {
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::One(item)));
    app.update();
    app.update();
}

#[test]
fn stack_item_accessors_tell_a_creator_from_a_reference() {
    let mut app = headless_app();
    let create = app.world_mut().register_system(spawn_panel_a);

    let direct = StackItem::new(create);
    assert_eq!(direct.creator(), Some(create));
    assert_eq!(direct.reference_id(), None);

    let reusable = StackItem::new_with_reference(7, create);
    assert_eq!(reusable.creator(), Some(create));
    assert_eq!(reusable.reference_id(), Some(7));

    let reference = StackItem::new_reference(7);
    assert_eq!(reference.creator(), None);
    assert_eq!(reference.reference_id(), Some(7));
}

#[test]
fn a_stack_reference_reuses_the_creator_and_the_child_of_its_item() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);
    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("a"), StackItem::new_with_reference(0, create_a)),
            (Item::new("b"), StackItem::new(create_b)),
            (Item::new("a again"), StackItem::new_reference(0)),
        ]))
        .id();
    let items = item_entities_of(&app, selection_ref);
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)));
    app.update();
    app.update();

    let panel_a = single_entity_with::<PanelA>(&mut app);
    select_one(&mut app, selection_ref, items[1]);
    let panel_b = single_entity_with::<PanelB>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::None);

    // The reference shows the child its creator's item already made; nothing new is spawned.
    select_one(&mut app, selection_ref, items[2]);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 1);
    assert_eq!(node_display(&app, panel_a), Display::Flex);
    assert_eq!(node_display(&app, panel_b), Display::None);

    // And back to the item itself: still the one child.
    select_one(&mut app, selection_ref, items[1]);
    select_one(&mut app, selection_ref, items[0]);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 1);
    assert_eq!(node_display(&app, panel_a), Display::Flex);
}

#[test]
fn a_stack_reference_selected_first_creates_the_child_with_the_referenced_creator() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);
    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("b"), StackItem::new(create_b)),
            (Item::new("a"), StackItem::new_with_reference(0, create_a)),
            (Item::new("a again"), StackItem::new_reference(0)),
        ]))
        .id();
    let items = item_entities_of(&app, selection_ref);
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)));
    app.update();
    app.update();
    assert_eq!(count_entities_with::<PanelA>(&mut app), 0);

    select_one(&mut app, selection_ref, items[2]);
    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::Flex);
    select_one(&mut app, selection_ref, items[1]);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 1);
}

#[test]
fn a_stack_reference_without_a_matching_creator_hides_the_current_child() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("a"), StackItem::new(create_a)),
            (Item::new("orphan"), StackItem::new_reference(3)),
        ]))
        .id();
    let items = item_entities_of(&app, selection_ref);
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)));
    app.update();
    app.update();
    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::Flex);

    select_one(&mut app, selection_ref, items[1]);
    assert_eq!(node_display(&app, panel_a), Display::None, "no stale panel");

    // The stack still works afterwards.
    select_one(&mut app, selection_ref, items[0]);
    assert_eq!(node_display(&app, panel_a), Display::Flex);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 1);
}

#[test]
fn a_stack_over_a_pending_selection_shows_nothing_until_the_first_item_arrives() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let selection_ref = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(Vec::<(Item<String>, StackItem)>::new()),
        )
        .id();
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)));
    app.update();
    app.update();
    assert_eq!(count_entities_with::<PanelA>(&mut app), 0);

    app.world_mut().spawn((
        Item::new("a"),
        StackItem::new(create_a),
        ItemOf(selection_ref),
    ));
    app.update();
    app.update();
    app.update();
    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::Flex);
}

#[test]
fn a_bread_crumb_trail_marks_the_last_crumb_current_when_an_item_without_text_is_skipped() {
    let mut app = headless_app();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one_or_more()
                .build_with_items([(Item::new("a"),), (Item::new("b"),)]),
        )
        .id();
    // A third item has no text, so it gets no crumb.
    let no_text = app
        .world_mut()
        .spawn(Item::<Color> { data: Color::WHITE })
        .id();
    let items = app
        .world()
        .get::<ItemValues>(selection)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();

    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut().spawn((
        Node::default(),
        BreadCrumbs,
        Selection::reference(selection),
        ChildOf(root),
    ));
    app.update();

    app.world_mut()
        .entity_mut(selection)
        .insert(Selection::Value(SelectionValue::OneOrMore(
            smallvec::smallvec![items[0], items[1], no_text],
        )));
    app.update();
    app.update();

    let world = app.world_mut();
    let mut links = world.query::<(&Link, &LinkState)>();
    let states = links
        .iter(world)
        .map(|(_, state)| *state)
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 2);
    assert_eq!(
        states
            .iter()
            .filter(|state| matches!(state, LinkState::Disabled))
            .count(),
        1,
        "exactly the last crumb is current"
    );
}

#[test]
fn a_despawned_stack_does_not_react_to_its_selection_changing() {
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
    let root = app.world_mut().spawn(Node::default()).id();
    let stack = app
        .world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)))
        .id();
    app.update();
    app.update();

    app.world_mut().entity_mut(stack).despawn();
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::One(items[1])));
    app.update();
    app.update();

    assert!(no_entity_with::<PanelB>(&mut app));
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
fn a_divider_has_the_divider_colour_and_follows_a_theme_change() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let horizontal = app
        .world_mut()
        .spawn((Divider::horizontal(), ChildOf(root)))
        .id();
    let vertical = app
        .world_mut()
        .spawn((Divider::vertical(), ChildOf(root)))
        .id();
    app.update();
    app.update();

    for divider in [horizontal, vertical] {
        assert_eq!(
            background(&app, divider),
            DEFAULT_LIGHT_THEME_COLORS.divider
        );
    }
    let node = app.world().get::<Node>(horizontal).unwrap();
    assert_eq!((node.height, node.width), (Val::Px(1.0), Val::Auto));
    let node = app.world().get::<Node>(vertical).unwrap();
    assert_eq!((node.width, node.height), (Val::Px(1.0), Val::Auto));

    toggle_theme(&mut app);
    app.update();

    for divider in [horizontal, vertical] {
        assert_eq!(background(&app, divider), DEFAULT_DARK_THEME_COLORS.divider);
    }
}

#[test]
fn a_kbd_shows_its_key_with_a_border_and_follows_a_theme_change() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let kbd = app.world_mut().spawn((Kbd::new("Esc"), ChildOf(root))).id();
    app.update();
    app.update();

    let state = |app: &App| {
        let e = app.world().entity(kbd);
        (
            e.get::<Text>().unwrap().0.clone(),
            e.get::<BorderColor>().unwrap().top,
            e.get::<TextColor>().unwrap().0,
        )
    };
    assert_eq!(
        state(&app),
        (
            "Esc".to_string(),
            DEFAULT_LIGHT_THEME_COLORS.border,
            DEFAULT_LIGHT_THEME_COLORS.text_secondary
        )
    );

    toggle_theme(&mut app);
    app.update();

    assert_eq!(
        state(&app),
        (
            "Esc".to_string(),
            DEFAULT_DARK_THEME_COLORS.border,
            DEFAULT_DARK_THEME_COLORS.text_secondary
        )
    );
}

#[test]
fn an_icon_is_tinted_with_the_text_colour_and_follows_a_theme_change() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let icon = app
        .world_mut()
        .spawn((Icon::bundle(Handle::default(), 18.0), ChildOf(root)))
        .id();
    let active = app
        .world_mut()
        .spawn((
            Icon::bundle_with_color(Handle::default(), 24.0, IconColor::AccentText),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();

    let tint = |app: &App, e: Entity| app.world().get::<ImageNode>(e).unwrap().color;
    assert_eq!(tint(&app, icon), DEFAULT_LIGHT_THEME_COLORS.text_primary);
    assert_eq!(tint(&app, active), DEFAULT_LIGHT_THEME_COLORS.accent_text);
    let node = app.world().get::<Node>(icon).unwrap();
    assert_eq!((node.width, node.height), (Val::Px(18.0), Val::Px(18.0)));

    toggle_theme(&mut app);
    app.update();

    assert_eq!(tint(&app, icon), DEFAULT_DARK_THEME_COLORS.text_primary);
    assert_eq!(tint(&app, active), DEFAULT_DARK_THEME_COLORS.accent_text);

    app.world_mut().entity_mut(icon).insert(Icon {
        color: IconColor::AccentSubtleText,
    });
    app.update();
    assert_eq!(
        tint(&app, icon),
        DEFAULT_DARK_THEME_COLORS.accent_subtle_text
    );
}

#[test]
fn an_icon_button_spawns_a_sized_icon_and_follows_its_image_and_name() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let regular = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Pause", IconButtonSize::Regular),
            ChildOf(root),
        ))
        .id();
    let small = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Close", IconButtonSize::Small),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();

    let icon_of = |app: &App, e: Entity| app.world().get::<IconButtonRefs>(e).unwrap().icon();
    let side = |app: &App, e: Entity| {
        let node = app.world().get::<Node>(e).unwrap();
        (node.width, node.height)
    };
    for (button, button_px, icon_px) in [(regular, 44.0, 22.0), (small, 30.0, 16.0)] {
        assert_eq!(side(&app, button), (Val::Px(button_px), Val::Px(button_px)));
        let icon = icon_of(&app, button);
        assert_eq!(side(&app, icon), (Val::Px(icon_px), Val::Px(icon_px)));
        assert_eq!(app.world().get::<ChildOf>(icon).unwrap().parent(), button);
    }
    assert_eq!(
        background(&app, regular),
        DEFAULT_LIGHT_THEME_COLORS.background_primary
    );
    let tint = |app: &App, e: Entity| app.world().get::<ImageNode>(icon_of(app, e)).unwrap().color;
    assert_eq!(tint(&app, regular), DEFAULT_LIGHT_THEME_COLORS.text_primary);
    assert_eq!(tint(&app, small), DEFAULT_LIGHT_THEME_COLORS.text_secondary);
    let label = |app: &App, e: Entity| {
        app.world()
            .get::<AccessibilityNode>(e)
            .unwrap()
            .label()
            .map(str::to_owned)
    };
    assert_eq!(label(&app, regular).as_deref(), Some("Pause"));

    let play: Handle<Image> = bevy::asset::uuid_handle!("0b6c1d2e-3f40-4a51-8b62-7c83d94ea5f6");
    app.world_mut().entity_mut(regular).insert(IconButton::new(
        play.clone(),
        "Play",
        IconButtonSize::Regular,
    ));
    app.update();
    app.update();

    let icon = icon_of(&app, regular);
    assert_eq!(app.world().get::<ImageNode>(icon).unwrap().image, play);
    assert_eq!(label(&app, regular).as_deref(), Some("Play"));
    assert_eq!(app.world().get::<Children>(regular).unwrap().len(), 1);
}

#[test]
fn activating_a_toggle_icon_button_flips_its_state_and_restyles_it() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let toggle = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Pause", IconButtonSize::Regular),
            ToggleState(false),
            ChildOf(root),
        ))
        .id();
    let plain = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Close", IconButtonSize::Regular),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();

    let colors = DEFAULT_LIGHT_THEME_COLORS;
    let tint = |app: &App, e: Entity| {
        let icon = app.world().get::<IconButtonRefs>(e).unwrap().icon();
        app.world().get::<ImageNode>(icon).unwrap().color
    };
    let state = |app: &App, e: Entity| app.world().get::<ToggleState>(e).copied();
    let bar_display = |app: &App, e: Entity| {
        let children = app.world().get::<Children>(e).unwrap();
        let bar = children[1];
        app.world().get::<Node>(bar).unwrap().display
    };
    assert_eq!(background(&app, toggle), colors.background_primary);
    assert_eq!(bar_display(&app, toggle), Display::None);

    app.world_mut().trigger(Activate { entity: toggle });
    app.update();
    app.update();
    assert_eq!(state(&app, toggle), Some(ToggleState(true)));
    assert_eq!(background(&app, toggle), colors.accent_subtle);
    assert_eq!(tint(&app, toggle), colors.accent_subtle_text);
    assert_eq!(bar_display(&app, toggle), Display::Flex);
    let bar = app.world().get::<Children>(toggle).unwrap()[1];
    assert_eq!(
        app.world().get::<BackgroundColor>(bar).unwrap().0,
        colors.accent_subtle_bar
    );

    app.world_mut().trigger(Activate { entity: toggle });
    app.update();
    app.update();
    assert_eq!(state(&app, toggle), Some(ToggleState(false)));
    assert_eq!(background(&app, toggle), colors.background_primary);
    assert_eq!(bar_display(&app, toggle), Display::None);

    // Set from outside.
    app.world_mut().entity_mut(toggle).insert(ToggleState(true));
    app.update();
    app.update();
    assert_eq!(background(&app, toggle), colors.accent_subtle);

    // A button without a ToggleState stays momentary.
    app.world_mut().trigger(Activate { entity: plain });
    app.update();
    app.update();
    assert_eq!(state(&app, plain), None);
    assert_eq!(app.world().get::<Children>(plain).unwrap().len(), 1);
}

#[test]
fn activating_a_toggle_push_button_flips_its_state_restyles_it_and_sets_its_toggled_state() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let toggle = app
        .world_mut()
        .spawn((
            PushButton::from_label("Grid"),
            ToggleState(false),
            ChildOf(root),
        ))
        .id();
    let plain = app
        .world_mut()
        .spawn((PushButton::from_label("Plain"), ChildOf(root)))
        .id();
    app.update();
    app.update();

    let colors = DEFAULT_LIGHT_THEME_COLORS;
    let state = |app: &App, e: Entity| app.world().get::<ToggleState>(e).copied();
    let label_color = |app: &App, e: Entity| {
        let label = app.world().get::<PushButtonRefs>(e).unwrap().label();
        app.world().get::<TextColor>(label).unwrap().0
    };
    let toggled = |app: &App, e: Entity| app.world().get::<AccessibilityNode>(e).unwrap().toggled();
    assert_eq!(background(&app, toggle), colors.background_primary);
    assert_eq!(label_color(&app, toggle), colors.text_primary);
    assert_eq!(toggled(&app, toggle), Some(accesskit::Toggled::False));
    assert_eq!(toggled(&app, plain), None);

    app.world_mut().trigger(Activate { entity: toggle });
    app.update();
    app.update();
    assert_eq!(state(&app, toggle), Some(ToggleState(true)));
    assert_eq!(background(&app, toggle), colors.accent_subtle);
    assert_eq!(label_color(&app, toggle), colors.accent_subtle_text);
    assert_eq!(toggled(&app, toggle), Some(accesskit::Toggled::True));

    app.world_mut().trigger(Activate { entity: toggle });
    app.update();
    app.update();
    assert_eq!(state(&app, toggle), Some(ToggleState(false)));
    assert_eq!(background(&app, toggle), colors.background_primary);
    assert_eq!(toggled(&app, toggle), Some(accesskit::Toggled::False));

    // A button without a ToggleState stays momentary.
    app.world_mut().trigger(Activate { entity: plain });
    app.update();
    app.update();
    assert_eq!(state(&app, plain), None);
    assert_eq!(background(&app, plain), colors.background_primary);
}

/// Says the pointer moves onto (`over`) or off `target`, as picking would.
fn pointer_over_or_out(app: &mut App, target: Entity, over: bool) {
    let hit = HitData::new(Entity::PLACEHOLDER, 0.0, None, None);
    if over {
        app.world_mut().trigger(PointerOver {
            entity: target,
            pointer: Pointer::new(PointerId::Mouse, pointer_location()),
            hit,
        });
    } else {
        app.world_mut().trigger(PointerOut {
            entity: target,
            pointer: Pointer::new(PointerId::Mouse, pointer_location()),
            hit,
        });
    }
    app.update();
}

#[test]
fn an_icon_button_shows_the_hover_colour_under_the_pointer_and_the_press_colour_when_held() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Pause", IconButtonSize::Regular),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, button), colors.background_primary);

    pointer_over_or_out(&mut app, button, true);
    assert_eq!(background(&app, button), colors.hovered_item);

    press(&mut app, button);
    assert_eq!(background(&app, button), colors.selected_hovered_item);

    click_and_release(&mut app, button);
    assert_eq!(background(&app, button), colors.hovered_item);

    pointer_over_or_out(&mut app, button, false);
    assert_eq!(background(&app, button), colors.background_primary);
}

#[test]
fn a_small_icon_button_turns_its_icon_from_secondary_to_primary_text_on_hover() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Close", IconButtonSize::Small),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(icon_tint(&app, button), colors.text_secondary);

    pointer_over_or_out(&mut app, button, true);
    app.update();
    assert_eq!(icon_tint(&app, button), colors.text_primary);

    pointer_over_or_out(&mut app, button, false);
    app.update();
    assert_eq!(icon_tint(&app, button), colors.text_secondary);
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
    app.world_mut().trigger(PointerPress {
        entity: link,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button: PointerButton::Primary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
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

/// A link in a root node, after two updates.
fn spawn_link(app: &mut App, state: LinkState) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let link = app
        .world_mut()
        .spawn((Link::new("link"), state, ChildOf(root)))
        .id();
    app.update();
    app.update();
    link
}

/// The text colour of a link.
fn link_text_color(app: &App, link: Entity) -> Color {
    let text = app.world().entity(link).get::<Children>().unwrap()[0];
    app.world().entity(text).get::<TextColor>().unwrap().0
}

#[test]
fn changing_the_theme_under_a_hovered_link_keeps_it_hovered() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, LinkState::Enabled);

    hover_and_toggle_theme(&mut app, link);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_secondary);
    assert_eq!(link_text_color(&app, link), dark.link);
}

#[test]
fn changing_the_theme_under_a_pressed_link_keeps_it_pressed() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, LinkState::Enabled);
    press(&mut app, link);

    hover_and_toggle_theme(&mut app, link);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_secondary);
    assert_eq!(link_text_color(&app, link), dark.link_selected);
}

#[test]
fn changing_the_theme_under_a_link_the_pointer_is_not_on_leaves_it_at_rest() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, LinkState::Enabled);

    toggle_theme(&mut app);
    app.update();

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_primary);
    assert_eq!(link_text_color(&app, link), dark.link);
}

#[test]
fn a_disabled_link_the_pointer_is_on_stays_at_rest_in_a_new_theme() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, LinkState::Disabled);

    hover_and_toggle_theme(&mut app, link);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_primary);
    assert_eq!(link_text_color(&app, link), dark.text_primary);
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
    app.world_mut().trigger(PointerPress {
        entity: link,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button: PointerButton::Primary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
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

/// `Activate` is a single-hop entity event: an observer on a button hears it, one on an ancestor
/// of the button does not. An action therefore observes `Activate` on the button itself.
#[test]
fn an_activate_does_not_bubble_to_an_observer_on_an_ancestor() {
    let mut app = headless_app();
    app.init_resource::<Activations>();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((PushButton::from_label("Go"), ChildOf(root)))
        .id();
    app.world_mut()
        .entity_mut(root)
        .observe(|_: On<Activate>, mut n: ResMut<Activations>| n.0 += 100);
    app.world_mut()
        .entity_mut(button)
        .observe(|_: On<Activate>, mut n: ResMut<Activations>| n.0 += 1);
    app.update();
    app.update();

    app.world_mut().trigger(Activate { entity: button });
    app.update();

    assert_eq!(
        app.world().get_resource::<Activations>().unwrap().0,
        1,
        "only the button's own observer hears Activate"
    );
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
    app.world_mut().trigger(PointerPress {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button: PointerButton::Primary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
    app.update();
}

/// Bevy's picking emits `Click` before `Release` when the button comes up.
fn click_and_release(app: &mut App, target: Entity) {
    app.world_mut().trigger(PointerClick {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button: PointerButton::Primary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        duration: std::time::Duration::ZERO,
    });
    app.world_mut().trigger(PointerRelease {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        button: PointerButton::Primary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
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
    assert_eq!(
        background(&app, button),
        colors.hovered_item,
        "the pointer is still on a button that was let go, which is no selection"
    );
}

/// Says the pointer is on `entity`, and changes the theme to the other one under it.
///
/// Only `Update` runs: `PreUpdate`'s `update_is_hovered` would put the `Hovered` of
/// the headless app, which has no pointer over anything, back to false first.
fn hover_and_toggle_theme(app: &mut App, entity: Entity) {
    app.world_mut().entity_mut(entity).insert(Hovered(true));
    toggle_theme(app);
    app.world_mut().run_schedule(Update);
}

#[test]
fn changing_the_theme_under_a_hovered_push_button_keeps_it_hovered() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);

    hover_and_toggle_theme(&mut app, button);

    assert_eq!(
        background(&app, button),
        DEFAULT_DARK_THEME_COLORS.hovered_item
    );
}

#[test]
fn changing_the_theme_under_a_pressed_push_button_keeps_it_pressed() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    press(&mut app, button);

    hover_and_toggle_theme(&mut app, button);

    assert_eq!(
        background(&app, button),
        DEFAULT_DARK_THEME_COLORS.selected_hovered_item
    );
}

#[test]
fn changing_the_theme_under_a_push_button_the_pointer_is_not_on_leaves_it_at_rest() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);

    toggle_theme(&mut app);
    app.update();

    assert_eq!(
        background(&app, button),
        DEFAULT_DARK_THEME_COLORS.background_primary
    );
}

#[test]
fn a_disabled_push_button_the_pointer_is_on_stays_at_rest_in_a_new_theme() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();

    hover_and_toggle_theme(&mut app, button);

    assert_eq!(
        background(&app, button),
        DEFAULT_DARK_THEME_COLORS.background_primary
    );
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
        .label();
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

#[test]
fn a_push_button_puts_its_icon_where_its_icon_placement_says() {
    let mut app = headless_app();
    let button = app.world_mut().spawn(PushButton::new("Label", ">")).id();
    app.update();
    app.update();
    let direction = |app: &App| {
        app.world()
            .entity(button)
            .get::<Node>()
            .unwrap()
            .flex_direction
    };
    // The default keeps the icon after the label, as before.
    assert_eq!(direction(&app), FlexDirection::Row);
    for (placement, expected) in [
        (IconPlacement::Left, FlexDirection::RowReverse),
        (IconPlacement::Top, FlexDirection::ColumnReverse),
        (IconPlacement::Bottom, FlexDirection::Column),
        (IconPlacement::Right, FlexDirection::Row),
    ] {
        app.world_mut().entity_mut(button).insert(placement);
        app.update();
        app.update();
        assert_eq!(direction(&app), expected);
    }
}

#[test]
fn a_compact_push_button_is_small_and_keeps_it_after_a_theme_change() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    app.world_mut()
        .entity_mut(button)
        .insert(ButtonSize::Compact);
    app.update();
    app.update();

    let label = app
        .world()
        .entity(button)
        .get::<PushButtonRefs>()
        .unwrap()
        .label();
    let check = |app: &App| {
        let theme = app.world().get_resource::<Theme>().unwrap();
        let node = app.world().entity(button).get::<Node>().unwrap();
        assert_eq!(node.padding, theme.compact_button_padding);
        assert_eq!(node.height, Val::Px(28.0));
        assert_eq!(node.border_radius, BorderRadius::all(Val::Px(6.0)));
        assert!(
            app.world()
                .entity(button)
                .get::<BoxShadow>()
                .unwrap()
                .0
                .is_empty()
        );
        let font = app.world().entity(label).get::<TextFont>().unwrap();
        assert_eq!(font.font_size, FontSize::Rem(0.6));
        assert_eq!(font.weight, FontWeight::SEMIBOLD);
    };
    check(&app);

    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .light_dark_toggle();
    app.update();
    app.update();
    check(&app);

    // Back to the regular size.
    app.world_mut()
        .entity_mut(button)
        .insert(ButtonSize::Regular);
    app.update();
    app.update();
    let theme = app.world().get_resource::<Theme>().unwrap();
    let node = app.world().entity(button).get::<Node>().unwrap();
    assert_eq!(node.padding, theme.item_padding);
    assert_eq!(node.height, Val::Auto);
    assert!(
        !app.world()
            .entity(button)
            .get::<BoxShadow>()
            .unwrap()
            .0
            .is_empty()
    );
}

#[test]
fn widgets_plugin_and_ui_widgets_plugins_work_in_either_order() {
    for widgets_first in [true, false] {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(WindowPlugin {
            primary_window: None,
            ..default()
        });
        app.add_plugins(InputPlugin);
        app.add_plugins(DefaultPickingPlugins);
        app.init_resource::<ClearColor>();
        if widgets_first {
            app.add_plugins(WidgetsPlugin);
            app.add_plugins(bevy::ui_widgets::UiWidgetsPlugins);
        } else {
            app.add_plugins(bevy::ui_widgets::UiWidgetsPlugins);
            app.add_plugins(WidgetsPlugin);
        }
        // Building and finishing the app is the test: a duplicate plugin panics.
        app.finish();
        app.cleanup();
    }
}

#[test]
fn a_disabled_color_push_button_shows_the_disabled_border_and_does_not_activate() {
    let mut app = headless_app();
    app.init_resource::<Activations>();
    let button = app.world_mut().spawn(ColorPushButton).id();
    app.world_mut()
        .entity_mut(button)
        .observe(|_: On<Activate>, mut n: ResMut<Activations>| n.0 += 1);
    app.update();
    app.update();
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let border = |app: &App| app.world().entity(button).get::<BorderColor>().unwrap().top;
    assert_eq!(border(&app), colors.border);

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    app.update();
    assert_eq!(border(&app), colors.text_disabled);
    press(&mut app, button);
    click_and_release(&mut app, button);
    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 0);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    app.update();
    assert_eq!(border(&app), colors.border);
    press(&mut app, button);
    click_and_release(&mut app, button);
    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 1);
}

/// Whether `entity` has a focus ring, an [`Outline`].
fn has_focus_ring(app: &App, entity: Entity) -> bool {
    app.world().entity(entity).contains::<Outline>()
}

#[test]
fn the_focused_tab_stop_gets_a_focus_ring_and_an_unfocused_one_does_not() {
    let mut app = headless_app();
    let focused = spawn_push_button(&mut app);
    let other = app.world_mut().spawn(PushButton::from_label("Other")).id();
    app.update();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(focused));
    app.world_mut().insert_resource(InputFocusVisible(true));
    app.update();
    app.update();

    let ring = app.world().entity(focused).get::<Outline>().unwrap();
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(ring.color, colors.focus_ring);
    assert_eq!(ring.width, Val::Px(2.0));
    assert_eq!(ring.offset, Val::Px(1.0));
    assert!(!has_focus_ring(&app, other));

    // A click hides the ring, Tab shows it again.
    app.world_mut().insert_resource(InputFocusVisible(false));
    app.update();
    app.update();
    assert!(!has_focus_ring(&app, focused));
    app.world_mut().insert_resource(InputFocusVisible(true));
    app.update();
    app.update();
    assert!(has_focus_ring(&app, focused));

    // The focus moves, and the ring with it.
    app.world_mut()
        .insert_resource(InputFocus::from_entity(other));
    app.update();
    app.update();
    assert!(!has_focus_ring(&app, focused));
    assert!(has_focus_ring(&app, other));
}

#[test]
fn the_focus_ring_keeps_an_outline_the_app_set_and_gives_it_back() {
    let mut app = headless_app();
    let own = Outline::new(Val::Px(5.0), Val::Px(3.0), Color::WHITE);
    let with_outline = spawn_push_button(&mut app);
    app.world_mut().entity_mut(with_outline).insert(own);
    let without = app.world_mut().spawn(PushButton::from_label("Other")).id();
    app.update();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(with_outline));
    app.world_mut().insert_resource(InputFocusVisible(true));
    app.update();
    app.update();
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let ring = app.world().entity(with_outline).get::<Outline>().unwrap();
    assert_eq!(ring.color, colors.focus_ring);

    app.world_mut()
        .insert_resource(InputFocus::from_entity(without));
    app.update();
    app.update();
    assert_eq!(
        app.world().entity(with_outline).get::<Outline>(),
        Some(&own)
    );
    assert!(has_focus_ring(&app, without));

    app.world_mut().insert_resource(InputFocusVisible(false));
    app.update();
    app.update();
    assert!(!has_focus_ring(&app, without));
    assert_eq!(
        app.world().entity(with_outline).get::<Outline>(),
        Some(&own)
    );
}

#[test]
fn the_focus_ring_keeps_an_outline_on_a_lists_active_row() {
    let mut app = headless_app();
    let own = Outline::new(Val::Px(5.0), Val::Px(3.0), Color::WHITE);
    let row = app.world_mut().spawn((Node::default(), ListItem, own)).id();
    let plain = app.world_mut().spawn((Node::default(), ListItem)).id();
    let list = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .entity_mut(list)
        .insert(ActiveDescendant(Some(row)));
    app.update();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(list));
    app.world_mut().insert_resource(InputFocusVisible(true));
    app.update();
    app.update();
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(
        app.world().entity(row).get::<Outline>().unwrap().color,
        colors.focus_ring
    );

    app.world_mut()
        .entity_mut(list)
        .insert(ActiveDescendant(Some(plain)));
    app.update();
    app.update();
    assert_eq!(app.world().entity(row).get::<Outline>(), Some(&own));
    assert!(has_focus_ring(&app, plain));

    app.world_mut().insert_resource(InputFocusVisible(false));
    app.update();
    app.update();
    assert!(!has_focus_ring(&app, plain));
    assert_eq!(app.world().entity(row).get::<Outline>(), Some(&own));
}

#[test]
fn a_tab_stop_is_reached_by_tab_only_while_it_is_enabled() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    let index = |app: &App| app.world().entity(button).get::<TabIndex>().unwrap().0;
    assert_eq!(index(&app), 0);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    assert_eq!(index(&app), -1);
    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(index(&app), 0);
}

#[test]
fn a_widget_under_a_display_none_grandparent_is_hidden_then_shown_again_and_follows_a_move() {
    let mut app = headless_app();
    let index = |app: &App, e: Entity| app.world().entity(e).get::<TabIndex>().unwrap().0;
    let hidden = |app: &App, e: Entity| {
        app.world()
            .entity(e)
            .get::<AccessibilityNode>()
            .unwrap()
            .is_hidden()
    };
    let set_display = |app: &mut App, e: Entity, display: Display| {
        app.world_mut()
            .entity_mut(e)
            .get_mut::<Node>()
            .unwrap()
            .display = display;
        app.update();
        app.update();
    };

    let outer = app.world_mut().spawn(Node::default()).id();
    let inner = app
        .world_mut()
        .spawn((Node::default(), ChildOf(outer)))
        .id();
    let button = app
        .world_mut()
        .spawn((PushButton::from_label("deep"), ChildOf(inner)))
        .id();
    let other = app.world_mut().spawn(Node::default()).id();
    app.update();
    app.update();
    assert_eq!(index(&app, button), 0);
    assert!(!hidden(&app, button));

    // Only the grandparent changes; nothing on the button or its parent does.
    set_display(&mut app, outer, Display::None);
    assert_eq!(index(&app, button), -1);
    assert!(hidden(&app, button));

    set_display(&mut app, outer, Display::Flex);
    assert_eq!(index(&app, button), 0);
    assert!(!hidden(&app, button));

    // Moved under a hidden parent, then back out.
    set_display(&mut app, other, Display::None);
    app.world_mut().entity_mut(inner).insert(ChildOf(other));
    app.update();
    app.update();
    assert_eq!(index(&app, button), -1);
    assert!(hidden(&app, button));
    app.world_mut().entity_mut(inner).insert(ChildOf(outer));
    app.update();
    app.update();
    assert_eq!(index(&app, button), 0);
    assert!(!hidden(&app, button));
}

#[test]
fn a_tab_stop_in_a_hidden_stack_panel_or_hidden_popup_is_not_reached_by_tab() {
    let mut app = headless_app();
    let index = |app: &App, e: Entity| app.world().entity(e).get::<TabIndex>().unwrap().0;

    // A button in a node that is `Display::None`, as a `Stack` hides its panels.
    let panel = app.world_mut().spawn(Node::default()).id();
    let in_panel = app
        .world_mut()
        .spawn((PushButton::from_label("in panel"), ChildOf(panel)))
        .id();
    // A button in a popup that is `Visibility::Hidden`.
    let popup = app.world_mut().spawn(Node::default()).id();
    let in_popup = app
        .world_mut()
        .spawn((PushButton::from_label("in popup"), ChildOf(popup)))
        .id();
    app.update();
    app.update();
    assert_eq!(index(&app, in_panel), 0);
    assert_eq!(index(&app, in_popup), 0);

    app.world_mut()
        .entity_mut(panel)
        .get_mut::<Node>()
        .unwrap()
        .display = Display::None;
    app.world_mut().entity_mut(popup).insert(Visibility::Hidden);
    app.update();
    app.update();
    assert_eq!(index(&app, in_panel), -1);
    assert_eq!(index(&app, in_popup), -1);

    app.world_mut()
        .entity_mut(panel)
        .get_mut::<Node>()
        .unwrap()
        .display = Display::Flex;
    app.world_mut()
        .entity_mut(popup)
        .insert(Visibility::Inherited);
    app.update();
    app.update();
    assert_eq!(index(&app, in_panel), 0);
    assert_eq!(index(&app, in_popup), 0);
}

/// Whether the [`AccessibilityNode`] of `entity` says `aria-expanded`, or nothing.
fn expanded(app: &App, entity: Entity) -> Option<bool> {
    app.world()
        .entity(entity)
        .get::<AccessibilityNode>()
        .unwrap()
        .is_expanded()
}

/// A press, then the release that makes it a click.
fn click(app: &mut App, target: Entity) {
    press(app, target);
    click_and_release(app, target);
}

/// A combo over two items in a root node, returning it and its button.
fn spawn_combo(app: &mut App) -> (Entity, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((
            Combo,
            Selection::builder()
                .zero_or_one()
                .build_with_items(["a", "b"].map(|s| Item::new(s.to_string()))),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    let button = app
        .world()
        .entity(combo)
        .get::<ComboSubRefs>()
        .unwrap()
        .button_id;
    (combo, button)
}

#[test]
fn hiding_the_panel_of_an_open_combo_hides_its_popup_without_closing_it() {
    let mut app = headless_app();
    let panel = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((
            Combo,
            Selection::builder()
                .zero_or_one()
                .build_with_items(["a", "b"].map(|s| Item::new(s.to_string()))),
            ChildOf(panel),
        ))
        .id();
    app.update();
    app.update();
    let refs = *app.world().entity(combo).get::<ComboSubRefs>().unwrap();
    click(&mut app, refs.button_id);
    app.update();
    app.update();
    let list_index = |app: &App| {
        app.world()
            .entity(refs.list_id)
            .get::<TabIndex>()
            .unwrap()
            .0
    };
    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Open)
    );
    assert_eq!(list_index(&app), 0, "an open popup is a tab stop");

    // No click or focus change: only the ancestor is hidden, as a `Stack` does.
    app.world_mut()
        .entity_mut(panel)
        .get_mut::<Node>()
        .unwrap()
        .display = Display::None;
    app.update();
    app.update();
    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Open),
        "the combo is still open, so the popup is hidden by the panel alone"
    );
    assert_eq!(list_index(&app), -1);
}

/// A colour palette combo over two colours in a root node, returning it and its button.
fn spawn_palette_combo(app: &mut App) -> (Entity, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((
            ColorPaletteCombo,
            Selection::builder()
                .zero_or_one()
                .build_with_items([Color::WHITE, Color::BLACK].map(|data| Item { data })),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    (combo, single_entity_with::<ColorPushButton>(app))
}

#[test]
fn a_combo_state_on_an_entity_that_is_not_a_combo_does_nothing() {
    let mut app = headless_app();
    let plain = app.world_mut().spawn(Node::default()).id();
    let other = app.world_mut().spawn(Node::default()).id();
    let before = app.world().entity(other).get::<Visibility>().copied();
    app.world_mut().entity_mut(plain).insert(ComboState::Open);
    app.update();
    app.update();
    assert_eq!(
        app.world().entity(plain).get::<ComboState>(),
        Some(&ComboState::Open)
    );
    assert_eq!(
        app.world().entity(other).get::<Visibility>().copied(),
        before
    );
    assert!(
        app.world()
            .get_resource::<InputFocus>()
            .is_none_or(|focus| focus.get().is_none())
    );
}

#[test]
fn a_combo_button_says_whether_its_popup_is_open() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    assert_eq!(expanded(&app, button), Some(false), "closed to begin with");

    click(&mut app, button);
    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Open)
    );
    assert_eq!(expanded(&app, button), Some(true));

    click(&mut app, button);
    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Closed)
    );
    assert_eq!(expanded(&app, button), Some(false));
}

#[test]
fn a_color_palette_combo_button_says_whether_its_popup_is_open() {
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let (combo, button) = spawn_palette_combo(&mut app);
    assert_eq!(expanded(&app, button), Some(false), "closed to begin with");

    click(&mut app, button);
    assert_eq!(
        app.world().entity(combo).get::<PaletteComboState>(),
        Some(&PaletteComboState::Open)
    );
    assert_eq!(expanded(&app, button), Some(true));

    click(&mut app, button);
    assert_eq!(
        app.world().entity(combo).get::<PaletteComboState>(),
        Some(&PaletteComboState::Closed)
    );
    assert_eq!(expanded(&app, button), Some(false));
}

/// The node ids the accessibility node of `entity` says it controls.
fn controls(app: &App, entity: Entity) -> Vec<u64> {
    app.world()
        .get::<AccessibilityNode>(entity)
        .unwrap()
        .controls()
        .iter()
        .map(|id| id.0)
        .collect()
}

#[test]
fn a_combo_button_controls_its_list() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    let list = app.world().get::<ComboSubRefs>(combo).unwrap().list_id;
    assert_eq!(controls(&app, button), vec![list.to_bits()]);
}

#[test]
fn a_combo_button_shows_a_chevron_image_after_its_label_not_text() {
    let mut app = headless_app();
    let (_, button) = spawn_combo(&mut app);
    let children = app.world().get::<Children>(button).unwrap();
    assert_eq!(children.len(), 2);
    let label = children[0];
    let chevron = children[1];
    assert!(app.world().get::<Text>(label).is_some());
    assert!(app.world().get::<Text>(chevron).is_none());
    assert!(app.world().get::<Icon>(chevron).is_some());
    assert!(app.world().get::<ImageNode>(chevron).is_some());
}

/// The tick at which the [`AccessibilityNode`] of `entity` last changed.
fn accessibility_changed_at(app: &App, entity: Entity) -> bevy::ecs::change_detection::Tick {
    app.world()
        .entity(entity)
        .get_ref::<AccessibilityNode>()
        .unwrap()
        .last_changed()
}

#[test]
fn a_combo_button_writes_its_accessibility_node_only_when_the_shown_text_changes() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    let before = accessibility_changed_at(&app, button);
    app.update();
    app.update();
    assert_eq!(accessibility_changed_at(&app, button), before, "idle");

    select_by_text(&mut app, combo, "b");
    let node = app.world().get::<AccessibilityNode>(button).unwrap();
    assert_eq!(node.value(), Some("b"));
    assert_ne!(accessibility_changed_at(&app, button), before, "changed");
    let after = accessibility_changed_at(&app, button);
    app.update();
    app.update();
    assert_eq!(accessibility_changed_at(&app, button), after, "idle again");
}

#[test]
fn a_color_palette_combo_button_writes_its_accessibility_node_only_when_the_colour_changes() {
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let (combo, button) = spawn_palette_combo(&mut app);
    let before = accessibility_changed_at(&app, button);
    app.update();
    app.update();
    assert_eq!(accessibility_changed_at(&app, button), before, "idle");

    app.world_mut()
        .run_system_once(
            move |mut selections: SelectionParam, items: Query<(Entity, &Item<Color>)>| {
                let item = items
                    .iter()
                    .find(|(_, i)| i.data == Color::BLACK)
                    .unwrap()
                    .0;
                selections.update_selection(combo, |s| s.set_selection([item]));
            },
        )
        .unwrap();
    app.update();
    app.update();
    assert_ne!(accessibility_changed_at(&app, button), before, "changed");
    let after = accessibility_changed_at(&app, button);
    app.update();
    app.update();
    assert_eq!(accessibility_changed_at(&app, button), after, "idle again");
}

#[test]
fn a_color_palette_combo_button_controls_its_palette() {
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let (_, button) = spawn_palette_combo(&mut app);
    let palette = single_entity_with::<ColorPalette>(&mut app);
    assert_eq!(controls(&app, button), vec![palette.to_bits()]);
}

#[test]
fn a_color_push_button_outside_a_combo_is_named_after_its_colour() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app.world_mut().spawn((ColorPushButton, ChildOf(root))).id();
    app.update();
    let square = app.world().get::<ColorSquareRef>(button).unwrap().entity();
    app.world_mut().entity_mut(square).insert(BackgroundColor(
        bevy::color::palettes::tailwind::RED_500.into(),
    ));
    app.update();
    let node = app.world().get::<AccessibilityNode>(button).unwrap();
    assert_eq!(node.label(), Some("red-500"));
}

/// A colour palette combo's selection over two colours.
fn two_color_selection() -> impl Bundle {
    Selection::builder()
        .zero_or_one()
        .build_with_items([Color::WHITE, Color::BLACK].map(|data| Item { data }))
}

/// The entities named `entity_name`.
fn entities_named(app: &mut App, entity_name: &str) -> Vec<Entity> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &Name)>();
    query
        .iter(world)
        .filter(|(_, name)| name.as_str() == entity_name)
        .map(|(entity, _)| entity)
        .collect()
}

/// The popups of palette combos that kept their default name.
fn palette_combo_floats(app: &mut App) -> Vec<Entity> {
    entities_named(app, "ColorPaletteCombo popup")
}

#[test]
fn a_color_palette_combo_gets_its_popup_when_the_app_has_two_ui_roots() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let other_root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)))
        .id();

    app.update();
    app.update();

    assert!(app.world().entity(root).contains::<UiRoot>());
    assert!(app.world().entity(other_root).contains::<UiRoot>());
    assert_eq!(
        app.world().entity(combo).get::<PaletteComboState>(),
        Some(&PaletteComboState::Closed)
    );
    let floats = palette_combo_floats(&mut app);
    assert_eq!(floats.len(), 1, "one popup, got {floats:?}");
}

#[test]
fn a_color_palette_combo_popup_is_named_after_its_combo() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    for name in ["first", "second"] {
        app.world_mut().spawn((
            ColorPaletteCombo,
            Name::new(name),
            two_color_selection(),
            ChildOf(root),
        ));
    }
    app.world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)));

    app.update();
    app.update();

    for popup in ["first popup", "second popup", "ColorPaletteCombo popup"] {
        let popups = entities_named(&mut app, popup);
        assert_eq!(popups.len(), 1, "one {popup:?}, got {popups:?}");
        assert_eq!(
            app.world()
                .entity(popups[0])
                .get::<ChildOf>()
                .and_then(|c| app.world().get::<Name>(c.0))
                .map(Name::as_str),
            Some(popup.trim_end_matches(" popup")),
            "{popup:?} is a child of its combo"
        );
    }
}

#[test]
fn a_color_palette_combo_spawns_its_popup_as_a_fixed_child_of_the_combo() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let panel = app.world_mut().spawn((Node::default(), ChildOf(root))).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(panel)))
        .id();

    app.update();
    app.update();

    let floats = palette_combo_floats(&mut app);
    assert_eq!(floats.len(), 1, "one popup, got {floats:?}");
    assert_eq!(
        app.world().entity(floats[0]).get::<ChildOf>().map(|c| c.0),
        Some(combo)
    );
    assert!(app.world().entity(floats[0]).contains::<FixedNode>());
}

#[test]
fn a_color_palette_combo_without_a_parent_gets_its_popup() {
    let mut app = headless_app();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection()))
        .id();

    app.update();
    app.update();

    assert_eq!(
        app.world().entity(combo).get::<PaletteComboState>(),
        Some(&PaletteComboState::Closed)
    );
    assert_eq!(palette_combo_floats(&mut app).len(), 1);
}

/// The selection of the only [`ColorPalette`] of the app.
fn only_palette_selection(app: &mut App) -> Option<Selection> {
    let palette = single_entity_with::<ColorPalette>(app);
    app.world().entity(palette).get::<Selection>().cloned()
}

#[test]
fn a_color_palette_combo_popup_refers_to_the_combos_selection_as_soon_as_it_is_spawned() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    // No update after the spawn: nothing has changed since, so the reference was wired
    // when the popup was spawned, not by a system that reacts to a change.
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)))
        .id();

    assert_eq!(
        only_palette_selection(&mut app),
        Some(Selection::reference(combo))
    );
}

#[test]
fn a_color_palette_combo_popup_refers_to_the_selection_the_combo_refers_to_as_soon_as_it_is_spawned()
 {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection_ref = app.world_mut().spawn(two_color_selection()).id();
    app.world_mut().spawn((
        ColorPaletteCombo,
        Selection::reference(selection_ref),
        ChildOf(root),
    ));

    assert_eq!(
        only_palette_selection(&mut app),
        Some(Selection::reference(selection_ref))
    );
}

#[test]
fn a_color_palette_combo_popup_follows_the_selection_the_combo_is_given_later() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let first = app.world_mut().spawn(two_color_selection()).id();
    let second = app.world_mut().spawn(two_color_selection()).id();
    let combo = app
        .world_mut()
        .spawn((
            ColorPaletteCombo,
            Selection::reference(first),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    assert_eq!(
        only_palette_selection(&mut app),
        Some(Selection::reference(first))
    );

    app.world_mut()
        .entity_mut(combo)
        .insert(Selection::reference(second));
    app.update();

    assert_eq!(
        only_palette_selection(&mut app),
        Some(Selection::reference(second))
    );
}

#[test]
fn a_color_palette_combo_given_its_selection_after_it_is_spawned_passes_it_to_its_popup() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection_ref = app.world_mut().spawn(two_color_selection()).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, ChildOf(root)))
        .id();
    app.update();
    assert_eq!(only_palette_selection(&mut app), None);

    app.world_mut()
        .entity_mut(combo)
        .insert(Selection::reference(selection_ref));
    app.update();

    assert_eq!(
        only_palette_selection(&mut app),
        Some(Selection::reference(selection_ref))
    );
}

#[test]
fn the_popup_swatches_of_a_color_palette_combo_show_the_selection_the_combo_refers_to() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection_ref = app
        .world_mut()
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items([RED, GREEN, BLUE].map(|data| Item { data })),
        )
        .id();
    app.update();
    let green = {
        let world = app.world_mut();
        let mut items = world.query::<(Entity, &Item<Color>)>();
        items
            .iter(world)
            .find(|(_, item)| item.data == GREEN)
            .map(|(entity, _)| entity)
            .unwrap()
    };
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(green))));
    app.world_mut().spawn((
        ColorPaletteCombo,
        Selection::reference(selection_ref),
        ChildOf(root),
    ));
    app.update();
    app.update();

    let palette = single_entity_with::<ColorPalette>(&mut app);
    assert_eq!(selected_colors(&app, palette), [GREEN]);
}

/// The two colours the palette combos below choose from.
fn white_and_black() -> Vec<Item<Color>> {
    [Color::WHITE, Color::BLACK]
        .into_iter()
        .map(|data| Item { data })
        .collect()
}

/// A colour palette combo over `selection` (two colours), opened: the combo, its popup's
/// palette and the palette's swatches.
fn open_palette_combo_over(app: &mut App, selection: impl Bundle) -> (Entity, Entity, Vec<Entity>) {
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, selection, ChildOf(root)))
        .id();
    app.update();
    app.update();
    let button = single_entity_with::<ColorPushButton>(app);
    click(app, button);
    assert_eq!(
        app.world().entity(combo).get::<PaletteComboState>(),
        Some(&PaletteComboState::Open)
    );
    let palette = single_entity_with::<ColorPalette>(app);
    let swatches = item_views(app, palette);
    assert_eq!(swatches.len(), 2);
    (combo, palette, swatches)
}

fn palette_combo_state(app: &App, combo: Entity) -> PaletteComboState {
    *app.world()
        .entity(combo)
        .get::<PaletteComboState>()
        .unwrap()
}

#[test]
fn a_click_on_a_swatch_closes_the_popup_of_a_palette_combo_that_holds_one_item_at_most() {
    for (name, selection) in [
        (
            "ZeroOrOne",
            Selection::builder()
                .zero_or_one()
                .build_with_items(white_and_black()),
        ),
        (
            "One",
            Selection::builder()
                .one()
                .build_with_items(white_and_black()),
        ),
    ] {
        let mut app = headless_app();
        let (combo, _, swatches) = open_palette_combo_over(&mut app, selection);

        click_row(&mut app, swatches[1]);

        assert_eq!(
            palette_combo_state(&app, combo),
            PaletteComboState::Closed,
            "{name}"
        );
    }
}

#[test]
fn a_click_on_the_selected_swatch_closes_the_popup_of_a_one_palette_combo_and_keeps_the_selection()
{
    let mut app = headless_app();
    let (combo, palette, swatches) = open_palette_combo_over(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(white_and_black()),
    );
    assert_eq!(selected_colors(&app, palette), [Color::WHITE]);

    click_row(&mut app, swatches[0]);

    assert_eq!(palette_combo_state(&app, combo), PaletteComboState::Closed);
    assert_eq!(selected_colors(&app, palette), [Color::WHITE]);
}

#[test]
fn a_click_on_the_popup_of_a_palette_combo_that_is_not_on_a_swatch_leaves_it_open() {
    let mut app = headless_app();
    let (combo, palette, swatches) = open_palette_combo_over(&mut app, two_color_selection());

    click_row(&mut app, palette);

    assert_eq!(palette_combo_state(&app, combo), PaletteComboState::Open);
    assert!(
        selected_colors(&app, palette).is_empty(),
        "nothing was picked"
    );
    assert!(
        swatches
            .iter()
            .all(|s| !app.world().entity(*s).contains::<Selected>())
    );
}

#[test]
fn a_click_with_another_button_on_a_swatch_leaves_the_popup_open() {
    let mut app = headless_app();
    let (combo, _, swatches) = open_palette_combo_over(&mut app, two_color_selection());

    app.world_mut().trigger(PointerClick {
        entity: swatches[1],
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button: PointerButton::Secondary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        duration: std::time::Duration::ZERO,
    });
    app.update();

    assert_eq!(palette_combo_state(&app, combo), PaletteComboState::Open);
}

#[test]
fn a_combo_opens_when_its_button_is_activated_and_not_when_it_is_only_pressed() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);

    press(&mut app, button);
    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Closed),
        "a press is not yet an Activate"
    );
    assert_eq!(expanded(&app, button), Some(false));

    activate(&mut app, button);
    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Open),
        "Enter or Space on a focused button is an Activate too"
    );
    assert_eq!(expanded(&app, button), Some(true));
}

#[test]
fn a_disabled_combo_button_does_not_open_its_popup() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();

    click(&mut app, button);

    assert_eq!(
        app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Closed)
    );
    assert_eq!(expanded(&app, button), Some(false));
}

/// A stepper over a `u8` from 0 to 3, starting at `initial`, returning it and its
/// decrease and increase buttons.
fn spawn_stepper(app: &mut App, initial: u8) -> (Entity, Entity, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let stepper = app
        .world_mut()
        .spawn((
            NumericStepper::default(),
            Value::new_u8(initial, 0, 3, 1).unwrap(),
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

fn level_text(value: &Value) -> String {
    match value.as_u8() {
        Some(0) => "Ground".to_string(),
        Some(level) => format!("+{level}"),
        None => String::new(),
    }
}

#[test]
fn a_numeric_stepper_shows_its_custom_format_and_updates_it_on_a_change() {
    let mut app = headless_app();
    let stepper = app
        .world_mut()
        .spawn((
            NumericStepper {
                format: level_text,
                decrease_name: "Lower the level".into(),
                increase_name: "Raise the level".into(),
                ..default()
            },
            Value::new_u8(0, 0, 14, 7).unwrap(),
        ))
        .id();
    app.update();
    app.update();
    let children: Vec<Entity> = app
        .world()
        .entity(stepper)
        .get::<Children>()
        .unwrap()
        .iter()
        .collect();
    let text = |app: &App| app.world().get::<Text>(children[1]).unwrap().0.clone();
    assert_eq!(text(&app), "Ground");
    assert_eq!(
        app.world().get::<IconButton>(children[0]).unwrap().name,
        "Lower the level"
    );
    assert_eq!(
        app.world().get::<IconButton>(children[2]).unwrap().name,
        "Raise the level"
    );

    activate(&mut app, children[2]);
    assert_eq!(text(&app), "+7");
    activate(&mut app, children[2]);
    assert_eq!(text(&app), "+14");
    assert!(is_disabled(&app, children[2]));
    activate(&mut app, children[0]);
    activate(&mut app, children[0]);
    assert_eq!(text(&app), "Ground");
    assert!(is_disabled(&app, children[0]));
}

#[test]
fn one_arrow_key_steps_a_reinserted_numeric_stepper_once() {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};

    let mut app = headless_app();
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    let (stepper, _, _) = spawn_stepper(&mut app, 0);
    app.world_mut()
        .entity_mut(stepper)
        .insert(NumericStepper::default());
    app.update();
    app.update();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(stepper));

    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::ArrowUp,
        logical_key: Key::ArrowUp,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    app.update();

    assert_eq!(stepper_value(&app, stepper), 1);
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

#[test]
fn a_list_grid_with_zero_columns_does_not_panic_and_acts_as_one_column() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let list = app
        .world_mut()
        .spawn((
            List,
            ListLayout::VerticalGrid { columns: 0 },
            Selection::builder()
                .one()
                .build_with_items(["a", "b", "c"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    let palette = app
        .world_mut()
        .spawn((
            ColorPalette,
            ListLayout::VerticalGrid { columns: 0 },
            Selection::builder().one().build_with_items(
                [Color::WHITE, Color::BLACK, Color::NONE].map(|data| Item { data }),
            ),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Dark);
    app.update();
    app.update();
    for widget in [list, palette] {
        assert_eq!(item_views(&app, widget).len(), 3);
        let node = app.world().entity(widget).get::<Node>().unwrap();
        assert_eq!(node.grid_template_columns.len(), 1, "one column track");
    }
}

#[test]
fn a_list_keeps_the_node_fields_the_app_gave_it() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let list = app
        .world_mut()
        .spawn((
            List,
            Node {
                width: Val::Px(120.0),
                margin: UiRect::all(Val::Px(7.0)),
                ..default()
            },
            Selection::builder()
                .one()
                .build_with_items(["a", "b"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    // Inserting the zero-sized `List` again must not reset the node either.
    app.world_mut().entity_mut(list).insert(List);
    app.update();
    app.update();
    let node = app.world().entity(list).get::<Node>().unwrap();
    assert_eq!(node.width, Val::Px(120.0));
    assert_eq!(node.margin, UiRect::all(Val::Px(7.0)));
    assert_eq!(node.display, Display::Grid);
    assert_eq!(node.position_type, PositionType::Relative);
}

#[test]
fn changing_the_layout_of_a_live_list_updates_its_grid() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let list = app
        .world_mut()
        .spawn((
            List,
            Node {
                width: Val::Px(120.0),
                ..default()
            },
            Selection::builder()
                .one()
                .build_with_items(["a", "b", "c"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    let reference = app
        .world_mut()
        .spawn((
            List,
            ListLayout::VerticalGrid { columns: 3 },
            Selection::builder()
                .one()
                .build_with_items(["a", "b", "c"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.world_mut()
        .entity_mut(list)
        .insert(ListLayout::VerticalGrid { columns: 3 });
    app.update();
    app.update();
    let node = app.world().entity(list).get::<Node>().unwrap();
    let expected = app.world().entity(reference).get::<Node>().unwrap();
    assert_eq!(node.grid_auto_flow, expected.grid_auto_flow);
    assert_eq!(node.grid_template_columns, expected.grid_template_columns);
    assert_eq!(node.grid_template_columns.len(), 1);
    assert_eq!(node.width, Val::Px(120.0), "the app's width survives");

    app.world_mut()
        .entity_mut(list)
        .insert(ListLayout::Horizontal);
    app.update();
    let node = app.world().entity(list).get::<Node>().unwrap();
    assert_eq!(node.grid_auto_flow, GridAutoFlow::ColumnDense);
    assert!(node.grid_template_columns.is_empty());
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

#[test]
fn a_theme_change_restyles_a_palette_after_one_without_swatches() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let empty = app
        .world_mut()
        .spawn((
            ColorPalette,
            Selection::builder()
                .one()
                .build_with_items(Vec::<Item<Color>>::new()),
            ChildOf(root),
        ))
        .id();
    let populated = app
        .world_mut()
        .spawn((
            ColorPalette,
            Selection::builder()
                .one()
                .build_with_items([Color::WHITE, Color::BLACK].map(|data| Item { data })),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    assert!(item_views(&app, empty).is_empty());
    let swatches = item_views(&app, populated);
    assert_eq!(swatches.len(), 2);

    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Dark);
    app.update();
    app.update();

    for swatch in swatches {
        assert_eq!(
            background(&app, swatch),
            DEFAULT_DARK_THEME_COLORS.background_primary
        );
    }
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
fn a_view_for_a_new_item_starts_unselected_unless_the_item_is_in_the_selection() {
    let mut app = headless_app();
    let (list, _, selection_ref) =
        spawn_list(&mut app, Selection::builder().zero_or_more(), &["a"]);
    assert_eq!(item_views(&app, list).len(), 1);

    // A new item that is not in the selection gets a view without `Selected`.
    app.world_mut()
        .spawn((Item::new("b"), ItemOf(selection_ref)));
    app.update();
    app.update();

    assert_eq!(item_views(&app, list).len(), 2);
    assert!(selected_labels(&app, list).is_empty());

    // A new item that the selection already holds gets a view with `Selected`.
    let item_c = app
        .world_mut()
        .spawn((Item::new("c"), ItemOf(selection_ref)))
        .id();
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::ZeroOrMore(
            smallvec::smallvec![item_c],
        )));
    app.update();
    app.update();

    assert_eq!(item_views(&app, list).len(), 3);
    assert_eq!(selected_labels(&app, list), ["c"]);
}

// ---- `ColorPalette`: swatches are `ListItem`s, and `Selected` follows the selection ----

const RED: Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const BLUE: Color = Color::srgb(0.0, 0.0, 1.0);

/// A palette of red, green and blue swatches over a selection of its own, as
/// `(palette, swatches, selection)`.
fn spawn_palette(
    app: &mut App,
    selection: SelectionBuilderWithValue,
) -> (Entity, Vec<Entity>, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let items: Vec<_> = [RED, GREEN, BLUE]
        .into_iter()
        .map(|data| Item { data })
        .collect();
    let selection_ref = app
        .world_mut()
        .spawn(selection.build_with_items(items))
        .id();
    let palette = spawn_palette_on(app, root, selection_ref);
    let swatches = item_views(app, palette);
    (palette, swatches, selection_ref)
}

/// A palette sharing the selection of `selection_ref`.
fn spawn_palette_on(app: &mut App, root: Entity, selection_ref: Entity) -> Entity {
    let palette = app
        .world_mut()
        .spawn((
            ColorPalette,
            Selection::reference(selection_ref),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    palette
}

/// The colour of a swatch.
fn swatch_color(app: &App, swatch: Entity) -> Color {
    let item_ref = app
        .world()
        .entity(swatch)
        .get::<ItemView>()
        .unwrap()
        .item_ref;
    app.world()
        .entity(item_ref)
        .get::<Item<Color>>()
        .unwrap()
        .data
}

/// The colours of the swatches of `palette` that carry [`Selected`], in the order of the palette.
fn selected_colors(app: &App, palette: Entity) -> Vec<Color> {
    item_views(app, palette)
        .into_iter()
        .filter(|swatch| app.world().entity(*swatch).contains::<Selected>())
        .map(|swatch| swatch_color(app, swatch))
        .collect()
}

/// The colours that `selection_ref` selects.
fn selection_colors(app: &App, selection_ref: Entity) -> Vec<Color> {
    let Selection::Value(selection) = app
        .world()
        .entity(selection_ref)
        .get::<Selection>()
        .unwrap()
    else {
        return Vec::new();
    };
    selection
        .iter()
        .map(|item| app.world().entity(item).get::<Item<Color>>().unwrap().data)
        .collect()
}

#[test]
fn a_palette_is_a_list_box_and_its_swatches_are_list_items() {
    let mut app = headless_app();
    let (palette, swatches, _) = spawn_palette(&mut app, Selection::builder().zero_or_one());

    assert!(app.world().entity(palette).contains::<ListBox>());
    assert_eq!(swatches.len(), 3);
    for swatch in swatches {
        assert!(app.world().entity(swatch).contains::<ListItem>());
    }
}

#[test]
fn the_swatches_of_the_selection_are_selected_and_the_others_are_not() {
    let mut app = headless_app();
    let (palette, swatches, selection_ref) =
        spawn_palette(&mut app, Selection::builder().zero_or_one());
    assert!(selected_colors(&app, palette).is_empty());

    // Selected follows the selection when something else changes it as well.
    let blue = app
        .world()
        .entity(swatches[2])
        .get::<ItemView>()
        .unwrap()
        .item_ref;
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(blue))));
    app.update();

    assert_eq!(selected_colors(&app, palette), [BLUE]);
}

#[test]
fn palettes_sharing_a_selection_all_show_it_as_selected() {
    let mut app = headless_app();
    let (first, swatches, selection_ref) =
        spawn_palette(&mut app, Selection::builder().zero_or_more());
    let root = app.world_mut().spawn(Node::default()).id();
    let second = spawn_palette_on(&mut app, root, selection_ref);
    assert!(selected_colors(&app, second).is_empty());

    click_row(&mut app, swatches[1]);

    assert_eq!(selected_colors(&app, first), [GREEN]);
    assert_eq!(selected_colors(&app, second), [GREEN]);
}

#[test]
fn a_palette_over_a_selection_of_many_is_a_multi_select_list_box() {
    let mut app = headless_app();
    for (selection, multiple) in [
        (Selection::builder().one(), false),
        (Selection::builder().zero_or_one(), false),
        (Selection::builder().one_or_more(), true),
        (Selection::builder().zero_or_more(), true),
    ] {
        let (palette, _, _) = spawn_palette(&mut app, selection);

        assert_eq!(
            app.world().entity(palette).contains::<ListBoxMultiSelect>(),
            multiple
        );
    }
}

#[test]
fn clicking_a_swatch_selects_it_and_clicking_the_selected_one_deselects_it() {
    let mut app = headless_app();
    let (palette, swatches, selection_ref) =
        spawn_palette(&mut app, Selection::builder().zero_or_one());

    click_row(&mut app, swatches[0]);
    assert_eq!(selection_colors(&app, selection_ref), [RED]);
    assert_eq!(selected_colors(&app, palette), [RED]);

    click_row(&mut app, swatches[2]);
    assert_eq!(selection_colors(&app, selection_ref), [BLUE]);
    assert_eq!(selected_colors(&app, palette), [BLUE]);

    click_row(&mut app, swatches[2]);
    assert!(selection_colors(&app, selection_ref).is_empty());
    assert!(selected_colors(&app, palette).is_empty());
}

#[test]
fn clicking_a_selected_swatch_that_is_not_the_first_deselects_it() {
    let mut app = headless_app();
    let (palette, swatches, selection_ref) =
        spawn_palette(&mut app, Selection::builder().zero_or_more());
    for swatch in &swatches[..2] {
        click_row(&mut app, *swatch);
    }
    assert_eq!(selected_colors(&app, palette), [RED, GREEN]);

    click_row(&mut app, swatches[1]);

    assert_eq!(selection_colors(&app, selection_ref), [RED]);
    assert_eq!(selected_colors(&app, palette), [RED]);
}

#[test]
fn the_selection_rules_decide_what_a_swatch_click_does() {
    let mut app = headless_app();
    let (palette, swatches, selection_ref) = spawn_palette(&mut app, Selection::builder().one());
    assert_eq!(selected_colors(&app, palette), [RED]);

    // A `One` selection replaces its swatch and cannot be emptied.
    click_row(&mut app, swatches[1]);
    assert_eq!(selected_colors(&app, palette), [GREEN]);
    click_row(&mut app, swatches[1]);
    assert_eq!(selection_colors(&app, selection_ref), [GREEN]);
    assert_eq!(selected_colors(&app, palette), [GREEN]);
}

#[test]
fn a_swatch_spawned_for_an_item_of_the_selection_starts_selected() {
    let mut app = headless_app();
    let (palette, _, selection_ref) = spawn_palette(&mut app, Selection::builder().zero_or_more());
    let yellow = app
        .world_mut()
        .spawn((
            Item {
                data: Color::srgb(1.0, 1.0, 0.0),
            },
            ItemOf(selection_ref),
        ))
        .id();
    app.world_mut()
        .entity_mut(selection_ref)
        .insert(Selection::Value(SelectionValue::ZeroOrMore(
            smallvec::smallvec![yellow],
        )));
    app.update();
    app.update();

    assert_eq!(item_views(&app, palette).len(), 4);
    assert_eq!(selected_colors(&app, palette), [Color::srgb(1.0, 1.0, 0.0)]);
}

#[test]
fn changing_the_theme_under_a_hovered_list_item_keeps_it_hovered() {
    let mut app = headless_app();
    let (_list, views, _selection) =
        spawn_list(&mut app, Selection::builder().zero_or_one(), &["a", "b"]);

    hover_and_toggle_theme(&mut app, views[1]);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, views[1]), dark.hovered_item);
    assert_eq!(background(&app, views[0]), dark.background_primary);
}

#[test]
fn changing_the_theme_under_a_hovered_selected_list_item_keeps_it_selected_and_hovered() {
    let mut app = headless_app();
    let (list, views, _selection) = spawn_list(&mut app, Selection::builder().one(), &["a", "b"]);
    assert_eq!(selected_labels(&app, list), ["a"], "the first is selected");

    hover_and_toggle_theme(&mut app, views[0]);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, views[0]), dark.selected_hovered_item);

    app.world_mut().entity_mut(views[0]).insert(Hovered(false));
    toggle_theme(&mut app);
    app.world_mut().run_schedule(Update);
    assert_eq!(
        background(&app, views[0]),
        DEFAULT_LIGHT_THEME_COLORS.selected_item,
        "the pointer left, so it is only selected"
    );
}

/// The borders of a swatch and of the colour square in it, which carry its highlight.
fn swatch_border(app: &App, swatch: Entity) -> (Color, Color) {
    let outer = app.world().entity(swatch).get::<BorderColor>().unwrap().top;
    let square = app.world().entity(swatch).get::<Children>().unwrap()[0];
    let inner = app.world().entity(square).get::<BorderColor>().unwrap().top;
    (outer, inner)
}

/// The border of `swatch` in the dark theme with the pointer off it, whatever it was.
fn resting_dark_border(app: &mut App, swatch: Entity) -> (Color, Color) {
    app.world_mut().entity_mut(swatch).insert(Hovered(false));
    // Light, then dark again, both with the pointer off the swatch.
    toggle_theme(app);
    app.world_mut().run_schedule(Update);
    toggle_theme(app);
    app.world_mut().run_schedule(Update);
    swatch_border(app, swatch)
}

#[test]
fn changing_the_theme_under_a_hovered_swatch_keeps_it_hovered() {
    let mut app = headless_app();
    let (_palette, swatches, _selection) =
        spawn_palette(&mut app, Selection::builder().zero_or_one());

    hover_and_toggle_theme(&mut app, swatches[1]);
    let hovered = swatch_border(&app, swatches[1]);

    assert_ne!(
        hovered,
        resting_dark_border(&mut app, swatches[1]),
        "the swatch under the pointer keeps its hover highlight"
    );
}

#[test]
fn changing_the_theme_under_a_hovered_selected_swatch_keeps_it_selected_and_hovered() {
    let mut app = headless_app();
    let (_palette, swatches, _selection) = spawn_palette(&mut app, Selection::builder().one());

    hover_and_toggle_theme(&mut app, swatches[0]);
    let hovered = swatch_border(&app, swatches[0]);

    assert_ne!(
        hovered,
        resting_dark_border(&mut app, swatches[0]),
        "the selected swatch under the pointer is highlighted more than a selected one"
    );
}

#[test]
fn changing_the_theme_fonts_restyles_the_text_of_widgets() {
    let body = bevy::asset::uuid_handle!("6f2b1c1e-3b0a-4f43-9d0e-0a3c5a1b7e01");
    let display = bevy::asset::uuid_handle!("6f2b1c1e-3b0a-4f43-9d0e-0a3c5a1b7e02");
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((PushButton::from_label("Go"), ChildOf(root)))
        .id();
    let regular = app
        .world_mut()
        .spawn((Label::regular("Regular"), ChildOf(root)))
        .id();
    let header = app
        .world_mut()
        .spawn((Label::header("Header"), ChildOf(root)))
        .id();
    app.update();
    app.update();

    {
        let mut theme = app.world_mut().get_resource_mut::<Theme>().unwrap();
        theme.body_font = body.clone();
        theme.display_font = display.clone();
        theme.font_size_secondary = 20.0;
        theme.font_size_title = 30.0;
    }
    app.update();
    app.update();

    let font_of = |app: &App, entity: Entity| {
        let font = app.world().entity(entity).get::<TextFont>().unwrap();
        (font.font.clone(), font.font_size)
    };
    let button_label = app
        .world()
        .entity(button)
        .get::<PushButtonRefs>()
        .unwrap()
        .label();
    assert_eq!(
        font_of(&app, button_label),
        (body.clone().into(), FontSize::Rem(1.0))
    );
    assert_eq!(font_of(&app, regular), (body.into(), FontSize::Rem(1.0)));
    assert_eq!(font_of(&app, header), (display.into(), FontSize::Rem(1.5)));
}

#[test]
fn each_label_variant_gets_its_colour_and_size_and_a_theme_change_restyles_it() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let mut spawn = |label: Label| app.world_mut().spawn((label, ChildOf(root))).id();
    let title = spawn(Label::title("Trains"));
    let regular = spawn(Label::regular("Row"));
    let secondary = spawn(Label::secondary("4 on the network"));
    let caption = spawn(Label::caption("A to B"));
    let overline = spawn(Label::overline("Locomotives"));
    app.update();
    app.update();

    let look = |app: &App, entity: Entity| {
        let e = app.world().entity(entity);
        (
            e.get::<TextColor>().unwrap().0,
            e.get::<TextFont>().unwrap().font_size,
        )
    };
    let check = |app: &App| {
        let theme = app.world().get_resource::<Theme>().unwrap();
        let c = theme.colors;
        assert_eq!(
            look(app, title),
            (
                c.text_primary,
                FontSize::Rem(theme.font_size_title / DEFAULT_REM_SIZE_PX)
            )
        );
        assert_eq!(
            look(app, regular),
            (
                c.text_primary,
                FontSize::Rem(theme.font_size_secondary / DEFAULT_REM_SIZE_PX)
            )
        );
        assert_eq!(
            look(app, secondary),
            (
                c.text_secondary,
                FontSize::Rem(theme.font_size_secondary / DEFAULT_REM_SIZE_PX)
            )
        );
        assert_eq!(
            look(app, caption),
            (
                c.text_tertiary,
                FontSize::Rem(theme.font_size_small / DEFAULT_REM_SIZE_PX)
            )
        );
        assert_eq!(
            look(app, overline),
            (
                c.text_secondary,
                FontSize::Rem(theme.font_size_overline / DEFAULT_REM_SIZE_PX)
            )
        );
    };
    check(&app);
    assert_eq!(
        app.world().entity(overline).get::<Text>().unwrap().0,
        "LOCOMOTIVES"
    );

    {
        let mut theme = app.world_mut().get_resource_mut::<Theme>().unwrap();
        theme.light_dark_toggle();
        theme.font_size_small = 13.0;
        theme.font_size_overline = 12.0;
    }
    app.update();
    app.update();
    check(&app);
    let overline_font = app.world().entity(overline).get::<TextFont>().unwrap();
    assert_eq!(overline_font.weight, FontWeight::SEMIBOLD);
    assert_eq!(
        app.world().entity(overline).get::<LetterSpacing>().unwrap(),
        &LetterSpacing::Px(0.5)
    );
    assert_eq!(
        app.world().entity(regular).get::<LetterSpacing>().unwrap(),
        &LetterSpacing::default()
    );
}

#[derive(Resource, Default)]
struct CreatedViews(Vec<(Entity, Entity)>);

#[derive(Component)]
struct RowButton;

#[derive(Component)]
struct RowText;

/// A creator of custom item views: a text and a nested button.
fn spawn_row(
    spawner: In<ItemViewSpawner>,
    mut commands: Commands,
    mut created: ResMut<CreatedViews>,
) {
    let ItemViewSpawner { item, view } = *spawner;
    created.0.push((item, view));
    commands.entity(view).with_children(|parent| {
        parent.spawn((Text::new("row"), RowText));
        parent.spawn((Node::default(), Button, RowButton));
    });
}

fn spawn_custom_list(app: &mut App, selection: SelectionBuilderWithValue) -> (Entity, Entity) {
    app.init_resource::<CreatedViews>();
    let create = app.world_mut().register_system(spawn_row);
    let root = app.world_mut().spawn(Node::default()).id();
    let selection_ref = app
        .world_mut()
        .spawn(selection.build_with_items([Item::new("a"), Item::new("b")]))
        .id();
    let list = app
        .world_mut()
        .spawn((
            List,
            ItemViewCreator::new(create),
            Selection::reference(selection_ref),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    (list, selection_ref)
}

/// The text and the button inside a custom view.
fn nested(app: &App, view: Entity) -> (Entity, Entity) {
    let children = app.world().entity(view).get::<Children>().unwrap();
    let text = children
        .iter()
        .find(|c| app.world().entity(*c).contains::<RowText>())
        .unwrap();
    let button = children
        .iter()
        .find(|c| app.world().entity(*c).contains::<RowButton>())
        .unwrap();
    (text, button)
}

#[test]
fn a_custom_view_creator_runs_once_per_item_and_fills_the_view() {
    let mut app = headless_app();
    let (list, _) = spawn_custom_list(&mut app, Selection::builder().one());

    let views = item_views(&app, list);
    assert_eq!(views.len(), 2);
    let created = app
        .world()
        .get_resource::<CreatedViews>()
        .unwrap()
        .0
        .clone();
    assert_eq!(created.len(), 2);
    for view in views {
        assert!(created.iter().any(|(_, v)| *v == view));
        assert!(!app.world().entity(view).contains::<Text>());
        nested(&app, view);
    }
    // Further updates do not run it again.
    app.update();
    assert_eq!(
        app.world().get_resource::<CreatedViews>().unwrap().0.len(),
        2
    );
}

#[test]
fn adding_and_removing_items_adds_and_removes_custom_views() {
    let mut app = headless_app();
    let (list, selection_ref) = spawn_custom_list(&mut app, Selection::builder().one());

    app.world_mut()
        .spawn((Item::new("c"), ItemOf(selection_ref)));
    app.update();
    app.update();
    assert_eq!(item_views(&app, list).len(), 3);
    assert_eq!(
        app.world().get_resource::<CreatedViews>().unwrap().0.len(),
        3
    );

    let third = item_views(&app, list)[2];
    let item = app
        .world()
        .entity(third)
        .get::<ItemView>()
        .unwrap()
        .item_ref;
    app.world_mut().entity_mut(item).despawn();
    app.update();
    app.update();
    assert_eq!(item_views(&app, list).len(), 2);
}

#[test]
fn selecting_a_custom_view_restyles_it_like_a_default_one() {
    let mut app = headless_app();
    let (list, _) = spawn_custom_list(&mut app, Selection::builder().one());
    let views = item_views(&app, list);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert!(app.world().entity(views[0]).contains::<Selected>());
    assert_eq!(background(&app, views[0]), colors.selected_item);
    assert_eq!(background(&app, views[1]), colors.background_primary);

    list_box_picks(&mut app, list, views[1]);

    assert!(!app.world().entity(views[0]).contains::<Selected>());
    assert!(app.world().entity(views[1]).contains::<Selected>());
    assert_eq!(background(&app, views[0]), colors.background_primary);
    assert_eq!(background(&app, views[1]), colors.selected_item);
}

#[test]
fn a_press_on_a_nested_button_leaves_the_custom_view_and_the_selection_alone() {
    let mut app = headless_app();
    let (list, selection_ref) = spawn_custom_list(&mut app, Selection::builder().one());
    let views = item_views(&app, list);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let (_, button) = nested(&app, views[1]);

    press(&mut app, button);
    assert_eq!(background(&app, views[1]), colors.background_primary);
    click_and_release(&mut app, button);

    assert_eq!(selection_labels(&app, selection_ref), ["a"]);
    assert!(!app.world().entity(views[1]).contains::<Selected>());
    assert_eq!(background(&app, views[1]), colors.background_primary);
}

#[test]
fn a_press_on_the_text_of_a_custom_view_presses_the_row() {
    let mut app = headless_app();
    let (list, _) = spawn_custom_list(&mut app, Selection::builder().one());
    let views = item_views(&app, list);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let (text, _) = nested(&app, views[1]);

    press(&mut app, text);

    assert_eq!(background(&app, views[1]), colors.selected_item);
}

#[test]
fn releasing_on_a_selected_custom_view_in_a_zero_or_one_list_deselects_it() {
    let mut app = headless_app();
    let (list, selection_ref) = spawn_custom_list(&mut app, Selection::builder().zero_or_one());
    let views = item_views(&app, list);
    list_box_picks(&mut app, list, views[1]);
    assert_eq!(selection_labels(&app, selection_ref), ["b"]);
    let (text, _) = nested(&app, views[1]);

    click_row(&mut app, text);

    assert!(selection_labels(&app, selection_ref).is_empty());
}

/// The segments of a control, in order.
fn segments_of(app: &App, control: Entity) -> Vec<Entity> {
    app.world()
        .entity(control)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|c| app.world().entity(*c).contains::<Segment>())
        .collect()
}

/// The text of a segment.
fn segment_text(app: &App, segment: Entity) -> String {
    let label = app.world().entity(segment).get::<Children>().unwrap()[0];
    app.world().entity(label).get::<Text>().unwrap().0.clone()
}

/// The text of the segments that carry `Selected`.
fn selected_segments(app: &App, control: Entity) -> Vec<String> {
    segments_of(app, control)
        .into_iter()
        .filter(|s| app.world().entity(*s).contains::<Selected>())
        .map(|s| segment_text(app, s))
        .collect()
}

/// The sliding indicator behind the segments of a control.
fn indicator_of(app: &App, control: Entity) -> Entity {
    app.world()
        .entity(control)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|c| app.world().entity(*c).contains::<SegmentIndicator>())
        .unwrap()
}

fn spawn_segmented_control(app: &mut App) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let control = app
        .world_mut()
        .spawn((
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["Auto", "Manual", "Off"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    control
}

/// Selects the item with the given text through the system param.
fn select_by_text(app: &mut App, selection: Entity, text: &'static str) {
    app.world_mut()
        .run_system_once(
            move |mut selections: SelectionParam, items: Query<(Entity, &Item<String>)>| {
                let item = items.iter().find(|(_, i)| i.data == text).unwrap().0;
                selections.update_selection(selection, |s| s.set_selection([item]));
            },
        )
        .unwrap();
    app.update();
    app.update();
}

#[test]
fn a_segmented_control_shows_a_segment_per_item_with_the_first_selected() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);

    let segments = segments_of(&app, control);
    let texts: Vec<_> = segments.iter().map(|s| segment_text(&app, *s)).collect();
    assert_eq!(texts, ["Auto", "Manual", "Off"]);
    assert_eq!(selected_segments(&app, control), ["Auto"]);
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, control), colors.segmented_track);
    assert_eq!(background(&app, segments[0]), Color::NONE);
    assert_eq!(background(&app, segments[1]), Color::NONE);
}

#[test]
fn clicking_a_segment_selects_it() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);

    activate(&mut app, segments[1]);

    assert_eq!(selected_segments(&app, control), ["Manual"]);
    assert_eq!(background(&app, segments[0]), Color::NONE);
    assert_eq!(background(&app, segments[1]), Color::NONE);

    // The selected segment stays selected when it is used again.
    activate(&mut app, segments[1]);
    assert_eq!(selected_segments(&app, control), ["Manual"]);
}

#[test]
fn a_selection_changed_through_the_selection_param_restyles_the_segments() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);

    select_by_text(&mut app, control, "Off");

    assert_eq!(selected_segments(&app, control), ["Off"]);
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, segments[0]), Color::NONE);
    assert_eq!(background(&app, segments[2]), Color::NONE);
    let label = app.world().entity(segments[2]).get::<Children>().unwrap()[0];
    assert_eq!(
        app.world().entity(label).get::<TextColor>().unwrap().0,
        colors.segmented_selected_text
    );
}

#[test]
fn a_segmented_control_follows_a_selection_it_refers_to() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["Dark", "Light"].map(Item::new)),
        )
        .id();
    let control = app
        .world_mut()
        .spawn((
            SegmentedControl,
            Selection::reference(selection),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(selected_segments(&app, control), ["Dark"]);

    // Clicking writes to the referenced selection ...
    let segments = segments_of(&app, control);
    activate(&mut app, segments[1]);
    assert_eq!(selected_segments(&app, control), ["Light"]);
    let value = app.world().entity(selection).get::<Selection>().unwrap();
    assert!(matches!(value, Selection::Value(SelectionValue::One(_))));

    // ... and a change made to it from elsewhere restyles the control.
    select_by_text(&mut app, selection, "Dark");
    assert_eq!(selected_segments(&app, control), ["Dark"]);
    assert_eq!(background(&app, segments[0]), Color::NONE);
    assert_eq!(background(&app, segments[1]), Color::NONE);
}

#[test]
fn a_segmented_control_follows_a_theme_change() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);

    toggle_theme(&mut app);
    app.update();

    let colors = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, control), colors.segmented_track);
    assert_eq!(
        background(&app, indicator_of(&app, control)),
        colors.segmented_selected
    );
    assert_eq!(background(&app, segments[0]), Color::NONE);
}

#[test]
fn an_unselected_segment_shows_primary_text_while_hovered_and_secondary_after() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    let label_color = |app: &App, segment: Entity| {
        let label = app.world().entity(segment).get::<Children>().unwrap()[0];
        app.world().entity(label).get::<TextColor>().unwrap().0
    };
    assert_eq!(label_color(&app, segments[1]), colors.text_secondary);

    app.world_mut()
        .entity_mut(segments[1])
        .insert(Hovered(true));
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(Update);
    assert_eq!(label_color(&app, segments[1]), colors.text_primary);
    assert_eq!(background(&app, segments[1]), Color::NONE);
    assert_eq!(label_color(&app, segments[2]), colors.text_secondary);

    app.world_mut()
        .entity_mut(segments[1])
        .insert(Hovered(false));
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(Update);
    assert_eq!(label_color(&app, segments[1]), colors.text_secondary);
}

#[test]
fn a_hovered_selected_segment_keeps_the_selected_text() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);

    app.world_mut()
        .entity_mut(segments[0])
        .insert(Hovered(true));
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(Update);

    let label = app.world().entity(segments[0]).get::<Children>().unwrap()[0];
    assert_eq!(
        app.world().entity(label).get::<TextColor>().unwrap().0,
        DEFAULT_LIGHT_THEME_COLORS.segmented_selected_text
    );
}

#[test]
fn adding_and_removing_items_adds_and_removes_segments() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let removed = app.world().entity(segments_of(&app, control)[1]);
    let removed_item = removed.get::<Segment>().unwrap().item_ref;
    // The control's own selection holds its items.
    let items_ref = control;

    app.world_mut().entity_mut(removed_item).despawn();
    app.world_mut().spawn((Item::new("New"), ItemOf(items_ref)));
    app.update();
    app.update();
    app.update();

    let texts: Vec<_> = segments_of(&app, control)
        .into_iter()
        .map(|s| segment_text(&app, s))
        .collect();
    assert_eq!(texts, ["Auto", "Off", "New"]);
    assert_eq!(selected_segments(&app, control), ["Auto"]);
    // The new segment is styled and works.
    let segments = segments_of(&app, control);
    assert_eq!(background(&app, segments[2]), Color::NONE);
    activate(&mut app, segments[2]);
    assert_eq!(selected_segments(&app, control), ["New"]);
}

// ---- Tab bars ----

/// The tabs of a bar, in order.
fn tabs_of(app: &App, bar: Entity) -> Vec<Entity> {
    app.world()
        .entity(bar)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|c| app.world().entity(*c).contains::<TabBarTab>())
        .collect()
}

/// The text of the tabs that carry `Selected`.
fn selected_tabs(app: &App, bar: Entity) -> Vec<String> {
    tabs_of(app, bar)
        .into_iter()
        .filter(|t| app.world().entity(*t).contains::<Selected>())
        .map(|t| segment_text(app, t))
        .collect()
}

/// The tab the bar's `SelectedTab` names.
fn selected_tab(app: &App, bar: Entity) -> Option<Entity> {
    app.world().entity(bar).get::<SelectedTab>().unwrap().0
}

/// Asks the bar's tab list for `tab`, as a click or Enter on it does.
fn request_tab(app: &mut App, bar: Entity, tab: Entity) {
    app.world_mut().trigger(ValueChange::<Option<Entity>> {
        source: bar,
        value: Some(tab),
        is_final: true,
    });
    app.update();
    app.update();
}

fn spawn_tab_bar(app: &mut App, selection: impl Bundle) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let bar = app
        .world_mut()
        .spawn((TabBar, selection, ChildOf(root)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    bar
}

#[test]
fn a_tab_bar_shows_a_tab_per_item_with_the_first_selected() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );

    let tabs = tabs_of(&app, bar);
    let texts: Vec<_> = tabs.iter().map(|t| segment_text(&app, *t)).collect();
    assert_eq!(texts, ["General", "Audio", "Input"]);
    assert!(app.world().entity(bar).contains::<TabList>());
    assert!(
        tabs.iter()
            .all(|t| app.world().entity(*t).contains::<Tab>())
    );
    assert_eq!(selected_tab(&app, bar), Some(tabs[0]));
    assert_eq!(selected_tabs(&app, bar), ["General"]);
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, bar), colors.segmented_track);
    assert_eq!(background(&app, tabs[0]), colors.segmented_selected);
    assert_eq!(background(&app, tabs[1]), colors.segmented_track);
}

#[test]
fn a_tab_the_tab_list_asks_for_becomes_the_selection() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let tabs = tabs_of(&app, bar);

    request_tab(&mut app, bar, tabs[2]);

    assert_eq!(selection_labels(&app, bar), ["Input"]);
    assert_eq!(selected_tab(&app, bar), Some(tabs[2]));
    assert_eq!(selected_tabs(&app, bar), ["Input"]);
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, tabs[0]), colors.segmented_track);
    assert_eq!(background(&app, tabs[2]), colors.segmented_selected);
}

#[test]
fn a_tab_bar_follows_a_selection_it_refers_to() {
    let mut app = headless_app();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["Dark", "Light"].map(Item::new)),
        )
        .id();
    let bar = spawn_tab_bar(&mut app, Selection::reference(selection));
    let tabs = tabs_of(&app, bar);
    assert_eq!(selected_tabs(&app, bar), ["Dark"]);

    // A tab asked for writes to the referenced selection ...
    request_tab(&mut app, bar, tabs[1]);
    assert_eq!(selection_labels(&app, selection), ["Light"]);
    assert_eq!(selected_tabs(&app, bar), ["Light"]);

    // ... and a change made to it from elsewhere moves the selected tab.
    select_by_text(&mut app, selection, "Dark");
    assert_eq!(selected_tab(&app, bar), Some(tabs[0]));
    assert_eq!(selected_tabs(&app, bar), ["Dark"]);
}

#[test]
fn a_tab_is_named_after_its_item_to_a_screen_reader() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio"].map(Item::new)),
    );
    let tabs = tabs_of(&app, bar);

    let bar_node = app.world().entity(bar).get::<AccessibilityNode>().unwrap();
    assert_eq!(bar_node.role(), accesskit::Role::TabList);
    let node = app
        .world()
        .entity(tabs[1])
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.role(), accesskit::Role::Tab);
    assert_eq!(node.label(), Some("Audio"));
}

#[test]
fn a_hidden_tab_bar_gives_its_tabs_tab_index_minus_one_until_shown() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let tabs = tabs_of(&app, bar);
    let panel = app.world().entity(bar).get::<ChildOf>().unwrap().parent();
    let indices = |app: &App| -> Vec<i32> {
        tabs.iter()
            .map(|t| app.world().entity(*t).get::<TabIndex>().unwrap().0)
            .collect()
    };
    let set_display = |app: &mut App, display: Display| {
        app.world_mut()
            .entity_mut(panel)
            .get_mut::<Node>()
            .unwrap()
            .display = display;
        app.update();
        app.update();
    };
    assert_eq!(indices(&app), [0, -1, -1]);

    // Its panel hidden by `Display::None`, as a `Stack` hides its panels.
    set_display(&mut app, Display::None);
    assert_eq!(indices(&app), [-1, -1, -1]);
    // A selection made while hidden is the tab Tab reaches once it shows.
    request_tab(&mut app, bar, tabs[1]);
    assert_eq!(indices(&app), [-1, -1, -1]);
    set_display(&mut app, Display::Flex);
    assert_eq!(indices(&app), [-1, 0, -1]);

    // The bar itself hidden by its visibility.
    app.world_mut().entity_mut(bar).insert(Visibility::Hidden);
    app.update();
    app.update();
    assert_eq!(indices(&app), [-1, -1, -1]);
    app.world_mut()
        .entity_mut(bar)
        .insert(Visibility::Inherited);
    app.update();
    app.update();
    assert_eq!(indices(&app), [-1, 0, -1]);
}

// ---- Separated lists ----

/// A `CornerStyle::Separated` list over a `One` selection of `labels`.
fn spawn_separated_list(app: &mut App, labels: &[&str]) -> (Entity, Vec<Entity>) {
    let root = app.world_mut().spawn(Node::default()).id();
    let items: Vec<_> = labels.iter().map(|label| Item::new(*label)).collect();
    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items(items))
        .id();
    let list = app
        .world_mut()
        .spawn((
            List,
            CornerStyle::Separated,
            Selection::reference(selection_ref),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    let views = item_views(app, list);
    (list, views)
}

fn ring(app: &App, view: Entity) -> Color {
    app.world().entity(view).get::<BorderColor>().unwrap().top
}

#[test]
fn a_separated_list_has_no_border_or_background_and_rounds_every_row() {
    let mut app = headless_app();
    let (list, views) = spawn_separated_list(&mut app, &["a", "b", "c"]);
    let radius = app.world().get_resource::<Theme>().unwrap().control_radius;

    let node = app.world().entity(list).get::<Node>().unwrap();
    assert_eq!(node.border, UiRect::ZERO);
    assert_eq!(node.padding, UiRect::all(Val::Px(6.0)));
    assert_eq!(
        (node.row_gap, node.column_gap),
        (Val::Px(2.0), Val::Px(2.0))
    );
    assert_eq!(background(&app, list), Color::NONE);

    assert_eq!(views.len(), 3);
    for view in views {
        let node = app.world().entity(view).get::<Node>().unwrap();
        assert_eq!(node.border_radius, BorderRadius::all(radius));
        assert_eq!(node.padding, UiRect::axes(Val::Px(12.0), Val::Px(10.0)));
    }
}

#[test]
fn the_selected_row_of_a_separated_list_has_a_ring_and_the_strong_hover_background() {
    let mut app = headless_app();
    let (_, views) = spawn_separated_list(&mut app, &["a", "b"]);
    let colors = DEFAULT_LIGHT_THEME_COLORS;

    assert_eq!(ring(&app, views[0]), colors.selection_ring);
    assert_eq!(background(&app, views[0]), colors.hovered_selected_item);
    assert_eq!(ring(&app, views[1]), Color::NONE);
    assert_eq!(background(&app, views[1]), Color::NONE);
}

#[test]
fn selecting_a_row_of_a_separated_list_moves_the_ring_to_it() {
    let mut app = headless_app();
    let (list, views) = spawn_separated_list(&mut app, &["a", "b"]);

    list_box_picks(&mut app, list, views[1]);

    assert_eq!(
        ring(&app, views[1]),
        DEFAULT_LIGHT_THEME_COLORS.selection_ring
    );
    assert_eq!(ring(&app, views[0]), Color::NONE);
    assert_eq!(background(&app, views[0]), Color::NONE);
}

#[test]
fn a_separated_list_follows_a_theme_change() {
    let mut app = headless_app();
    let (_, views) = spawn_separated_list(&mut app, &["a", "b"]);

    toggle_theme(&mut app);
    app.update();

    assert_eq!(
        ring(&app, views[0]),
        DEFAULT_DARK_THEME_COLORS.selection_ring
    );
    assert_eq!(
        background(&app, views[0]),
        DEFAULT_DARK_THEME_COLORS.hovered_selected_item
    );
    assert_eq!(background(&app, views[1]), Color::NONE);
}

#[test]
fn a_pill_gets_the_colours_of_its_tone_and_a_new_tone_restyles_it() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let pill = app
        .world_mut()
        .spawn((Pill::new("running"), ChildOf(root)))
        .id();
    app.update();
    app.update();

    let state = |app: &App| {
        let e = app.world().entity(pill);
        (
            e.get::<Text>().unwrap().0.clone(),
            e.get::<BackgroundColor>().unwrap().0,
            e.get::<TextColor>().unwrap().0,
        )
    };
    let c = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(
        state(&app),
        ("RUNNING".to_string(), c.pill_background, c.pill_text)
    );

    for (tone, text, bg, fg) in [
        (PillTone::Info, "RUNNING", c.info_background, c.info_text),
        (PillTone::Warning, "AT STOP", c.wait_background, c.wait_text),
        (
            PillTone::Problem,
            "NO PATH",
            c.warning_background,
            c.warning_text,
        ),
        (PillTone::Neutral, "IDLE", c.pill_background, c.pill_text),
    ] {
        app.world_mut()
            .entity_mut(pill)
            .insert(Pill::new(text).with_tone(tone));
        app.update();
        app.update();
        assert_eq!(state(&app), (text.to_string(), bg, fg), "{tone:?}");
    }

    app.world_mut()
        .entity_mut(pill)
        .insert(Pill::new("x").with_tone(PillTone::Warning));
    toggle_theme(&mut app);
    app.update();
    app.update();
    let d = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(
        state(&app),
        ("X".to_string(), d.wait_background, d.wait_text)
    );
}
fn spawn_chips(app: &mut App, chips: [Chip; 3]) -> (Entity, Entity) {
    let root = app.world_mut().spawn(Node::default()).id();
    let chips_entity = app
        .world_mut()
        .spawn((
            FilterChips,
            Selection::builder()
                .one()
                .build_with_items(chips.map(|data| Item { data })),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.update();
    (root, chips_entity)
}

fn chip_views(app: &App, chips: Entity) -> Vec<Entity> {
    app.world()
        .entity(chips)
        .get::<Children>()
        .map(|children| {
            children
                .iter()
                .filter(|c| app.world().entity(*c).contains::<ChipView>())
                .collect()
        })
        .unwrap_or_default()
}

/// The label and the count text of a chip.
fn chip_texts(app: &App, view: Entity) -> (String, String) {
    let refs = app.world().entity(view).get::<ChipRefs>().unwrap();
    let text = |e: Entity| app.world().entity(e).get::<Text>().unwrap().0.clone();
    (text(refs.label), text(refs.count))
}

fn three_chips() -> [Chip; 3] {
    [
        Chip::with_count("All", 4),
        Chip::with_count("Running", 2),
        Chip::new("Problems"),
    ]
}

#[test]
fn filter_chips_show_a_chip_per_item_with_label_and_count_and_the_first_active() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    assert_eq!(views.len(), 3);
    assert_eq!(chip_texts(&app, views[0]), ("All".into(), "4".into()));
    assert_eq!(
        chip_texts(&app, views[2]),
        ("Problems".into(), String::new())
    );

    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(background(&app, views[0]), colors.chip_on);
    assert_eq!(background(&app, views[1]), Color::NONE);
    let label = app
        .world()
        .entity(views[0])
        .get::<ChipRefs>()
        .unwrap()
        .label;
    let ink = app.world().entity(label).get::<TextColor>().unwrap().0;
    assert_eq!(ink, colors.chip_on_ink);
}

#[test]
fn activating_a_chip_selects_it_and_fills_it() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;

    activate(&mut app, views[1]);

    let selection = app.world().entity(chips).get::<Selection>().unwrap();
    let item = app.world().entity(views[1]).get::<ChipView>().unwrap().item;
    assert_eq!(selection.as_selection().unwrap().single(), Some(item));
    assert_eq!(background(&app, views[1]), colors.chip_on);
    assert_eq!(background(&app, views[0]), Color::NONE);

    // The active chip stays active when it is activated again.
    activate(&mut app, views[1]);
    assert_eq!(background(&app, views[1]), colors.chip_on);
}

#[test]
fn activating_the_active_chip_leaves_the_selection_unchanged() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    let before = app
        .world()
        .entity(chips)
        .get::<Selection>()
        .unwrap()
        .clone();

    activate(&mut app, views[0]);

    let after = app.world().entity(chips).get::<Selection>().unwrap();
    assert_eq!(*after, before);
}

#[test]
fn updating_the_count_of_an_item_changes_the_text_of_its_chip_without_respawning_it() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    let item = app.world().entity(views[2]).get::<ChipView>().unwrap().item;

    app.world_mut()
        .entity_mut(item)
        .get_mut::<Item<Chip>>()
        .unwrap()
        .data
        .count = Some(7);
    app.update();
    assert_eq!(chip_texts(&app, views[2]), ("Problems".into(), "7".into()));

    app.world_mut()
        .entity_mut(item)
        .get_mut::<Item<Chip>>()
        .unwrap()
        .data
        .count = None;
    app.update();
    assert_eq!(chip_views(&app, chips), views);
    assert_eq!(
        chip_texts(&app, views[2]),
        ("Problems".into(), String::new())
    );
}

#[test]
fn adding_and_removing_items_adds_and_removes_chips() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let items = app.world().entity(chips).get::<ItemValues>().unwrap();
    let removed = items[1];

    app.world_mut().entity_mut(removed).despawn();
    app.world_mut().spawn((
        Item {
            data: Chip::new("New"),
        },
        ItemOf(chips),
    ));
    app.update();
    app.update();

    let texts = chip_views(&app, chips)
        .into_iter()
        .map(|view| chip_texts(&app, view).0)
        .collect::<Vec<_>>();
    assert_eq!(texts, ["All", "Problems", "New"]);
}

#[test]
fn filter_chips_share_a_selection_by_reference() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(three_chips().map(|data| Item { data })),
        )
        .id();
    let chips = app
        .world_mut()
        .spawn((FilterChips, Selection::reference(selection), ChildOf(root)))
        .id();
    app.update();
    app.update();
    app.update();
    let views = chip_views(&app, chips);
    assert_eq!(views.len(), 3);

    activate(&mut app, views[2]);

    let item = app.world().entity(views[2]).get::<ChipView>().unwrap().item;
    let selected = app.world().entity(selection).get::<Selection>().unwrap();
    assert_eq!(selected.as_selection().unwrap().single(), Some(item));
}

#[test]
fn a_theme_change_restyles_the_chips() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);

    toggle_theme(&mut app);
    app.update();

    assert_eq!(
        background(&app, views[0]),
        DEFAULT_DARK_THEME_COLORS.chip_on
    );
    let border = app.world().entity(views[1]).get::<BorderColor>().unwrap();
    assert_eq!(border.top, DEFAULT_DARK_THEME_COLORS.chip_border);
}

#[derive(Resource, Default)]
struct PanelsClosed(Vec<Entity>);

#[test]
fn a_panel_spawns_its_header_parts_and_hides_the_empty_sections() {
    let mut app = headless_app();
    let icon: Handle<Image> = bevy::asset::uuid_handle!("1c7d2e3f-4051-4b62-9c73-8d94ea5f6a07");
    let panel = app
        .world_mut()
        .spawn(
            Panel::new("Trains")
                .with_subtitle("4 on the network")
                .with_close_icon(icon.clone()),
        )
        .id();
    app.update();
    app.update();

    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let text = |app: &App, e: Entity| app.world().get::<Text>(e).unwrap().0.clone();
    let display = |app: &App, e: Entity| app.world().get::<Node>(e).unwrap().display;
    assert_eq!(text(&app, refs.title()), "Trains");
    assert_eq!(text(&app, refs.subtitle()), "4 on the network");
    assert_eq!(display(&app, refs.subtitle()), Display::Flex);
    assert_eq!(display(&app, refs.close()), Display::Flex);
    let close_icon = app
        .world()
        .get::<IconButtonRefs>(refs.close())
        .unwrap()
        .icon();
    assert_eq!(
        app.world().get::<ImageNode>(close_icon).unwrap().image,
        icon
    );
    assert_eq!(display(&app, refs.strip()), Display::None);
    assert_eq!(display(&app, refs.footer()), Display::None);
    assert_eq!(
        app.world()
            .get::<Node>(refs.scroll_area())
            .unwrap()
            .max_height,
        // The 620 px of the body plus the 12 px corner radius it stops short of the bottom edge.
        Val::Px(632.0)
    );

    let footer_text = app
        .world_mut()
        .spawn((Label::caption("Pick a train."), ChildOf(refs.footer())))
        .id();
    app.update();
    assert_eq!(display(&app, refs.footer()), Display::Flex);
    app.world_mut().despawn(footer_text);
    app.update();
    assert_eq!(display(&app, refs.footer()), Display::None);

    let plain = app.world_mut().spawn(Panel::new("Stops")).id();
    app.update();
    app.update();
    let plain_refs = *app.world().get::<PanelRefs>(plain).unwrap();
    assert_eq!(display(&app, plain_refs.subtitle()), Display::None);
    assert_eq!(display(&app, plain_refs.close()), Display::None);
}

#[test]
fn a_panel_body_stops_a_corner_radius_short_of_the_bottom_unless_a_footer_holds_the_corner() {
    let mut app = headless_app();
    let radius = app.world().get_resource::<Theme>().unwrap().panel_radius;
    let panel = app.world_mut().spawn(Panel::new("Trains")).id();
    app.update();
    app.update();
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let margin = |app: &App| {
        app.world()
            .get::<Node>(refs.scroll_area())
            .unwrap()
            .margin
            .bottom
    };
    assert_ne!(radius, Val::ZERO);
    assert_eq!(margin(&app), radius);

    let footer_text = app
        .world_mut()
        .spawn((Label::caption("Pick a train."), ChildOf(refs.footer())))
        .id();
    app.update();
    assert_eq!(margin(&app), Val::ZERO);
    app.world_mut().despawn(footer_text);
    app.update();
    assert_eq!(margin(&app), radius);
}

#[test]
fn activating_the_close_button_of_a_panel_triggers_panel_closed_on_it() {
    let mut app = headless_app();
    app.init_resource::<PanelsClosed>();
    app.add_observer(|t: On<PanelClosed>, mut closed: ResMut<PanelsClosed>| {
        closed.0.push(t.event().event_target());
    });
    let panel = app
        .world_mut()
        .spawn(Panel::new("Warnings").with_close_icon(Handle::default()))
        .id();
    let other = app
        .world_mut()
        .spawn(Panel::new("Schedules").with_close_icon(Handle::default()))
        .id();
    app.update();
    app.update();

    let close = app.world().get::<PanelRefs>(panel).unwrap().close();
    app.world_mut().trigger(Activate { entity: close });
    app.update();

    assert_eq!(
        app.world().get_resource::<PanelsClosed>().unwrap().0,
        vec![panel]
    );
    assert!(app.world().get::<PanelRefs>(other).is_some());
}

#[test]
fn changing_the_subtitle_of_a_live_panel_updates_its_text() {
    let mut app = headless_app();
    let panel = app
        .world_mut()
        .spawn(Panel::new("Warnings").with_subtitle("3 open"))
        .id();
    app.update();
    app.update();
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let text = |app: &App, e: Entity| app.world().get::<Text>(e).unwrap().0.clone();
    assert_eq!(text(&app, refs.subtitle()), "3 open");

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Alerts").with_subtitle("2 open"));
    app.update();
    app.update();
    assert_eq!(text(&app, refs.subtitle()), "2 open");
    assert_eq!(text(&app, refs.title()), "Alerts");
    assert_eq!(
        app.world().get::<Children>(refs.header()).unwrap().len(),
        3,
        "the parts are spawned once"
    );

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Alerts"));
    app.update();
    app.update();
    assert_eq!(
        app.world().get::<Node>(refs.subtitle()).unwrap().display,
        Display::None
    );
}

#[test]
fn a_panels_close_button_is_named_close_unless_the_app_names_it() {
    let mut app = headless_app();
    let close_label = |app: &App, panel: Entity| {
        let close = app.world().get::<PanelRefs>(panel).unwrap().close();
        app.world()
            .get::<AccessibilityNode>(close)
            .unwrap()
            .label()
            .map(str::to_owned)
    };
    let plain = app
        .world_mut()
        .spawn(Panel::new("A").with_close_icon(Handle::default()))
        .id();
    let named = app
        .world_mut()
        .spawn(
            Panel::new("B")
                .with_close_icon(Handle::default())
                .with_close_name("Sluiten"),
        )
        .id();
    app.update();
    app.update();
    assert_eq!(close_label(&app, plain).as_deref(), Some("Close"));
    assert_eq!(close_label(&app, named).as_deref(), Some("Sluiten"));

    app.world_mut().entity_mut(plain).insert(
        Panel::new("A")
            .with_close_icon(Handle::default())
            .with_close_name("Fermer"),
    );
    app.update();
    app.update();
    assert_eq!(close_label(&app, plain).as_deref(), Some("Fermer"));

    app.world_mut()
        .entity_mut(plain)
        .insert(Panel::new("A").with_close_icon(Handle::default()));
    app.update();
    app.update();
    assert_eq!(close_label(&app, plain).as_deref(), Some("Close"));
}

#[test]
fn inserting_a_panel_again_keeps_its_parts_content_and_one_close_notification() {
    let mut app = headless_app();
    app.init_resource::<PanelsClosed>();
    app.add_observer(|t: On<PanelClosed>, mut closed: ResMut<PanelsClosed>| {
        closed.0.push(t.event().event_target());
    });
    let panel = app
        .world_mut()
        .spawn(Panel::new("Trains").with_close_icon(Handle::default()))
        .id();
    app.update();
    app.update();
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let content = app
        .world_mut()
        .spawn((Label::caption("Pick a train."), ChildOf(refs.footer())))
        .id();

    app.world_mut().entity_mut(panel).insert(
        Panel::new("Stops")
            .with_subtitle("6 stops")
            .with_close_icon(Handle::default())
            .with_close_name("Sluiten")
            .with_max_body_height(200.0)
            .unwrap(),
    );
    app.update();
    app.update();

    assert_eq!(*app.world().get::<PanelRefs>(panel).unwrap(), refs);
    assert_eq!(app.world().get::<Text>(refs.title()).unwrap().0, "Stops");
    assert_eq!(
        app.world().get::<Text>(refs.subtitle()).unwrap().0,
        "6 stops"
    );
    assert!(
        app.world()
            .get::<Children>(refs.footer())
            .unwrap()
            .contains(&content)
    );
    assert!(matches!(
        app.world().get::<Node>(refs.scroll_area()).unwrap().max_height,
        Val::Px(px) if (200.0..200.0 + 100.0).contains(&px)
    ));
    app.world_mut().trigger(Activate {
        entity: refs.close(),
    });
    app.update();
    assert_eq!(
        app.world().get_resource::<PanelsClosed>().unwrap().0,
        vec![panel],
        "one notification per activation"
    );
}

#[test]
fn a_setting_row_has_a_title_a_description_and_a_control_slot() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let row = app
        .world_mut()
        .spawn((SettingRow::new("Theme", "Light or dark"), ChildOf(root)))
        .id();
    app.update();
    app.update();

    let refs = *app.world().get::<SettingRowRefs>(row).unwrap();
    let text = |entity| app.world().get::<Text>(entity).unwrap().0.clone();
    assert_eq!(text(refs.title_id), "Theme");
    assert_eq!(text(refs.description_id), "Light or dark");
    assert!(app.world().get::<Node>(refs.control_slot).is_some());

    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(
        app.world().get::<TextColor>(refs.title_id).unwrap().0,
        colors.text_primary
    );
    assert_eq!(
        app.world().get::<TextColor>(refs.description_id).unwrap().0,
        colors.text_secondary
    );

    // The app spawns its control into the slot.
    let control = app
        .world_mut()
        .spawn((Label::regular("control"), ChildOf(refs.control_slot)))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<ChildOf>(control).unwrap().parent(),
        refs.control_slot
    );
}

#[test]
fn a_setting_row_is_a_group_labelled_with_its_title_and_described_by_its_description() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let row = app
        .world_mut()
        .spawn((SettingRow::new("Theme", "Light or dark"), ChildOf(root)))
        .id();
    app.update();
    app.update();

    let node = app.world().get::<AccessibilityNode>(row).unwrap();
    assert_eq!(node.role(), accesskit::Role::Group);
    assert_eq!(node.label(), Some("Theme"));
    assert_eq!(node.description(), Some("Light or dark"));
}

#[test]
fn a_panel_is_a_group_labelled_by_its_title_which_follows_a_new_panel() {
    let mut app = headless_app();
    let panel = app.world_mut().spawn(Panel::new("Trains")).id();
    app.update();
    app.update();
    let node = |app: &App| {
        let node = app.world().get::<AccessibilityNode>(panel).unwrap();
        let title = app.world().get::<PanelRefs>(panel).unwrap().title();
        (
            node.role(),
            node.label().map(str::to_owned),
            node.labelled_by() == [accesskit::NodeId(title.to_bits())],
            app.world().get::<Text>(title).unwrap().0.clone(),
        )
    };
    assert_eq!(
        node(&app),
        (accesskit::Role::Group, None, true, "Trains".to_owned())
    );

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Stops"));
    app.update();
    app.update();
    assert_eq!(
        node(&app),
        (accesskit::Role::Group, None, true, "Stops".to_owned())
    );
}

#[test]
fn of_three_setting_rows_only_the_first_two_show_a_divider() {
    let mut app = headless_app();
    let rows = app.world_mut().spawn(SettingRows).id();
    let spawned: Vec<Entity> = ["A", "B", "C"]
        .into_iter()
        .map(|title| {
            app.world_mut()
                .spawn((SettingRow::new(title, "description"), ChildOf(rows)))
                .id()
        })
        .collect();
    app.update();
    app.update();

    let display = |app: &App, row: Entity| {
        let refs = *app.world().get::<SettingRowRefs>(row).unwrap();
        app.world().get::<Node>(refs.divider_id).unwrap().display
    };
    let displays: Vec<Display> = spawned.iter().map(|row| display(&app, *row)).collect();
    assert_eq!(displays, [Display::Flex, Display::Flex, Display::None]);

    // Adding a fourth row moves the missing divider to it.
    let fourth = app
        .world_mut()
        .spawn((SettingRow::new("D", "description"), ChildOf(rows)))
        .id();
    app.update();
    app.update();
    assert_eq!(display(&app, spawned[2]), Display::Flex);
    assert_eq!(display(&app, fourth), Display::None);
}

fn tools() -> [Tool; 4] {
    [
        Tool::in_group(Handle::default(), "Select", 0),
        Tool::in_group(Handle::default(), "Rail", 1),
        Tool::in_group(Handle::default(), "Signal", 1),
        Tool::in_group(Handle::default(), "Train", 2),
    ]
}

fn toolbar_children(app: &App, toolbar: Entity) -> Vec<Entity> {
    app.world()
        .entity(toolbar)
        .get::<Children>()
        .map(|c| c.iter().collect())
        .unwrap_or_default()
}

fn toolbar_buttons(app: &App, toolbar: Entity) -> Vec<Entity> {
    toolbar_children(app, toolbar)
        .into_iter()
        .filter(|c| app.world().entity(*c).contains::<ToolbarButton>())
        .collect()
}

fn spawn_toolbar(app: &mut App, style: ToolbarStyle, builder: SelectionBuilderWithValue) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let toolbar = app
        .world_mut()
        .spawn((
            Toolbar::new(style),
            builder.build_with_items(tools().map(|data| Item { data })),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.update();
    toolbar
}

fn selected_label(app: &App, selection_holder: Entity) -> Option<String> {
    let selection = app
        .world()
        .entity(selection_holder)
        .get::<Selection>()
        .unwrap();
    let item = selection.as_selection().unwrap().single()?;
    Some(
        app.world()
            .entity(item)
            .get::<Item<Tool>>()
            .unwrap()
            .data
            .label
            .clone(),
    )
}

#[test]
fn toolbar_buttons_always_have_the_children_of_their_toolbars_style() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let check = |app: &App, style: ToolbarStyle| {
        let buttons = toolbar_buttons(app, toolbar);
        assert!(!buttons.is_empty());
        for button in buttons {
            let refs = app.world().get::<ToolbarButtonRefs>(button).unwrap();
            assert_eq!(refs.style(), style);
            assert_eq!(refs.label().is_some(), style == ToolbarStyle::Filled);
            assert_eq!(refs.bar().is_some(), style == ToolbarStyle::Underlined);
            assert_eq!(
                app.world().get::<ToolbarButton>(button).unwrap().toolbar(),
                toolbar
            );
        }
    };
    check(&app, ToolbarStyle::Filled);
    app.world_mut()
        .entity_mut(toolbar)
        .insert(Toolbar::new(ToolbarStyle::Underlined));
    for _ in 0..3 {
        app.update();
    }
    check(&app, ToolbarStyle::Underlined);
}

#[test]
fn a_tool_with_a_key_hint_gets_a_positioned_hint_text_and_one_without_gets_none() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let toolbar = app
        .world_mut()
        .spawn((
            Toolbar::new(ToolbarStyle::Filled),
            Selection::builder().one().build_with_items(
                [
                    Tool::new(Handle::default(), "Select").with_key_hint("1"),
                    Tool::new(Handle::default(), "Rail"),
                ]
                .map(|data| Item { data }),
            ),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let buttons = toolbar_buttons(&app, toolbar);
    let hint_of = |button: Entity| {
        app.world()
            .entity(button)
            .get::<ToolbarButtonRefs>()
            .unwrap()
            .hint()
    };
    let hint = hint_of(buttons[0]).unwrap();
    assert_eq!(hint_of(buttons[1]), None);
    let entity = app.world().entity(hint);
    assert_eq!(entity.get::<Text>().unwrap().0, "1");
    let node = entity.get::<Node>().unwrap();
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!((node.top, node.right), (Val::Px(5.0), Val::Px(7.0)));
    assert!((entity.get::<TextColor>().unwrap().0.alpha() - 0.6).abs() < 1e-6);
}

#[test]
fn a_toolbar_has_a_button_per_item_with_a_divider_between_groups() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());

    let children = toolbar_children(&app, toolbar);
    assert_eq!(children.len(), 6);
    let is_divider = |e: Entity| app.world().entity(e).contains::<Divider>();
    assert_eq!(
        children.iter().map(|c| is_divider(*c)).collect::<Vec<_>>(),
        [false, true, false, false, true, false]
    );
    let buttons = toolbar_buttons(&app, toolbar);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(background(&app, buttons[0]), colors.accent);
    assert_eq!(background(&app, buttons[1]), Color::NONE);
    let node = app.world().entity(buttons[0]).get::<Node>().unwrap();
    assert_eq!((node.width, node.height), (Val::Px(66.0), Val::Px(60.0)));
}

#[test]
fn clicking_a_tool_of_a_one_toolbar_selects_it_and_fills_it() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let buttons = toolbar_buttons(&app, toolbar);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;

    activate(&mut app, buttons[2]);
    assert_eq!(selected_label(&app, toolbar).as_deref(), Some("Signal"));
    assert_eq!(background(&app, buttons[2]), colors.accent);
    assert_eq!(background(&app, buttons[0]), Color::NONE);

    // The active tool stays active when it is activated again.
    activate(&mut app, buttons[2]);
    assert_eq!(selected_label(&app, toolbar).as_deref(), Some("Signal"));
    assert_eq!(background(&app, buttons[2]), colors.accent);
}

#[test]
fn the_label_of_a_filled_button_is_its_text_and_takes_the_accent_text_colour_when_active() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let buttons = toolbar_buttons(&app, toolbar);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let label = |app: &App, b: Entity| {
        let refs = app.world().entity(b).get::<ToolbarButtonRefs>().unwrap();
        let e = app.world().entity(refs.label().unwrap());
        (
            e.get::<Text>().unwrap().0.clone(),
            e.get::<TextColor>().unwrap().0,
        )
    };
    assert_eq!(
        label(&app, buttons[0]),
        ("Select".into(), colors.accent_text)
    );
    assert_eq!(
        label(&app, buttons[1]),
        ("Rail".into(), colors.text_secondary)
    );
}

#[test]
fn clicking_the_open_panel_of_a_zero_or_one_toolbar_closes_it() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().zero_or_one(),
    );
    let buttons = toolbar_buttons(&app, toolbar);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(selected_label(&app, toolbar), None);

    activate(&mut app, buttons[1]);
    assert_eq!(selected_label(&app, toolbar).as_deref(), Some("Rail"));
    assert_eq!(background(&app, buttons[1]), colors.accent_subtle);
    let refs = *app
        .world()
        .entity(buttons[1])
        .get::<ToolbarButtonRefs>()
        .unwrap();
    assert_eq!(node_display(&app, refs.bar().unwrap()), Display::Flex);

    activate(&mut app, buttons[1]);
    assert_eq!(selected_label(&app, toolbar), None);
    assert_eq!(background(&app, buttons[1]), Color::NONE);
    assert_eq!(node_display(&app, refs.bar().unwrap()), Display::None);
}

#[test]
fn a_selection_changed_through_the_selection_param_restyles_the_buttons() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let buttons = toolbar_buttons(&app, toolbar);
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    let target = app
        .world()
        .entity(buttons[3])
        .get::<ToolbarButton>()
        .unwrap()
        .item();

    app.world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            selections.update_selection(toolbar, |s| s.set_selection([target]));
        })
        .unwrap();
    app.update();
    app.update();

    assert_eq!(background(&app, buttons[3]), colors.accent);
    assert_eq!(background(&app, buttons[0]), Color::NONE);
}

#[test]
fn a_toolbar_shares_a_selection_by_reference() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(tools().map(|data| Item { data })),
        )
        .id();
    let toolbar = app
        .world_mut()
        .spawn((
            Toolbar::new(ToolbarStyle::Filled),
            Selection::reference(selection),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.update();
    let buttons = toolbar_buttons(&app, toolbar);
    assert_eq!(buttons.len(), 4);

    activate(&mut app, buttons[3]);

    assert_eq!(selected_label(&app, selection).as_deref(), Some("Train"));
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(background(&app, buttons[3]), colors.accent);
}

#[test]
fn a_toolbar_button_is_named_after_its_tool_for_assistive_tech() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().zero_or_one(),
    );
    let buttons = toolbar_buttons(&app, toolbar);
    let node = app
        .world()
        .entity(buttons[1])
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.label(), Some("Rail"));
}

#[test]
fn a_theme_change_restyles_the_toolbar() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let buttons = toolbar_buttons(&app, toolbar);

    toggle_theme(&mut app);
    app.update();
    app.update();

    assert_eq!(
        background(&app, buttons[0]),
        DEFAULT_DARK_THEME_COLORS.accent
    );
}

fn spawn_gallery(app: &mut App) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let cards = [
        ("Steam", "Locomotives"),
        ("Diesel", "Locomotives"),
        ("Electric", "Locomotives"),
        ("Passenger", "Wagons"),
        ("Box wagon", "Wagons"),
    ]
    .map(|(label, group)| Item {
        data: GalleryCard::new(Handle::default(), label, group),
    });
    let gallery = app
        .world_mut()
        .spawn((
            Gallery,
            Selection::builder().one().build_with_items(cards),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.update();
    gallery
}

fn children_of(app: &App, entity: Entity) -> Vec<Entity> {
    app.world()
        .entity(entity)
        .get::<Children>()
        .map(|children| children.iter().collect())
        .unwrap_or_default()
}

/// The heading and the cards of each group of a gallery.
fn gallery_groups(app: &App, gallery: Entity) -> Vec<(String, Vec<Entity>)> {
    children_of(app, gallery)
        .into_iter()
        .filter(|child| !app.world().entity(*child).contains::<Divider>())
        .map(|group| {
            let parts = children_of(app, group);
            let heading = app
                .world()
                .entity(parts[0])
                .get::<Text>()
                .unwrap()
                .0
                .clone();
            (heading, children_of(app, parts[1]))
        })
        .collect()
}

fn card_label(app: &App, card: Entity) -> String {
    let refs = app.world().entity(card).get::<GalleryCardRefs>().unwrap();
    app.world()
        .entity(refs.label)
        .get::<Text>()
        .unwrap()
        .0
        .clone()
}

fn picked_card_label(app: &App, gallery: Entity) -> Option<String> {
    let selection = app.world().entity(gallery).get::<Selection>().unwrap();
    let item = selection.as_selection().unwrap().single()?;
    app.world()
        .entity(item)
        .get::<Item<GalleryCard>>()
        .map(|item| item.data.label.clone())
}

fn border_top(app: &App, entity: Entity) -> Color {
    app.world().entity(entity).get::<BorderColor>().unwrap().top
}

#[test]
fn a_gallery_creates_a_card_per_item_under_the_heading_of_its_group() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);

    let groups = gallery_groups(&app, gallery)
        .into_iter()
        .map(|(heading, cards)| {
            let labels = cards
                .iter()
                .map(|card| card_label(&app, *card))
                .collect::<Vec<_>>();
            (heading, labels)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        groups,
        [
            (
                "LOCOMOTIVES".to_string(),
                vec!["Steam".to_string(), "Diesel".into(), "Electric".into()]
            ),
            (
                "WAGONS".to_string(),
                vec!["Passenger".to_string(), "Box wagon".into()]
            ),
        ]
    );
    // One vertical divider sits between the two groups.
    let children = children_of(&app, gallery);
    assert_eq!(children.len(), 3);
    assert!(app.world().entity(children[1]).contains::<Divider>());
}

#[test]
fn clicking_a_card_in_the_second_group_deselects_the_one_in_the_first() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let groups = gallery_groups(&app, gallery);
    let first = groups[0].1[0];
    let wagon = groups[1].1[1];
    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(picked_card_label(&app, gallery).as_deref(), Some("Steam"));
    assert_eq!(border_top(&app, first), colors.selection_ring);

    activate(&mut app, wagon);

    assert_eq!(
        picked_card_label(&app, gallery).as_deref(),
        Some("Box wagon")
    );
    assert_eq!(border_top(&app, wagon), colors.selection_ring);
    assert_eq!(border_top(&app, first), colors.border);
    assert_eq!(background(&app, first), colors.card_surface);
}

#[test]
fn a_selection_changed_through_selection_param_moves_the_ring() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let groups = gallery_groups(&app, gallery);
    let target = groups[0].1[2];
    let item = app
        .world()
        .entity(target)
        .get::<GalleryCardView>()
        .unwrap()
        .item;
    let colors = app.world().get_resource::<Theme>().unwrap().colors;

    app.world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            selections.update_selection(gallery, |selection| selection.set_selection([item]));
        })
        .unwrap();
    app.update();
    app.update();

    assert_eq!(border_top(&app, target), colors.selection_ring);
    assert_eq!(background(&app, target), colors.hovered_selected_item);
    assert_eq!(border_top(&app, groups[0].1[0]), colors.border);
    let thickness = |card: Entity| app.world().entity(card).get::<Node>().unwrap().border.left;
    assert_eq!(thickness(target), Val::Px(2.0));
    assert_eq!(thickness(groups[0].1[0]), Val::Px(1.0));
}

#[test]
fn a_gallery_shares_a_selection_by_reference_and_names_its_cards() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["a", "b"].map(|label| Item {
                    data: GalleryCard::new(Handle::default(), label, "G"),
                })),
        )
        .id();
    let gallery = app
        .world_mut()
        .spawn((Gallery, Selection::reference(selection), ChildOf(root)))
        .id();
    app.update();
    app.update();
    app.update();
    let cards = gallery_groups(&app, gallery).remove(0).1;
    assert_eq!(cards.len(), 2);

    activate(&mut app, cards[1]);

    let item = app
        .world()
        .entity(cards[1])
        .get::<GalleryCardView>()
        .unwrap()
        .item;
    let selected = app.world().entity(selection).get::<Selection>().unwrap();
    assert_eq!(selected.as_selection().unwrap().single(), Some(item));
    let node = app
        .world()
        .entity(cards[1])
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.label(), Some("b"));
}

#[test]
fn a_theme_change_restyles_the_gallery() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let groups = gallery_groups(&app, gallery);

    toggle_theme(&mut app);
    app.update();
    app.update();

    let colors = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(border_top(&app, groups[0].1[0]), colors.selection_ring);
    assert_eq!(background(&app, groups[1].1[0]), colors.card_surface);
}

#[test]
fn hiding_the_tool_labels_shrinks_the_buttons_and_showing_them_restores_it() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let buttons = toolbar_buttons(&app, toolbar);
    let state = |app: &App| {
        let world = app.world();
        let node = world.entity(buttons[0]).get::<Node>().unwrap();
        let refs = world.entity(buttons[0]).get::<ToolbarButtonRefs>().unwrap();
        let label = world.entity(refs.label().unwrap()).get::<Node>().unwrap();
        (node.width, node.height, label.display)
    };
    assert_eq!(state(&app), (Val::Px(66.0), Val::Px(60.0), Display::Flex));

    app.world_mut()
        .entity_mut(toolbar)
        .insert(ToolbarLabels::Hidden);
    app.update();
    app.update();
    assert_eq!(state(&app), (Val::Px(56.0), Val::Px(52.0), Display::None));
    // The buttons were laid out again, not respawned.
    assert_eq!(toolbar_buttons(&app, toolbar), buttons);

    app.world_mut()
        .entity_mut(toolbar)
        .insert(ToolbarLabels::Shown);
    app.update();
    app.update();
    assert_eq!(state(&app), (Val::Px(66.0), Val::Px(60.0), Display::Flex));
}

/// Advances `Time` by 200 ms and runs `Update` only: `PreUpdate`'s `update_is_hovered` would
/// put the `Hovered` of the headless app, which has no pointer over anything, back to false.
fn tick(app: &mut App) {
    app.world_mut()
        .get_resource_mut::<Time>()
        .unwrap()
        .advance_by(std::time::Duration::from_millis(200));
    app.world_mut().run_schedule(Update);
}

/// An icon button under a root.
fn tooltip_app() -> (App, Entity) {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Pause", IconButtonSize::Regular),
            ChildOf(root),
        ))
        .id();
    tick(&mut app);
    tick(&mut app);
    (app, button)
}

fn tooltip_popups(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Children, With<TooltipPopup>>();
    let children: Vec<Entity> = query
        .iter(world)
        .flat_map(RelationshipTarget::iter)
        .collect();
    children
        .into_iter()
        .filter_map(|c| world.get::<Text>(c).map(|t| t.0.clone()))
        .collect()
}

#[test]
fn hovering_an_icon_button_longer_than_the_delay_shows_its_name_and_leaving_hides_it() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    tick(&mut app);
    tick(&mut app);
    assert!(
        tooltip_popups(&mut app).is_empty(),
        "shown before the delay"
    );

    for _ in 0..3 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);

    app.world_mut().entity_mut(button).insert(Hovered(false));
    tick(&mut app);
    tick(&mut app);
    assert!(tooltip_popups(&mut app).is_empty());
}

#[test]
fn a_press_hides_the_tooltip_until_the_pointer_has_left_and_returned() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);

    app.world_mut().entity_mut(button).insert(Pressed);
    tick(&mut app);
    tick(&mut app);
    assert!(tooltip_popups(&mut app).is_empty());

    app.world_mut().entity_mut(button).remove::<Pressed>();
    for _ in 0..6 {
        tick(&mut app);
    }
    assert!(
        tooltip_popups(&mut app).is_empty(),
        "came back while still hovered"
    );

    app.world_mut().entity_mut(button).insert(Hovered(false));
    tick(&mut app);
    app.world_mut().entity_mut(button).insert(Hovered(true));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);
}

#[test]
fn a_tooltip_popup_is_a_fixed_child_of_its_node() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    for _ in 0..6 {
        tick(&mut app);
    }
    let world = app.world_mut();
    let popups: Vec<(Option<Entity>, bool)> = world
        .query_filtered::<(Option<&ChildOf>, Has<FixedNode>), With<TooltipPopup>>()
        .iter(world)
        .map(|(parent, fixed)| (parent.map(ChildOf::parent), fixed))
        .collect();
    assert_eq!(popups, [(Some(button), true)]);
}

/// A tooltip app with a primary window of `window` logical pixels and a laid-out node centred at
/// `center`, `size` big: the headless app has no layout, so the sizes are set by hand.
fn placement_app(window: (u32, u32), center: Vec2, size: Vec2) -> (App, Entity) {
    let mut app = headless_app();
    app.world_mut().spawn((
        Window {
            resolution: window.into(),
            ..default()
        },
        PrimaryWindow,
    ));
    let node = app
        .world_mut()
        .spawn((
            Node::default(),
            Tooltip::new("Tip"),
            ComputedNode {
                size,
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from_translation(center),
        ))
        .id();
    app.world_mut().entity_mut(node).insert(Hovered(true));
    for _ in 0..4 {
        tick(&mut app);
    }
    (app, node)
}

/// Gives the popup a laid-out size of `popup`, places it and returns its `left` and `top`.
fn placed_popup(app: &mut App, popup_size: Vec2) -> (f32, f32) {
    let popup = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<TooltipPopup>>()
            .single(world)
            .unwrap()
    };
    app.world_mut().entity_mut(popup).insert(ComputedNode {
        size: popup_size,
        inverse_scale_factor: 1.0,
        ..default()
    });
    tick(app);
    let node = app.world().get::<Node>(popup).unwrap();
    let px = |val: Val| match val {
        Val::Px(px) => Some(px),
        _ => None,
    };
    (px(node.left).unwrap(), px(node.top).unwrap())
}

#[test]
fn a_tooltip_popup_keeps_six_pixels_from_the_left_and_right_window_edges() {
    let popup = Vec2::new(100.0, 20.0);
    let (mut app, _) = placement_app((800, 600), Vec2::new(10.0, 100.0), Vec2::new(20.0, 20.0));
    assert!((placed_popup(&mut app, popup).0 - 6.0).abs() < 0.01);
    let (mut app, _) = placement_app((800, 600), Vec2::new(790.0, 100.0), Vec2::new(20.0, 20.0));
    let (left, _) = placed_popup(&mut app, popup);
    assert!(
        (left + popup.x - (800.0 - 6.0)).abs() < 0.01,
        "left {left} leaves less than 6 px at the right edge"
    );
    let (mut app, _) = placement_app((800, 600), Vec2::new(400.0, 100.0), Vec2::new(20.0, 20.0));
    assert!(
        (placed_popup(&mut app, popup).0 - 350.0).abs() < 0.01,
        "a node away from the edges has its popup centred"
    );
}

#[test]
fn a_tooltip_popup_is_below_its_node_or_above_it_when_it_would_run_off_the_bottom() {
    let popup = Vec2::new(100.0, 20.0);
    let (mut app, _) = placement_app((800, 600), Vec2::new(400.0, 100.0), Vec2::new(20.0, 20.0));
    // Below: the node's bottom (110) and the 6 px gap.
    assert!((placed_popup(&mut app, popup).1 - 116.0).abs() < 0.01);
    let (mut app, _) = placement_app((800, 600), Vec2::new(400.0, 580.0), Vec2::new(20.0, 20.0));
    // Above: the node's top (570), the gap and the popup's height.
    assert!((placed_popup(&mut app, popup).1 - 544.0).abs() < 0.01);
}

#[test]
fn a_tooltip_popup_above_a_node_in_a_short_window_stays_below_the_window_margin() {
    let (mut app, _) = placement_app((800, 60), Vec2::new(400.0, 30.0), Vec2::new(20.0, 20.0));
    let (_, top) = placed_popup(&mut app, Vec2::new(100.0, 40.0));
    assert!(top >= 6.0, "top {top} is above the window margin");
}

#[test]
fn a_tooltip_with_empty_text_shows_no_popup() {
    let (mut app, node) = placement_app((800, 600), Vec2::new(400.0, 100.0), Vec2::new(20.0, 20.0));
    app.world_mut().entity_mut(node).insert(Tooltip::new(""));
    for _ in 0..4 {
        tick(&mut app);
    }
    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, With<TooltipPopup>>()
            .iter(world)
            .count(),
        0,
        "an empty tooltip shows a popup"
    );
    app.world_mut()
        .entity_mut(node)
        .insert(Tooltip::new("Back"));
    tick(&mut app);
    tick(&mut app);
    assert_eq!(tooltip_popups(&mut app), ["Back"]);
}

#[test]
fn a_tooltip_removed_and_inserted_again_shows_its_popup_on_the_next_hover() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);

    app.world_mut().entity_mut(button).remove::<Tooltip>();
    tick(&mut app);
    tick(&mut app);
    assert!(tooltip_popups(&mut app).is_empty());

    app.world_mut()
        .entity_mut(button)
        .insert(Tooltip::new("Again"));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Again"]);
}

#[test]
fn despawning_the_icon_button_removes_its_tooltip() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);

    app.world_mut().entity_mut(button).despawn();
    tick(&mut app);
    assert!(tooltip_popups(&mut app).is_empty());
}

#[test]
fn only_the_buttons_of_an_underlined_toolbar_have_a_tooltip_with_their_label() {
    let mut app = headless_app();
    let underlined = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().zero_or_one(),
    );
    let filled = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    app.update();
    let rail = toolbar_buttons(&app, underlined)[1];
    assert_eq!(
        app.world().get::<Tooltip>(rail).map(|t| t.text.as_str()),
        Some("Rail")
    );
    for button in toolbar_buttons(&app, filled) {
        assert!(app.world().get::<Tooltip>(button).is_none());
    }
}

#[test]
fn a_tooltip_the_app_put_on_an_underlined_toolbar_button_is_kept() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().zero_or_one(),
    );
    let select = toolbar_buttons(&app, toolbar)[0];
    app.world_mut()
        .entity_mut(select)
        .insert(Tooltip::new("Select (1)"));
    app.update();
    app.update();
    assert_eq!(
        app.world().get::<Tooltip>(select).map(|t| t.text.as_str()),
        Some("Select (1)")
    );
}

/// The badge node under `host`, with its text, if there is one.
fn badge_under(app: &App, host: Entity) -> Option<(Entity, String)> {
    let children = app.world().entity(host).get::<Children>()?;
    children.iter().find_map(|c| {
        let e = app.world().entity(c);
        (e.get::<Name>().is_some_and(|n| n.as_str() == "Badge"))
            .then(|| (c, e.get::<Text>().map(|t| t.0.clone()).unwrap_or_default()))
    })
}

fn settle(app: &mut App) {
    for _ in 0..3 {
        app.update();
    }
}

#[test]
fn a_badge_on_an_icon_button_appears_updates_and_disappears() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Trains", IconButtonSize::Regular),
            ChildOf(root),
        ))
        .id();
    settle(&mut app);
    assert_eq!(badge_under(&app, button), None);

    app.world_mut().entity_mut(button).insert(Badge::count(4));
    settle(&mut app);
    let (badge, text) = badge_under(&app, button).unwrap();
    assert_eq!(text, "4");
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    let world = app.world();
    assert_eq!(
        world.get::<BackgroundColor>(badge).unwrap().0,
        colors.pill_background
    );
    assert_eq!(world.get::<TextColor>(badge).unwrap().0, colors.pill_text);
    let node = world.get::<Node>(badge).unwrap();
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.height, Val::Rem(0.8));
    assert_eq!(node.min_width, Val::Rem(0.8));
    assert_eq!(world.get::<Pickable>(badge).unwrap(), &Pickable::IGNORE);

    app.world_mut()
        .entity_mut(button)
        .insert(Badge::count(12).with_tone(BadgeTone::Warning));
    settle(&mut app);
    let (same, text) = badge_under(&app, button).unwrap();
    assert_eq!((same, text.as_str()), (badge, "12"));
    assert_eq!(
        app.world().get::<BackgroundColor>(badge).unwrap().0,
        colors.warning_text
    );

    app.world_mut().entity_mut(button).insert(Badge::new(""));
    settle(&mut app);
    assert_eq!(badge_under(&app, button), None);

    app.world_mut().entity_mut(button).insert(Badge::count(1));
    settle(&mut app);
    assert!(badge_under(&app, button).is_some());
    app.world_mut().entity_mut(button).remove::<Badge>();
    settle(&mut app);
    assert_eq!(badge_under(&app, button), None);
}

#[test]
fn a_badge_on_a_toolbar_item_shows_on_its_button_and_follows_the_item() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().one(),
    );
    let buttons = toolbar_buttons(&app, toolbar);
    let item = app.world().get::<ToolbarButton>(buttons[1]).unwrap().item();
    assert_eq!(badge_under(&app, buttons[1]), None);

    app.world_mut().entity_mut(item).insert(Badge::count(6));
    settle(&mut app);
    assert_eq!(badge_under(&app, buttons[1]).unwrap().1, "6");
    assert_eq!(badge_under(&app, buttons[0]), None);

    app.world_mut().entity_mut(item).insert(Badge::count(7));
    settle(&mut app);
    assert_eq!(badge_under(&app, buttons[1]).unwrap().1, "7");

    app.world_mut().entity_mut(item).remove::<Badge>();
    settle(&mut app);
    assert_eq!(badge_under(&app, buttons[1]), None);
}

#[test]
fn a_badge_on_a_toolbar_item_describes_its_button_for_accessibility() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().one(),
    );
    let buttons = toolbar_buttons(&app, toolbar);
    let item = app.world().get::<ToolbarButton>(buttons[1]).unwrap().item();
    let description = |app: &App, button: Entity| {
        app.world()
            .get::<AccessibilityNode>(button)
            .and_then(|node| node.description().map(str::to_owned))
    };
    assert_eq!(description(&app, buttons[1]), None);

    app.world_mut().entity_mut(item).insert(Badge::count(6));
    settle(&mut app);
    assert_eq!(description(&app, buttons[1]).as_deref(), Some("6"));
    assert_eq!(description(&app, buttons[0]), None);
    let (badge, _) = badge_under(&app, buttons[1]).unwrap();
    assert!(app.world().get::<AccessibilityNode>(badge).is_none());

    app.world_mut().entity_mut(item).insert(Badge::count(7));
    settle(&mut app);
    assert_eq!(description(&app, buttons[1]).as_deref(), Some("7"));

    app.world_mut().entity_mut(item).remove::<Badge>();
    settle(&mut app);
    assert_eq!(description(&app, buttons[1]), None);
}

fn spawn_toned_icon_button(app: &mut App, tone: Tone, size: IconButtonSize) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Tone", size).with_tone(tone),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    button
}

fn icon_tint(app: &App, button: Entity) -> Color {
    let icon = app.world().get::<IconButtonRefs>(button).unwrap().icon();
    app.world().get::<ImageNode>(icon).unwrap().color
}

#[test]
fn a_warning_icon_button_has_the_warning_colours_at_rest_hovered_and_on() {
    let mut app = headless_app();
    let button = spawn_toned_icon_button(&mut app, Tone::Warning, IconButtonSize::Large);
    let light = DEFAULT_LIGHT_THEME_COLORS;

    let node = app.world().get::<Node>(button).unwrap();
    assert_eq!((node.width, node.height), (Val::Px(54.0), Val::Px(54.0)));
    assert_eq!(node.border, UiRect::all(Val::Px(1.0)));
    assert_eq!(background(&app, button), light.warning_background);
    assert_eq!(border_top(&app, button), light.warning_border);
    assert_eq!(icon_tint(&app, button), light.warning_text);
    assert_eq!(
        app.world().get::<BoxShadow>(button).unwrap().0[0].color,
        light.large_shadow
    );

    // Hovered, under the dark theme.
    let dark = DEFAULT_DARK_THEME_COLORS;
    hover_and_toggle_theme(&mut app, button);
    app.update();
    let brighter = dark.warning_background.mix(&dark.warning_border, 0.25);
    assert_eq!(background(&app, button), brighter);
    assert_ne!(brighter, dark.warning_background);
    assert_eq!(border_top(&app, button), dark.warning_border);
    assert_eq!(icon_tint(&app, button), dark.warning_text);

    // On: a 3 px bar in the border colour.
    app.world_mut().entity_mut(button).insert(ToggleState(true));
    app.update();
    app.update();
    let bar = app.world().get::<Children>(button).unwrap()[1];
    let bar_node = app.world().get::<Node>(bar).unwrap();
    assert_eq!(bar_node.display, Display::Flex);
    assert_eq!(bar_node.height, Val::Px(3.0));
    assert_eq!(
        app.world().get::<BackgroundColor>(bar).unwrap().0,
        dark.warning_border
    );
}

#[test]
fn a_danger_icon_button_turns_danger_when_hovered_and_filled_when_on() {
    let mut app = headless_app();
    let button = spawn_toned_icon_button(&mut app, Tone::Danger, IconButtonSize::Regular);
    let light = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, button), light.background_primary);
    assert_eq!(icon_tint(&app, button), light.text_primary);
    assert_eq!(
        app.world().get::<Node>(button).unwrap().border,
        UiRect::ZERO
    );

    let dark = DEFAULT_DARK_THEME_COLORS;
    hover_and_toggle_theme(&mut app, button);
    app.update();
    assert_eq!(background(&app, button), dark.hovered_item);
    assert_eq!(icon_tint(&app, button), dark.danger_hover_text);

    app.world_mut().entity_mut(button).insert(ToggleState(true));
    app.update();
    app.update();
    assert_eq!(background(&app, button), dark.danger);
    assert_eq!(icon_tint(&app, button), dark.danger_text);
}

#[test]
fn a_neutral_icon_button_keeps_its_ordinary_colours() {
    let mut app = headless_app();
    let button = spawn_toned_icon_button(&mut app, Tone::Neutral, IconButtonSize::Regular);
    let light = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, button), light.background_primary);
    assert_eq!(icon_tint(&app, button), light.text_primary);

    app.world_mut().entity_mut(button).insert(ToggleState(true));
    app.update();
    app.update();
    assert_eq!(background(&app, button), light.accent_subtle);
}

#[test]
fn a_danger_tool_gets_the_danger_colours_at_rest_hovered_and_active() {
    for style in [ToolbarStyle::Filled, ToolbarStyle::Underlined] {
        let mut app = headless_app();
        let root = app.world_mut().spawn(Node::default()).id();
        let toolbar = app
            .world_mut()
            .spawn((
                Toolbar::new(style),
                Selection::builder().zero_or_one().build_with_items([
                    Item {
                        data: Tool::new(Handle::default(), "Select"),
                    },
                    Item {
                        data: Tool::new(Handle::default(), "Bulldoze").with_tone(Tone::Danger),
                    },
                ]),
                ChildOf(root),
            ))
            .id();
        for _ in 0..3 {
            app.update();
        }
        let buttons = toolbar_buttons(&app, toolbar);
        let bulldoze = buttons[1];
        let tint = |app: &App, b: Entity| {
            let icon = app.world().get::<ToolbarButtonRefs>(b).unwrap().icon();
            app.world().get::<ImageNode>(icon).unwrap().color
        };
        let light = DEFAULT_LIGHT_THEME_COLORS;
        assert_eq!(background(&app, bulldoze), Color::NONE);

        app.world_mut().entity_mut(bulldoze).insert(Hovered(true));
        app.world_mut().run_schedule(Update);
        app.world_mut().run_schedule(Update);
        assert_eq!(background(&app, bulldoze), light.hovered_item);
        assert_eq!(tint(&app, bulldoze), light.danger_hover_text);
        let refs = *app.world().get::<ToolbarButtonRefs>(bulldoze).unwrap();
        if let Some(label) = refs.label() {
            assert_eq!(
                app.world().get::<TextColor>(label).unwrap().0,
                light.danger_hover_text
            );
        }

        app.world_mut().entity_mut(bulldoze).insert(Hovered(false));
        activate(&mut app, bulldoze);
        app.update();
        assert_eq!(background(&app, bulldoze), light.danger);
        assert_eq!(tint(&app, bulldoze), light.danger_text);
        // The neutral tool next to it is untouched.
        assert_eq!(background(&app, buttons[0]), Color::NONE);
    }
}

#[test]
fn a_scene_points_a_stack_at_a_selection_named_in_the_same_scene() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins((
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        MeshPlugin,
        VisibilityPlugin,
    ));
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins((InputPlugin, DefaultPickingPlugins));
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let items = [(Item::new("a"), StackItem::new(create_a))];
    app.world_mut().commands().spawn_scene(bsn! {
        Node
        Children [
            #sel
            template(move |context| {
                context
                    .entity
                    .insert(Selection::builder().one().build_with_items(items.clone()));
                Ok(bevy::scene::EmptySceneEffect)
            })
            --
            Stack { selection_ref: #sel }
        ]
    });
    for _ in 0..4 {
        app.update();
    }
    single_entity_with::<PanelA>(&mut app);
}

#[test]
fn the_hud_example_builds_and_runs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    // The HUD is spawned from scenes (`bsn!`), which need the asset and scene plugins.
    app.add_plugins((AssetPlugin::default(), bevy::scene::ScenePlugin));
    app.init_resource::<ClearColor>();
    app.init_resource::<Assets<Image>>();
    app.add_plugins((WidgetsPlugin, hud_ui::HudPlugin));
    app.finish();
    app.cleanup();
    for _ in 0..6 {
        app.update();
    }
    assert_eq!(ui_root_count(&mut app), 1);
    let world = app.world_mut();
    // The Trains panel is open at the start, with a row for each of the four trains,
    // and the Warnings panel is made hidden.
    assert_eq!(world.query::<&Panel>().iter(world).count(), 2);
    assert_eq!(world.query::<&ItemView>().iter(world).count(), 4);
    // The run-state card has the simulation speed slider.
    assert_eq!(world.query::<&Slider>().iter(world).count(), 1);
    // Four panel tools and five dock tools.
    assert_eq!(world.query::<&ToolbarButton>().iter(world).count(), 9);
}

#[test]
fn the_overview_example_builds_and_every_page_runs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.add_plugins((AssetPlugin::default(), bevy::scene::ScenePlugin));
    app.init_resource::<ClearColor>();
    app.init_resource::<Assets<Image>>();
    app.add_plugins((WidgetsPlugin, overview::OverviewPlugin));
    app.finish();
    app.cleanup();
    for _ in 0..6 {
        app.update();
    }
    assert_eq!(ui_root_count(&mut app), 1);
    let bar = single_entity_with::<TabBar>(&mut app);
    // Showing each page in turn builds it: none of them panics or logs a missing widget.
    for tab in [
        "Buttons",
        "Icon buttons",
        "Labels",
        "Tags",
        "Lists",
        "Combos",
        "Colours",
        "Segmented control",
        "Filter chips",
        "Tabs and trails",
        "Slider and stepper",
        "Text field",
        "Toolbar",
        "Gallery",
        "Cards and panels",
        "Setting rows",
        "Scroll area",
        "Tooltip and toast",
    ] {
        select_by_text(&mut app, bar, tab);
        for _ in 0..4 {
            app.update();
        }
    }
    assert_eq!(ui_root_count(&mut app), 1);
}

#[test]
fn widgets_with_handles_and_scene_constructors_are_written_directly_in_a_scene() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.add_plugins((AssetPlugin::default(), bevy::scene::ScenePlugin));
    app.init_resource::<ClearColor>();
    app.init_resource::<Assets<Image>>();
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    let icon = app
        .world_mut()
        .get_resource_mut::<Assets<Image>>()
        .unwrap()
        .add(Image::default());
    app.world_mut()
        .spawn_scene(bsn! {
            Node
            Children [
                IconButton { icon: {icon.clone()}, name: "Close", size: IconButtonSize::Large, tone: Tone::Danger }
                --
                Panel { title: "Trains", close_icon: {HandleTemplate::Handle(icon.clone())} }
                --
                NumericStepper { decrease_name: "Lower" }
                --
                @{Divider::vertical_scene()}
                --
                SegmentedControl
                @{Selection::builder().one().scene_with_items(["Auto", "Manual"].map(Item::new))}
            ]
        })
        .unwrap();
    for _ in 0..3 {
        app.update();
    }
    let world = app.world_mut();
    // The panel's close button is an `IconButton` too.
    let button = world
        .query::<&IconButton>()
        .iter(world)
        .find(|button| button.tone == Tone::Danger)
        .unwrap();
    assert_eq!(button.icon, icon);
    assert_eq!(button.name, "Close");
    assert_eq!(button.size, IconButtonSize::Large);
    let panel = world.query::<&Panel>().single(world).unwrap();
    assert_eq!(panel.close_icon.as_ref(), Some(&icon));
    // What is not written is as `Panel::new` has it.
    assert!((panel.max_body_height() - Panel::new("").max_body_height()).abs() < f32::EPSILON);
    let stepper = world.query::<&NumericStepper>().single(world).unwrap();
    assert_eq!(stepper.decrease_name, "Lower");
    assert_eq!(stepper.increase_name, "Increase");
    // The panel has two dividers of its own; the scene's is the vertical one.
    assert_eq!(
        world
            .query::<(&Divider, &Node)>()
            .iter(world)
            .filter(|(_, node)| node.width == Val::Px(1.0))
            .count(),
        1
    );
    let selection = world.query::<&Selection>().single(world).unwrap();
    assert!(matches!(selection, Selection::Value(_)));
    assert_eq!(world.query::<&Item<String>>().iter(world).count(), 2);
}

#[test]
fn a_scroll_area_scene_has_its_contents_as_a_scene_and_both_scroll_bars() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: None,
        ..default()
    });
    app.add_plugins(InputPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.add_plugins((AssetPlugin::default(), bevy::scene::ScenePlugin));
    app.init_resource::<ClearColor>();
    app.add_plugins(WidgetsPlugin);
    app.finish();
    app.cleanup();
    app.world_mut().spawn(Camera2d);
    app.world_mut()
        .spawn_scene(bsn! {
            Node
            Children [
                @ScrollArea::scene(
                    bsn_list! {
                        Node
                        --
                        Node
                        --
                        Node
                    },
                    true,
                    true,
                )
            ]
        })
        .unwrap();
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    let (area, node) = world
        .query_filtered::<(Entity, &Node), With<ScrollArea>>()
        .single(world)
        .unwrap();
    assert_eq!(node.display, Display::Grid);
    assert!(world.get::<ChildOf>(area).is_some());
    let contents = world
        .query_filtered::<&Children, With<ScrollContents>>()
        .single(world)
        .unwrap();
    assert_eq!(contents.len(), 3);
    assert_eq!(world.query::<&ScrollBar>().iter(world).count(), 2);
    assert_eq!(ui_root_count_world(world), 1);
}

fn ui_root_count_world(world: &mut World) -> usize {
    world
        .query_filtered::<(), With<UiRoot>>()
        .iter(world)
        .count()
}

fn ui_root_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<UiRoot>>()
        .iter(app.world())
        .count()
}

#[test]
fn a_toolbar_in_either_style_leaves_exactly_one_ui_root() {
    for style in [ToolbarStyle::Filled, ToolbarStyle::Underlined] {
        let mut app = headless_app();
        spawn_toolbar(&mut app, style, Selection::builder().one());
        assert_eq!(ui_root_count(&mut app), 1, "{style:?}");
    }
}

#[test]
fn filter_chips_and_a_gallery_leave_exactly_one_ui_root() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let chips_selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(three_chips().map(|data| Item { data })),
        )
        .id();
    app.world_mut().spawn((
        FilterChips,
        Selection::reference(chips_selection),
        ChildOf(root),
    ));
    let cards_selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["a", "b"].map(|label| Item {
                    data: GalleryCard::new(Handle::default(), label, "G"),
                })),
        )
        .id();
    app.world_mut().spawn((
        Gallery,
        Selection::reference(cards_selection),
        ChildOf(root),
    ));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(ui_root_count(&mut app), 1);
}

#[test]
fn a_numeric_stepper_is_a_spin_button_with_numeric_bounds_and_step() {
    let mut app = headless_app();
    let (stepper, _, increase) = spawn_stepper(&mut app, 1);

    let node = app
        .world()
        .entity(stepper)
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.role(), accesskit::Role::SpinButton);
    assert_eq!(node.numeric_value(), Some(1.0));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(3.0));
    assert_eq!(node.numeric_value_step(), Some(1.0));
    assert_eq!(node.value(), Some("1"));
    assert_eq!(node.label(), None);

    activate(&mut app, increase);
    let node = app
        .world()
        .entity(stepper)
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.numeric_value(), Some(2.0));
    assert_eq!(node.value(), Some("2"));
}

/// A stepper over `value`, returning it and its decrease and increase buttons.
fn spawn_stepper_of(app: &mut App, value: Value) -> (Entity, Entity, Entity) {
    let stepper = app
        .world_mut()
        .spawn((NumericStepper::default(), value))
        .id();
    app.update();
    app.update();
    let children = app.world().entity(stepper).get::<Children>().unwrap();
    (stepper, children[0], children[2])
}

#[test]
fn a_stepper_with_equal_bounds_disables_both_buttons_and_reports_its_numbers() {
    let mut app = headless_app();
    let (stepper, decrease, increase) =
        spawn_stepper_of(&mut app, Value::new_u8(5, 5, 5, 1).unwrap());

    assert!(is_disabled(&app, decrease));
    assert!(is_disabled(&app, increase));
    let node = app
        .world()
        .entity(stepper)
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.numeric_value(), Some(5.0));
    assert_eq!(node.min_numeric_value(), Some(5.0));
    assert_eq!(node.max_numeric_value(), Some(5.0));
    assert_eq!(node.numeric_value_step(), Some(1.0));
    assert_eq!(node.value(), Some("5"));
}

#[test]
fn a_stepper_with_a_step_larger_than_its_range_disables_both_buttons() {
    let mut app = headless_app();
    let (stepper, decrease, increase) =
        spawn_stepper_of(&mut app, Value::new_u16(2, 0, 3, 10).unwrap());

    assert!(is_disabled(&app, decrease));
    assert!(is_disabled(&app, increase));
    let node = app
        .world()
        .entity(stepper)
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.numeric_value(), Some(2.0));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(3.0));
    assert_eq!(node.numeric_value_step(), Some(10.0));

    let (_, decrease, increase) =
        spawn_stepper_of(&mut app, Value::new_f32(0.5, 0.0, 1.0, 2.0).unwrap());
    assert!(is_disabled(&app, decrease));
    assert!(is_disabled(&app, increase));
}

#[test]
fn mirrors_of_a_shared_stepper_value_follow_it_and_disable_at_its_limits() {
    let mut app = headless_app();
    let (shared, _, _) = spawn_stepper_of(&mut app, Value::new_u8(1, 0, 2, 1).unwrap());
    let mirror = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(shared)))
        .id();
    app.update();
    app.update();
    let children = app.world().entity(mirror).get::<Children>().unwrap();
    let (decrease, increase) = (children[0], children[2]);

    activate(&mut app, increase);
    assert_eq!(stepper_value(&app, shared), 2);
    assert!(is_disabled(&app, increase));
    assert!(!is_disabled(&app, decrease));
    let node = app
        .world()
        .entity(mirror)
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.numeric_value(), Some(2.0));
    assert_eq!(node.max_numeric_value(), Some(2.0));
    assert_eq!(node.value(), Some("2"));
}

#[test]
fn a_slider_is_a_slider_node_with_its_value_bounds_and_step() {
    let mut app = headless_app();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let stepped = world
        .spawn(
            Slider::new(2.0, 0.0, 10.0)
                .unwrap()
                .with_step(1.0)
                .unwrap()
                .with_format(|value| format!("{value:.0} km"))
                .with_name("Distance"),
        )
        .id();
    let continuous = world.spawn(Slider::new(25.0, 0.0, 100.0).unwrap()).id();
    app.update();
    app.update();

    let node = |app: &App, entity: Entity| -> AccessibilityNode {
        app.world()
            .entity(entity)
            .get::<AccessibilityNode>()
            .unwrap()
            .clone()
    };
    let stepped_node = node(&app, stepped);
    assert_eq!(stepped_node.role(), accesskit::Role::Slider);
    assert_eq!(stepped_node.numeric_value(), Some(2.0));
    assert_eq!(stepped_node.min_numeric_value(), Some(0.0));
    assert_eq!(stepped_node.max_numeric_value(), Some(10.0));
    assert_eq!(stepped_node.numeric_value_step(), Some(1.0));
    assert_eq!(stepped_node.value(), Some("2 km"));
    assert_eq!(stepped_node.label(), Some("Distance"));

    // A continuous slider steps by a hundredth of its range.
    let continuous_node = node(&app, continuous);
    assert_eq!(continuous_node.role(), accesskit::Role::Slider);
    assert_eq!(continuous_node.numeric_value(), Some(25.0));
    assert_eq!(continuous_node.min_numeric_value(), Some(0.0));
    assert_eq!(continuous_node.max_numeric_value(), Some(100.0));
    assert_eq!(continuous_node.numeric_value_step(), Some(1.0));
    assert_eq!(continuous_node.value(), None);
    assert_eq!(continuous_node.label(), None);

    app.world_mut().trigger(SetSliderValue {
        entity: stepped,
        change: SliderValueChange::Absolute(7.0),
    });
    app.update();
    let stepped_node = node(&app, stepped);
    assert_eq!(stepped_node.numeric_value(), Some(7.0));
    assert_eq!(stepped_node.value(), Some("7 km"));

    // Inserted again without a format, it has no value text left.
    app.world_mut()
        .entity_mut(stepped)
        .insert(Slider::new(7.0, 0.0, 10.0).unwrap().with_step(1.0).unwrap());
    app.update();
    assert_eq!(node(&app, stepped).value(), None);
}

#[test]
fn a_zero_range_slider_has_finite_data_and_keys_cannot_change_its_value() {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};

    #[derive(Resource, Default)]
    struct Seen(Vec<f32>);

    let mut app = headless_app();
    app.init_resource::<Seen>();
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let slider = world
        .spawn(Slider::new(9.0, 3.0, 3.0).unwrap().with_step(1.0).unwrap())
        .observe(|change: On<ValueChange<f32>>, mut seen: ResMut<Seen>| {
            seen.0.push(change.value);
        })
        .id();
    app.update();
    app.update();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(slider));
    for key in [
        (KeyCode::ArrowRight, Key::ArrowRight),
        (KeyCode::ArrowLeft, Key::ArrowLeft),
        (KeyCode::Home, Key::Home),
        (KeyCode::End, Key::End),
    ] {
        app.world_mut().write_message(KeyboardInput {
            key_code: key.0,
            logical_key: key.1,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
        app.update();
    }

    let entity = app.world().entity(slider);
    assert!((entity.get::<SliderValue>().unwrap().0 - 3.0).abs() < f32::EPSILON);
    assert!(
        app.world()
            .get_resource::<Seen>()
            .unwrap()
            .0
            .iter()
            .all(|value| (*value - 3.0).abs() < f32::EPSILON),
        "no key may report a changed value"
    );
    let node = entity.get::<AccessibilityNode>().unwrap();
    for number in [
        node.numeric_value(),
        node.min_numeric_value(),
        node.max_numeric_value(),
        node.numeric_value_step(),
    ] {
        assert!(number.unwrap().is_finite());
    }
    assert_eq!(node.numeric_value(), Some(3.0));
    let refs = *entity.get::<SliderRefs>().unwrap();
    let left = app.world().entity(refs.thumb).get::<Node>().unwrap().left;
    let width = app.world().entity(refs.fill).get::<Node>().unwrap().width;
    for val in [left, width] {
        assert!(matches!(val, Val::Percent(percent) if percent.is_finite()));
    }
}

#[test]
fn a_slider_without_a_label_leaves_the_apps_right_margin_alone() {
    let mut app = headless_app();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let margin = UiRect::all(Val::Px(8.0));
    let slider = world
        .spawn((
            Slider::new(0.0, 0.0, 1.0).unwrap(),
            Node {
                margin,
                ..default()
            },
        ))
        .id();
    app.update();
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .light_dark_toggle();
    app.update();
    let node = app.world().entity(slider).get::<Node>().unwrap();
    assert_eq!(node.margin, margin);
}

#[test]
fn a_slider_thumb_takes_the_themes_control_radius_and_keeps_it_while_used() {
    let mut app = headless_app();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let slider = world.spawn(Slider::new(2.0, 0.0, 10.0).unwrap()).id();
    app.update();
    app.update();
    let thumb = app
        .world()
        .entity(slider)
        .get::<SliderRefs>()
        .unwrap()
        .thumb;
    let radius = |app: &App| {
        app.world()
            .entity(thumb)
            .get::<Node>()
            .unwrap()
            .border_radius
    };
    let control_radius = app.world().get_resource::<Theme>().unwrap().control_radius;
    assert_eq!(radius(&app), BorderRadius::all(control_radius));

    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .control_radius = Val::Px(3.0);
    app.update();
    assert_eq!(radius(&app), BorderRadius::all(Val::Px(3.0)));

    // A value change restyles the thumb but does not write its radius: a radius
    // the theme would not give is still there after it.
    let own_radius = BorderRadius::all(Val::Px(9.0));
    app.world_mut()
        .entity_mut(thumb)
        .get_mut::<Node>()
        .unwrap()
        .border_radius = own_radius;
    app.world_mut().trigger(SetSliderValue {
        entity: slider,
        change: SliderValueChange::Absolute(7.0),
    });
    app.update();
    // The thumb moved, so the slider was restyled.
    assert_eq!(
        app.world().entity(thumb).get::<Node>().unwrap().left,
        Val::Percent(70.0)
    );
    assert_eq!(radius(&app), own_radius);
}

#[test]
fn a_list_row_shows_its_items_note_and_follows_changes_to_it() {
    let mut app = headless_app();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let list = world
        .spawn((
            Node::default(),
            List,
            Selection::builder().one().build_with_items([
                (Item::new("Delft"), ItemNote::warning("< unreachable >")),
                (Item::new("Utrecht"), ItemNote::new("")),
            ]),
        ))
        .id();
    for _ in 0..4 {
        app.update();
    }

    let row = |app: &mut App, label: &str| -> Entity {
        let world = app.world_mut();
        let mut views = world.query::<(Entity, &ItemView)>();
        let mut items = Vec::new();
        for (view, ItemView { item_ref, .. }) in views.iter(world) {
            items.push((view, *item_ref));
        }
        items
            .into_iter()
            .find(|(_, item)| {
                world
                    .get::<Item<String>>(*item)
                    .is_some_and(|i| i.data == label)
            })
            .map(|(view, _)| view)
            .unwrap()
    };
    let name = |app: &App, view: Entity| {
        app.world()
            .get::<AccessibilityNode>(view)
            .and_then(|node| node.label().map(str::to_string))
    };
    let texts = |app: &mut App, view: Entity| -> Vec<String> {
        let world = app.world_mut();
        let children: Vec<Entity> = world
            .get::<Children>(view)
            .map(|c| c.iter().collect())
            .unwrap_or_default();
        children
            .into_iter()
            .filter_map(|c| world.get::<Text>(c).map(|t| t.0.clone()))
            .collect()
    };

    let delft = row(&mut app, "Delft");
    let utrecht = row(&mut app, "Utrecht");
    assert_eq!(texts(&mut app, delft), ["Delft", "< unreachable >"]);
    assert_eq!(name(&app, delft).as_deref(), Some("Delft < unreachable >"));
    assert_eq!(texts(&mut app, utrecht), ["Utrecht"]);
    assert_eq!(name(&app, utrecht).as_deref(), Some("Utrecht"));

    // Changing a note, adding one and removing one update the rows in place.
    let item_of = |app: &App, view: Entity| app.world().get::<ItemView>(view).unwrap().item_ref;
    let (delft_item, utrecht_item) = (item_of(&app, delft), item_of(&app, utrecht));
    app.world_mut()
        .entity_mut(delft_item)
        .insert(ItemNote::new("closed"));
    app.world_mut()
        .entity_mut(utrecht_item)
        .insert(ItemNote::new("4 km"));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(texts(&mut app, delft), ["Delft", "closed"]);
    assert_eq!(name(&app, delft).as_deref(), Some("Delft closed"));
    assert_eq!(texts(&mut app, utrecht), ["Utrecht", "4 km"]);
    app.world_mut().entity_mut(delft_item).remove::<ItemNote>();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(texts(&mut app, delft), ["Delft"]);
    assert_eq!(name(&app, delft).as_deref(), Some("Delft"));
    assert!(app.world().get_entity(list).is_ok());
}

#[test]
fn reinserting_a_toast_on_a_live_entity_restarts_its_timer() {
    use bevy::time::TimeUpdateStrategy;
    use rin_bevy_ui::toast::Toast;
    use std::time::Duration;

    let mut app = headless_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        250,
    )));
    let root = app.world_mut().spawn(Node::default()).id();
    let toast = app
        .world_mut()
        .spawn((Toast::new("Saved").with_duration(2.0), ChildOf(root)))
        .id();
    for _ in 0..7 {
        app.update();
    }
    // About 1.5 s of the 2 s have passed (a frame is 0.25 s, the virtual clock's largest step); re-inserting restarts the clock.
    app.world_mut()
        .entity_mut(toast)
        .insert(Toast::new("Saved").with_duration(2.0));
    for _ in 0..6 {
        app.update();
    }
    assert!(
        app.world().get_entity(toast).is_ok(),
        "the toast went after its first timer"
    );
    for _ in 0..4 {
        app.update();
    }
    assert!(
        app.world().get_entity(toast).is_err(),
        "the toast outlived its restarted timer"
    );
}

/// The option of the toggle that sets `preference`.
fn light_dark_option(app: &mut App, preference: ThemePreference) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &LightDarkOption)>()
        .iter(world)
        .find(|(_, option)| option.preference == preference)
        .map(|(entity, _)| entity)
        .unwrap()
}

/// Tell the app that the system theme became `theme`.
fn system_theme_changes(app: &mut App, theme: WindowTheme) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .write_message(WindowThemeChanged { window, theme });
    app.update();
    app.update();
}

#[test]
fn a_light_dark_toggle_has_a_light_a_system_and_a_dark_option() {
    let mut app = headless_app();
    let toggle = app.world_mut().spawn(LightDarkToggle).id();
    app.update();
    app.update();
    let world = app.world_mut();
    let mut options = world
        .query::<(&LightDarkOption, &ChildOf)>()
        .iter(world)
        .map(|(option, parent)| (option.preference, parent.parent()))
        .collect::<Vec<_>>();
    options.sort_by_key(|(preference, _)| *preference as u8);
    assert_eq!(
        options,
        vec![
            (ThemePreference::Light, toggle),
            (ThemePreference::System, toggle),
            (ThemePreference::Dark, toggle),
        ]
    );
    assert_eq!(
        *app.world().get_resource::<ThemePreference>().unwrap(),
        ThemePreference::System,
        "the toggle follows the system by default"
    );
}

#[test]
fn activating_a_light_dark_option_sets_the_preference_and_the_theme_mode() {
    let mut app = headless_app();
    app.world_mut().spawn(LightDarkToggle);
    app.update();
    let dark = light_dark_option(&mut app, ThemePreference::Dark);
    app.world_mut().trigger(Activate { entity: dark });
    app.update();
    app.update();
    assert_eq!(
        *app.world().get_resource::<ThemePreference>().unwrap(),
        ThemePreference::Dark
    );
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Dark
    );
    assert!(app.world().entity(dark).contains::<Selected>());
    let light = light_dark_option(&mut app, ThemePreference::Light);
    app.world_mut().trigger(Activate { entity: light });
    app.update();
    app.update();
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Light
    );
    assert!(!app.world().entity(dark).contains::<Selected>());
    assert!(app.world().entity(light).contains::<Selected>());
}

#[test]
fn a_system_preference_follows_the_system_theme_as_it_changes() {
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.world_mut().spawn(LightDarkToggle);
    app.update();
    system_theme_changes(&mut app, WindowTheme::Dark);
    assert_eq!(
        app.world().get_resource::<SystemTheme>().unwrap().0,
        Some(ThemeMode::Dark)
    );
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Dark
    );
    system_theme_changes(&mut app, WindowTheme::Light);
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Light
    );
}

#[test]
fn a_window_created_with_a_fixed_theme_is_not_taken_for_the_system_theme() {
    let mut app = headless_app();
    app.world_mut().spawn((
        Window {
            window_theme: Some(WindowTheme::Dark),
            ..default()
        },
        PrimaryWindow,
    ));
    app.world_mut().spawn(LightDarkToggle);
    app.update();
    app.update();
    assert_eq!(
        app.world().get_resource::<SystemTheme>().unwrap().0,
        None,
        "the window's fixed theme says nothing about the system"
    );
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Light,
        "an unknown system theme means light"
    );
    system_theme_changes(&mut app, WindowTheme::Dark);
    assert_eq!(
        app.world().get_resource::<SystemTheme>().unwrap().0,
        Some(ThemeMode::Dark)
    );
}

#[test]
fn a_window_with_a_fixed_theme_that_exists_before_the_plugin_is_not_taken_for_the_system_theme() {
    let mut app = headless_app_with(|app| {
        app.world_mut().spawn((
            Window {
                window_theme: Some(WindowTheme::Dark),
                ..default()
            },
            PrimaryWindow,
        ));
    });
    app.world_mut().spawn(LightDarkToggle);
    app.update();
    app.update();
    assert_eq!(app.world().get_resource::<SystemTheme>().unwrap().0, None);
    system_theme_changes(&mut app, WindowTheme::Dark);
    assert_eq!(
        app.world().get_resource::<SystemTheme>().unwrap().0,
        Some(ThemeMode::Dark)
    );
}

#[test]
fn a_fixed_preference_ignores_the_system_theme_and_system_goes_back_to_it() {
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.world_mut().spawn(LightDarkToggle);
    app.update();
    let light = light_dark_option(&mut app, ThemePreference::Light);
    app.world_mut().trigger(Activate { entity: light });
    app.update();
    system_theme_changes(&mut app, WindowTheme::Dark);
    assert_eq!(
        app.world().get_resource::<SystemTheme>().unwrap().0,
        Some(ThemeMode::Dark),
        "the system theme is still tracked"
    );
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Light
    );
    let system = light_dark_option(&mut app, ThemePreference::System);
    app.world_mut().trigger(Activate { entity: system });
    app.update();
    app.update();
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Dark
    );
}

#[test]
fn the_primary_window_theme_follows_the_preference() {
    let mut app = headless_app();
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.world_mut().spawn(LightDarkToggle);
    app.update();
    app.update();
    let window_theme = |app: &App| app.world().get::<Window>(window).unwrap().window_theme;
    assert_eq!(
        window_theme(&app),
        None,
        "System leaves the window to the system"
    );
    for (preference, expected) in [
        (ThemePreference::Dark, Some(WindowTheme::Dark)),
        (ThemePreference::Light, Some(WindowTheme::Light)),
        (ThemePreference::System, None),
    ] {
        let option = light_dark_option(&mut app, preference);
        app.world_mut().trigger(Activate { entity: option });
        app.update();
        app.update();
        assert_eq!(window_theme(&app), expected, "{preference:?}");
    }
}

#[test]
fn a_light_dark_toggle_is_a_radio_group_whose_toggle_follows_the_preference() {
    let mut app = headless_app();
    let toggle = app.world_mut().spawn(LightDarkToggle).id();
    app.update();
    app.update();
    assert_eq!(
        app.world().get::<AccessibilityNode>(toggle).unwrap().role(),
        accesskit::Role::RadioGroup
    );
    let dark = light_dark_option(&mut app, ThemePreference::Dark);
    app.world_mut().trigger(Activate { entity: dark });
    app.update();
    app.update();
    for (preference, label, toggled) in [
        (ThemePreference::Light, "Light", false),
        (ThemePreference::System, "System", false),
        (ThemePreference::Dark, "Dark", true),
    ] {
        let option = light_dark_option(&mut app, preference);
        let node = app.world().get::<AccessibilityNode>(option).unwrap();
        assert_eq!(node.role(), accesskit::Role::RadioButton);
        assert_eq!(node.label(), Some(label));
        assert_eq!(node.toggled(), Some(accesskit::Toggled::from(toggled)));
    }
}

fn two_axis_scroll_area(app: &mut App) -> (Entity, Entity) {
    app.world_mut().spawn((
        Node::default(),
        ScrollArea::bundle(|_: &mut ChildSpawner| {}, true, true),
    ));
    app.update();
    app.update();
    let bar = |app: &mut App, orientation| {
        let world = app.world_mut();
        world
            .query_filtered::<(Entity, &Scrollbar), With<ScrollBar>>()
            .iter(world)
            .find(|(_, scrollbar)| scrollbar.orientation == orientation)
            .map(|(entity, _)| entity)
            .unwrap()
    };
    let vertical = bar(app, ControlOrientation::Vertical);
    (vertical, bar(app, ControlOrientation::Horizontal))
}

fn thumbs_of(app: &mut App, bar: Entity) -> usize {
    let world = app.world_mut();
    world
        .get::<Children>(bar)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|child| world.get::<ScrollbarThumb>(*child).is_some())
        .count()
}

fn cell(app: &App, bar: Entity) -> (GridPlacement, GridPlacement) {
    let node = app.world().get::<Node>(bar).unwrap();
    (node.grid_row, node.grid_column)
}

#[test]
fn scroll_bars_sit_in_the_grid_cell_of_their_headless_orientation_with_one_thumb_each() {
    let mut app = headless_app();
    let (vertical, horizontal) = two_axis_scroll_area(&mut app);
    assert_eq!(
        cell(&app, vertical),
        (GridPlacement::start(1), GridPlacement::start(2))
    );
    assert_eq!(
        cell(&app, horizontal),
        (GridPlacement::start(2), GridPlacement::start(1))
    );
    assert_eq!(thumbs_of(&mut app, vertical), 1);
    assert_eq!(thumbs_of(&mut app, horizontal), 1);
}

#[test]
fn reinserting_the_scroll_bar_marker_keeps_exactly_one_thumb() {
    let mut app = headless_app();
    let (vertical, _) = two_axis_scroll_area(&mut app);

    // Across frames.
    app.world_mut().entity_mut(vertical).insert(ScrollBar);
    app.update();
    app.update();
    assert_eq!(thumbs_of(&mut app, vertical), 1);

    // Twice before the commands are applied.
    app.world_mut()
        .entity_mut(vertical)
        .insert(ScrollBar)
        .insert(ScrollBar);
    app.update();
    app.update();
    assert_eq!(thumbs_of(&mut app, vertical), 1);
    assert_eq!(app.world().get::<Children>(vertical).unwrap().len(), 1);
}

#[test]
fn a_bar_without_a_thumb_gets_exactly_one_when_the_marker_is_inserted_twice_at_once() {
    let mut app = headless_app();
    let target = app.world_mut().spawn(Node::default()).id();
    let bar = app
        .world_mut()
        .spawn(Scrollbar::new(target, ControlOrientation::Horizontal, 0.0))
        .id();
    app.world_mut()
        .entity_mut(bar)
        .insert(ScrollBar)
        .insert(ScrollBar);
    app.update();
    app.update();
    assert_eq!(thumbs_of(&mut app, bar), 1);
    assert_eq!(
        cell(&app, bar),
        (GridPlacement::start(2), GridPlacement::start(1))
    );
}

#[test]
fn changing_the_headless_orientation_moves_and_resizes_the_bar_and_keeps_its_thumb() {
    let mut app = headless_app();
    let (vertical, _) = two_axis_scroll_area(&mut app);
    let thickness = app
        .world()
        .get_resource::<Theme>()
        .unwrap()
        .scroll_bar_width;
    assert_eq!(app.world().get::<Node>(vertical).unwrap().width, thickness);

    app.world_mut()
        .get_mut::<Scrollbar>(vertical)
        .unwrap()
        .orientation = ControlOrientation::Horizontal;
    app.update();
    app.update();

    let node = app.world().get::<Node>(vertical).unwrap();
    assert_eq!(node.height, thickness);
    assert_eq!(node.width, Val::Auto);
    assert_eq!(
        cell(&app, vertical),
        (GridPlacement::start(2), GridPlacement::start(1))
    );
    assert_eq!(thumbs_of(&mut app, vertical), 1);
}

#[test]
fn a_scroll_area_gives_its_bottom_margin_back_when_it_stops_being_clear_of_corners() {
    let mut app = headless_app();
    let radius = app.world().get_resource::<Theme>().unwrap().panel_radius;
    let area = app
        .world_mut()
        .spawn((
            Node::default(),
            ScrollArea::bundle(|_: &mut ChildSpawner| {}, true, false),
            ClearOfCorners,
        ))
        .id();
    app.update();
    app.update();
    let margin = |app: &App| app.world().get::<Node>(area).unwrap().margin.bottom;
    assert_eq!(margin(&app), radius);

    app.world_mut().entity_mut(area).remove::<ClearOfCorners>();
    assert_eq!(margin(&app), Val::ZERO);
    app.update();
    assert_eq!(margin(&app), Val::ZERO);
}

/// A control over "Grid" (given an icon) and "List" (without one).
fn spawn_mixed_icon_control(app: &mut App) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let control = app
        .world_mut()
        .spawn((
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["Grid", "List"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let grid = item_with_text(app, "Grid");
    app.world_mut()
        .entity_mut(grid)
        .insert(ItemIcon(Handle::default()));
    for _ in 0..3 {
        app.update();
    }
    control
}

/// The icon child of a segment, if any.
fn icon_of(app: &App, segment: Entity) -> Option<Entity> {
    app.world().entity(segment).get::<Children>().and_then(|c| {
        c.iter()
            .find(|c| app.world().entity(*c).contains::<SegmentIcon>())
    })
}

/// The display of the text child of a segment.
fn label_display(app: &App, segment: Entity) -> Display {
    let label = app
        .world()
        .entity(segment)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|c| app.world().entity(*c).contains::<Text>())
        .unwrap();
    app.world().entity(label).get::<Node>().unwrap().display
}

/// The entity of the item with the given text.
fn item_with_text(app: &mut App, text: &'static str) -> Entity {
    app.world_mut()
        .run_system_once(move |items: Query<(Entity, &Item<String>)>| {
            items.iter().find(|(_, i)| i.data == text).unwrap().0
        })
        .unwrap()
}

#[test]
fn a_segment_of_an_item_with_an_icon_gets_a_sixteen_pixel_icon_before_its_text() {
    let mut app = headless_app();
    let control = spawn_mixed_icon_control(&mut app);
    let segments = segments_of(&app, control);
    let icon = icon_of(&app, segments[0]).unwrap();
    assert!(icon_of(&app, segments[1]).is_none());
    assert_eq!(
        app.world().entity(segments[0]).get::<Children>().unwrap()[0],
        icon
    );
    let node = app.world().entity(icon).get::<Node>().unwrap();
    assert_eq!((node.width, node.height), (Val::Px(16.0), Val::Px(16.0)));
    assert_eq!(label_display(&app, segments[0]), Display::Flex);
}

#[test]
fn the_icon_of_a_segment_follows_the_colour_of_its_text_and_the_theme() {
    let mut app = headless_app();
    let control = spawn_mixed_icon_control(&mut app);
    let segment = segments_of(&app, control)[0];
    let icon = icon_of(&app, segment).unwrap();
    let colour = |app: &App| app.world().entity(icon).get::<ImageNode>().unwrap().color;
    assert_eq!(
        colour(&app),
        DEFAULT_LIGHT_THEME_COLORS.segmented_selected_text
    );
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .light_dark_toggle();
    app.update();
    app.update();
    assert_eq!(
        colour(&app),
        DEFAULT_DARK_THEME_COLORS.segmented_selected_text
    );
}

#[test]
fn hidden_labels_hide_the_text_only_of_items_with_an_icon_and_add_a_tooltip() {
    let mut app = headless_app();
    let control = spawn_mixed_icon_control(&mut app);
    let segments = segments_of(&app, control);
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Hidden);
    app.update();
    app.update();
    assert_eq!(label_display(&app, segments[0]), Display::None);
    assert_eq!(label_display(&app, segments[1]), Display::Flex);
    assert_eq!(
        app.world()
            .entity(segments[0])
            .get::<Tooltip>()
            .unwrap()
            .text,
        "Grid"
    );
    assert!(app.world().entity(segments[1]).get::<Tooltip>().is_none());
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Shown);
    app.update();
    app.update();
    assert_eq!(label_display(&app, segments[0]), Display::Flex);
    assert!(app.world().entity(segments[0]).get::<Tooltip>().is_none());
}

#[test]
fn an_app_tooltip_swapped_in_over_the_widgets_own_is_kept_and_not_removed_on_shown() {
    let mut app = headless_app();
    let control = spawn_mixed_icon_control(&mut app);
    let segment = segments_of(&app, control)[0];
    let tip = |app: &App| {
        app.world()
            .entity(segment)
            .get::<Tooltip>()
            .map(|t| t.text.clone())
    };
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Hidden);
    app.update();
    app.update();
    assert_eq!(tip(&app).as_deref(), Some("Grid"));
    app.world_mut()
        .entity_mut(segment)
        .insert(Tooltip::new("Mine"));
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Shown);
    app.update();
    app.update();
    assert_eq!(tip(&app).as_deref(), Some("Mine"));
}

#[test]
fn an_app_tooltip_on_a_segment_survives_label_and_icon_changes_and_is_not_overwritten() {
    let mut app = headless_app();
    let control = spawn_mixed_icon_control(&mut app);
    let segments = segments_of(&app, control);
    let grid = item_with_text(&mut app, "Grid");
    app.world_mut()
        .entity_mut(segments[0])
        .insert(Tooltip::new("Mine"));
    let tip = |app: &App| {
        app.world()
            .entity(segments[0])
            .get::<Tooltip>()
            .map(|t| t.text.clone())
    };
    let settle = |app: &mut App| {
        app.update();
        app.update();
    };
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Shown);
    settle(&mut app);
    assert_eq!(tip(&app).as_deref(), Some("Mine"));
    app.world_mut()
        .entity_mut(grid)
        .insert(ItemIcon(Handle::default()));
    settle(&mut app);
    assert_eq!(tip(&app).as_deref(), Some("Mine"));
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Hidden);
    settle(&mut app);
    assert_eq!(tip(&app).as_deref(), Some("Mine"));
    app.world_mut()
        .entity_mut(control)
        .insert(SegmentLabels::Shown);
    settle(&mut app);
    assert_eq!(tip(&app).as_deref(), Some("Mine"));
}

#[test]
fn changing_or_removing_an_item_icon_updates_its_segment_in_place() {
    let mut app = headless_app();
    let control = spawn_mixed_icon_control(&mut app);
    let segments = segments_of(&app, control);
    let indicator = indicator_of(&app, control);
    let icon = icon_of(&app, segments[0]).unwrap();
    let grid = item_with_text(&mut app, "Grid");
    let other = Handle::<Image>::Uuid(
        bevy::asset::uuid::Uuid::from_u128(7),
        std::marker::PhantomData,
    );
    app.world_mut()
        .entity_mut(grid)
        .insert(ItemIcon(other.clone()));
    app.update();
    app.update();
    assert_eq!(
        app.world().entity(icon).get::<ImageNode>().unwrap().image,
        other
    );
    assert_eq!(segments_of(&app, control), segments);

    app.world_mut().entity_mut(grid).remove::<ItemIcon>();
    app.update();
    app.update();
    assert!(icon_of(&app, segments[0]).is_none());
    assert_eq!(segments_of(&app, control), segments);
    assert_eq!(indicator_of(&app, control), indicator);

    let list = item_with_text(&mut app, "List");
    app.world_mut()
        .entity_mut(list)
        .insert(ItemIcon(Handle::default()));
    app.update();
    app.update();
    assert!(icon_of(&app, segments[1]).is_some());
    assert_eq!(segments_of(&app, control), segments);
}

#[test]
fn changing_an_items_text_updates_its_segment_in_place() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);
    let item = app
        .world()
        .entity(segments[1])
        .get::<Segment>()
        .unwrap()
        .item_ref;

    app.world_mut()
        .entity_mut(item)
        .get_mut::<Item<String>>()
        .unwrap()
        .data = "Custom".into();
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(segments_of(&app, control), segments);
    let texts: Vec<_> = segments.iter().map(|s| segment_text(&app, *s)).collect();
    assert_eq!(texts, ["Auto", "Custom", "Off"]);
}

#[test]
fn an_item_inserted_in_the_middle_puts_its_segment_there() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let before = segments_of(&app, control);

    let new = app.world_mut().spawn(Item::new("Middle")).id();
    app.world_mut()
        .entity_mut(control)
        .insert_related::<ItemOf>(1, &[new]);
    for _ in 0..3 {
        app.update();
    }

    let segments = segments_of(&app, control);
    let texts: Vec<_> = segments.iter().map(|s| segment_text(&app, *s)).collect();
    assert_eq!(texts, ["Auto", "Middle", "Manual", "Off"]);
    assert_eq!(segments[0], before[0]);
    assert_eq!(segments[2], before[1]);
    assert_eq!(segments[3], before[2]);
}

#[test]
fn reordering_items_reorders_their_segments() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let before = segments_of(&app, control);
    let last = app
        .world()
        .entity(before[2])
        .get::<Segment>()
        .unwrap()
        .item_ref;

    app.world_mut()
        .entity_mut(control)
        .insert_related::<ItemOf>(0, &[last]);
    for _ in 0..3 {
        app.update();
    }

    let segments = segments_of(&app, control);
    assert_eq!(segments, [before[2], before[0], before[1]]);
}

#[test]
fn changing_an_items_text_updates_its_tab_in_place() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let tabs = tabs_of(&app, bar);
    let item = app
        .world()
        .entity(tabs[1])
        .get::<TabBarTab>()
        .unwrap()
        .item_ref;

    app.world_mut()
        .entity_mut(item)
        .get_mut::<Item<String>>()
        .unwrap()
        .data = "Sound".into();
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(tabs_of(&app, bar), tabs);
    let texts: Vec<_> = tabs.iter().map(|t| segment_text(&app, *t)).collect();
    assert_eq!(texts, ["General", "Sound", "Input"]);
}

#[test]
fn an_item_inserted_in_the_middle_puts_its_tab_there() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let before = tabs_of(&app, bar);

    let new = app.world_mut().spawn(Item::new("Video")).id();
    app.world_mut()
        .entity_mut(bar)
        .insert_related::<ItemOf>(1, &[new]);
    for _ in 0..3 {
        app.update();
    }

    let tabs = tabs_of(&app, bar);
    let texts: Vec<_> = tabs.iter().map(|t| segment_text(&app, *t)).collect();
    assert_eq!(texts, ["General", "Video", "Audio", "Input"]);
    assert_eq!(
        [tabs[0], tabs[2], tabs[3]],
        [before[0], before[1], before[2]]
    );
}

#[test]
fn reordering_items_reorders_their_tabs() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let before = tabs_of(&app, bar);
    let last = app
        .world()
        .entity(before[2])
        .get::<TabBarTab>()
        .unwrap()
        .item_ref;

    app.world_mut()
        .entity_mut(bar)
        .insert_related::<ItemOf>(0, &[last]);
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(tabs_of(&app, bar), [before[2], before[0], before[1]]);
}

#[test]
fn a_one_and_a_one_or_more_selection_are_repaired_when_their_items_arrive_later() {
    let mut app = headless_app();
    let one = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(Vec::<Item<String>>::new()),
        )
        .id();
    let one_or_more = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one_or_more()
                .build_with_items(Vec::<Item<String>>::new()),
        )
        .id();
    for _ in 0..3 {
        app.update();
    }

    let first_of_one = app.world_mut().spawn((Item::new("a"), ItemOf(one))).id();
    let first_of_many = app
        .world_mut()
        .spawn((Item::new("b"), ItemOf(one_or_more)))
        .id();
    app.world_mut().spawn((Item::new("c"), ItemOf(one)));
    app.world_mut().spawn((Item::new("d"), ItemOf(one_or_more)));
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(
        app.world().entity(one).get::<Selection>(),
        Some(&Selection::Value(SelectionValue::One(first_of_one)))
    );
    assert_eq!(
        app.world().entity(one_or_more).get::<Selection>(),
        Some(&Selection::Value(SelectionValue::OneOrMore(
            std::iter::once(first_of_many).collect()
        )))
    );
}

/// A `Selection::Ref` that points to a holder of a value is followed.
#[test]
fn a_selection_ref_to_a_value_resolves_to_its_holder() {
    let mut app = headless_app();
    let holder = app
        .world_mut()
        .spawn(Selection::Value(SelectionValue::ZeroOrOne(None)))
        .id();
    let reference = app.world_mut().spawn(Selection::Ref(holder)).id();
    app.update();
    // `SelectionParam<Entity>` filters on `Entity`, which matches both.
    let got = app
        .world_mut()
        .run_system_once(move |param: SelectionParam| {
            (
                param.get_ref(reference),
                param.get_with_ref(reference).map(|(e, _)| e),
            )
        })
        .unwrap();
    assert_eq!(got, (Some(holder), Some(holder)));
}

/// A `Ref` to an entity without a `Selection` resolves to nothing.
#[test]
fn a_selection_ref_to_an_entity_without_a_selection_resolves_to_nothing() {
    let mut app = headless_app();
    let target = app.world_mut().spawn_empty().id();
    let reference = app.world_mut().spawn((Selection::Ref(target), Marker)).id();
    app.update();
    assert_eq!(
        resolve_marked(&mut app, reference),
        (None, false, None, false)
    );
}

/// A `Ref` to a despawned entity resolves to nothing.
#[test]
fn a_selection_ref_to_a_despawned_entity_resolves_to_nothing() {
    let mut app = headless_app();
    let target = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(target);
    let reference = app.world_mut().spawn((Selection::Ref(target), Marker)).id();
    app.update();
    assert_eq!(
        resolve_marked(&mut app, reference),
        (None, false, None, false)
    );
}

/// A `Ref` to its own entity resolves to nothing.
#[test]
fn a_selection_ref_to_itself_resolves_to_nothing() {
    let mut app = headless_app();
    let reference = app.world_mut().spawn(Marker).id();
    app.world_mut()
        .entity_mut(reference)
        .insert(Selection::Ref(reference));
    app.update();
    assert_eq!(
        resolve_marked(&mut app, reference),
        (None, false, None, false)
    );
}

/// A two-hop chain is not followed: the first `Ref` points to a `Ref`.
#[test]
fn a_selection_ref_chain_is_not_followed() {
    let mut app = headless_app();
    let holder = app
        .world_mut()
        .spawn(Selection::Value(SelectionValue::ZeroOrOne(None)))
        .id();
    let middle = app.world_mut().spawn(Selection::Ref(holder)).id();
    let first = app.world_mut().spawn((Selection::Ref(middle), Marker)).id();
    app.update();
    assert_eq!(resolve_marked(&mut app, first), (None, false, None, false));
    // One hop still works for the middle `Ref`.
    let middle_ref = app
        .world_mut()
        .run_system_once(move |param: SelectionParam| param.get_ref(middle))
        .unwrap();
    assert_eq!(middle_ref, Some(holder));
}

/// Marks the one `Ref` entity that `resolve_marked` looks at.
#[derive(Component)]
struct Marker;

/// [`resolve_selection`] over the `Marker` entity only, so `single` sees one.
fn resolve_marked(app: &mut App, entity: Entity) -> (Option<Entity>, bool, Option<Entity>, bool) {
    app.world_mut()
        .run_system_once(move |param: SelectionParam<With<Marker>>| {
            (
                param.iter().next().map(|(holder, _)| holder),
                param.single().is_ok(),
                param.get_ref(entity),
                param.get_with_ref(entity).is_some(),
            )
        })
        .unwrap()
}

/// An icon button in a root node, counting its [`Activate`]s in [`Activations`].
fn spawn_counting_icon_button(app: &mut App, tone: Tone, toggle: Option<bool>) -> Entity {
    app.init_resource::<Activations>();
    let root = app.world_mut().spawn(Node::default()).id();
    let mut entity = app.world_mut().spawn((
        IconButton::new(Handle::default(), "Go", IconButtonSize::Regular).with_tone(tone),
        ChildOf(root),
    ));
    if let Some(on) = toggle {
        entity.insert(ToggleState(on));
    }
    let button = entity
        .observe(|_: On<Activate>, mut activations: ResMut<Activations>| activations.0 += 1)
        .id();
    app.update();
    app.update();
    button
}

#[test]
fn a_disabled_icon_button_rests_dimmed_ignores_the_pointer_and_does_not_toggle() {
    let mut app = headless_app();
    let button = spawn_counting_icon_button(&mut app, Tone::Neutral, Some(false));
    let light = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(icon_tint(&app, button), light.text_primary);

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    app.update();
    assert_eq!(background(&app, button), light.background_primary);
    assert_eq!(icon_tint(&app, button), light.text_disabled);

    // Neither hover nor press changes the colours.
    pointer_over_or_out(&mut app, button, true);
    assert_eq!(background(&app, button), light.background_primary);
    assert_eq!(icon_tint(&app, button), light.text_disabled);
    press(&mut app, button);
    assert!(!app.world().entity(button).contains::<Pressed>());
    assert_eq!(background(&app, button), light.background_primary);
    click_and_release(&mut app, button);
    assert_eq!(background(&app, button), light.background_primary);

    // An Activate does not flip the toggle.
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert_eq!(
        app.world().get::<ToggleState>(button),
        Some(&ToggleState(false))
    );

    // Once enabled, the button is restyled and toggles again.
    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    app.update();
    assert_eq!(icon_tint(&app, button), light.text_primary);
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert_eq!(
        app.world().get::<ToggleState>(button),
        Some(&ToggleState(true))
    );
    assert_eq!(background(&app, button), light.accent_subtle);
}

#[test]
fn a_disabled_warning_icon_button_that_is_on_has_no_bar_and_no_border() {
    let mut app = headless_app();
    let button = spawn_counting_icon_button(&mut app, Tone::Warning, Some(true));
    let light = DEFAULT_LIGHT_THEME_COLORS;
    let bar = app.world().get::<Children>(button).unwrap()[1];
    assert_eq!(app.world().get::<Node>(bar).unwrap().display, Display::Flex);
    assert_eq!(border_top(&app, button), light.warning_border);

    app.world_mut()
        .entity_mut(button)
        .insert(InteractionDisabled);
    app.update();
    app.update();
    assert_eq!(background(&app, button), light.background_primary);
    assert_eq!(border_top(&app, button), Color::NONE);
    assert_eq!(
        app.world().get::<Node>(button).unwrap().border,
        UiRect::ZERO
    );
    assert_eq!(icon_tint(&app, button), light.text_disabled);
    assert_eq!(app.world().get::<Node>(bar).unwrap().display, Display::None);

    app.world_mut()
        .entity_mut(button)
        .remove::<InteractionDisabled>();
    app.update();
    app.update();
    assert_eq!(app.world().get::<Node>(bar).unwrap().display, Display::Flex);
    assert_eq!(border_top(&app, button), light.warning_border);
    assert_eq!(icon_tint(&app, button), light.warning_text);
}

#[test]
fn a_push_button_with_no_default_hover_rests_at_the_primary_background() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let owned = app
        .world_mut()
        .spawn((
            PushButton::from_label("Owned"),
            NoDefaultHover,
            ChildOf(root),
        ))
        .id();
    let plain = app
        .world_mut()
        .spawn((PushButton::from_label("Plain"), ChildOf(root)))
        .id();
    app.update();
    app.update();
    let light = DEFAULT_LIGHT_THEME_COLORS;
    let dark = DEFAULT_DARK_THEME_COLORS;

    pointer_over_or_out(&mut app, owned, true);
    pointer_over_or_out(&mut app, plain, true);
    assert_eq!(background(&app, owned), light.background_primary);
    assert_eq!(background(&app, plain), light.hovered_item);

    press(&mut app, owned);
    press(&mut app, plain);
    assert!(app.world().entity(owned).contains::<Pressed>());
    assert_eq!(background(&app, owned), light.background_primary);
    assert_eq!(background(&app, plain), light.selected_hovered_item);

    // A theme change under the pointer keeps the owned button at rest too.
    app.world_mut().entity_mut(plain).insert(Hovered(true));
    hover_and_toggle_theme(&mut app, owned);
    assert_eq!(background(&app, owned), dark.background_primary);
    assert_eq!(background(&app, plain), dark.selected_hovered_item);
}

/// The string of the `Text` on `entity`.
fn text_of(app: &App, entity: Entity) -> String {
    app.world().entity(entity).get::<Text>().unwrap().0.clone()
}

/// The accessible label of `entity`.
fn accessible_label(app: &App, entity: Entity) -> Option<String> {
    app.world()
        .entity(entity)
        .get::<AccessibilityNode>()
        .unwrap()
        .label()
        .map(str::to_owned)
}

#[test]
fn inserting_a_push_button_again_changes_its_text_in_place_and_keeps_its_label_in_step() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    let refs = *app.world().entity(button).get::<PushButtonRefs>().unwrap();

    app.world_mut()
        .entity_mut(button)
        .insert(PushButton::from_label("Stop"));
    app.update();
    app.update();

    let after = *app.world().entity(button).get::<PushButtonRefs>().unwrap();
    assert_eq!(after, refs, "no respawn, refs kept");
    assert_eq!(text_of(&app, after.label()), "Stop");
    assert_eq!(accessible_label(&app, button).as_deref(), Some("Stop"));
    assert_eq!(
        app.world().entity(button).get::<Children>().unwrap().len(),
        1,
        "no second label"
    );

    // The observers were not registered twice.
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 1);
}

#[test]
fn a_push_button_inserted_several_times_keeps_one_label_and_one_observer() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    let refs = *app.world().entity(button).get::<PushButtonRefs>().unwrap();

    for text in ["One", "Two", "Three"] {
        app.world_mut()
            .entity_mut(button)
            .insert(PushButton::from_label(text));
        app.update();
        app.update();
    }

    let after = *app.world().entity(button).get::<PushButtonRefs>().unwrap();
    assert_eq!(after, refs);
    assert_eq!(text_of(&app, after.label()), "Three");
    assert_eq!(
        app.world().entity(button).get::<Children>().unwrap().len(),
        1
    );
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert_eq!(app.world().get_resource::<Activations>().unwrap().0, 1);
}

#[test]
fn a_push_button_gains_and_loses_its_icon_when_inserted_again() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    let label_id = app
        .world()
        .entity(button)
        .get::<PushButtonRefs>()
        .unwrap()
        .label();

    app.world_mut()
        .entity_mut(button)
        .insert(PushButton::new("Go", "+"));
    app.update();
    app.update();

    let refs = *app.world().entity(button).get::<PushButtonRefs>().unwrap();
    assert_eq!(refs.label(), label_id);
    let icon = refs.icon().unwrap();
    assert_eq!(text_of(&app, icon), "+");
    assert_eq!(accessible_label(&app, button).as_deref(), Some("Go"));
    assert!(
        app.world().entity(icon).get::<TextFont>().is_some(),
        "the new icon is themed"
    );

    app.world_mut()
        .entity_mut(button)
        .insert(PushButton::new("Go", "-"));
    app.update();
    app.update();
    let refs = *app.world().entity(button).get::<PushButtonRefs>().unwrap();
    assert_eq!(refs.icon(), Some(icon));
    assert_eq!(text_of(&app, icon), "-");

    app.world_mut()
        .entity_mut(button)
        .insert(PushButton::from_label("Go"));
    app.update();
    app.update();
    let refs = *app.world().entity(button).get::<PushButtonRefs>().unwrap();
    assert_eq!(refs.icon(), None);
    assert!(app.world().get_entity(icon).is_err(), "the icon is gone");
    assert_eq!(
        app.world().entity(button).get::<Children>().unwrap().len(),
        1
    );
}

#[test]
fn inserting_a_link_again_changes_its_text_in_place_and_keeps_its_label_in_step() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, LinkState::Enabled);
    let text = app.world().entity(link).get::<Children>().unwrap()[0];
    assert_eq!(text_of(&app, text), "link");

    app.world_mut().entity_mut(link).insert(Link::new("other"));
    app.update();
    app.update();

    let children = app.world().entity(link).get::<Children>().unwrap();
    assert_eq!(children.len(), 1, "no second text");
    assert_eq!(children[0], text, "the text is kept");
    assert_eq!(text_of(&app, text), "other");
    assert_eq!(accessible_label(&app, link).as_deref(), Some("other"));
}

#[test]
fn no_default_hover_inserted_after_spawn_stops_the_hover_colours() {
    let mut app = headless_app();
    let button = spawn_push_button(&mut app);
    app.world_mut().entity_mut(button).insert(NoDefaultHover);
    app.update();

    press(&mut app, button);

    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(background(&app, button), colors.background_primary);
}

#[test]
fn no_default_hover_removed_after_spawn_brings_the_hover_colours_back() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((PushButton::from_label("Go"), NoDefaultHover, ChildOf(root)))
        .id();
    app.update();
    app.update();
    app.world_mut()
        .entity_mut(button)
        .remove::<NoDefaultHover>();
    app.update();

    press(&mut app, button);

    let colors = app.world().get_resource::<Theme>().unwrap().colors;
    assert_eq!(background(&app, button), colors.selected_hovered_item);
}

// ---- Re-pointing a `Selection::Ref` stops the list following the old items ----

#[test]
fn a_list_pointed_at_other_items_ignores_changes_to_the_old_ones() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let old = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["a", "b"].map(Item::new)),
        )
        .id();
    let new = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["x", "y", "z"].map(Item::new)),
        )
        .id();
    let list = app
        .world_mut()
        .spawn((List, Selection::reference(old), ChildOf(root)))
        .id();
    app.update();
    app.update();
    assert_eq!(item_views(&app, list).len(), 2);

    app.world_mut()
        .entity_mut(list)
        .insert(Selection::reference(new));
    app.update();
    app.update();
    let labels = |app: &App| {
        item_views(app, list)
            .into_iter()
            .map(|view| item_label(app, app.world().get::<ItemView>(view).unwrap().item_ref))
            .collect::<Vec<_>>()
    };
    assert_eq!(labels(&app), ["x", "y", "z"]);

    app.world_mut().spawn((Item::new("c"), ItemOf(old)));
    app.update();
    app.update();
    assert_eq!(
        labels(&app),
        ["x", "y", "z"],
        "old items no longer drive the list"
    );

    app.world_mut().spawn((Item::new("w"), ItemOf(new)));
    app.update();
    app.update();
    assert_eq!(labels(&app), ["x", "y", "z", "w"]);

    // Back to the old items: still one set of views, no duplicate observer effects.
    app.world_mut()
        .entity_mut(list)
        .insert(Selection::reference(old));
    app.update();
    app.update();
    assert_eq!(labels(&app), ["a", "b", "c"]);
}

#[test]
fn a_palette_pointed_at_other_items_ignores_changes_to_the_old_ones() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let palette_items = |data: Vec<Color>| {
        Selection::builder()
            .one()
            .build_with_items(data.into_iter().map(|data| Item { data }))
    };
    let old = app.world_mut().spawn(palette_items(vec![RED, GREEN])).id();
    let new = app
        .world_mut()
        .spawn(palette_items(vec![BLUE, RED, GREEN, Color::WHITE]))
        .id();
    let palette = app
        .world_mut()
        .spawn((ColorPalette, Selection::reference(old), ChildOf(root)))
        .id();
    app.update();
    app.update();
    assert_eq!(item_views(&app, palette).len(), 2);

    app.world_mut()
        .entity_mut(palette)
        .insert(Selection::reference(new));
    app.update();
    app.update();
    assert_eq!(item_views(&app, palette).len(), 4);

    app.world_mut().spawn((Item { data: BLUE }, ItemOf(old)));
    app.update();
    app.update();
    assert_eq!(
        item_views(&app, palette).len(),
        4,
        "old items no longer drive the palette"
    );

    app.world_mut().spawn((Item { data: BLUE }, ItemOf(new)));
    app.update();
    app.update();
    assert_eq!(item_views(&app, palette).len(), 5);
}

/// How many buttons, popups and palettes are descendants of the combo.
fn palette_combo_parts(app: &mut App, combo: Entity) -> (usize, usize, usize) {
    let world = app.world_mut();
    let entities: Vec<Entity> = world.query::<Entity>().iter(world).collect();
    let below = |has: fn(EntityRef) -> bool| {
        entities
            .iter()
            .filter(|&&e| has(world.entity(e)) && is_below(world, e, combo))
            .count()
    };
    (
        below(|e| e.contains::<ColorPushButton>()),
        below(|e| e.contains::<Float>()),
        below(|e| e.contains::<ColorPalette>()),
    )
}

/// Whether `entity` is a descendant of `ancestor`.
fn is_below(world: &World, entity: Entity, ancestor: Entity) -> bool {
    let mut current = entity;
    while let Some(child_of) = world.entity(current).get::<ChildOf>() {
        current = child_of.parent();
        if current == ancestor {
            return true;
        }
    }
    false
}

#[test]
fn inserting_a_color_palette_combo_again_keeps_one_button_one_popup_and_one_palette() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)))
        .id();
    app.update();
    app.update();
    assert_eq!(palette_combo_parts(&mut app, combo), (1, 1, 1));

    app.world_mut().entity_mut(combo).insert(ColorPaletteCombo);
    app.update();
    app.update();

    assert_eq!(palette_combo_parts(&mut app, combo), (1, 1, 1));
}

#[test]
fn removing_a_color_palette_combo_removes_its_button_popup_and_palette() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)))
        .id();
    app.update();
    app.update();
    assert_eq!(palette_combo_parts(&mut app, combo), (1, 1, 1));

    app.world_mut()
        .entity_mut(combo)
        .remove::<ColorPaletteCombo>();
    app.update();
    app.update();

    assert_eq!(palette_combo_parts(&mut app, combo), (0, 0, 0));
    let has_children = app
        .world()
        .entity(combo)
        .get::<Children>()
        .is_some_and(|c| !c.is_empty());
    assert!(!has_children, "no children left");

    // And it can be inserted again.
    app.world_mut().entity_mut(combo).insert(ColorPaletteCombo);
    app.update();
    app.update();
    assert_eq!(palette_combo_parts(&mut app, combo), (1, 1, 1));
}

fn observers_of(app: &mut App, entity: Entity) -> usize {
    app.world_mut()
        .query::<&Observer>()
        .iter(app.world())
        .filter(|observer| observer.descriptor().entities().contains(&entity))
        .count()
}

#[test]
fn despawning_a_segmented_control_or_tab_bar_leaves_no_items_observer_on_shared_items() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let selection = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["Dark", "Light"].map(Item::new)),
        )
        .id();
    app.update();
    let before = observers_of(&mut app, selection);

    let control = app
        .world_mut()
        .spawn((
            SegmentedControl,
            Selection::reference(selection),
            ChildOf(root),
        ))
        .id();
    let bar = app
        .world_mut()
        .spawn((TabBar, Selection::reference(selection), ChildOf(root)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(observers_of(&mut app, selection), before + 2);

    // Inserting the selection again does not add a second observer.
    app.world_mut()
        .entity_mut(control)
        .insert(Selection::reference(selection));
    for _ in 0..2 {
        app.update();
    }
    assert_eq!(observers_of(&mut app, selection), before + 2);

    app.world_mut().entity_mut(control).despawn();
    app.world_mut().entity_mut(bar).despawn();
    for _ in 0..2 {
        app.update();
    }
    assert_eq!(observers_of(&mut app, selection), before);
}

/// An entity with the given items, to draw a selection from through `ItemsRef`.
fn spawn_item_source(app: &mut App, texts: &[&'static str]) -> Entity {
    let source = app.world_mut().spawn_empty().id();
    for text in texts {
        app.world_mut().spawn((Item::new(*text), ItemOf(source)));
    }
    app.update();
    source
}

fn source_texts(app: &App, source: Entity) -> Vec<String> {
    app.world()
        .get::<ItemValues>(source)
        .map(|values| values.iter().map(|item| item_label(app, item)).collect())
        .unwrap_or_default()
}

#[test]
fn a_changed_item_data_still_updates_an_existing_view() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let first = item_entities_of(&app, control)[0];

    app.world_mut()
        .entity_mut(first)
        .get_mut::<Item<String>>()
        .unwrap()
        .data = "Changed".to_string();
    for _ in 0..3 {
        app.update();
    }

    let texts: Vec<_> = segments_of(&app, control)
        .iter()
        .map(|s| segment_text(&app, *s))
        .collect();
    assert_eq!(texts, ["Changed", "Manual", "Off"]);
}

#[test]
fn reinserting_item_of_moves_an_item_between_item_values() {
    let mut app = headless_app();
    let old = spawn_item_source(&mut app, &["a", "b"]);
    let new = spawn_item_source(&mut app, &["c"]);
    let moved = item_entities_of(&app, old)[0];

    app.world_mut().entity_mut(moved).insert(ItemOf(new));
    app.update();

    assert_eq!(source_texts(&app, old), ["b"]);
    assert_eq!(source_texts(&app, new), ["c", "a"]);

    // The last item leaving removes the collection, as Bevy's relationships do.
    let last = item_entities_of(&app, old)[0];
    app.world_mut().entity_mut(last).insert(ItemOf(new));
    app.update();
    assert!(source_texts(&app, old).is_empty());
    assert_eq!(source_texts(&app, new), ["c", "a", "b"]);
}

fn spawn_control_over(app: &mut App, source: Entity) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let control = app
        .world_mut()
        .spawn((
            SegmentedControl,
            Selection::builder().one().build_with_items_ref(source),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    control
}

fn segment_texts(app: &App, control: Entity) -> Vec<String> {
    segments_of(app, control)
        .iter()
        .map(|s| segment_text(app, *s))
        .collect()
}

#[test]
fn replacing_items_ref_refreshes_the_selection_and_the_widget() {
    let mut app = headless_app();
    let old = spawn_item_source(&mut app, &["a", "b"]);
    let new = spawn_item_source(&mut app, &["c", "d"]);
    let control = spawn_control_over(&mut app, old);
    assert_eq!(segment_texts(&app, control), ["a", "b"]);
    select_by_text(&mut app, control, "b");
    assert_eq!(selected_segments(&app, control), ["b"]);

    app.world_mut().entity_mut(control).insert(ItemsRef(new));
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(segment_texts(&app, control), ["c", "d"]);
    assert_eq!(selection_labels(&app, control), ["c"]);
    assert_eq!(selected_segments(&app, control), ["c"]);

    // Items of the former source no longer touch the control.
    app.world_mut().spawn((Item::new("e"), ItemOf(old)));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(segment_texts(&app, control), ["c", "d"]);
    // And items of the new one do.
    app.world_mut().spawn((Item::new("f"), ItemOf(new)));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(segment_texts(&app, control), ["c", "d", "f"]);
}

#[test]
fn replacing_items_ref_with_a_source_whose_items_arrive_later_selects_the_first_of_them() {
    let mut app = headless_app();
    let old = spawn_item_source(&mut app, &["a", "b"]);
    let empty = app.world_mut().spawn_empty().id();
    let control = spawn_control_over(&mut app, old);

    app.world_mut().entity_mut(control).insert(ItemsRef(empty));
    for _ in 0..3 {
        app.update();
    }
    // Nothing is selected while the source has no items (the selection waits for its first).
    assert!(selection_labels(&app, control).is_empty());
    assert!(selected_segments(&app, control).is_empty());

    app.world_mut().spawn((Item::new("x"), ItemOf(empty)));
    app.world_mut().spawn((Item::new("y"), ItemOf(empty)));
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(segment_texts(&app, control), ["x", "y"]);
    assert_eq!(selection_labels(&app, control), ["x"]);
    assert_eq!(selected_segments(&app, control), ["x"]);
}

fn stack_over(app: &mut App, selection_ref: Entity) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let stack = app
        .world_mut()
        .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    stack
}

#[test]
fn replacing_a_live_stacks_selection_ref_shows_the_new_selection_and_ignores_the_old() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);
    let first = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items([(Item::new("a"), StackItem::new(create_a))]),
        )
        .id();
    let second = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items([(Item::new("b"), StackItem::new(create_b))]),
        )
        .id();
    let stack = stack_over(&mut app, first);
    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert!(no_entity_with::<PanelB>(&mut app));

    app.world_mut().entity_mut(stack).insert(Stack {
        selection_ref: second,
    });
    for _ in 0..3 {
        app.update();
    }

    let panel_b = single_entity_with::<PanelB>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::None);
    assert_eq!(node_display(&app, panel_b), Display::Flex);

    // The former source no longer switches the stack.
    let item_a = item_entities_of(&app, first)[0];
    app.world_mut()
        .entity_mut(first)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(None)));
    select_one(&mut app, first, item_a);
    assert_eq!(node_display(&app, panel_a), Display::None);
    assert_eq!(node_display(&app, panel_b), Display::Flex);

    // Pointing back shows the kept child again, without creating another.
    app.world_mut().entity_mut(stack).insert(Stack {
        selection_ref: first,
    });
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(node_display(&app, panel_a), Display::Flex);
    assert_eq!(node_display(&app, panel_b), Display::None);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 1);
}

#[test]
fn replacing_a_stacks_selection_ref_with_an_entity_without_a_selection_hides_the_children() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let first = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items([(Item::new("a"), StackItem::new(create_a))]),
        )
        .id();
    let stack = stack_over(&mut app, first);
    let panel_a = single_entity_with::<PanelA>(&mut app);
    let nothing = app.world_mut().spawn_empty().id();

    app.world_mut().entity_mut(stack).insert(Stack {
        selection_ref: nothing,
    });
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(node_display(&app, panel_a), Display::None);
}

// ---- ColorPaletteCombo over a selection of many ----

/// Red, blue and green, in that order.
fn three_colors() -> Vec<Item<Color>> {
    use bevy::color::palettes::tailwind::{BLUE_500, GREEN_500, RED_500};
    [RED_500, BLUE_500, GREEN_500]
        .into_iter()
        .map(|data| Item { data: data.into() })
        .collect()
}

/// The items of the selection entity `selection_ref`, in order.
fn items_of(app: &App, selection_ref: Entity) -> Vec<Entity> {
    app.world()
        .entity(selection_ref)
        .get::<ItemValues>()
        .unwrap()
        .iter()
        .collect()
}

/// The colour of the square in the palette combo's button.
fn combo_square(app: &App, button: Entity) -> Color {
    let square = app.world().get::<ColorSquareRef>(button).unwrap().entity();
    app.world().get::<BackgroundColor>(square).unwrap().0
}

/// The value the palette combo's button tells a screen reader.
fn combo_value(app: &App, button: Entity) -> String {
    app.world()
        .get::<AccessibilityNode>(button)
        .unwrap()
        .value()
        .unwrap_or_default()
        .to_owned()
}

fn select_items(app: &mut App, selection_ref: Entity, items: &[Entity]) {
    let items = items.to_vec();
    app.world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            selections.update_selection(selection_ref, |s| s.set_selection(items.clone()));
        })
        .unwrap();
    app.update();
    app.update();
}

#[test]
fn a_color_palette_combo_over_many_shows_the_first_selected_colour_and_names_them_all() {
    use bevy::color::palettes::tailwind::{BLUE_500, GREEN_500, RED_500};
    let none = Color::Srgba(Srgba::NONE);
    let (red, blue, green): (Color, Color, Color) =
        (RED_500.into(), BLUE_500.into(), GREEN_500.into());
    for (name, builder, first) in [
        ("ZeroOrMore", Selection::builder().zero_or_more(), none),
        ("OneOrMore", Selection::builder().one_or_more(), red),
    ] {
        let mut app = headless_app();
        let (combo, button) = {
            app.world_mut().spawn((Window::default(), PrimaryWindow));
            let root = app.world_mut().spawn(Node::default()).id();
            let combo = app
                .world_mut()
                .spawn((
                    ColorPaletteCombo,
                    builder.build_with_items(three_colors()),
                    ChildOf(root),
                ))
                .id();
            app.update();
            app.update();
            (combo, single_entity_with::<ColorPushButton>(&mut app))
        };
        let items = items_of(&app, combo);
        assert_eq!(combo_square(&app, button), first, "{name}: initial");
        assert_eq!(
            combo_value(&app, button),
            if first == none { "" } else { "red-500" },
            "{name}: initial value"
        );

        select_items(&mut app, combo, &[items[1], items[0]]);
        assert_eq!(combo_square(&app, button), blue, "{name}: first selected");
        assert_eq!(
            combo_value(&app, button),
            "blue-500, red-500",
            "{name}: order"
        );

        select_items(&mut app, combo, &[items[0], items[1], items[2]]);
        assert_eq!(combo_square(&app, button), red, "{name}");
        assert_eq!(combo_value(&app, button), "red-500, blue-500, green-500");

        select_items(&mut app, combo, &[items[1], items[2]]);
        assert_eq!(combo_square(&app, button), blue, "{name}: red deselected");
        assert_eq!(combo_value(&app, button), "blue-500, green-500");

        select_items(&mut app, combo, &[items[2]]);
        assert_eq!(combo_square(&app, button), green, "{name}");
        assert_eq!(combo_value(&app, button), "green-500");

        if name == "ZeroOrMore" {
            select_items(&mut app, combo, &[]);
            assert_eq!(combo_square(&app, button), none, "{name}: nothing selected");
            assert_eq!(combo_value(&app, button), "");
        }
        let accessible = app.world().get::<AccessibilityNode>(button).unwrap();
        assert_eq!(accessible.label(), None, "{name}: no label");
    }
}

#[test]
fn a_color_palette_combo_over_a_shared_selection_of_many_follows_it() {
    use bevy::color::palettes::tailwind::{BLUE_500, GREEN_500};
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let shared = app
        .world_mut()
        .spawn(
            Selection::builder()
                .zero_or_more()
                .build_with_items(three_colors()),
        )
        .id();
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut().spawn((
        ColorPaletteCombo,
        Selection::reference(shared),
        ChildOf(root),
    ));
    app.update();
    app.update();
    let button = single_entity_with::<ColorPushButton>(&mut app);
    let items = items_of(&app, shared);
    assert_eq!(combo_square(&app, button), Color::Srgba(Srgba::NONE));
    assert_eq!(combo_value(&app, button), "");

    select_items(&mut app, shared, &[items[2], items[1]]);
    assert_eq!(combo_square(&app, button), Color::from(GREEN_500));
    assert_eq!(combo_value(&app, button), "green-500, blue-500");

    select_items(&mut app, shared, &[items[1]]);
    assert_eq!(combo_square(&app, button), Color::from(BLUE_500));
    assert_eq!(combo_value(&app, button), "blue-500");
}

#[test]
fn a_color_palette_combo_with_a_mandatory_selection_that_is_pending_is_transparent_and_unnamed() {
    for (name, builder) in [
        ("One", Selection::builder().one()),
        ("OneOrMore", Selection::builder().one_or_more()),
    ] {
        let mut app = headless_app();
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        let root = app.world_mut().spawn(Node::default()).id();
        let combo = app
            .world_mut()
            .spawn((
                ColorPaletteCombo,
                builder.build_with_items(Vec::<Item<Color>>::new()),
                ChildOf(root),
            ))
            .id();
        app.update();
        app.update();
        let button = single_entity_with::<ColorPushButton>(&mut app);
        assert_eq!(
            combo_square(&app, button),
            Color::Srgba(Srgba::NONE),
            "{name}"
        );
        assert_eq!(combo_value(&app, button), "", "{name}");

        // The first item arrives and the pending selection takes it.
        app.world_mut().spawn((
            Item {
                data: Color::from(bevy::color::palettes::tailwind::RED_500),
            },
            ItemOf(combo),
        ));
        app.update();
        app.update();
        app.update();
        assert_eq!(
            combo_square(&app, button),
            Color::from(bevy::color::palettes::tailwind::RED_500),
            "{name}"
        );
        assert_eq!(combo_value(&app, button), "red-500", "{name}");
    }
}

#[test]
fn a_swatch_click_leaves_the_popup_of_a_palette_combo_over_many_open() {
    for (name, builder) in [
        ("ZeroOrMore", Selection::builder().zero_or_more()),
        ("OneOrMore", Selection::builder().one_or_more()),
    ] {
        let mut app = headless_app();
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        let root = app.world_mut().spawn(Node::default()).id();
        let combo = app
            .world_mut()
            .spawn((
                ColorPaletteCombo,
                builder.build_with_items(three_colors()),
                ChildOf(root),
            ))
            .id();
        app.update();
        app.update();
        let button = single_entity_with::<ColorPushButton>(&mut app);
        click(&mut app, button);
        assert_eq!(palette_combo_state(&app, combo), PaletteComboState::Open);
        let palette = single_entity_with::<ColorPalette>(&mut app);
        let swatches = item_views(&app, palette);
        assert_eq!(swatches.len(), 3);

        click_row(&mut app, swatches[1]);
        click_row(&mut app, swatches[2]);

        assert_eq!(
            palette_combo_state(&app, combo),
            PaletteComboState::Open,
            "{name}"
        );
        assert!(selection_colors(&app, combo).len() >= 2, "{name}");
        assert_eq!(
            combo_value(&app, button).split(", ").count(),
            selection_colors(&app, combo).len(),
            "{name}"
        );
    }
}
