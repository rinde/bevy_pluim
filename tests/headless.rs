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
use bevy::ecs::event::PropagateEntityTrigger;
use bevy::ecs::system::RunSystemOnce;
use bevy::input::InputPlugin;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::mesh::MeshPlugin;
use bevy::picking::DefaultPickingPlugins;
use bevy::picking::backend::HitData;
use bevy::picking::events::{
    Pointer, PointerClick, PointerDrag, PointerOut, PointerOver, PointerPress, PointerRelease,
};
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::prelude::*;
use bevy::reflect::structs::Struct;
use bevy::text::{DEFAULT_REM_SIZE_PX, LetterSpacing};
use bevy::ui::{
    ComputedNode, ComputedUiRenderTargetInfo, FixedNode, InteractionDisabled, Pressed, Selected,
    UiGlobalTransform,
};
use bevy::ui_widgets::{
    Activate, ActiveDescendant, Button, ControlOrientation, ListBox, ListBoxMultiSelect, ListItem,
    ScrollIntoView, Scrollbar, ScrollbarThumb, SelectedTab, SetSliderValue, SliderRange,
    SliderStep, SliderValue, SliderValueChange, Tab, TabList, ValueChange,
};
use bevy::window::{PrimaryWindow, Window, WindowPlugin, WindowTheme, WindowThemeChanged};
use bevy_pluim::UiRoot;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::badge::{Badge, BadgeTone};
use bevy_pluim::breadcrumbs::BreadCrumbs;
use bevy_pluim::card::Card;
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::color_palette_combo::{
    ColorPaletteCombo, ColorPaletteComboButton, ColorPaletteComboFloat, ColorPaletteComboRefs,
    ColorPaletteComboState as PaletteComboState,
};
use bevy_pluim::color_push_button::{ColorPushButton, ColorSquareRef};
use bevy_pluim::combo::{Combo, ComboButton, ComboList, ComboPlaceholder, ComboRefs, ComboState};
use bevy_pluim::divider::Divider;
use bevy_pluim::filter_chips::{Chip, ChipRefs, ChipView, FilterChips};
use bevy_pluim::filter_combo::{FilterCombo, FilterComboRefs};
use bevy_pluim::float::Float;
use bevy_pluim::gallery::{Gallery, GalleryCard, GalleryCardRefs, GalleryCardView};
use bevy_pluim::icon::{Icon, IconColor};
use bevy_pluim::icon_button::{IconButton, IconButtonRefs, IconButtonSize, ToggleState, Tone};
use bevy_pluim::kbd::Kbd;
use bevy_pluim::label::Label;
use bevy_pluim::light_dark_toggle::{
    LightDarkOption, LightDarkToggle, SystemTheme, ThemePreference,
};
use bevy_pluim::link::Link;
use bevy_pluim::list::{
    CornerStyle, ItemNote, ItemView, ItemViewCreator, ItemViewSpawner, List, ListLayout,
};
use bevy_pluim::numeric_stepper::{NumericStepper, Value, ValueRef};
use bevy_pluim::panel::{Panel, PanelClosed, PanelRefs};
use bevy_pluim::pill::{Pill, PillTone};
use bevy_pluim::push_button::{
    ButtonSize, IconPlacement, NoDefaultHover, PushButton, PushButtonRefs,
};
use bevy_pluim::scroll_area::{ClearOfCorners, ScrollArea, ScrollBar, ScrollContents};
use bevy_pluim::segmented_control::{
    ItemIcon, Segment, SegmentIcon, SegmentIndicator, SegmentLabels, SegmentedControl,
};
use bevy_pluim::select::{
    Item, ItemOf, ItemValues, ItemsRef, Selection, SelectionBuilder, SelectionBuilderWithValue,
    SelectionParam, SelectionValue,
};
use bevy_pluim::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use bevy_pluim::slider::{Slider, SliderRefs};
use bevy_pluim::stack::{Stack, StackChildSpawner, StackItem};
use bevy_pluim::tab_bar::{TabBar, TabBarTab};
use bevy_pluim::text_field::{TextField, TextFieldChange, TextFieldCommit, TextFieldRefs};
use bevy_pluim::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme, ThemeMode};
use bevy_pluim::toast::Toast;
use bevy_pluim::toolbar::{
    Tool, Toolbar, ToolbarButton, ToolbarButtonRefs, ToolbarLabels, ToolbarStyle,
};
use bevy_pluim::tooltip::{Tooltip, TooltipPopup};
use bevy_pluim::virtual_list::{RowHeight, VirtualList, VirtualListRow};

#[path = "../examples/hud/ui/mod.rs"]
mod hud_ui;

#[path = "../examples/overview/ui/mod.rs"]
#[expect(dead_code, reason = "the example's own `main` is not run by the test")]
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
    assert_eq!(width, Val::Px(bevy_pluim::panel::PANEL_WIDTH_PX));
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
    assert_eq!(width, Val::Px(bevy_pluim::panel::PANEL_WIDTH_PX));
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
    let mut links = world.query::<(&Link, Has<InteractionDisabled>)>();
    let states = links
        .iter(world)
        .map(|(_, disabled)| disabled)
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 2);
    assert_eq!(
        states.iter().filter(|disabled| **disabled).count(),
        1,
        "exactly the last crumb is current"
    );
}

#[test]
fn bread_crumbs_follow_a_reordered_selection_and_trim_from_the_new_order() {
    let mut app = headless_app();
    let selection = app
        .world_mut()
        .spawn(Selection::builder().one_or_more().build_with_items([
            (Item::new("a"),),
            (Item::new("b"),),
            (Item::new("c"),),
        ]))
        .id();
    let items = app
        .world()
        .get::<ItemValues>(selection)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    let root = app.world_mut().spawn(Node::default()).id();
    let crumbs = app
        .world_mut()
        .spawn((
            Node::default(),
            BreadCrumbs,
            Selection::reference(selection),
            ChildOf(root),
        ))
        .id();
    app.update();

    let (a, b, c) = (items[0], items[1], items[2]);
    let set = |app: &mut App, order: Vec<Entity>| {
        app.world_mut()
            .run_system_once(move |mut selections: SelectionParam| {
                selections.update_selection(selection, |s| s.set_selection(order.clone()))
            })
            .unwrap()
    };
    assert!(set(&mut app, vec![a, b, c]));
    app.update();
    app.update();
    assert_eq!(crumb_texts(&app, crumbs), ["a", "b", "c"]);

    // The same items in another order are a change: the trail follows it.
    assert!(set(&mut app, vec![c, a, b]));
    app.update();
    app.update();
    assert_eq!(crumb_texts(&app, crumbs), ["c", "a", "b"]);
    // Setting the same order again is no change.
    assert!(!set(&mut app, vec![c, a, b]));

    // Trimming (what a click on a crumb does) keeps the first of the new order.
    app.world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            selections.update_selection(selection, |s| s.trim_to_length(2))
        })
        .unwrap();
    app.update();
    app.update();
    assert_eq!(crumb_texts(&app, crumbs), ["c", "a"]);
}

#[test]
fn bread_crumbs_follow_item_text_changes_and_trim_to_the_current_position() {
    let mut app = headless_app();
    let selection = app
        .world_mut()
        .spawn(Selection::builder().one_or_more().build_with_items([
            (Item::new("a"),),
            (Item::new("b"),),
            (Item::new("c"),),
        ]))
        .id();
    let items = app
        .world()
        .get::<ItemValues>(selection)
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    let (a, b, c) = (items[0], items[1], items[2]);
    let root = app.world_mut().spawn(Node::default()).id();
    let crumbs = app
        .world_mut()
        .spawn((
            Node::default(),
            BreadCrumbs,
            Selection::reference(selection),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            selections.update_selection(selection, |s| s.set_selection(vec![a, b, c]))
        })
        .unwrap();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(crumb_texts(&app, crumbs), ["a", "b", "c"]);

    // Editing the text of a selected item alone updates its crumb.
    app.world_mut().get_mut::<Item<String>>(b).unwrap().data = "B".into();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(crumb_texts(&app, crumbs), ["a", "B", "c"]);

    // Removing the text drops the crumb; the later crumb still trims to its position.
    app.world_mut().entity_mut(b).remove::<Item<String>>();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(crumb_texts(&app, crumbs), ["a", "c"]);

    // A formerly unlabelled item gaining text gets a crumb again, in its place.
    app.world_mut().entity_mut(b).insert(Item::new("again"));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(crumb_texts(&app, crumbs), ["a", "again", "c"]);

    // Exactly the last crumb is current, and clicking "again" trims to its position (2).
    let world = app.world_mut();
    let mut links = world.query::<(Entity, &Link, Has<InteractionDisabled>)>();
    let found = links
        .iter(world)
        .map(|(e, _, d)| (e, d))
        .collect::<Vec<_>>();
    assert_eq!(found.iter().filter(|(_, d)| *d).count(), 1);
    let again = found
        .iter()
        .map(|(e, _)| *e)
        .find(|e| {
            world
                .get::<Children>(*e)
                .into_iter()
                .flatten()
                .any(|c| world.get::<Text>(*c).is_some_and(|t| t.0 == "again"))
        })
        .unwrap();
    app.world_mut().trigger(Activate { entity: again });
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(crumb_texts(&app, crumbs), ["a", "again"]);
    let sel = app
        .world_mut()
        .run_system_once(move |selections: SelectionParam| {
            selections
                .get(selection)
                .map(|s| s.iter().collect::<Vec<_>>())
        })
        .unwrap()
        .unwrap();
    assert_eq!(sel, [a, b]);
}

/// The texts of the crumbs of a trail, in the order shown (separators left out).
fn crumb_texts(app: &App, crumbs: Entity) -> Vec<String> {
    fn collect(world: &World, entity: Entity, out: &mut Vec<String>) {
        if let Some(text) = world.get::<Text>(entity)
            && text.0 != " > "
        {
            out.push(text.0.clone());
        }
        for child in world.get::<Children>(entity).into_iter().flatten() {
            collect(world, *child, out);
        }
    }
    let mut out = Vec::new();
    collect(app.world(), crumbs, &mut out);
    out
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
        .colors()
        .background_secondary;
    assert_ne!(before.0, expected);
    assert_eq!(after.0, expected);
}

/// A link in a root node, after two updates.
fn spawn_link(app: &mut App, disabled: bool) -> Entity {
    let root = app.world_mut().spawn(Node::default()).id();
    let link = app
        .world_mut()
        .spawn((Link::new("link"), ChildOf(root)))
        .id();
    if disabled {
        app.world_mut().entity_mut(link).insert(InteractionDisabled);
    }
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
    let link = spawn_link(&mut app, false);

    hover_and_toggle_theme(&mut app, link);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_secondary);
    assert_eq!(link_text_color(&app, link), dark.link);
}

#[test]
fn changing_the_theme_under_a_pressed_link_keeps_it_pressed() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, false);
    press(&mut app, link);

    hover_and_toggle_theme(&mut app, link);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_secondary);
    assert_eq!(link_text_color(&app, link), dark.link_selected);
}

#[test]
fn changing_the_theme_under_a_link_the_pointer_is_not_on_leaves_it_at_rest() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, false);

    toggle_theme(&mut app);
    app.update();

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_primary);
    assert_eq!(link_text_color(&app, link), dark.link);
}

#[test]
fn a_disabled_link_the_pointer_is_on_stays_at_rest_in_a_new_theme() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, true);

    hover_and_toggle_theme(&mut app, link);

    let dark = DEFAULT_DARK_THEME_COLORS;
    assert_eq!(background(&app, link), dark.background_primary);
    assert_eq!(link_text_color(&app, link), dark.text_primary);
}

/// The text colour of a link whose own text is found by marker, not position.
fn link_label_color(app: &App, link: Entity) -> Color {
    let children = app.world().entity(link).get::<Children>().unwrap();
    children
        .iter()
        .find_map(|child| {
            let entity = app.world().entity(child);
            (entity.contains::<Text>() && entity.get::<Text>().unwrap().0 == "link")
                .then(|| entity.get::<TextColor>().unwrap().0)
        })
        .unwrap()
}

#[test]
fn a_link_with_an_app_child_before_its_text_still_themes_hovers_and_disables() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, false);
    let extra = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .entity_mut(link)
        .insert_children(0, &[extra]);
    app.update();
    assert_eq!(
        app.world().entity(link).get::<Children>().unwrap()[0],
        extra
    );

    // Hover: the label takes the link colour on the background of a hover.
    app.world_mut().trigger(PointerOver {
        entity: link,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
    app.update();
    assert_eq!(
        background(&app, link),
        DEFAULT_LIGHT_THEME_COLORS.background_secondary
    );

    // Press changes the label colour, which is only reachable through the owned child.
    app.world_mut().trigger(PointerPress {
        entity: link,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button: PointerButton::Primary,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
    app.update();
    assert_eq!(
        link_label_color(&app, link),
        DEFAULT_LIGHT_THEME_COLORS.link_selected
    );

    // Disabled: plain text colour.
    app.world_mut().entity_mut(link).insert(InteractionDisabled);
    app.update();
    assert_eq!(
        link_label_color(&app, link),
        DEFAULT_LIGHT_THEME_COLORS.text_primary
    );

    // Theme change restyles the label, and the app child is kept.
    app.world_mut()
        .entity_mut(link)
        .remove::<InteractionDisabled>();
    app.update();
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Dark);
    app.update();
    assert_ne!(
        link_label_color(&app, link),
        DEFAULT_LIGHT_THEME_COLORS.link_selected
    );
    assert!(app.world().get_entity(extra).is_ok());
    assert_eq!(app.world().entity(link).get::<Children>().unwrap().len(), 2);
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

    app.world_mut().entity_mut(link).insert(InteractionDisabled);
    app.update();
    assert!(app.world().entity(link).contains::<Button>());
    assert!(app.world().entity(link).contains::<InteractionDisabled>());

    app.world_mut()
        .entity_mut(link)
        .remove::<InteractionDisabled>();
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
        .spawn((Link::new("link"), InteractionDisabled, ChildOf(root)))
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
            .colors()
            .text_primary
    );
}

#[test]
fn inserting_and_removing_the_marker_on_a_live_link_restyles_it_under_the_pointer() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, false);
    app.world_mut().entity_mut(link).insert(Hovered(true));
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();

    app.world_mut().entity_mut(link).insert(InteractionDisabled);
    app.world_mut().run_schedule(Update);
    assert_eq!(background(&app, link), colors.background_primary);
    assert_eq!(link_text_color(&app, link), colors.text_primary);

    app.world_mut()
        .entity_mut(link)
        .remove::<InteractionDisabled>();
    app.world_mut().run_schedule(Update);
    assert_eq!(background(&app, link), colors.background_secondary);
    assert_eq!(link_text_color(&app, link), colors.link);
}

#[test]
fn a_link_enabled_again_reacts_to_the_pointer_again() {
    let mut app = headless_app();
    let link = spawn_link(&mut app, true);
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
    press(&mut app, link);
    assert_eq!(background(&app, link), colors.background_primary);

    app.world_mut()
        .entity_mut(link)
        .remove::<InteractionDisabled>();
    app.update();
    assert_eq!(background(&app, link), colors.background_primary);
    assert_eq!(link_text_color(&app, link), colors.link);

    press(&mut app, link);
    assert_eq!(background(&app, link), colors.background_secondary);
    assert_eq!(link_text_color(&app, link), colors.link_selected);
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

    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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

    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
fn overlapping_display_flips_and_a_reparent_in_one_frame_reach_every_consumer_of_a_deep_tree() {
    let mut app = headless_app();
    let index = |app: &App, e: Entity| app.world().entity(e).get::<TabIndex>().unwrap().0;
    let hidden = |app: &App, e: Entity| {
        app.world()
            .entity(e)
            .get::<AccessibilityNode>()
            .unwrap()
            .is_hidden()
    };
    let display = |app: &mut App, e: Entity, display: Display| {
        app.world_mut()
            .entity_mut(e)
            .get_mut::<Node>()
            .unwrap()
            .display = display;
    };

    // A chain of twenty nodes with a button at the bottom, and a spare parent.
    let mut chain = Vec::new();
    let mut parent = app.world_mut().spawn(Node::default()).id();
    for _ in 0..20 {
        chain.push(parent);
        parent = app
            .world_mut()
            .spawn((Node::default(), ChildOf(parent)))
            .id();
    }
    let button = app
        .world_mut()
        .spawn((PushButton::from_label("deep"), ChildOf(parent)))
        .id();
    let spare = app.world_mut().spawn(Node::default()).id();
    app.update();
    app.update();
    assert_eq!(index(&app, button), 0);

    // Two nested roots flip in one frame, so their subtrees overlap.
    display(&mut app, chain[2], Display::None);
    display(&mut app, chain[10], Display::None);
    app.update();
    app.update();
    assert_eq!(index(&app, button), -1);
    assert!(hidden(&app, button));

    // The outer one is shown again: the inner one still hides the button.
    display(&mut app, chain[2], Display::Flex);
    app.update();
    app.update();
    assert_eq!(index(&app, button), -1);
    assert!(hidden(&app, button));

    // The inner flips back and the chain is moved under a spare parent in the same frame.
    display(&mut app, chain[10], Display::Flex);
    app.world_mut().entity_mut(chain[0]).insert(ChildOf(spare));
    app.update();
    app.update();
    assert_eq!(index(&app, button), 0);
    assert!(!hidden(&app, button));

    // Hiding the spare parent and removing a middle node's `Node` in one frame.
    display(&mut app, spare, Display::None);
    app.update();
    app.update();
    assert_eq!(index(&app, button), -1);
    app.world_mut().entity_mut(spare).despawn();
    app.update();
    app.update();
    assert!(app.world().get_entity(button).is_err());
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
        .get::<ComboRefs>()
        .unwrap()
        .button();
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
    let refs = *app.world().entity(combo).get::<ComboRefs>().unwrap();
    click(&mut app, refs.button());
    app.update();
    app.update();
    let list_index = |app: &App| app.world().entity(refs.list()).get::<TabIndex>().unwrap().0;
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
fn the_root_refs_of_a_combo_point_at_its_live_parts() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    let refs = *app.world().get::<ComboRefs>(combo).unwrap();
    assert_eq!(refs.button(), button);
    assert!(app.world().entity(button).contains::<ComboButton>());
    assert!(app.world().entity(refs.list()).contains::<ComboList>());

    // Inserting the component again keeps one parts tree.
    app.world_mut().entity_mut(combo).insert(Combo);
    app.update();
    assert_eq!(
        app.world().get::<ComboRefs>(combo).unwrap().button(),
        button
    );
    let mut buttons = app.world_mut().query::<&ComboButton>();
    assert_eq!(buttons.iter(app.world()).count(), 1);

    app.world_mut().entity_mut(combo).despawn();
    app.update();
    assert_eq!(buttons.iter(app.world()).count(), 0);
    let mut lists = app.world_mut().query::<&ComboList>();
    assert_eq!(lists.iter(app.world()).count(), 0);
}

#[test]
fn the_root_refs_of_a_palette_combo_point_at_its_live_parts() {
    let mut app = headless_app();
    let (combo, button) = spawn_palette_combo(&mut app);
    let refs = *app.world().get::<ColorPaletteComboRefs>(combo).unwrap();
    assert_eq!(refs.button(), button);
    let world = app.world();
    assert!(world.entity(button).contains::<ColorPaletteComboButton>());
    assert_eq!(
        world.get::<Name>(button).map(Name::as_str),
        Some("ColorPaletteComboButton")
    );
    assert!(
        world
            .entity(refs.popup())
            .contains::<ColorPaletteComboFloat>()
    );
    assert!(world.entity(refs.palette()).contains::<ColorPalette>());

    // Inserting the component again keeps one parts tree.
    app.world_mut().entity_mut(combo).insert(ColorPaletteCombo);
    app.update();
    let again = *app.world().get::<ColorPaletteComboRefs>(combo).unwrap();
    assert_eq!(again.popup(), refs.popup());
    let mut popups = app.world_mut().query::<&ColorPaletteComboFloat>();
    assert_eq!(popups.iter(app.world()).count(), 1);

    app.world_mut().entity_mut(combo).despawn();
    app.update();
    assert_eq!(popups.iter(app.world()).count(), 0);
    let mut palettes = app.world_mut().query::<&ColorPalette>();
    assert_eq!(palettes.iter(app.world()).count(), 0);
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
    let list = app.world().get::<ComboRefs>(combo).unwrap().list();
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
fn a_color_palette_combo_popup_border_is_the_themes_highlight_when_spawned_and_after_a_change() {
    let mut app = headless_app();
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Dark);
    let root = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)));
    // No update since the spawn: the popup is themed when it is spawned.
    assert_popup_borders_are_highlight(&mut app, 1);

    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Light);
    app.update();
    app.update();
    assert_popup_borders_are_highlight(&mut app, 1);

    // A combo spawned after the theme changed is themed as well.
    app.world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(root)));
    assert_popup_borders_are_highlight(&mut app, 2);
}

/// Every palette combo popup has the active theme's `border_highlight` border.
fn assert_popup_borders_are_highlight(app: &mut App, count: usize) {
    let expected = BorderColor::all(
        app.world()
            .get_resource::<Theme>()
            .unwrap()
            .colors()
            .border_highlight,
    );
    let floats = palette_combo_floats(app);
    assert_eq!(floats.len(), count);
    for float in floats {
        assert_eq!(
            app.world().entity(float).get::<BorderColor>(),
            Some(&expected)
        );
    }
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
    let button = single_entity_with::<ColorPushButton>(&mut app);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    click(&mut app, button);
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
    // The closed popup holds no swatches, so reopen it to read the kept selection.
    let button = single_entity_with::<ColorPushButton>(&mut app);
    click(&mut app, button);
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
    (stepper, children[1], children[3])
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

#[derive(Resource, Default)]
struct StepperChanges(Vec<u8>);

fn count_stepper_changes(app: &mut App) {
    app.init_resource::<StepperChanges>();
    app.add_observer(
        |change: On<bevy_pluim::numeric_stepper::ValueChange>, mut seen: ResMut<StepperChanges>| {
            seen.0.push(change.value.as_u8().unwrap());
        },
    );
}

#[test]
fn a_value_change_is_sent_for_a_step_only_not_for_spawning_or_setting_the_value() {
    let mut app = headless_app();
    count_stepper_changes(&mut app);
    let (stepper, _, increase) = spawn_stepper(&mut app, 1);
    assert!(
        app.world()
            .get_resource::<StepperChanges>()
            .unwrap()
            .0
            .is_empty()
    );

    app.world_mut()
        .entity_mut(stepper)
        .insert(Value::new_u8(3, 0, 3, 1).unwrap());
    app.update();
    app.update();
    assert!(
        app.world()
            .get_resource::<StepperChanges>()
            .unwrap()
            .0
            .is_empty()
    );

    app.world_mut()
        .entity_mut(stepper)
        .insert(Value::new_u8(1, 0, 3, 1).unwrap());
    app.update();
    activate(&mut app, increase);
    assert_eq!(
        app.world().get_resource::<StepperChanges>().unwrap().0,
        vec![2]
    );
}

#[test]
fn a_stepper_without_a_value_or_with_a_value_ref_to_none_is_not_built_and_gets_no_default() {
    let mut app = headless_app();
    let bare = app.world_mut().spawn(NumericStepper::default()).id();
    let target = app.world_mut().spawn_empty().id();
    let dangling = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(target)))
        .id();
    app.update();
    app.update();
    for stepper in [bare, dangling] {
        assert!(app.world().entity(stepper).get::<Value>().is_none());
        assert!(app.world().entity(stepper).get::<Children>().is_none());
    }
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
    let text = |app: &App| app.world().get::<Text>(children[2]).unwrap().0.clone();
    assert_eq!(text(&app), "Ground");
    assert_eq!(
        app.world().get::<IconButton>(children[1]).unwrap().name,
        "Lower the level"
    );
    assert_eq!(
        app.world().get::<IconButton>(children[3]).unwrap().name,
        "Raise the level"
    );

    activate(&mut app, children[3]);
    assert_eq!(text(&app), "+7");
    activate(&mut app, children[3]);
    assert_eq!(text(&app), "+14");
    assert!(is_disabled(&app, children[3]));
    activate(&mut app, children[1]);
    activate(&mut app, children[1]);
    assert_eq!(text(&app), "Ground");
    assert!(is_disabled(&app, children[1]));
}

/// The parts of a stepper: fill, decrease, label, increase (child order: track, "-", label, "+").
fn stepper_label(app: &App, stepper: Entity) -> String {
    let children: Vec<Entity> = app
        .world()
        .entity(stepper)
        .get::<Children>()
        .unwrap()
        .iter()
        .collect();
    app.world().get::<Text>(children[2]).unwrap().0.clone()
}

fn stepper_spin_text(app: &App, stepper: Entity) -> Option<String> {
    app.world()
        .get::<AccessibilityNode>(stepper)
        .unwrap()
        .value()
        .map(ToString::to_string)
}

#[test]
fn an_external_value_change_updates_only_the_steppers_that_show_that_value() {
    let mut app = headless_app();
    let shared = app
        .world_mut()
        .spawn(Value::new_u8(1, 0, 9, 1).unwrap())
        .id();
    let mirror_a = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(shared)))
        .id();
    let mirror_b = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(shared)))
        .id();
    let (other, _, _) = spawn_stepper(&mut app, 3);
    app.update();
    app.update();
    assert_eq!(stepper_label(&app, mirror_a), "1");
    assert_eq!(stepper_label(&app, other), "3");

    app.world_mut()
        .entity_mut(shared)
        .get_mut::<Value>()
        .unwrap()
        .increment();
    app.update();
    app.update();
    for mirror in [mirror_a, mirror_b] {
        assert_eq!(stepper_label(&app, mirror), "2");
        assert_eq!(stepper_spin_text(&app, mirror).as_deref(), Some("2"));
    }
    assert_eq!(stepper_label(&app, other), "3");
}

#[test]
fn a_self_held_stepper_follows_an_external_change_of_its_own_value() {
    let mut app = headless_app();
    let (stepper, decrease, increase) = spawn_stepper(&mut app, 0);
    app.update();
    app.update();
    assert!(is_disabled(&app, decrease));
    app.world_mut()
        .entity_mut(stepper)
        .get_mut::<Value>()
        .unwrap()
        .increment();
    app.update();
    app.update();
    assert_eq!(stepper_label(&app, stepper), "1");
    assert_eq!(stepper_spin_text(&app, stepper).as_deref(), Some("1"));
    assert!(!is_disabled(&app, decrease));
    assert!(!is_disabled(&app, increase));
}

#[test]
fn the_buttons_and_fill_of_every_mirror_follow_a_shared_value_to_its_limits() {
    let mut app = headless_app();
    let shared = app
        .world_mut()
        .spawn(Value::new_u8(0, 0, 2, 1).unwrap())
        .id();
    let mirrors: Vec<Entity> = (0..3)
        .map(|_| {
            app.world_mut()
                .spawn((NumericStepper::default(), ValueRef(shared)))
                .id()
        })
        .collect();
    app.update();
    app.update();
    let parts = |app: &App, mirror: Entity| -> Vec<Entity> {
        app.world()
            .get::<Children>(mirror)
            .unwrap()
            .iter()
            .collect()
    };
    let fill_width = |app: &App, mirror: Entity| {
        let track = parts(app, mirror)[0];
        let fill = app.world().get::<Children>(track).unwrap()[0];
        app.world().get::<Node>(fill).unwrap().width
    };
    for mirror in &mirrors {
        assert!(is_disabled(&app, parts(&app, *mirror)[1]));
        assert!(!is_disabled(&app, parts(&app, *mirror)[3]));
        assert_eq!(fill_width(&app, *mirror), Val::Percent(0.0));
    }
    for _ in 0..2 {
        app.world_mut()
            .entity_mut(shared)
            .get_mut::<Value>()
            .unwrap()
            .increment();
    }
    app.update();
    app.update();
    for mirror in &mirrors {
        assert!(!is_disabled(&app, parts(&app, *mirror)[1]));
        assert!(is_disabled(&app, parts(&app, *mirror)[3]));
        assert_eq!(fill_width(&app, *mirror), Val::Percent(100.0));
        assert_eq!(stepper_label(&app, *mirror), "2");
    }
}

#[test]
fn a_stepper_pointed_at_another_value_stops_following_the_old_one() {
    let mut app = headless_app();
    let first = app
        .world_mut()
        .spawn(Value::new_u8(1, 0, 9, 1).unwrap())
        .id();
    let second = app
        .world_mut()
        .spawn(Value::new_u8(7, 0, 9, 1).unwrap())
        .id();
    let mirror = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(first)))
        .id();
    app.update();
    app.update();
    app.world_mut()
        .entity_mut(mirror)
        .insert((NumericStepper::default(), ValueRef(second)));
    app.world_mut()
        .entity_mut(second)
        .get_mut::<Value>()
        .unwrap()
        .increment();
    app.update();
    app.update();
    assert_eq!(stepper_label(&app, mirror), "8");
    app.world_mut()
        .entity_mut(first)
        .get_mut::<Value>()
        .unwrap()
        .increment();
    app.update();
    app.update();
    assert_eq!(stepper_label(&app, mirror), "8");
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

/// Names of the children of `entity`, in order.
fn child_names(app: &App, entity: Entity) -> Vec<String> {
    app.world()
        .entity(entity)
        .get::<Children>()
        .map(|children| {
            children
                .iter()
                .map(|child| {
                    app.world()
                        .entity(child)
                        .get::<Name>()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn inserting_a_numeric_stepper_keeps_app_children_at_first_and_every_later_insert() {
    let mut app = headless_app();
    let stepper = app
        .world_mut()
        .spawn(Value::new_u8(1, 0, 3, 1).unwrap())
        .id();
    let mine = app
        .world_mut()
        .spawn((Name::new("Mine"), ChildOf(stepper)))
        .id();
    app.world_mut()
        .entity_mut(stepper)
        .insert(NumericStepper::default());
    app.update();
    app.update();
    let first = child_names(&app, stepper);
    assert_eq!(first.len(), 5);
    assert!(first.contains(&"Mine".to_owned()));

    // Repeated inserts, also several before the commands run, never duplicate or drop.
    for _ in 0..3 {
        app.world_mut()
            .entity_mut(stepper)
            .insert(NumericStepper::default());
    }
    let late = app
        .world_mut()
        .spawn((Name::new("Late"), ChildOf(stepper)))
        .id();
    app.update();
    app.update();
    app.world_mut()
        .entity_mut(stepper)
        .insert(NumericStepper::default());
    app.update();
    app.update();

    let names = child_names(&app, stepper);
    assert_eq!(names.len(), 6, "{names:?}");
    for kept in ["Mine", "Late"] {
        assert_eq!(names.iter().filter(|n| *n == kept).count(), 1);
    }
    for part in ["ScrubTrack", "Decrease", "Value", "Increase"] {
        assert_eq!(names.iter().filter(|n| *n == part).count(), 1, "{part}");
    }
    assert!(app.world().get_entity(mine).is_ok());
    assert!(app.world().get_entity(late).is_ok());
    assert_eq!(app.world().get::<ChildOf>(mine).map(|c| c.0), Some(stepper));
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

#[test]
fn a_numeric_stepper_shows_a_resize_cursor_and_a_text_cursor_on_its_value() {
    use bevy::picking::cursor::EntityCursor;
    use bevy::window::SystemCursorIcon;

    let mut app = headless_app();
    let (stepper, _, _) = spawn_stepper(&mut app, 1);
    let label = app.world().entity(stepper).get::<Children>().unwrap()[2];

    let cursor =
        |app: &App, entity: Entity| app.world().entity(entity).get::<EntityCursor>().cloned();
    assert_eq!(
        cursor(&app, stepper),
        Some(EntityCursor::System(SystemCursorIcon::ColResize)),
        "a drag anywhere on the stepper scrubs it"
    );
    assert_eq!(
        cursor(&app, label),
        Some(EntityCursor::System(SystemCursorIcon::Text)),
        "a click on the value opens the editor"
    );

    click_and_release(&mut app, label);
    app.update();
    let editor = single_entity_with::<TextField>(&mut app);
    assert_eq!(
        cursor(&app, editor),
        Some(EntityCursor::System(SystemCursorIcon::Text)),
        "the open editor is a text field"
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
        let c = theme.colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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

/// The child of `parent` named `name`, such as the indicator of a tab bar.
fn child_named(app: &App, parent: Entity, name: &str) -> Entity {
    app.world()
        .entity(parent)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|c| {
            app.world()
                .get::<Name>(*c)
                .is_some_and(|n| n.as_str() == name)
        })
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
    let removed_item = removed.get::<Segment>().unwrap().item();
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
    // The selected look is the sliding indicator's; the tabs themselves are transparent.
    assert_eq!(
        background(&app, child_named(&app, bar, "TabIndicator")),
        colors.segmented_selected
    );
    assert_eq!(background(&app, tabs[0]), Color::NONE);
    assert_eq!(background(&app, tabs[1]), Color::NONE);
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
    assert_eq!(background(&app, tabs[0]), Color::NONE);
    assert_eq!(background(&app, tabs[2]), Color::NONE);
    assert_eq!(
        background(&app, child_named(&app, bar, "TabIndicator")),
        colors.segmented_selected
    );
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

#[test]
fn a_quiet_tab_bar_is_not_revisited_and_a_change_to_its_tabs_or_focus_is() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let tabs = tabs_of(&app, bar);
    let index = |app: &App, tab: Entity| app.world().entity(tab).get::<TabIndex>().unwrap().0;

    // Plant a wrong index without marking it changed: a quiet frame must leave it alone.
    app.world_mut()
        .entity_mut(tabs[2])
        .get_mut::<TabIndex>()
        .unwrap()
        .bypass_change_detection()
        .0 = 0;
    app.update();
    app.update();
    assert_eq!(index(&app, tabs[2]), 0, "an idle bar was looked at");

    // A move of the focus onto a tab brings the bar back in step.
    app.world_mut()
        .insert_resource(InputFocus::from_entity(tabs[1]));
    app.update();
    app.update();
    assert_eq!(
        tabs.iter().map(|t| index(&app, *t)).collect::<Vec<_>>(),
        [-1, 0, -1]
    );

    // Focus gone: back to the selected tab.
    app.world_mut().insert_resource(InputFocus::default());
    app.update();
    app.update();
    assert_eq!(
        tabs.iter().map(|t| index(&app, *t)).collect::<Vec<_>>(),
        [0, -1, -1]
    );

    // Disabling the roving tab moves the stop to the next enabled one; enabling restores it.
    app.world_mut()
        .entity_mut(tabs[0])
        .insert(InteractionDisabled);
    app.update();
    app.update();
    assert_eq!(
        tabs.iter().map(|t| index(&app, *t)).collect::<Vec<_>>(),
        [-1, 0, -1]
    );
    app.world_mut()
        .entity_mut(tabs[0])
        .remove::<InteractionDisabled>();
    app.update();
    app.update();
    assert_eq!(
        tabs.iter().map(|t| index(&app, *t)).collect::<Vec<_>>(),
        [0, -1, -1]
    );
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

    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();

    activate(&mut app, views[1]);

    let selection = app.world().entity(chips).get::<Selection>().unwrap();
    let item = app
        .world()
        .entity(views[1])
        .get::<ChipView>()
        .unwrap()
        .item();
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
    let item = app
        .world()
        .entity(views[2])
        .get::<ChipView>()
        .unwrap()
        .item();

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

    let item = app
        .world()
        .entity(views[2])
        .get::<ChipView>()
        .unwrap()
        .item();
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
fn inserting_a_panel_again_sets_the_body_maximum_height_and_keeps_the_corner_margin() {
    let mut app = headless_app();
    let radius = app.world().get_resource::<Theme>().unwrap().panel_radius;
    let radius_px = 12.0;
    assert_eq!(radius, Val::Px(radius_px));
    let panel = app.world_mut().spawn(Panel::new("Trains")).id();
    app.update();
    app.update();
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let area = |app: &App| {
        let node = app.world().get::<Node>(refs.scroll_area()).unwrap();
        (node.max_height, node.margin.bottom)
    };
    assert_eq!(area(&app), (Val::Px(620.0 + radius_px), radius));

    // A new maximum reaches the scroll area, with the margin of the corners added to it.
    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Trains").with_max_body_height(150.0).unwrap());
    app.update();
    app.update();
    assert_eq!(area(&app), (Val::Px(150.0 + radius_px), radius));

    // With a footer holding the corners there is no margin to add.
    let footer_text = app
        .world_mut()
        .spawn((Label::caption("Pick a train."), ChildOf(refs.footer())))
        .id();
    app.update();
    assert_eq!(area(&app), (Val::Px(150.0), Val::ZERO));
    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Trains").with_max_body_height(90.0).unwrap());
    app.update();
    app.update();
    assert_eq!(area(&app), (Val::Px(90.0), Val::ZERO));

    // Without the footer again, a re-insert does not drop the margin.
    app.world_mut().despawn(footer_text);
    app.update();
    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Trains").with_max_body_height(70.0).unwrap());
    app.update();
    app.update();
    assert_eq!(area(&app), (Val::Px(70.0 + radius_px), radius));
}

#[test]
fn inserting_a_panel_again_adds_and_removes_its_close_button() {
    let mut app = headless_app();
    let icon: Handle<Image> = bevy::asset::uuid_handle!("2d8e3f40-5162-4c73-8da4-9fa5fb607a18");
    let panel = app.world_mut().spawn(Panel::new("Trains")).id();
    app.update();
    app.update();
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let display = |app: &App| app.world().get::<Node>(refs.close()).unwrap().display;
    assert_eq!(display(&app), Display::None);

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Trains").with_close_icon(icon.clone()));
    app.update();
    app.update();
    assert_eq!(display(&app), Display::Flex);
    let close_icon = app
        .world()
        .get::<IconButtonRefs>(refs.close())
        .unwrap()
        .icon();
    assert_eq!(
        app.world().get::<ImageNode>(close_icon).unwrap().image,
        icon
    );

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Trains"));
    app.update();
    app.update();
    assert_eq!(display(&app), Display::None);
    assert_eq!(*app.world().get::<PanelRefs>(panel).unwrap(), refs);
}

#[test]
fn a_panel_stays_labelled_by_its_title_after_it_is_inserted_again_with_other_parts() {
    let mut app = headless_app();
    let panel = app.world_mut().spawn(Panel::new("Trains")).id();
    app.update();
    app.update();
    let title = app.world().get::<PanelRefs>(panel).unwrap().title();

    app.world_mut().entity_mut(panel).insert(
        Panel::new("Stops")
            .with_subtitle("6 stops")
            .with_close_icon(Handle::default())
            .with_max_body_height(200.0)
            .unwrap(),
    );
    app.update();
    app.update();

    let node = app.world().get::<AccessibilityNode>(panel).unwrap();
    assert_eq!(node.role(), accesskit::Role::Group);
    assert_eq!(node.label(), None);
    assert_eq!(node.labelled_by(), [accesskit::NodeId(title.to_bits())]);
    assert_eq!(
        app.world().get::<PanelRefs>(panel).unwrap().title(),
        title,
        "the title is the same entity"
    );
    assert_eq!(app.world().get::<Text>(title).unwrap().0, "Stops");
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

    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
fn inserting_a_setting_row_again_changes_its_text_and_keeps_its_control() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let row = app
        .world_mut()
        .spawn((SettingRow::new("A", "first"), ChildOf(root)))
        .id();
    app.update();
    app.update();
    let refs = *app.world().get::<SettingRowRefs>(row).unwrap();
    let control = app
        .world_mut()
        .spawn((Node::default(), ChildOf(refs.control_slot)))
        .id();
    let children_before: Vec<Entity> = app.world().get::<Children>(row).unwrap().to_vec();

    app.world_mut()
        .entity_mut(row)
        .insert(SettingRow::new("B", "second"));
    app.update();
    app.update();

    assert_eq!(app.world().get::<Text>(refs.title_id).unwrap().0, "B");
    assert_eq!(
        app.world().get::<Text>(refs.description_id).unwrap().0,
        "second"
    );
    let node = app.world().get::<AccessibilityNode>(row).unwrap();
    assert_eq!(node.label(), Some("B"));
    assert_eq!(node.description(), Some("second"));
    assert_eq!(*app.world().get::<SettingRowRefs>(row).unwrap(), refs);
    assert_eq!(
        app.world().get::<Children>(row).unwrap().to_vec(),
        children_before
    );
    assert_eq!(
        app.world().get::<ChildOf>(control).unwrap().parent(),
        refs.control_slot
    );
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

fn setting_row_divider_displays(app: &App, column: Entity) -> Vec<Display> {
    app.world()
        .get::<Children>(column)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| app.world().get::<SettingRowRefs>(child).copied())
                .map(|refs| app.world().get::<Node>(refs.divider_id).unwrap().display)
                .collect()
        })
        .unwrap_or_default()
}

fn spawn_setting_rows(app: &mut App, titles: &[&str]) -> (Entity, Vec<Entity>) {
    let column = app.world_mut().spawn(SettingRows).id();
    let rows = titles
        .iter()
        .map(|title| {
            app.world_mut()
                .spawn((SettingRow::new(*title, "description"), ChildOf(column)))
                .id()
        })
        .collect();
    app.update();
    app.update();
    (column, rows)
}

#[test]
fn removing_the_last_setting_row_hides_the_divider_of_the_new_last_one() {
    let mut app = headless_app();
    let (column, rows) = spawn_setting_rows(&mut app, &["A", "B", "C"]);
    app.world_mut().entity_mut(rows[2]).despawn();
    app.update();
    assert_eq!(
        setting_row_divider_displays(&app, column),
        [Display::Flex, Display::None]
    );
}

#[test]
fn reordering_setting_rows_moves_the_missing_divider_to_the_new_last_row() {
    let mut app = headless_app();
    let (column, rows) = spawn_setting_rows(&mut app, &["A", "B", "C"]);
    app.world_mut()
        .entity_mut(column)
        .insert_children(0, &[rows[2]]);
    app.update();
    assert_eq!(
        setting_row_divider_displays(&app, column),
        [Display::Flex, Display::Flex, Display::None]
    );
    let last = *app.world().get::<Children>(column).unwrap().last().unwrap();
    assert_eq!(last, rows[1]);
    let refs = *app.world().get::<SettingRowRefs>(rows[1]).unwrap();
    assert_eq!(
        app.world().get::<Node>(refs.divider_id).unwrap().display,
        Display::None
    );
}

#[test]
fn non_row_children_of_setting_rows_do_not_count_as_the_last_row() {
    let mut app = headless_app();
    let (column, rows) = spawn_setting_rows(&mut app, &["A", "B"]);
    app.world_mut().spawn((Node::default(), ChildOf(column)));
    app.update();
    assert_eq!(
        setting_row_divider_displays(&app, column),
        [Display::Flex, Display::None]
    );
    let extra = app
        .world_mut()
        .spawn((Node::default(), ChildOf(column)))
        .id();
    app.update();
    app.world_mut().entity_mut(extra).despawn();
    app.world_mut().entity_mut(rows[1]).despawn();
    app.update();
    assert_eq!(setting_row_divider_displays(&app, column), [Display::None]);
}

#[test]
fn empty_setting_rows_and_a_column_with_only_other_children_are_left_alone() {
    let mut app = headless_app();
    let empty = app.world_mut().spawn(SettingRows).id();
    let other = app.world_mut().spawn(SettingRows).id();
    app.world_mut().spawn((Node::default(), ChildOf(other)));
    app.update();
    app.update();
    assert!(app.world().get::<Children>(empty).is_none());
    assert!(setting_row_divider_displays(&app, other).is_empty());
}

#[test]
fn a_column_gets_its_dividers_when_the_marker_is_added_to_an_existing_parent() {
    let mut app = headless_app();
    let parent = app.world_mut().spawn(Node::default()).id();
    app.world_mut()
        .spawn((SettingRow::new("A", "d"), ChildOf(parent)));
    app.world_mut()
        .spawn((SettingRow::new("B", "d"), ChildOf(parent)));
    app.update();
    app.update();
    app.world_mut().entity_mut(parent).insert(SettingRows);
    app.update();
    assert_eq!(
        setting_row_divider_displays(&app, parent),
        [Display::Flex, Display::None]
    );
}

#[test]
fn a_quiet_frame_leaves_setting_row_dividers_alone_and_a_change_touches_only_its_column() {
    let mut app = headless_app();
    let (first, first_rows) = spawn_setting_rows(&mut app, &["A", "B"]);
    let (second, second_rows) = spawn_setting_rows(&mut app, &["C", "D"]);
    let divider =
        |app: &App, row: Entity| app.world().get::<SettingRowRefs>(row).unwrap().divider_id;
    // Someone else's change to a divider is not undone by a quiet frame.
    let first_divider = divider(&app, first_rows[0]);
    app.world_mut()
        .get_mut::<Node>(first_divider)
        .unwrap()
        .display = Display::None;
    app.update();
    assert_eq!(
        app.world().get::<Node>(first_divider).unwrap().display,
        Display::None
    );
    // A change in the second column does not look at the first.
    app.world_mut().entity_mut(second_rows[1]).despawn();
    app.update();
    assert_eq!(
        app.world().get::<Node>(first_divider).unwrap().display,
        Display::None
    );
    assert_eq!(setting_row_divider_displays(&app, second), [Display::None]);
    assert_eq!(setting_row_divider_displays(&app, first)[1], Display::None);
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
                app.world().get::<ToolbarButton>(button).unwrap().owner(),
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();

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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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

#[derive(Component)]
struct AppOwnedChild;

#[test]
fn a_gallery_keeps_children_of_the_app_through_the_first_build_and_rebuilds() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let card = |label: &str, group: &str| Item {
        data: GalleryCard::new(Handle::default(), label.to_string(), group.to_string()),
    };
    let gallery = app
        .world_mut()
        .spawn((
            Gallery,
            Selection::builder()
                .one()
                .build_with_items([card("Steam", "Locomotives")]),
            ChildOf(root),
        ))
        .id();
    // Present before the first build.
    let mine = app
        .world_mut()
        .spawn((AppOwnedChild, Node::default(), ChildOf(gallery)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().entity(mine).contains::<AppOwnedChild>());
    assert_eq!(gallery_groups_skipping(&app, gallery, mine).len(), 1);

    // Adding an item in a new group rebuilds the groups.
    let new_item = app
        .world_mut()
        .spawn((card("Passenger", "Wagons"), ItemOf(gallery)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().get_entity(mine).is_ok());
    assert!(app.world().entity(mine).contains::<AppOwnedChild>());
    assert_eq!(gallery_groups_skipping(&app, gallery, mine).len(), 2);

    // Changing a group rebuilds again.
    app.world_mut()
        .entity_mut(new_item)
        .get_mut::<Item<GalleryCard>>()
        .unwrap()
        .data
        .group = "Locomotives".into();
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().entity(mine).contains::<AppOwnedChild>());
    assert_eq!(gallery_groups_skipping(&app, gallery, mine).len(), 1);
    assert!(children_of(&app, gallery).contains(&mine));

    // A child added after a build is kept through a later rebuild too.
    let later = app
        .world_mut()
        .spawn((AppOwnedChild, Node::default(), ChildOf(gallery)))
        .id();
    app.world_mut()
        .entity_mut(new_item)
        .get_mut::<Item<GalleryCard>>()
        .unwrap()
        .data
        .group = "Wagons".into();
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().entity(later).contains::<AppOwnedChild>());
    assert!(app.world().entity(mine).contains::<AppOwnedChild>());
    let children = children_of(&app, gallery);
    assert!(children.contains(&later) && children.contains(&mine));
    assert_eq!(children[0], mine);
}

/// Like [`gallery_groups`], leaving out the app's own child.
fn gallery_groups_skipping(app: &App, gallery: Entity, skip: Entity) -> Vec<(String, Vec<Entity>)> {
    let mut app_children = Vec::new();
    let world = app.world();
    for child in children_of(app, gallery) {
        if child != skip && !world.entity(child).contains::<Divider>() {
            app_children.push(child);
        }
    }
    app_children
        .into_iter()
        .map(|group| {
            let parts = children_of(app, group);
            (String::new(), children_of(app, parts[1]))
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
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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
        .item();
    let colors = *app.world().get_resource::<Theme>().unwrap().colors();

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
        .item();
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

/// Gives `node` the target data Bevy would propagate from its camera: a viewport of `physical`
/// pixels on a target with `scale_factor` (the headless app has no camera to compute it).
fn set_render_target(app: &mut App, node: Entity, physical: UVec2, scale_factor: f32) {
    let mut info = ComputedUiRenderTargetInfo::default();
    info.field_mut("physical_size").unwrap().apply(&physical);
    info.field_mut("scale_factor").unwrap().apply(&scale_factor);
    app.world_mut().entity_mut(node).insert(info);
}

#[test]
fn a_tooltip_popup_stays_inside_a_restricted_viewport_not_the_primary_window() {
    let popup = Vec2::new(100.0, 20.0);
    // Without target data the primary window (800 wide) is the space: centred on 390.
    let (mut app, _) = placement_app((800, 600), Vec2::new(390.0, 100.0), Vec2::new(20.0, 20.0));
    assert!((placed_popup(&mut app, popup).0 - 340.0).abs() < 0.01);
    // A 400 x 300 logical viewport (800 x 600 physical at scale 2) pushes it in from its edge.
    let (mut app, node) = placement_app((800, 600), Vec2::new(390.0, 100.0), Vec2::new(20.0, 20.0));
    set_render_target(&mut app, node, UVec2::new(800, 600), 2.0);
    let (left, _) = placed_popup(&mut app, popup);
    assert!(
        (left + popup.x - (400.0 - 6.0)).abs() < 0.01,
        "left {left} does not keep 6 px from the viewport's right edge"
    );
    // An empty viewport is unusable data and falls back to the primary window.
    let (mut app, node) = placement_app((800, 600), Vec2::new(390.0, 100.0), Vec2::new(20.0, 20.0));
    set_render_target(&mut app, node, UVec2::ZERO, 1.0);
    assert!((placed_popup(&mut app, popup).0 - 340.0).abs() < 0.01);
}

#[test]
fn a_combo_list_stays_inside_the_ui_target_of_its_button_and_falls_back_to_the_window() {
    let list_left = |target: Option<(UVec2, f32)>| {
        let mut app = headless_app();
        app.world_mut().spawn((
            Window {
                resolution: (800, 600).into(),
                ..default()
            },
            PrimaryWindow,
        ));
        let (combo, button) = spawn_combo(&mut app);
        let list = app.world().entity(combo).get::<ComboRefs>().unwrap().list();
        app.world_mut().entity_mut(button).insert((
            ComputedNode {
                size: Vec2::new(40.0, 20.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from_translation(Vec2::new(390.0, 100.0)),
        ));
        app.world_mut().entity_mut(list).insert(ComputedNode {
            size: Vec2::new(100.0, 50.0),
            inverse_scale_factor: 1.0,
            ..default()
        });
        if let Some((physical, scale)) = target {
            set_render_target(&mut app, button, physical, scale);
        }
        click(&mut app, button);
        app.update();
        let Val::Px(left) = app.world().get::<Node>(list).unwrap().left else {
            return f32::NAN;
        };
        left
    };
    // The button's left edge, 370, fits the window.
    assert!((list_left(None) - 370.0).abs() < 0.01);
    // A 400 wide viewport keeps the 100 wide list 10 px inside its right edge.
    assert!((list_left(Some((UVec2::new(400, 300), 1.0))) - 290.0).abs() < 0.01);
    assert!((list_left(Some((UVec2::new(800, 600), 2.0))) - 290.0).abs() < 0.01);
    // Unusable target data falls back to the primary window.
    assert!((list_left(Some((UVec2::ZERO, 1.0))) - 370.0).abs() < 0.01);
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
fn a_showing_tooltip_follows_a_changed_text() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);

    app.world_mut()
        .entity_mut(button)
        .insert(Tooltip::new("Resume"));
    tick(&mut app);
    tick(&mut app);
    assert_eq!(tooltip_popups(&mut app), ["Resume"]);
}

#[test]
fn a_tooltip_inserted_on_a_node_the_pointer_already_rests_on_shows_after_the_delay() {
    let (mut app, button) = tooltip_app();
    app.world_mut().entity_mut(button).insert(Hovered(true));
    tick(&mut app);
    tick(&mut app);
    // Removed and inserted again while the pointer stays: `Hovered` does not change.
    app.world_mut().entity_mut(button).remove::<Tooltip>();
    tick(&mut app);
    app.world_mut()
        .entity_mut(button)
        .insert(Tooltip::new("Again"));
    tick(&mut app);
    assert!(
        tooltip_popups(&mut app).is_empty(),
        "shown before the delay"
    );
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Again"]);
}

#[test]
fn a_tooltip_that_was_left_starts_its_delay_again_on_the_next_hover() {
    let (mut app, button) = tooltip_app();
    // Rest for less than the delay, leave, return: the earlier time must not count.
    app.world_mut().entity_mut(button).insert(Hovered(true));
    tick(&mut app);
    tick(&mut app);
    app.world_mut().entity_mut(button).insert(Hovered(false));
    tick(&mut app);
    tick(&mut app);
    app.world_mut().entity_mut(button).insert(Hovered(true));
    tick(&mut app);
    tick(&mut app);
    assert!(tooltip_popups(&mut app).is_empty(), "the delay was kept");
    for _ in 0..4 {
        tick(&mut app);
    }
    assert_eq!(tooltip_popups(&mut app), ["Pause"]);
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

#[test]
fn replacing_a_badge_on_a_toolbar_item_updates_its_button_without_a_second_pill() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().one(),
    );
    let button = toolbar_buttons(&app, toolbar)[1];
    let item = app.world().get::<ToolbarButton>(button).unwrap().item();
    let description = |app: &App| {
        app.world()
            .get::<AccessibilityNode>(button)
            .and_then(|node| node.description().map(str::to_owned))
    };
    let pills = |app: &App| {
        app.world().get::<Children>(button).map_or(0, |children| {
            children
                .iter()
                .filter(|c| {
                    app.world()
                        .get::<Name>(*c)
                        .is_some_and(|n| n.as_str() == "Badge")
                })
                .count()
        })
    };
    let colors = DEFAULT_LIGHT_THEME_COLORS;

    app.world_mut().entity_mut(item).insert(Badge::count(6));
    settle(&mut app);
    let (pill, text) = badge_under(&app, button).unwrap();
    assert_eq!(text, "6");
    assert_eq!(
        app.world().get::<BackgroundColor>(pill).unwrap().0,
        colors.pill_background
    );

    // A replacement with another text and tone updates the same pill.
    app.world_mut()
        .entity_mut(item)
        .insert(Badge::new("9+").with_tone(BadgeTone::Warning));
    settle(&mut app);
    let (same, text) = badge_under(&app, button).unwrap();
    assert_eq!((same, text.as_str()), (pill, "9+"));
    assert_eq!(pills(&app), 1);
    assert_eq!(
        app.world().get::<BackgroundColor>(pill).unwrap().0,
        colors.warning_text
    );
    assert_eq!(description(&app).as_deref(), Some("9+"));

    // An empty replacement removes the overlay and the description.
    app.world_mut().entity_mut(item).insert(Badge::new(""));
    settle(&mut app);
    assert_eq!(pills(&app), 0);
    assert_eq!(description(&app), None);

    app.world_mut().entity_mut(item).insert(Badge::count(2));
    settle(&mut app);
    assert_eq!(pills(&app), 1);
    assert_eq!(badge_under(&app, button).unwrap().1, "2");
    assert_eq!(description(&app).as_deref(), Some("2"));
}

#[test]
fn a_badge_follows_its_item_to_the_buttons_made_when_the_toolbar_style_changes() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(
        &mut app,
        ToolbarStyle::Underlined,
        Selection::builder().one(),
    );
    let old_button = toolbar_buttons(&app, toolbar)[1];
    let item = app.world().get::<ToolbarButton>(old_button).unwrap().item();
    app.world_mut().entity_mut(item).insert(Badge::count(5));
    settle(&mut app);
    assert_eq!(badge_under(&app, old_button).unwrap().1, "5");

    app.world_mut()
        .entity_mut(toolbar)
        .insert(Toolbar::new(ToolbarStyle::Filled));
    settle(&mut app);
    let buttons = toolbar_buttons(&app, toolbar);
    let new_button = buttons
        .into_iter()
        .find(|b| app.world().get::<ToolbarButton>(*b).unwrap().item() == item)
        .unwrap();
    assert_eq!(badge_under(&app, new_button).unwrap().1, "5");
    let description = app
        .world()
        .get::<AccessibilityNode>(new_button)
        .and_then(|node| node.description().map(str::to_owned));
    assert_eq!(description.as_deref(), Some("5"));
}

#[test]
fn a_theme_change_recolours_a_badge_that_did_not_change() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let button = app
        .world_mut()
        .spawn((
            IconButton::new(Handle::default(), "Trains", IconButtonSize::Regular),
            Badge::count(3),
            ChildOf(root),
        ))
        .id();
    settle(&mut app);
    // Quiet frames change nothing.
    settle(&mut app);
    let (badge, _) = badge_under(&app, button).unwrap();
    assert_eq!(
        app.world().get::<BackgroundColor>(badge).unwrap().0,
        DEFAULT_LIGHT_THEME_COLORS.pill_background
    );
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Dark);
    settle(&mut app);
    assert_eq!(
        app.world().get::<BackgroundColor>(badge).unwrap().0,
        DEFAULT_DARK_THEME_COLORS.pill_background
    );
}

#[test]
fn a_panel_keeps_its_sections_and_corner_margin_across_theme_changes_and_quiet_frames() {
    let mut app = headless_app();
    let panel = app.world_mut().spawn(Panel::new("Trains")).id();
    settle(&mut app);
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    let margin = |app: &App| {
        app.world()
            .get::<Node>(refs.scroll_area())
            .unwrap()
            .margin
            .bottom
    };
    let strip_text = app
        .world_mut()
        .spawn((Label::caption("Filter"), ChildOf(refs.strip())))
        .id();
    settle(&mut app);
    let display = |app: &App, e: Entity| app.world().get::<Node>(e).unwrap().display;
    assert_eq!(display(&app, refs.strip()), Display::Flex);
    app.world_mut().despawn(strip_text);
    settle(&mut app);
    assert_eq!(display(&app, refs.strip()), Display::None);

    let new_radius = Val::Px(20.0);
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .panel_radius = new_radius;
    settle(&mut app);
    assert_eq!(margin(&app), new_radius);
    assert_eq!(
        app.world()
            .get::<Node>(refs.scroll_area())
            .unwrap()
            .max_height,
        Val::Px(640.0)
    );

    // A new maximum height is applied through a new Panel.
    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Trains").with_max_body_height(100.0).unwrap());
    settle(&mut app);
    assert_eq!(
        app.world()
            .get::<Node>(refs.scroll_area())
            .unwrap()
            .max_height,
        Val::Px(120.0)
    );
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

fn theme_editor_app() -> App {
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
    let bar = single_entity_with::<TabBar>(&mut app);
    select_by_text(&mut app, bar, "Theme editor");
    for _ in 0..6 {
        app.update();
    }
    app
}

#[test]
fn the_theme_editor_has_a_row_for_each_editable_colour_slot() {
    let mut app = theme_editor_app();
    assert_eq!(ui_root_count(&mut app), 1);
    let slots = bevy_pluim::theme_editor::editable_color_slots();
    assert!(slots.len() > 20);
    let world = app.world_mut();
    assert_eq!(
        world
            .query::<&bevy_pluim::theme_editor::ThemeColorSlot>()
            .iter(world)
            .count(),
        slots.len()
    );
    assert_eq!(
        world
            .query::<&bevy_pluim::theme_editor::ThemeColorSwatch>()
            .iter(world)
            .count(),
        slots.len()
    );
}

#[test]
fn picking_a_colour_in_the_theme_editor_sets_that_slot_of_the_theme_in_use() {
    let mut app = theme_editor_app();
    let (combo, items) = {
        let world = app.world_mut();
        let combo = world
            .query::<(Entity, &bevy_pluim::theme_editor::ThemeColorSlot)>()
            .iter(world)
            .find(|(_, slot)| slot.0 == "accent")
            .map(|(e, _)| e)
            .unwrap();
        let red = world
            .query::<&Item<Color>>()
            .iter(world)
            .map(|i| i.data)
            .nth(60)
            .unwrap();
        (combo, red)
    };
    let picked = {
        let world = app.world_mut();
        world
            .query::<(Entity, &Item<Color>)>()
            .iter(world)
            .find(|(_, i)| i.data == items)
            .map(|(e, _)| e)
            .unwrap()
    };
    let dark_before = app
        .world()
        .get_resource::<Theme>()
        .unwrap()
        .colors_of(ThemeMode::Dark);
    app.world_mut()
        .entity_mut(combo)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(picked))));
    for _ in 0..3 {
        app.update();
    }
    let theme = app.world().get_resource::<Theme>().unwrap();
    let mode = theme.mode();
    assert_eq!(
        bevy_pluim::theme_editor::color_slot(theme.colors(), "accent"),
        Some(items)
    );
    assert_eq!(
        bevy_pluim::theme_editor::color_slot(&theme.colors_of(mode), "accent"),
        Some(items)
    );
    if mode == ThemeMode::Light {
        assert_eq!(theme.colors_of(ThemeMode::Dark), dark_before);
    }
}

/// Picks the colour at index 60 of the palette (red-500) for the "accent" row of the theme
/// editor and returns it.
fn pick_accent_in_the_theme_editor(app: &mut App) -> Color {
    let combo = {
        let world = app.world_mut();
        world
            .query::<(Entity, &bevy_pluim::theme_editor::ThemeColorSlot)>()
            .iter(world)
            .find(|(_, slot)| slot.0 == "accent")
            .map(|(e, _)| e)
            .unwrap()
    };
    let (picked, color) = {
        let world = app.world_mut();
        let color = world
            .query::<&Item<Color>>()
            .iter(world)
            .map(|i| i.data)
            .nth(60)
            .unwrap();
        let picked = world
            .query::<(Entity, &Item<Color>)>()
            .iter(world)
            .find(|(_, i)| i.data == color)
            .map(|(e, _)| e)
            .unwrap();
        (picked, color)
    };
    app.world_mut()
        .entity_mut(combo)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(picked))));
    for _ in 0..3 {
        app.update();
    }
    color
}

#[test]
fn picking_a_colour_in_the_theme_editor_in_dark_mode_sets_the_dark_half_only() {
    let mut app = theme_editor_app();
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .light_dark_toggle();
    app.update();
    app.update();
    let before = app.world().get_resource::<Theme>().unwrap().clone();
    assert_eq!(before.mode(), ThemeMode::Dark);

    let red = pick_accent_in_the_theme_editor(&mut app);

    let theme = app.world().get_resource::<Theme>().unwrap();
    assert_eq!(theme.mode(), ThemeMode::Dark);
    assert_ne!(
        bevy_pluim::theme_editor::color_slot(&before.colors_of(ThemeMode::Dark), "accent"),
        Some(red),
        "the pick is a change"
    );
    assert_eq!(
        bevy_pluim::theme_editor::color_slot(&theme.colors_of(ThemeMode::Dark), "accent"),
        Some(red)
    );
    assert_eq!(
        bevy_pluim::theme_editor::color_slot(theme.colors(), "accent"),
        Some(red)
    );
    assert_eq!(
        theme.colors_of(ThemeMode::Light),
        before.colors_of(ThemeMode::Light)
    );
}

#[test]
fn the_theme_editor_pair_source_has_both_blocks_and_follows_an_edit_to_the_dark_half() {
    let mut theme = Theme::default();
    let source = overview::theme_editor::pair_source(&theme);
    assert!(source.contains("const LIGHT: ThemeColors = ThemeColors {\n"));
    assert!(source.contains("const DARK: ThemeColors = ThemeColors {\n"));

    let mut dark = theme.colors_of(ThemeMode::Dark);
    assert!(bevy_pluim::theme_editor::set_color_slot(
        &mut dark,
        "accent",
        Color::srgba(0.25, 0.5, 0.75, 1.0)
    ));
    theme.set_colors(ThemeMode::Dark, dark);
    let edited = overview::theme_editor::pair_source(&theme);
    let (light_block, dark_block) = edited.split_once("const DARK").unwrap();
    assert!(dark_block.contains("    accent: Color::srgba(0.25, 0.5, 0.75, 1"));
    assert!(!light_block.contains("0.25, 0.5, 0.75"));
    assert_eq!(
        light_block,
        overview::theme_editor::pair_source(&Theme::default())
            .split_once("const DARK")
            .unwrap()
            .0,
        "the light block is untouched"
    );
}

#[test]
fn activating_the_theme_editors_export_button_spawns_a_toast_under_the_ui_root() {
    let mut app = theme_editor_app();
    let export = {
        let world = app.world_mut();
        let label = world
            .query::<(&Text, &ChildOf)>()
            .iter(world)
            .find(|(text, _)| text.0 == "Export")
            .map(|(_, child_of)| child_of.parent())
            .unwrap();
        assert!(
            world.get::<PushButton>(label).is_some(),
            "the label is in a PushButton"
        );
        label
    };
    let root = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<UiRoot>>()
            .single(world)
            .unwrap()
    };
    assert_eq!(count_toasts(&mut app), 0);

    app.world_mut().trigger(Activate { entity: export });
    app.update();
    app.update();

    let world = app.world_mut();
    let toasts: Vec<Entity> = world
        .query_filtered::<Entity, With<Toast>>()
        .iter(world)
        .collect();
    assert_eq!(toasts.len(), 1);
    assert!(
        world
            .get::<ChildOf>(toasts[0])
            .is_some_and(|child_of| child_of.parent() == root
                || world
                    .get::<ChildOf>(child_of.parent())
                    .is_some_and(|stack| stack.parent() == root)),
        "the toast is under the UI root (directly or in its stack)"
    );
}

fn count_toasts(app: &mut App) -> usize {
    let world = app.world_mut();
    world
        .query_filtered::<(), With<Toast>>()
        .iter(world)
        .count()
}

#[test]
fn the_theme_editor_exports_every_colour_slot_as_rust_text() {
    use bevy::reflect::structs::Struct as _;
    let source = overview::theme_editor::colors_source("LIGHT", &DEFAULT_LIGHT_THEME_COLORS);
    assert!(source.starts_with("const LIGHT: ThemeColors = ThemeColors {\n"));
    assert!(source.ends_with("};\n"));
    let lines = source.lines().count();
    assert_eq!(lines, DEFAULT_LIGHT_THEME_COLORS.field_len() + 2);
    assert!(source.contains("    accent: Color::srgba("));
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
    // Showing each page in turn builds it: none of them panics or logs a missing widget, and the
    // page has its header (the same text as its tab, so a shown page makes two of them).
    let pages = [
        "Buttons",
        "Icon buttons",
        "Labels",
        "Tags",
        "Lists",
        "Combos",
        "Filter combos",
        "Colours",
        "Segmented control",
        "Light dark toggle",
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
        "Theme editor",
    ];
    {
        let world = app.world_mut();
        assert_eq!(world.query::<&TabBarTab>().iter(world).count(), pages.len());
    }
    for page in pages {
        select_by_text(&mut app, bar, page);
        for _ in 0..4 {
            app.update();
        }
        let world = app.world_mut();
        let texts = world
            .query::<&Text>()
            .iter(world)
            .filter(|text| text.0 == page)
            .count();
        assert_eq!(texts, 2, "the {page} page has its header");
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
                    || {
                        bsn_list! {
                            Node
                            --
                            Node
                            --
                            Node
                        }
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

fn scene_app() -> App {
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
    app.world_mut().spawn(Camera2d);
    app
}

/// Each area's contents entity with the entities below it, and the targets of its scroll bars.
fn scroll_areas_of(world: &mut World) -> Vec<(Entity, Vec<Entity>, Vec<Entity>)> {
    let areas: Vec<Entity> = world
        .query_filtered::<Entity, With<ScrollArea>>()
        .iter(world)
        .collect();
    areas
        .into_iter()
        .map(|area| {
            let contents = world
                .query_filtered::<(Entity, &ChildOf), With<ScrollContents>>()
                .iter(world)
                .find(|(_, parent)| parent.parent() == area)
                .map(|(entity, _)| entity)
                .unwrap();
            let children = world
                .get::<Children>(contents)
                .map(|children| children.iter().collect())
                .unwrap_or_default();
            let targets = world
                .query::<(&ChildOf, &Scrollbar)>()
                .iter(world)
                .filter(|(parent, _)| parent.parent() == area)
                .map(|(_, bar)| bar.target)
                .collect();
            (contents, children, targets)
        })
        .collect()
}

fn assert_two_distinct_areas(world: &mut World, children: usize) {
    let areas = scroll_areas_of(world);
    assert_eq!(areas.len(), 2);
    let (first, second) = (&areas[0], &areas[1]);
    assert_ne!(first.0, second.0);
    for (contents, kids, targets) in &areas {
        assert_eq!(kids.len(), children);
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().all(|target| target == contents));
    }
    assert!(first.1.iter().all(|kid| !second.1.contains(kid)));
}

#[test]
fn a_scroll_area_scene_gives_every_build_its_own_fresh_contents() {
    let mut app = scene_app();
    let area = || {
        ScrollArea::scene(
            || {
                bsn_list! {
                    Node
                    --
                    Node
                    --
                    Node
                }
            },
            true,
            true,
        )
    };
    app.world_mut()
        .spawn_scene(bsn! { Node Children [ @area() -- @area() ] })
        .unwrap();
    for _ in 0..4 {
        app.update();
    }
    assert_two_distinct_areas(app.world_mut(), 3);
}

#[test]
fn a_cached_scroll_area_scene_builds_fresh_contents_each_time_it_is_spawned() {
    let mut app = scene_app();
    let server = app.world().get_resource::<AssetServer>().unwrap().clone();
    let mut patch = bevy::scene::ScenePatch::load(
        &server,
        bsn! {
            Node
            Children [
                @ScrollArea::scene(
                    || bsn_list! { Node -- Node },
                    true,
                    true,
                )
            ]
        },
    );
    patch
        .resolve(
            &server,
            app.world()
                .get_resource::<Assets<bevy::scene::ScenePatch>>()
                .unwrap(),
        )
        .unwrap();
    // One resolved scene, applied twice: the contents are made again for each.
    patch.spawn(app.world_mut()).unwrap();
    patch.spawn(app.world_mut()).unwrap();
    for _ in 0..4 {
        app.update();
    }
    assert_two_distinct_areas(app.world_mut(), 2);
}

#[test]
fn a_scroll_area_factory_with_a_captured_handle_builds_asset_scenes_again_and_again() {
    let mut app = scene_app();
    let icon = app
        .world_mut()
        .get_resource_mut::<Assets<Image>>()
        .unwrap()
        .add(Image::default());
    let area = move || {
        let icon = icon.clone();
        ScrollArea::scene(
            move || {
                bsn_list! {
                    IconButton { icon: {icon.clone()}, name: "Close", size: IconButtonSize::Regular }
                }
            },
            true,
            true,
        )
    };
    app.world_mut()
        .spawn_scene(bsn! { Node Children [ @area() -- @area() ] })
        .unwrap();
    for _ in 0..6 {
        app.update();
    }
    let world = app.world_mut();
    assert_two_distinct_areas(world, 1);
    let names: Vec<String> = world
        .query::<&IconButton>()
        .iter(world)
        .map(|button| button.name.clone())
        .collect();
    assert_eq!(names, ["Close", "Close"]);
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
    (stepper, children[1], children[3])
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
    let (decrease, increase) = (children[1], children[3]);

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
fn a_note_change_resyncs_only_the_rows_of_its_item_and_a_theme_change_all_of_them() {
    let mut app = headless_app();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let spawn_list = |world: &mut World, label: &str| {
        world
            .spawn((
                Node::default(),
                List,
                Selection::builder()
                    .one()
                    .build_with_items([(Item::new(label.to_string()), ItemNote::new("note"))]),
            ))
            .id()
    };
    let first = spawn_list(world, "First");
    let second = spawn_list(world, "Second");
    // A third list draws from the first one's items, so one note shows in two lists.
    let sharing = world
        .spawn((
            Node::default(),
            List,
            Selection::builder().one().build_with_items_ref(first),
        ))
        .id();
    for _ in 0..4 {
        app.update();
    }

    // The note text of each row of a list, found through its views.
    let notes_of = |app: &mut App, list: Entity| -> Vec<Entity> {
        let world = app.world_mut();
        let mut views = world.query::<(Entity, &ItemView)>();
        let views: Vec<Entity> = views
            .iter(world)
            .filter(|(_, view)| view.list_ref == list)
            .map(|(view, _)| view)
            .collect();
        views
            .into_iter()
            .flat_map(|view| {
                world
                    .get::<Children>(view)
                    .map(|c| c.iter().collect::<Vec<_>>())
                    .unwrap_or_default()
            })
            .filter(|child| {
                world
                    .get::<Text>(*child)
                    .is_some_and(|t| t.0 == "note" || t.0 == "stale")
            })
            .collect()
    };
    let (first_notes, second_notes, shared_notes) = (
        notes_of(&mut app, first),
        notes_of(&mut app, second),
        notes_of(&mut app, sharing),
    );
    assert_eq!(
        (first_notes.len(), second_notes.len(), shared_notes.len()),
        (1, 1, 1)
    );

    // Mark every note text so a row that is looked at again shows.
    for note in [first_notes[0], second_notes[0], shared_notes[0]] {
        app.world_mut().get_mut::<Text>(note).unwrap().0 = "stale".to_string();
    }
    let item_of = |app: &App, list: Entity| {
        let mut views = app.world().try_query::<&ItemView>().unwrap();
        views
            .iter(app.world())
            .find(|view| view.list_ref == list)
            .unwrap()
            .item_ref
    };
    let first_item = item_of(&app, first);
    app.world_mut()
        .entity_mut(first_item)
        .insert(ItemNote::warning("changed"));
    for _ in 0..3 {
        app.update();
    }
    let text = |app: &App, note: Entity| app.world().get::<Text>(note).unwrap().0.clone();
    assert_eq!(text(&app, first_notes[0]), "changed");
    assert_eq!(text(&app, shared_notes[0]), "changed", "a shared note");
    assert_eq!(
        text(&app, second_notes[0]),
        "stale",
        "an unrelated list is not looked at"
    );

    // Emptying and removing a note still takes the text from the shared rows.
    app.world_mut()
        .entity_mut(first_item)
        .insert(ItemNote::new(""));
    for _ in 0..3 {
        app.update();
    }
    assert!(notes_of(&mut app, first).is_empty());
    assert!(app.world().get_entity(shared_notes[0]).is_err());
    assert_eq!(text(&app, second_notes[0]), "stale");

    // A theme change restyles every row.
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(ThemeMode::Dark);
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(text(&app, second_notes[0]), "note");
}

#[test]
fn reinserting_a_toast_on_a_live_entity_restarts_its_timer() {
    use bevy::time::TimeUpdateStrategy;
    use bevy_pluim::toast::Toast;
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

#[test]
fn toasts_with_unusual_durations_do_not_panic_and_the_zero_ones_go_after_a_tick() {
    use bevy_pluim::toast::Toast;

    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let mut spawn = |text: &str, duration: f32| {
        app.world_mut()
            .spawn((Toast::new(text).with_duration(duration), ChildOf(root)))
            .id()
    };
    let long: Vec<Entity> = [("inf", f32::INFINITY), ("max", f32::MAX)]
        .into_iter()
        .map(|(text, duration)| spawn(text, duration))
        .collect();
    let short: Vec<Entity> = [("nan", f32::NAN), ("negative", -1.0), ("zero", 0.0)]
        .into_iter()
        .map(|(text, duration)| spawn(text, duration))
        .collect();
    for _ in 0..5 {
        app.update();
    }
    for toast in short {
        assert!(
            app.world().get_entity(toast).is_err(),
            "a zero toast stayed"
        );
    }
    for toast in long {
        assert!(app.world().get_entity(toast).is_ok(), "a capped toast went");
    }
    // Re-inserting a huge duration on a live entity must not panic either.
    let again = app
        .world_mut()
        .spawn((Toast::new("again"), ChildOf(root)))
        .id();
    app.update();
    app.world_mut()
        .entity_mut(again)
        .insert(Toast::new("again").with_duration(f32::INFINITY));
    app.update();
    app.update();
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
        .item();

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
        .item();

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
        .item();

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
        .item();

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
    let link = spawn_link(&mut app, false);
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

    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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

    let colors = *app.world().get_resource::<Theme>().unwrap().colors();
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

#[test]
fn filter_chips_do_not_look_at_their_chips_in_a_frame_without_a_change() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    let item = app.world().get::<ChipView>(views[1]).unwrap().item();

    // Break the children by hand: a frame with nothing to react to must leave it alone.
    app.world_mut().entity_mut(views[1]).despawn();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(chip_views(&app, chips).len(), 2);

    // A change to an item makes the chips look again, and they are built as they should be.
    app.world_mut()
        .get_mut::<Item<Chip>>(item)
        .unwrap()
        .set_changed();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(chip_views(&app, chips).len(), 3);
}

/// Breaks a chips row by hand (one chip view despawned) so only a rebuild repairs it.
fn break_chips(app: &mut App, chips: Entity) -> Entity {
    let views = chip_views(app, chips);
    let item = app.world().get::<ChipView>(views[1]).unwrap().item();
    app.world_mut().entity_mut(views[1]).despawn();
    item
}

#[test]
fn a_change_to_one_chips_source_leaves_independent_chips_alone() {
    let mut app = headless_app();
    let (_, first) = spawn_chips(&mut app, three_chips());
    let (_, second) = spawn_chips(&mut app, three_chips());
    let item = break_chips(&mut app, first);
    break_chips(&mut app, second);
    settle(&mut app);
    assert_eq!(chip_views(&app, first).len(), 2);
    assert_eq!(chip_views(&app, second).len(), 2);

    app.world_mut()
        .get_mut::<Item<Chip>>(item)
        .unwrap()
        .set_changed();
    settle(&mut app);
    assert_eq!(
        chip_views(&app, first).len(),
        3,
        "the changed source rebuilds"
    );
    assert_eq!(
        chip_views(&app, second).len(),
        2,
        "an independent one is not looked at"
    );
}

#[test]
fn chips_sharing_a_selection_or_items_are_all_rebuilt_by_a_change_to_the_source() {
    let mut app = headless_app();
    let (root, first) = spawn_chips(&mut app, three_chips());
    let (_, other) = spawn_chips(&mut app, three_chips());
    let shared = app
        .world_mut()
        .spawn((FilterChips, Selection::reference(first), ChildOf(root)))
        .id();
    let over_items = app
        .world_mut()
        .spawn((
            FilterChips,
            Selection::Value(SelectionValue::ZeroOrOne(None)),
            ItemsRef(first),
            ChildOf(root),
        ))
        .id();
    settle(&mut app);
    for chips in [first, shared, over_items, other] {
        assert_eq!(chip_views(&app, chips).len(), 3);
    }
    let item = break_chips(&mut app, first);
    break_chips(&mut app, shared);
    break_chips(&mut app, over_items);
    break_chips(&mut app, other);
    settle(&mut app);

    app.world_mut()
        .get_mut::<Item<Chip>>(item)
        .unwrap()
        .set_changed();
    settle(&mut app);
    for chips in [first, shared, over_items] {
        assert_eq!(chip_views(&app, chips).len(), 3);
    }
    assert_eq!(chip_views(&app, other).len(), 2);

    // A new item of the source reaches every widget that draws from it, and only those.
    app.world_mut().spawn((
        Item {
            data: Chip::new("Extra"),
        },
        ItemOf(first),
    ));
    settle(&mut app);
    for chips in [first, shared, over_items] {
        assert_eq!(chip_views(&app, chips).len(), 4);
    }
    assert_eq!(chip_views(&app, other).len(), 2);
}

#[test]
fn replacing_the_items_ref_moves_the_chips_to_the_new_source_and_removals_are_seen() {
    let mut app = headless_app();
    let (root, first) = spawn_chips(&mut app, three_chips());
    let (_, second) = spawn_chips(&mut app, [Chip::new("X"), Chip::new("Y"), Chip::new("Z")]);
    let over = app
        .world_mut()
        .spawn((
            FilterChips,
            Selection::Value(SelectionValue::ZeroOrOne(None)),
            ItemsRef(first),
            ChildOf(root),
        ))
        .id();
    settle(&mut app);
    let texts = |app: &App| -> Vec<String> {
        chip_views(app, over)
            .into_iter()
            .map(|v| chip_texts(app, v).0)
            .collect()
    };
    assert_eq!(texts(&app), ["All", "Running", "Problems"]);

    app.world_mut().entity_mut(over).insert(ItemsRef(second));
    settle(&mut app);
    assert_eq!(texts(&app), ["X", "Y", "Z"]);

    // A removal in the old source no longer reaches it; one in the new source does.
    let old_item = app
        .world()
        .get::<ChipView>(chip_views(&app, first)[0])
        .unwrap()
        .item();
    app.world_mut().entity_mut(old_item).despawn();
    settle(&mut app);
    assert_eq!(texts(&app), ["X", "Y", "Z"]);
    assert_eq!(chip_views(&app, first).len(), 2);

    let new_item = app
        .world()
        .get::<ChipView>(chip_views(&app, second)[2])
        .unwrap()
        .item();
    app.world_mut().entity_mut(new_item).despawn();
    settle(&mut app);
    assert_eq!(texts(&app), ["X", "Y"]);
    assert_eq!(chip_views(&app, second).len(), 2);

    // An item that stays under its owner but stops being a chip.
    let kept = app
        .world()
        .get::<ChipView>(chip_views(&app, second)[0])
        .unwrap()
        .item();
    app.world_mut().entity_mut(kept).remove::<Item<Chip>>();
    settle(&mut app);
    assert_eq!(texts(&app), ["Y"]);
    assert_eq!(chip_views(&app, second).len(), 1);
}

#[test]
fn a_pending_selection_is_repaired_when_its_items_arrive_and_the_chips_follow() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let chips = app
        .world_mut()
        .spawn((
            FilterChips,
            Selection::builder()
                .one()
                .build_with_items(std::iter::empty::<Item<Chip>>()),
            ChildOf(root),
        ))
        .id();
    let other = app
        .world_mut()
        .spawn((
            FilterChips,
            Selection::builder().one().build_with_items([Item {
                data: Chip::new("Only"),
            }]),
            ChildOf(root),
        ))
        .id();
    settle(&mut app);
    assert!(chip_views(&app, chips).is_empty());
    let other_before = chip_views(&app, other);
    assert_eq!(other_before.len(), 1);
    app.world_mut().entity_mut(other_before[0]).despawn();

    app.world_mut().spawn((
        Item {
            data: Chip::new("First"),
        },
        ItemOf(chips),
    ));
    settle(&mut app);
    let views = chip_views(&app, chips);
    assert_eq!(views.len(), 1);
    assert_eq!(chip_texts(&app, views[0]).0, "First");
    let selected = app
        .world()
        .get::<Selection>(chips)
        .unwrap()
        .as_selection()
        .unwrap()
        .single();
    assert_eq!(
        selected,
        Some(app.world().get::<ChipView>(views[0]).unwrap().item())
    );
    assert!(
        chip_views(&app, other).is_empty(),
        "the unrelated row is left alone"
    );
}

#[test]
fn a_tab_bar_follows_only_the_selection_it_draws_from() {
    let mut app = headless_app();
    let items = || ["General", "Audio", "Input"].map(Item::new);
    let first = spawn_tab_bar(
        &mut app,
        Selection::builder().one().build_with_items(items()),
    );
    let second = spawn_tab_bar(
        &mut app,
        Selection::builder().one().build_with_items(items()),
    );
    let root = app.world().get::<ChildOf>(first).unwrap().parent();
    let shared = app
        .world_mut()
        .spawn((TabBar, Selection::reference(first), ChildOf(root)))
        .id();
    settle(&mut app);
    let tab_item = |app: &App, bar: Entity, i: usize| {
        app.world()
            .get::<TabBarTab>(tabs_of(app, bar)[i])
            .unwrap()
            .item()
    };
    let wanted = tab_item(&app, first, 2);

    // Knock the second bar's `SelectedTab` out of step without a selection change.
    app.world_mut().entity_mut(second).insert(SelectedTab(None));
    settle(&mut app);
    assert_eq!(selected_tab(&app, second), None);

    app.world_mut()
        .entity_mut(first)
        .insert(Selection::Value(SelectionValue::One(wanted)));
    settle(&mut app);
    assert_eq!(selected_tab(&app, first), Some(tabs_of(&app, first)[2]));
    assert_eq!(selected_tab(&app, shared), Some(tabs_of(&app, shared)[2]));
    assert_eq!(
        selected_tab(&app, second),
        None,
        "the independent bar is not looked at"
    );

    // Its own selection change does reach it.
    let own = tab_item(&app, second, 1);
    app.world_mut()
        .entity_mut(second)
        .insert(Selection::Value(SelectionValue::One(own)));
    settle(&mut app);
    assert_eq!(selected_tab(&app, second), Some(tabs_of(&app, second)[1]));
}

#[test]
fn a_toolbar_does_not_look_at_its_buttons_in_a_frame_without_a_change() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let buttons = toolbar_buttons(&app, toolbar);
    let item = app.world().get::<ToolbarButton>(buttons[0]).unwrap().item();

    app.world_mut().entity_mut(buttons[0]).despawn();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(toolbar_buttons(&app, toolbar).len(), buttons.len() - 1);

    app.world_mut()
        .get_mut::<Item<Tool>>(item)
        .unwrap()
        .set_changed();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(toolbar_buttons(&app, toolbar).len(), buttons.len());
}

#[test]
fn a_gallery_does_not_look_at_its_items_in_a_frame_without_a_change() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let groups = gallery_groups(&app, gallery);
    assert_eq!(groups.len(), 2);
    let card = groups[1].1[1];
    let item = app.world().get::<GalleryCardView>(card).unwrap().item();

    // Move the last card to a group of its own without Bevy noticing: a frame with nothing to
    // react to must not look at the items, so the gallery stays as it was.
    app.world_mut()
        .get_mut::<Item<GalleryCard>>(item)
        .unwrap()
        .bypass_change_detection()
        .data
        .group = "Specials".into();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(gallery_groups(&app, gallery), groups);

    // A change to an item makes the gallery look again, and the groups are built as they should be.
    app.world_mut()
        .get_mut::<Item<GalleryCard>>(item)
        .unwrap()
        .set_changed();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(gallery_groups(&app, gallery).len(), 3);
}

#[test]
fn a_gallery_still_rebuilds_its_groups_when_the_group_of_an_item_changes() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let groups = gallery_groups(&app, gallery);
    assert_eq!(groups.len(), 2);
    let card = groups[1].1[1];
    let item = app.world().get::<GalleryCardView>(card).unwrap().item();

    // The last card moves to a group of its own; quiet frames before and after change nothing.
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(gallery_groups(&app, gallery), groups);
    app.world_mut()
        .get_mut::<Item<GalleryCard>>(item)
        .unwrap()
        .data
        .group = "Specials".into();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(gallery_groups(&app, gallery).len(), 3);
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

fn children_with<C: Component>(app: &mut App, stack: Entity) -> Vec<Entity> {
    let world = app.world_mut();
    let children = world
        .get::<Children>(stack)
        .map(|c| c.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    children
        .into_iter()
        .filter(|c| world.entity(*c).contains::<C>())
        .collect()
}

#[test]
fn two_stacks_sharing_a_selection_each_get_a_child_of_their_own() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);
    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("a"), StackItem::new_with_reference(0, create_a)),
            (Item::new("a again"), StackItem::new_reference(0)),
            (Item::new("b"), StackItem::new(create_b)),
        ]))
        .id();
    let items = item_entities_of(&app, selection_ref);
    let first = stack_over(&mut app, selection_ref);
    let second = stack_over(&mut app, selection_ref);

    // Both stacks show the selected item, each with a child parented to itself.
    let a_first = children_with::<PanelA>(&mut app, first);
    let a_second = children_with::<PanelA>(&mut app, second);
    assert_eq!((a_first.len(), a_second.len()), (1, 1));
    assert_ne!(a_first[0], a_second[0]);
    assert_eq!(node_display(&app, a_first[0]), Display::Flex);
    assert_eq!(node_display(&app, a_second[0]), Display::Flex);

    // Switching shows the matching child in both; nothing is created lazily before it is picked.
    select_one(&mut app, selection_ref, items[2]);
    assert_eq!(count_entities_with::<PanelB>(&mut app), 2);
    assert_eq!(node_display(&app, a_first[0]), Display::None);
    assert_eq!(node_display(&app, a_second[0]), Display::None);

    // A reference reuses the creator's child of each stack, not the other stack's.
    select_one(&mut app, selection_ref, items[1]);
    assert_eq!(children_with::<PanelA>(&mut app, first), a_first);
    assert_eq!(children_with::<PanelA>(&mut app, second), a_second);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 2);
    assert_eq!(node_display(&app, a_first[0]), Display::Flex);
    assert_eq!(node_display(&app, a_second[0]), Display::Flex);
}

#[test]
fn two_stacks_sharing_a_selection_each_build_their_own_scene_child() {
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
    let create_b = app.world_mut().register_system(spawn_panel_b);
    let selection_ref = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("scene"), StackItem::new(create)),
            (Item::new("b"), StackItem::new(create_b)),
        ]))
        .id();
    let items = item_entities_of(&app, selection_ref);
    let root = app.world_mut().spawn(Node::default()).id();
    let stacks = [(); 2].map(|()| {
        app.world_mut()
            .spawn((Node::default(), Stack { selection_ref }, ChildOf(root)))
            .id()
    });
    for _ in 0..4 {
        app.update();
    }
    for stack in stacks {
        assert_eq!(children_with::<PanelA>(&mut app, stack).len(), 1);
    }
    // Switching away and back reuses each stack's scene root.
    select_one(&mut app, selection_ref, items[1]);
    select_one(&mut app, selection_ref, items[0]);
    for stack in stacks {
        let roots = children_with::<PanelA>(&mut app, stack);
        assert_eq!(roots.len(), 1);
        assert_eq!(node_display(&app, roots[0]), Display::Flex);
    }
    assert_eq!(count_entities_with::<PanelA>(&mut app), 2);
}

#[test]
fn despawning_a_stack_leaves_the_other_one_and_a_recreated_stack_gets_new_children() {
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
    let items = item_entities_of(&app, selection_ref);
    let first = stack_over(&mut app, selection_ref);
    let second = stack_over(&mut app, selection_ref);
    let kept = children_with::<PanelA>(&mut app, second)[0];
    assert_eq!(count_entities_with::<PanelA>(&mut app), 2);

    app.world_mut().entity_mut(first).despawn();
    app.update();
    assert_eq!(count_entities_with::<PanelA>(&mut app), 1);
    select_one(&mut app, selection_ref, items[1]);
    select_one(&mut app, selection_ref, items[0]);
    assert_eq!(children_with::<PanelA>(&mut app, second), [kept]);
    assert_eq!(node_display(&app, kept), Display::Flex);

    // A new stack over the same items makes its own children; the old stack's are not reused.
    let third = stack_over(&mut app, selection_ref);
    let fresh = children_with::<PanelA>(&mut app, third);
    assert_eq!(fresh.len(), 1);
    assert_ne!(fresh[0], kept);
    assert_eq!(count_entities_with::<PanelA>(&mut app), 2);
    select_one(&mut app, selection_ref, items[1]);
    assert_eq!(children_with::<PanelB>(&mut app, third).len(), 1);
    assert_eq!(children_with::<PanelB>(&mut app, second).len(), 1);
}

#[test]
fn a_stack_over_a_selection_ref_follows_the_selection_it_refers_to() {
    let mut app = headless_app();
    let create_a = app.world_mut().register_system(spawn_panel_a);
    let create_b = app.world_mut().register_system(spawn_panel_b);
    let source = app
        .world_mut()
        .spawn(Selection::builder().one().build_with_items([
            (Item::new("a"), StackItem::new(create_a)),
            (Item::new("b"), StackItem::new(create_b)),
        ]))
        .id();
    let reference = app.world_mut().spawn(Selection::reference(source)).id();
    let stack = stack_over(&mut app, reference);
    let panel_a = single_entity_with::<PanelA>(&mut app);
    assert!(no_entity_with::<PanelB>(&mut app));

    let items = item_entities_of(&app, source);
    select_one(&mut app, source, items[1]);
    for _ in 0..3 {
        app.update();
    }
    let panel_b = single_entity_with::<PanelB>(&mut app);
    assert_eq!(node_display(&app, panel_a), Display::None);
    assert_eq!(node_display(&app, panel_b), Display::Flex);

    // Despawning the stack leaves nothing that reacts to the selection.
    app.world_mut().entity_mut(stack).despawn();
    select_one(&mut app, source, items[0]);
    for _ in 0..3 {
        app.update();
    }
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

/// Presses `key_code` once with `list` focused, as a keyboard would.
fn press_key_on(app: &mut App, key_code: KeyCode) {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};

    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    app.update();
}

/// A focused `VirtualList` of three rows over a selection built by `selection`, with the
/// first row active, and the rows' items.
fn focused_virtual_list(
    selection: impl FnOnce(SelectionBuilder) -> SelectionBuilderWithValue,
) -> (App, Entity, Vec<Entity>) {
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let root = app.world_mut().spawn(Node::default()).id();
    let list = app
        .world_mut()
        .spawn((
            VirtualList::new(RowHeight::new(28.0).unwrap()),
            selection(Selection::builder()).build_with_items(Vec::<Item<String>>::new()),
            ChildOf(root),
        ))
        .id();
    let items: Vec<Entity> = (1..=3)
        .map(|index| {
            app.world_mut()
                .spawn((Item::new(format!("Row {index}")), ItemOf(list)))
                .id()
        })
        .collect();
    for _ in 0..3 {
        app.update();
    }
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(list, bevy::input_focus::FocusCause::Navigated);
    app.update();
    (app, list, items)
}

/// The items the selection of `list` holds.
fn selected_items(app: &App, list: Entity) -> Vec<Entity> {
    match app.world().get::<Selection>(list) {
        Some(Selection::Value(value)) => value.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
fn enter_or_space_on_a_selected_virtual_list_row_deselects_it_as_far_as_the_selection_allows() {
    for key in [KeyCode::Enter, KeyCode::Space] {
        let (mut app, list, items) = focused_virtual_list(SelectionBuilder::zero_or_one);
        press_key_on(&mut app, key);
        assert_eq!(
            selected_items(&app, list),
            vec![items[0]],
            "{key:?} selects"
        );
        press_key_on(&mut app, key);
        assert_eq!(
            selected_items(&app, list),
            Vec::<Entity>::new(),
            "{key:?} again"
        );

        let (mut app, list, items) = focused_virtual_list(SelectionBuilder::zero_or_more);
        press_key_on(&mut app, key);
        press_key_on(&mut app, KeyCode::ArrowDown);
        press_key_on(&mut app, key);
        assert_eq!(selected_items(&app, list), vec![items[0], items[1]]);
        press_key_on(&mut app, key);
        assert_eq!(
            selected_items(&app, list),
            vec![items[0]],
            "only the active row goes"
        );

        let (mut app, list, items) = focused_virtual_list(SelectionBuilder::one);
        press_key_on(&mut app, key);
        press_key_on(&mut app, key);
        assert_eq!(
            selected_items(&app, list),
            vec![items[0]],
            "a `One` keeps its item"
        );
    }
}

/// The pool row of `list` that shows `item`.
fn virtual_row_of(app: &mut App, item: Entity) -> Entity {
    let mut rows = app.world_mut().query::<(Entity, &VirtualListRow)>();
    rows.iter(app.world())
        .find(|(_, row)| row.shown.is_some_and(|shown| shown.item == item))
        .map(|(row, _)| row)
        .unwrap()
}

fn click_with(app: &mut App, target: Entity, button: PointerButton, pointer: PointerId) {
    app.world_mut().trigger(PointerClick {
        entity: target,
        pointer: Pointer::new(pointer, pointer_location()),
        count: 1,
        button,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        duration: std::time::Duration::ZERO,
    });
    app.update();
}

#[test]
fn only_a_primary_click_or_touch_picks_a_virtual_list_row() {
    let (mut app, list, items) = focused_virtual_list(SelectionBuilder::zero_or_more);
    let row = virtual_row_of(&mut app, items[1]);
    for button in [PointerButton::Secondary, PointerButton::Middle] {
        click_with(&mut app, row, button, PointerId::Mouse);
        assert!(
            selected_items(&app, list).is_empty(),
            "{button:?} picks nothing"
        );
    }
    click_with(&mut app, row, PointerButton::Primary, PointerId::Touch(1));
    assert_eq!(selected_items(&app, list), vec![items[1]], "a touch picks");
    click_with(&mut app, row, PointerButton::Secondary, PointerId::Mouse);
    assert_eq!(selected_items(&app, list), vec![items[1]], "no deselect");
    click_with(&mut app, row, PointerButton::Primary, PointerId::Mouse);
    assert!(
        selected_items(&app, list).is_empty(),
        "a primary click toggles"
    );
}

/// What the events of a [`TextField`] and the keys that bubbled past it came to.
#[derive(Resource, Default)]
struct TextFieldLog {
    changes: Vec<String>,
    commits: Vec<String>,
    bubbled_keys: Vec<KeyCode>,
}

/// An app with a primary window, a root and one [`TextField`] holding `value`, which logs its
/// events into [`TextFieldLog`]. The root counts the keys that reach it.
fn text_field_app(value: &str) -> (App, Entity, Entity, Entity) {
    let mut app = headless_app_with(|app| {
        app.add_plugins((bevy::text::TextPlugin, bevy::ui_widgets::TextInputPlugin));
    });
    app.init_resource::<TextFieldLog>();
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.world_mut().spawn(Camera2d);
    let root = app
        .world_mut()
        .spawn(Node::default())
        .observe(
            |key: On<bevy::input_focus::FocusedInput<bevy::input::keyboard::KeyboardInput>>,
             mut log: ResMut<TextFieldLog>| {
                log.bubbled_keys.push(key.event().input.key_code);
            },
        )
        .id();
    let field = app
        .world_mut()
        .spawn((TextField::new(value), ChildOf(root)))
        .observe(
            |change: On<TextFieldChange>, mut log: ResMut<TextFieldLog>| {
                log.changes.push(change.value.clone());
            },
        )
        .observe(
            |commit: On<TextFieldCommit>, mut log: ResMut<TextFieldLog>| {
                log.commits.push(commit.value.clone());
            },
        )
        .id();
    for _ in 0..3 {
        app.update();
    }
    let input = app
        .world()
        .entity(field)
        .get::<TextFieldRefs>()
        .unwrap()
        .input;
    (app, window, field, input)
}

fn input_text(app: &App, input: Entity) -> String {
    app.world()
        .entity(input)
        .get::<bevy::text::EditableText>()
        .unwrap()
        .value()
        .to_string()
}

fn focus_text_input(app: &mut App, input: Entity) {
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(input, bevy::input_focus::FocusCause::Navigated);
    for _ in 0..2 {
        app.update();
    }
}

fn press_text_key(
    app: &mut App,
    window: Entity,
    key_code: KeyCode,
    logical_key: bevy::input::keyboard::Key,
) {
    app.world_mut()
        .write_message(bevy::input::keyboard::KeyboardInput {
            key_code,
            logical_key,
            state: bevy::input::ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
    for _ in 0..3 {
        app.update();
    }
}

#[test]
fn inserting_a_new_text_field_sets_the_text_without_a_change_event() {
    let (mut app, _, field, input) = text_field_app("alpha");
    assert_eq!(input_text(&app, input), "alpha");
    app.world_mut()
        .entity_mut(field)
        .insert(TextField::new("beta").with_placeholder("Name"));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(input_text(&app, input), "beta");
    let log = app.world().get_resource::<TextFieldLog>().unwrap();
    assert!(log.changes.is_empty(), "set by the app: {:?}", log.changes);
    assert!(log.commits.is_empty());
}

#[test]
fn enter_and_numpad_enter_commit_the_text_and_stop_there() {
    use bevy::input::keyboard::Key;
    for (key_code, key) in [
        (KeyCode::Enter, Key::Enter),
        (KeyCode::NumpadEnter, Key::Enter),
    ] {
        let (mut app, window, _, input) = text_field_app("alpha");
        focus_text_input(&mut app, input);
        press_text_key(&mut app, window, key_code, key);
        let log = app.world().get_resource::<TextFieldLog>().unwrap();
        assert_eq!(log.commits, vec!["alpha"], "{key_code:?} commits");
        assert!(log.bubbled_keys.is_empty(), "{key_code:?} stops");
        assert_eq!(
            app.world().get_resource::<InputFocus>().unwrap().get(),
            None
        );
    }
}

#[test]
fn tab_commits_the_text_even_when_it_did_not_change() {
    use bevy::input::keyboard::Key;
    let (mut app, window, field, input) = text_field_app("alpha");
    app.world_mut()
        .spawn((Node::default(), Button, TabIndex(1), ChildOf(field)));
    focus_text_input(&mut app, input);
    press_text_key(&mut app, window, KeyCode::Tab, Key::Tab);
    let log = app.world().get_resource::<TextFieldLog>().unwrap();
    assert_eq!(log.commits, vec!["alpha"]);
    assert!(log.changes.is_empty());
    assert_ne!(
        app.world().get_resource::<InputFocus>().unwrap().get(),
        Some(input)
    );
}

#[test]
fn escape_puts_the_text_back_without_a_commit_and_stops_like_enter() {
    use bevy::input::keyboard::Key;
    let (mut app, window, _, input) = text_field_app("alpha");
    focus_text_input(&mut app, input);
    app.world_mut()
        .entity_mut(input)
        .get_mut::<bevy::text::EditableText>()
        .unwrap()
        .queue_edit(bevy::text::TextEdit::Insert("!".into()));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(input_text(&app, input), "alpha!");
    assert_eq!(
        app.world().get_resource::<TextFieldLog>().unwrap().changes,
        vec!["alpha!"]
    );
    press_text_key(&mut app, window, KeyCode::Escape, Key::Escape);
    assert_eq!(input_text(&app, input), "alpha");
    let log = app.world().get_resource::<TextFieldLog>().unwrap();
    assert!(log.commits.is_empty(), "Escape does not commit");
    assert!(log.bubbled_keys.is_empty(), "Escape stops like Enter");
    assert_eq!(
        app.world().get_resource::<InputFocus>().unwrap().get(),
        None
    );
}

/// A `u8` stepper from 0 to 9 at `initial` in a text-capable app, with its value label.
fn scrub_stepper_app(initial: u8) -> (App, Entity, Entity, Entity) {
    let mut app = headless_app_with(|app| {
        app.add_plugins((bevy::text::TextPlugin, bevy::ui_widgets::TextInputPlugin));
    });
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.world_mut().spawn(Camera2d);
    let root = app.world_mut().spawn(Node::default()).id();
    let stepper = app
        .world_mut()
        .spawn((
            NumericStepper::default(),
            Value::new_u8(initial, 0, 9, 1).unwrap(),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let label = app
        .world()
        .entity(stepper)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|kid| app.world().entity(*kid).contains::<Text>())
        .unwrap();
    (app, window, stepper, label)
}

/// The input of the number editor of `stepper`, if one is open.
fn scrub_editor_input(app: &App, stepper: Entity) -> Option<Entity> {
    app.world()
        .entity(stepper)
        .get::<Children>()?
        .iter()
        .find_map(|kid| {
            app.world()
                .entity(kid)
                .get::<TextFieldRefs>()
                .map(|refs| refs.input)
        })
}

/// Opens the number editor with a click on the value and types `text` into it.
fn open_scrub_editor_and_type(app: &mut App, stepper: Entity, label: Entity, text: &str) -> Entity {
    click_and_release(app, label);
    for _ in 0..3 {
        app.update();
    }
    let input = scrub_editor_input(app, stepper);
    assert!(input.is_some(), "a click on the value opens the editor");
    let input = input.unwrap();
    focus_text_input(app, input);
    app.world_mut()
        .entity_mut(input)
        .get_mut::<bevy::text::EditableText>()
        .unwrap()
        .queue_edit(bevy::text::TextEdit::Insert(text.into()));
    for _ in 0..3 {
        app.update();
    }
    input
}

#[test]
fn escape_in_the_number_editor_closes_it_and_keeps_the_value() {
    use bevy::input::keyboard::Key;
    let (mut app, window, stepper, label) = scrub_stepper_app(3);
    open_scrub_editor_and_type(&mut app, stepper, label, "7");
    press_text_key(&mut app, window, KeyCode::Escape, Key::Escape);
    app.update();
    assert_eq!(stepper_value(&app, stepper), 3);
    assert!(scrub_editor_input(&app, stepper).is_none());
    assert_ne!(
        app.world().entity(label).get::<Node>().unwrap().display,
        Display::None,
        "the value shows again"
    );
}

#[test]
fn enter_in_the_number_editor_sets_the_typed_number_and_closes_it() {
    use bevy::input::keyboard::Key;
    let (mut app, window, stepper, label) = scrub_stepper_app(3);
    open_scrub_editor_and_type(&mut app, stepper, label, "7");
    press_text_key(&mut app, window, KeyCode::Enter, Key::Enter);
    app.update();
    assert_eq!(stepper_value(&app, stepper), 7);
    assert!(scrub_editor_input(&app, stepper).is_none());
}

#[test]
fn text_that_is_not_a_number_changes_nothing_and_closes_the_editor() {
    use bevy::input::keyboard::Key;
    let (mut app, window, stepper, label) = scrub_stepper_app(3);
    open_scrub_editor_and_type(&mut app, stepper, label, "abc");
    press_text_key(&mut app, window, KeyCode::Enter, Key::Enter);
    app.update();
    assert_eq!(stepper_value(&app, stepper), 3);
    assert!(scrub_editor_input(&app, stepper).is_none());
}

/// Gives `stepper` a laid-out width of 100 logical pixels, which a scrub measures against.
fn give_width(app: &mut App, stepper: Entity) {
    app.world_mut().entity_mut(stepper).insert(ComputedNode {
        size: Vec2::new(100.0, 20.0),
        inverse_scale_factor: 1.0,
        ..default()
    });
}

/// A press of `button` on `target` followed by a drag of `dx` logical pixels.
fn press_and_drag(app: &mut App, target: Entity, button: PointerButton, dx: f32) {
    app.world_mut().trigger(PointerPress {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        count: 1,
        button,
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
    });
    app.world_mut().trigger(PointerDrag {
        entity: target,
        pointer: Pointer::new(PointerId::Mouse, pointer_location()),
        button,
        distance: Vec2::new(dx, 0.0),
        delta: Vec2::new(dx, 0.0),
    });
    app.update();
}

/// The width of the `ScrubFill` under `stepper`, as a percentage.
fn fill_percent(app: &App, stepper: Entity) -> f32 {
    let track = app.world().entity(stepper).get::<Children>().unwrap()[0];
    let fill = app
        .world()
        .entity(track)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|kid| {
            app.world()
                .entity(*kid)
                .get::<Name>()
                .is_some_and(|name| name.as_str() == "ScrubFill")
        })
        .unwrap();
    match app.world().entity(fill).get::<Node>().unwrap().width {
        Val::Percent(percent) => percent,
        _ => f32::NAN,
    }
}

#[test]
fn a_primary_drag_scrubs_but_a_right_button_drag_does_not() {
    let (mut app, _, stepper, label) = scrub_stepper_app(0);
    give_width(&mut app, stepper);
    press_and_drag(&mut app, label, PointerButton::Secondary, 100.0);
    assert_eq!(stepper_value(&app, stepper), 0);
    press_and_drag(&mut app, label, PointerButton::Primary, 100.0);
    assert_eq!(stepper_value(&app, stepper), 9);
}

#[test]
fn scrubbing_a_mirror_stepper_changes_the_shared_value() {
    let (mut app, _, shared, _) = scrub_stepper_app(0);
    let mirror = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(shared)))
        .id();
    app.update();
    app.update();
    give_width(&mut app, mirror);
    press_and_drag(&mut app, mirror, PointerButton::Primary, 100.0);
    assert_eq!(stepper_value(&app, shared), 9);
    assert!(app.world().entity(mirror).get::<Value>().is_none());
}

#[test]
fn the_scrub_fill_follows_the_value() {
    let (mut app, _, stepper, _) = scrub_stepper_app(0);
    assert!(fill_percent(&app, stepper).abs() < 0.01);
    *app.world_mut()
        .entity_mut(stepper)
        .get_mut::<Value>()
        .unwrap() = Value::new_u8(9, 0, 9, 1).unwrap();
    app.update();
    app.update();
    assert!((fill_percent(&app, stepper) - 100.0).abs() < 0.01);
    *app.world_mut()
        .entity_mut(stepper)
        .get_mut::<Value>()
        .unwrap() = Value::new_u8(3, 0, 6, 1).unwrap();
    app.update();
    app.update();
    assert!((fill_percent(&app, stepper) - 50.0).abs() < 0.01);
}

#[test]
fn a_drag_does_not_swallow_the_next_activate_once_all_pointers_are_up() {
    let (mut app, _, stepper, _) = scrub_stepper_app(0);
    give_width(&mut app, stepper);
    let increase = app.world().entity(stepper).get::<Children>().unwrap()[3];
    press_and_drag(&mut app, increase, PointerButton::Primary, 10.0);
    app.update();
    let before = stepper_value(&app, stepper);
    activate(&mut app, increase);
    assert_eq!(stepper_value(&app, stepper), before + 1);
}

#[test]
fn a_typed_number_above_the_maximum_is_clamped() {
    use bevy::input::keyboard::Key;
    let (mut app, window, stepper, label) = scrub_stepper_app(3);
    open_scrub_editor_and_type(&mut app, stepper, label, "99");
    press_text_key(&mut app, window, KeyCode::Enter, Key::Enter);
    app.update();
    assert_eq!(stepper_value(&app, stepper), 9);
}

#[test]
fn a_slider_gives_the_label_margin_back_when_inserted_again_without_a_format() {
    let mut app = headless_app();
    let world = app.world_mut();
    world.spawn((Camera2d, UiRoot));
    let slider = world
        .spawn(
            Slider::new(1.0, 0.0, 10.0)
                .unwrap()
                .with_format(|v| format!("{v}")),
        )
        .id();
    app.update();
    app.update();
    let margin = |app: &App| {
        app.world()
            .entity(slider)
            .get::<Node>()
            .unwrap()
            .margin
            .right
    };
    assert!(matches!(margin(&app), Val::Px(px) if px > 0.0));
    app.world_mut()
        .entity_mut(slider)
        .insert(Slider::new(1.0, 0.0, 10.0).unwrap());
    app.update();
    app.update();
    assert_eq!(margin(&app), Val::Px(0.0));
}

/// A scroll-contents node for `scroll_into_view`: 100 by 100 logical pixels at the window's top
/// left, its content 400 by 400, scrolling along the axes given, drawn at `scale`. The headless
/// app has no layout, so the sizes and places are set by hand.
fn scrolling_contents(
    app: &mut App,
    parent: Option<Entity>,
    overflow: Overflow,
    scroll: Vec2,
    scale: f32,
) -> Entity {
    let mut area = app.world_mut().spawn((
        Node {
            overflow,
            ..default()
        },
        ScrollContents,
        ScrollPosition(scroll),
        ComputedNode {
            size: Vec2::splat(100.0 * scale),
            content_size: Vec2::splat(400.0 * scale),
            inverse_scale_factor: 1.0 / scale,
            ..default()
        },
        UiGlobalTransform::from_translation(Vec2::splat(50.0 * scale)),
    ));
    if let Some(parent) = parent {
        area.insert(ChildOf(parent));
    }
    area.id()
}

/// A 20 by 20 logical pixel target in `parent`, whose top left the layout last put at `visible`
/// logical pixels from the window's top left (already shifted by the scroll position).
fn target_at(app: &mut App, parent: Option<Entity>, visible: Vec2, scale: f32) -> Entity {
    let mut target = app.world_mut().spawn((
        Node::default(),
        ComputedNode {
            size: Vec2::splat(20.0 * scale),
            inverse_scale_factor: 1.0 / scale,
            ..default()
        },
        UiGlobalTransform::from_translation((visible + Vec2::splat(10.0)) * scale),
    ));
    if let Some(parent) = parent {
        target.insert(ChildOf(parent));
    }
    target.id()
}

fn scroll_to(app: &mut App, entity: Entity) {
    app.world_mut().trigger(ScrollIntoView { entity });
    tick(app);
}

fn scroll_of(app: &App, area: Entity) -> Vec2 {
    app.world().get::<ScrollPosition>(area).unwrap().0
}

const SCROLL_BOTH: Overflow = Overflow::scroll();
const SCROLL_X: Overflow = Overflow {
    x: OverflowAxis::Scroll,
    y: OverflowAxis::Visible,
};

#[test]
fn a_horizontal_area_scrolls_a_target_sideways_and_leaves_the_vertical_axis_alone() {
    // Past the right and the bottom edge: only x moves, as y does not scroll.
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, SCROLL_X, Vec2::ZERO, 1.0);
    let target = target_at(&mut app, Some(area), Vec2::new(150.0, 150.0), 1.0);
    scroll_to(&mut app, target);
    // Right edge at 170 plus the 3 px ring, in a 100 px view.
    assert_eq!(scroll_of(&app, area), Vec2::new(73.0, 0.0));
}

#[test]
fn a_vertical_area_leaves_the_horizontal_axis_alone() {
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, Overflow::scroll_y(), Vec2::ZERO, 1.0);
    let target = target_at(&mut app, Some(area), Vec2::new(150.0, 150.0), 1.0);
    scroll_to(&mut app, target);
    assert_eq!(scroll_of(&app, area), Vec2::new(0.0, 73.0));
}

#[test]
fn a_two_way_area_scrolls_both_axes_to_show_a_target_with_its_ring() {
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, 1.0);
    let target = target_at(&mut app, Some(area), Vec2::new(150.0, 150.0), 1.0);
    scroll_to(&mut app, target);
    // The target ends 3 px from the view's right and bottom edges, not touching them.
    assert_eq!(scroll_of(&app, area), Vec2::new(73.0, 73.0));
}

#[test]
fn a_target_scrolled_in_from_before_the_start_clamps_to_zero_not_below() {
    // The target lies 1 px into the contents, so with its ring it starts at -2.
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::splat(200.0), 1.0);
    let target = target_at(&mut app, Some(area), Vec2::splat(-199.0), 1.0);
    scroll_to(&mut app, target);
    assert_eq!(scroll_of(&app, area), Vec2::ZERO);
}

#[test]
fn a_target_before_the_view_scrolls_back_to_show_its_ring() {
    // The target is at 50 in the contents, the view starts at 200: the ring starts at 47.
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::splat(200.0), 1.0);
    let target = target_at(&mut app, Some(area), Vec2::splat(-150.0), 1.0);
    scroll_to(&mut app, target);
    assert_eq!(scroll_of(&app, area), Vec2::splat(47.0));
}

#[test]
fn a_target_already_in_view_does_not_move_the_area() {
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::splat(30.0), 1.0);
    let target = target_at(&mut app, Some(area), Vec2::splat(40.0), 1.0);
    scroll_to(&mut app, target);
    assert_eq!(scroll_of(&app, area), Vec2::splat(30.0));
}

#[test]
fn a_horizontal_area_scrolls_the_same_logical_distance_at_scale_factor_two() {
    let mut app = headless_app();
    let area = scrolling_contents(&mut app, None, SCROLL_X, Vec2::ZERO, 2.0);
    let target = target_at(&mut app, Some(area), Vec2::new(150.0, 150.0), 2.0);
    scroll_to(&mut app, target);
    assert_eq!(scroll_of(&app, area), Vec2::new(73.0, 0.0));
    // And back to the start, clamped to 0.
    let back = target_at(&mut app, Some(area), Vec2::new(-72.0, 0.0), 2.0);
    scroll_to(&mut app, back);
    assert_eq!(scroll_of(&app, area), Vec2::ZERO);
}

#[test]
fn a_target_outside_any_scroll_area_is_left_alone_without_a_panic() {
    let mut app = headless_app();
    // No parent at all.
    let loose = target_at(&mut app, None, Vec2::splat(500.0), 1.0);
    scroll_to(&mut app, loose);
    // A parent that scrolls but is not a `ScrollContents`.
    let parent = app
        .world_mut()
        .spawn((
            Node {
                overflow: SCROLL_BOTH,
                ..default()
            },
            ScrollPosition(Vec2::splat(7.0)),
        ))
        .id();
    let child = target_at(&mut app, Some(parent), Vec2::splat(500.0), 1.0);
    scroll_to(&mut app, child);
    assert_eq!(scroll_of(&app, parent), Vec2::splat(7.0));
}

#[test]
fn only_the_innermost_scroll_area_takes_a_scroll_into_view() {
    let mut app = headless_app();
    let outer = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, 1.0);
    let inner = scrolling_contents(&mut app, Some(outer), SCROLL_BOTH, Vec2::ZERO, 1.0);
    let target = target_at(&mut app, Some(inner), Vec2::new(150.0, 150.0), 1.0);
    scroll_to(&mut app, target);
    assert_eq!(scroll_of(&app, inner), Vec2::splat(73.0));
    assert_eq!(scroll_of(&app, outer), Vec2::ZERO);
}

#[test]
fn a_scroll_into_view_that_propagates_stops_at_the_innermost_scroll_area() {
    // `World::trigger` runs the observers of the first hop only, so it cannot tell whether the
    // observer stops the event. Trigger it with propagation switched on, so it climbs from
    // the target's area to the outer one unless `scroll_into_view` stops it.
    let mut app = headless_app();
    let outer = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, 1.0);
    let inner = scrolling_contents(&mut app, Some(outer), SCROLL_BOTH, Vec2::ZERO, 1.0);
    // The inner area lies 300 px down and right in the outer one, outside its 100 px view, so
    // an outer area that took the event would scroll to show it.
    app.world_mut()
        .entity_mut(inner)
        .insert(UiGlobalTransform::from_translation(Vec2::splat(350.0)));
    let target = target_at(&mut app, Some(inner), Vec2::splat(450.0), 1.0);
    let mut trigger = PropagateEntityTrigger::<false, ScrollIntoView, &ChildOf>::default();
    trigger.propagate = true;
    app.world_mut()
        .trigger_with(ScrollIntoView { entity: target }, trigger);
    tick(&mut app);
    assert_eq!(scroll_of(&app, inner), Vec2::splat(73.0));
    assert_eq!(scroll_of(&app, outer), Vec2::ZERO);
}

#[test]
fn a_target_at_the_end_of_the_contents_stops_at_the_last_scroll_position_at_any_scale() {
    // The target ends at 400, the contents' end; with its ring it would want 303, but 300 is the
    // most the 400 px contents scroll in a 100 px view. Physical sizes double at scale 2.
    for scale in [1.0, 2.0] {
        let mut app = headless_app();
        let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, scale);
        let target = target_at(&mut app, Some(area), Vec2::splat(380.0), scale);
        scroll_to(&mut app, target);
        assert_eq!(scroll_of(&app, area), Vec2::splat(300.0), "scale {scale}");
    }
}

/// The accessible value of `entity`.
fn accessible_value(app: &App, entity: Entity) -> Option<String> {
    app.world()
        .get::<AccessibilityNode>(entity)
        .unwrap()
        .value()
        .map(str::to_string)
}

/// The text of a combo button's label.
fn combo_label(app: &App, button: Entity) -> String {
    let label = app.world().get::<PushButtonRefs>(button).unwrap().label();
    app.world().get::<Text>(label).unwrap().0.clone()
}

#[test]
fn a_combo_shows_select_dots_by_default_while_nothing_is_selected() {
    let mut app = headless_app();
    let (_, button) = spawn_combo(&mut app);
    assert_eq!(combo_label(&app, button), "Select..");
}

#[test]
fn a_combo_shows_its_placeholder_while_nothing_is_selected_and_the_selection_after() {
    let mut app = headless_app();
    let combo = app
        .world_mut()
        .spawn((
            Combo,
            ComboPlaceholder("Pick a country".to_string()),
            Selection::builder()
                .zero_or_one()
                .build_with_items(["a", "b"].map(|s| Item::new(s.to_string()))),
        ))
        .id();
    app.update();
    app.update();
    let button = app.world().get::<ComboRefs>(combo).unwrap().button();
    assert_eq!(combo_label(&app, button), "Pick a country");
    assert_eq!(accessible_value(&app, button), Some(String::new()));

    // Select the first item through the open popup.
    click(&mut app, button);
    app.update();
    let list = app.world().get::<ComboRefs>(combo).unwrap().list();
    let item = app
        .world()
        .get::<Children>(list)
        .unwrap()
        .iter()
        .find(|child| app.world().get::<ItemView>(*child).is_some())
        .unwrap();
    click(&mut app, item);
    app.update();
    app.update();
    assert_eq!(combo_label(&app, button), "a");
}

#[test]
fn inserting_a_placeholder_changes_the_text_of_a_live_combo() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    app.world_mut()
        .entity_mut(combo)
        .insert(ComboPlaceholder("Nothing yet".to_string()));
    app.update();
    app.update();
    assert_eq!(combo_label(&app, button), "Nothing yet");
    assert_eq!(accessible_value(&app, button), Some(String::new()));
}

/// The `ScrollContents` of the area `two_axis_scroll_area` spawned, given a laid-out `size` and
/// `content_size` (headless apps have no layout).
fn lay_out_scroll_contents(app: &mut App, size: Vec2, content_size: Vec2) {
    let contents = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<ScrollContents>>()
            .single(world)
            .unwrap()
    };
    app.world_mut().entity_mut(contents).insert(ComputedNode {
        size,
        content_size,
        inverse_scale_factor: 1.0,
        ..default()
    });
}

fn bar_visibility(app: &App, bar: Entity) -> Visibility {
    *app.world().get::<Visibility>(bar).unwrap()
}

#[test]
fn changing_the_headless_orientation_shows_or_hides_the_bar_by_its_new_axis() {
    let mut app = headless_app();
    let (vertical, horizontal) = two_axis_scroll_area(&mut app);
    // Taller than the area, no wider than it.
    lay_out_scroll_contents(&mut app, Vec2::new(100.0, 100.0), Vec2::new(100.0, 300.0));
    app.update();
    app.update();
    assert_eq!(bar_visibility(&app, vertical), Visibility::Inherited);
    assert_eq!(bar_visibility(&app, horizontal), Visibility::Hidden);

    // Swap the axes of the two bars without the contents changing at all.
    app.world_mut()
        .get_mut::<Scrollbar>(vertical)
        .unwrap()
        .orientation = ControlOrientation::Horizontal;
    app.world_mut()
        .get_mut::<Scrollbar>(horizontal)
        .unwrap()
        .orientation = ControlOrientation::Vertical;
    app.update();
    app.update();

    assert_eq!(
        bar_visibility(&app, vertical),
        Visibility::Hidden,
        "now horizontal, and the contents fit across"
    );
    assert_eq!(
        bar_visibility(&app, horizontal),
        Visibility::Inherited,
        "now vertical, and the contents overflow down"
    );
}

#[derive(Component)]
struct AppBarChild;

#[test]
fn an_app_owned_child_of_a_scroll_bar_survives_theming_and_reinserting_the_marker() {
    let mut app = headless_app();
    let (vertical, _) = two_axis_scroll_area(&mut app);
    let node = Node {
        width: px(7),
        ..default()
    };
    let child = app
        .world_mut()
        .spawn((
            AppBarChild,
            node.clone(),
            BackgroundColor(Color::WHITE),
            ChildOf(vertical),
        ))
        .id();
    app.update();

    let unchanged = |app: &App| {
        let world = app.world();
        assert!(
            world
                .get::<Children>(vertical)
                .unwrap()
                .iter()
                .any(|c| c == child),
            "the child is still under the bar"
        );
        assert_eq!(world.get::<Node>(child).unwrap(), &node);
        assert_eq!(world.get::<BackgroundColor>(child).unwrap().0, Color::WHITE);
        assert!(
            world
                .get::<ChildOf>(child)
                .is_some_and(|c| c.parent() == vertical)
        );
    };

    // A theme change restyles the bar and its thumb, not the app's child.
    let mode = app.world().get_resource::<Theme>().unwrap().mode();
    let other = if mode == ThemeMode::Light {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .set_mode(other);
    app.update();
    app.update();
    unchanged(&app);
    assert_eq!(thumbs_of(&mut app, vertical), 1);

    // So does inserting the marker again.
    app.world_mut().entity_mut(vertical).insert(ScrollBar);
    app.update();
    app.update();
    unchanged(&app);
    assert_eq!(thumbs_of(&mut app, vertical), 1);
    assert_eq!(app.world().get::<Children>(vertical).unwrap().len(), 2);
}

#[test]
fn reinserting_a_numeric_stepper_updates_every_setting_without_duplicating_its_parts() {
    let mut app = headless_app();
    let (stepper, old_decrease, _) = spawn_stepper(&mut app, 0);
    let icon = Handle::<Image>::default();

    app.world_mut().entity_mut(stepper).insert(NumericStepper {
        format: level_text,
        decrease_icon: icon.clone(),
        increase_icon: icon.clone(),
        decrease_name: "Lower".into(),
        increase_name: "Raise".into(),
        label_min_width: 90.0,
        name: Some("Level".into()),
    });
    app.update();
    app.update();

    let children: Vec<Entity> = app
        .world()
        .entity(stepper)
        .get::<Children>()
        .unwrap()
        .iter()
        .collect();
    assert_eq!(children.len(), 4, "no duplicate buttons or label");
    assert!(
        app.world().get_entity(old_decrease).is_err(),
        "the old buttons are gone"
    );
    assert_eq!(
        app.world().get::<IconButton>(children[1]).unwrap().name,
        "Lower"
    );
    assert_eq!(
        app.world().get::<IconButton>(children[3]).unwrap().name,
        "Raise"
    );
    assert_eq!(
        app.world().get::<IconButton>(children[1]).unwrap().icon,
        icon
    );
    assert_eq!(
        app.world().get::<IconButton>(children[3]).unwrap().icon,
        icon
    );
    assert_eq!(app.world().get::<Text>(children[2]).unwrap().0, "Ground");
    assert_eq!(
        app.world().get::<Node>(children[2]).unwrap().min_width,
        Val::Px(90.0)
    );
    let node = app
        .world()
        .entity(stepper)
        .get::<AccessibilityNode>()
        .unwrap();
    assert_eq!(node.label(), Some("Level"));
    assert_eq!(node.value(), Some("Ground"));

    // The new buttons step the value once each, through one observer.
    activate(&mut app, children[3]);
    assert_eq!(stepper_value(&app, stepper), 1);
    assert_eq!(app.world().get::<Text>(children[2]).unwrap().0, "+1");
    assert!(!is_disabled(&app, children[1]));
    activate(&mut app, children[1]);
    assert_eq!(stepper_value(&app, stepper), 0);
}

#[test]
fn reinserting_a_mirror_numeric_stepper_keeps_its_value_ref() {
    let mut app = headless_app();
    let (shared, _, _) = spawn_stepper_of(&mut app, Value::new_u8(1, 0, 3, 1).unwrap());
    let mirror = app
        .world_mut()
        .spawn((NumericStepper::default(), ValueRef(shared)))
        .id();
    app.update();
    app.update();
    let old_parts: Vec<Entity> = app
        .world()
        .entity(mirror)
        .get::<Children>()
        .unwrap()
        .iter()
        .collect();

    app.world_mut().entity_mut(mirror).insert(NumericStepper {
        format: level_text,
        increase_name: "Raise".into(),
        ..default()
    });
    app.update();
    app.update();

    for old in old_parts {
        assert!(app.world().get_entity(old).is_err(), "no orphaned part");
    }
    assert_eq!(
        app.world().get::<ValueRef>(mirror).map(|v| v.0),
        Some(shared)
    );
    let children: Vec<Entity> = app
        .world()
        .entity(mirror)
        .get::<Children>()
        .unwrap()
        .iter()
        .collect();
    assert_eq!(children.len(), 4);
    assert_eq!(app.world().get::<Text>(children[2]).unwrap().0, "+1");
    activate(&mut app, children[3]);
    assert_eq!(stepper_value(&app, shared), 2);
    assert_eq!(app.world().get::<Text>(children[2]).unwrap().0, "+2");
}

#[test]
fn reinserting_a_slider_updates_its_range_format_name_and_value() {
    let mut app = headless_app();
    app.world_mut().spawn((Camera2d, UiRoot));
    let slider = app
        .world_mut()
        .spawn(
            Slider::new(2.0, 0.0, 10.0)
                .unwrap()
                .with_step(1.0)
                .unwrap()
                .with_format(|value| format!("{value:.0} km"))
                .with_name("Distance"),
        )
        .id();
    app.update();
    app.update();
    let parts_before = app.world().get::<SliderRefs>(slider).unwrap().label;

    app.world_mut().entity_mut(slider).insert(
        Slider::new(50.0, 0.0, 100.0)
            .unwrap()
            .with_step(10.0)
            .unwrap()
            .with_format(|value| format!("{value:.0} %"))
            .with_name("Speed"),
    );
    app.update();
    app.update();

    let world = app.world();
    assert_eq!(world.get::<SliderValue>(slider), Some(&SliderValue(50.0)));
    assert_eq!(
        world.get::<SliderRange>(slider),
        Some(&SliderRange::new(0.0, 100.0))
    );
    assert_eq!(world.get::<SliderStep>(slider), Some(&SliderStep(10.0)));
    let node = world.get::<AccessibilityNode>(slider).unwrap();
    assert_eq!(node.label(), Some("Speed"));
    assert_eq!(node.value(), Some("50 %"));
    assert_eq!(node.max_numeric_value(), Some(100.0));
    let parts_after = world.get::<SliderRefs>(slider).unwrap().label;
    assert_ne!(parts_before, parts_after, "the parts are rebuilt");
    if let Some(old) = parts_before {
        assert!(world.get_entity(old).is_err(), "no stale label");
    }
}

#[test]
fn a_toast_with_no_parent_is_logged_and_despawned() {
    use bevy::log::tracing::{self, Level};
    use bevy::log::tracing_subscriber::{self, layer::SubscriberExt};
    use bevy_pluim::toast::Toast;
    use std::sync::{Arc, Mutex};

    struct Errors(Arc<Mutex<usize>>);
    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Errors {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if *event.metadata().level() == Level::ERROR
                && let Ok(mut count) = self.0.lock()
            {
                *count += 1;
            }
        }
    }

    let count = Arc::new(Mutex::new(0));
    let subscriber = tracing_subscriber::registry().with(Errors(count.clone()));
    let mut app = headless_app();
    // The subscriber is thread-local, so the systems must run on this thread.
    app.edit_schedule(Update, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });
    let toast = app.world_mut().spawn(Toast::new("Orphan")).id();
    tracing::subscriber::with_default(subscriber, || {
        for _ in 0..3 {
            app.update();
        }
    });
    assert!(
        app.world().get_entity(toast).is_err(),
        "a parentless toast stayed"
    );
    assert_eq!(*count.lock().unwrap(), 1, "one error should be logged");
}

#[test]
fn two_roots_each_get_their_own_toast_stack() {
    use bevy_pluim::toast::{Toast, ToastStack};

    let mut app = headless_app();
    let first = app.world_mut().spawn(Node::default()).id();
    let second = app.world_mut().spawn(Node::default()).id();
    let a = app
        .world_mut()
        .spawn((Toast::new("Saved"), ChildOf(first)))
        .id();
    app.update();
    // The second root's toast has the same text and must not join (or be
    // dropped for) the first root's stack.
    let b = app
        .world_mut()
        .spawn((Toast::new("Saved"), ChildOf(second)))
        .id();
    app.update();
    app.update();
    let parent_of = |app: &App, entity: Entity| {
        let stack = app.world().get::<ChildOf>(entity)?.parent();
        let root = app.world().get::<ChildOf>(stack)?.parent();
        app.world().get::<ToastStack>(stack)?;
        Some(root)
    };
    assert_eq!(parent_of(&app, a), Some(first));
    assert_eq!(parent_of(&app, b), Some(second));
}

#[test]
fn same_frame_duplicate_toasts_restart_the_kept_one_with_the_latest_duration() {
    use bevy::time::TimeUpdateStrategy;
    use bevy_pluim::toast::Toast;
    use std::time::Duration;

    let mut app = headless_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        250,
    )));
    let first_root = app.world_mut().spawn(Node::default()).id();
    let second_root = app.world_mut().spawn(Node::default()).id();
    let spawn = |app: &mut App, root: Entity, duration: f32| {
        app.world_mut()
            .spawn((Toast::new("Saved").with_duration(duration), ChildOf(root)))
            .id()
    };
    // One update: three duplicates under one parent, one under another.
    let kept = spawn(&mut app, first_root, 0.5);
    let dup_a = spawn(&mut app, first_root, 1.0);
    let dup_b = spawn(&mut app, first_root, 3.0);
    let other = spawn(&mut app, second_root, 0.5);
    app.update();
    app.update();
    assert!(
        app.world().get_entity(kept).is_ok(),
        "the first was dropped"
    );
    assert!(app.world().get_entity(dup_a).is_err(), "a duplicate stayed");
    assert!(app.world().get_entity(dup_b).is_err(), "a duplicate stayed");
    assert!(
        app.world().get_entity(other).is_ok(),
        "another parent's toast was dropped"
    );
    // After about 2 s the kept toast (restarted to 3 s by the last duplicate)
    // lives, and the other parent's 0.5 s toast is gone.
    for _ in 0..7 {
        app.update();
    }
    assert!(
        app.world().get_entity(kept).is_ok(),
        "the kept toast did not take the latest duration"
    );
    assert!(
        app.world().get_entity(other).is_err(),
        "the other parent's toast took a duration of another parent's"
    );
    for _ in 0..8 {
        app.update();
    }
    assert!(
        app.world().get_entity(kept).is_err(),
        "the kept toast stayed"
    );
}

/// A headless app with a root node, for the toast tests below.
fn toast_app() -> (App, Entity) {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    (app, root)
}

fn toast_stacks(app: &mut App) -> Vec<Entity> {
    use bevy_pluim::toast::ToastStack;

    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<ToastStack>>()
        .iter(world)
        .collect()
}

#[test]
fn the_toast_stack_sits_at_the_placement_bottom_and_follows_a_change_of_it() {
    use bevy_pluim::toast::{Toast, ToastPlacement};

    let (mut app, root) = toast_app();
    assert_eq!(
        app.world().get_resource::<ToastPlacement>().unwrap().bottom,
        Val::Px(24.0),
        "24 px by default"
    );
    app.world_mut().spawn((Toast::new("Saved"), ChildOf(root)));
    for _ in 0..3 {
        app.update();
    }
    let stack = toast_stacks(&mut app)[0];
    let bottom = |app: &App| app.world().get::<Node>(stack).unwrap().bottom;
    assert_eq!(bottom(&app), Val::Px(24.0));

    app.world_mut()
        .get_resource_mut::<ToastPlacement>()
        .unwrap()
        .bottom = Val::Px(90.0);
    app.update();
    assert_eq!(bottom(&app), Val::Px(90.0), "the existing stack moved");

    // A stack made later starts at the new distance.
    let other = app.world_mut().spawn(Node::default()).id();
    app.world_mut().spawn((Toast::new("Other"), ChildOf(other)));
    for _ in 0..3 {
        app.update();
    }
    let later = toast_stacks(&mut app)
        .into_iter()
        .find(|s| *s != stack)
        .unwrap();
    assert_eq!(
        app.world().get::<Node>(later).unwrap().bottom,
        Val::Px(90.0)
    );
}

#[test]
fn a_stack_made_after_idle_frames_gets_a_placement_set_before_it_and_left_unchanged() {
    use bevy_pluim::toast::{Toast, ToastPlacement};

    let (mut app, root) = toast_app();
    app.world_mut()
        .get_resource_mut::<ToastPlacement>()
        .unwrap()
        .bottom = Val::Px(70.0);
    // Idle frames: no toast, no stack, placement unchanged.
    for _ in 0..5 {
        app.update();
    }
    assert!(
        toast_stacks(&mut app).is_empty(),
        "idle frames made a stack"
    );
    app.world_mut().spawn((Toast::new("Late"), ChildOf(root)));
    for _ in 0..3 {
        app.update();
    }
    let stacks = toast_stacks(&mut app);
    assert_eq!(stacks.len(), 1);
    assert_eq!(
        app.world().get::<Node>(stacks[0]).unwrap().bottom,
        Val::Px(70.0),
        "a late stack missed the placement that did not change after it"
    );
}

#[test]
fn bursts_of_toasts_under_many_parents_keep_one_stack_each_across_idle_frames() {
    use bevy_pluim::toast::Toast;

    let (mut app, _) = toast_app();
    let parents: Vec<Entity> = (0..20)
        .map(|_| app.world_mut().spawn(Node::default()).id())
        .collect();
    let burst = |app: &mut App, text: &str| -> Vec<Entity> {
        parents
            .iter()
            .flat_map(|parent| {
                // Two same-text toasts per parent: the second is a duplicate.
                [0, 1].map(|_| {
                    app.world_mut()
                        .spawn((Toast::new(text).with_duration(1000.0), ChildOf(*parent)))
                        .id()
                })
            })
            .collect()
    };
    let first = burst(&mut app, "A");
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(toast_stacks(&mut app).len(), 20, "one stack per parent");
    let alive = |app: &App, toasts: &[Entity]| {
        toasts
            .iter()
            .filter(|toast| app.world().get_entity(**toast).is_ok())
            .count()
    };
    assert_eq!(alive(&app, &first), 20, "duplicates are dropped per parent");
    // Idle frames, then a second burst with the same and with a new text.
    for _ in 0..10 {
        app.update();
    }
    let again = burst(&mut app, "A");
    let other = burst(&mut app, "B");
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        alive(&app, &again),
        0,
        "text still showing restarts, not stacks"
    );
    assert_eq!(alive(&app, &other), 20);
    let stacks = toast_stacks(&mut app);
    assert_eq!(stacks.len(), 20, "the stacks were reused, none made again");
    for stack in stacks {
        let toasts = app.world().get::<Children>(stack).unwrap().len();
        assert_eq!(toasts, 2, "A and B in every stack");
    }
}

#[test]
fn reinserting_a_shown_toast_after_idle_frames_updates_it_in_its_stack() {
    use bevy_pluim::toast::Toast;

    let (mut app, root) = toast_app();
    let toast = app
        .world_mut()
        .spawn((Toast::new("Before").with_duration(1000.0), ChildOf(root)))
        .id();
    for _ in 0..10 {
        app.update();
    }
    let stack = app.world().get::<ChildOf>(toast).unwrap().parent();
    app.world_mut()
        .entity_mut(toast)
        .insert(Toast::new("After").with_duration(1000.0));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(app.world().get::<Text>(toast).unwrap().0, "After");
    assert_eq!(app.world().get::<ChildOf>(toast).unwrap().parent(), stack);
    assert_eq!(toast_stacks(&mut app), vec![stack], "no second stack");
}

#[test]
fn a_toast_after_the_first_has_gone_reuses_the_stack_that_stayed_when_empty() {
    use bevy_pluim::toast::Toast;

    let (mut app, root) = toast_app();
    let first = app
        .world_mut()
        .spawn((Toast::new("One").with_duration(0.0), ChildOf(root)))
        .id();
    for _ in 0..4 {
        app.update();
    }
    assert!(
        app.world().get_entity(first).is_err(),
        "the first toast went"
    );
    let stacks = toast_stacks(&mut app);
    assert_eq!(stacks.len(), 1, "the empty stack stays");
    assert!(
        app.world()
            .get::<Children>(stacks[0])
            .is_none_or(RelationshipTarget::is_empty)
    );

    let second = app
        .world_mut()
        .spawn((Toast::new("Two"), ChildOf(root)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(toast_stacks(&mut app), stacks, "no second stack was made");
    assert_eq!(
        app.world().get::<ChildOf>(second).unwrap().parent(),
        stacks[0]
    );
}

#[test]
fn two_toasts_with_one_text_raised_in_the_same_frame_keep_one_entity() {
    use bevy_pluim::toast::Toast;

    let (mut app, root) = toast_app();
    let a = app
        .world_mut()
        .spawn((Toast::new("Saved"), ChildOf(root)))
        .id();
    let b = app
        .world_mut()
        .spawn((Toast::new("Saved"), ChildOf(root)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let alive = [a, b]
        .into_iter()
        .filter(|e| app.world().get_entity(*e).is_ok())
        .count();
    assert_eq!(alive, 1, "the later duplicate is despawned");
    let world = app.world_mut();
    assert_eq!(world.query::<&Toast>().iter(world).count(), 1);
}

#[test]
fn the_toast_stack_is_absolute_above_everything_and_ignores_the_pointer() {
    use bevy_pluim::toast::Toast;

    let (mut app, root) = toast_app();
    let toast = app
        .world_mut()
        .spawn((Toast::new("Saved"), ChildOf(root)))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let stack = toast_stacks(&mut app)[0];
    assert_eq!(app.world().get::<GlobalZIndex>(stack).unwrap().0, 1100);
    assert_eq!(
        app.world().get::<Node>(stack).unwrap().position_type,
        PositionType::Absolute
    );
    assert_eq!(app.world().get::<Pickable>(stack), Some(&Pickable::IGNORE));
    assert_eq!(app.world().get::<Pickable>(toast), Some(&Pickable::IGNORE));
}

/// The accessibility node of `entity`: role, label, value and description.
fn access_of(
    app: &App,
    entity: Entity,
) -> (
    accesskit::Role,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let node = app.world().get::<AccessibilityNode>(entity).unwrap();
    (
        node.role(),
        node.label().map(str::to_owned),
        node.value().map(str::to_owned),
        node.description().map(str::to_owned),
    )
}

#[test]
fn replacing_a_kbd_pill_and_label_updates_text_and_accessibility_and_empty_text_keeps_the_node() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let kbd = app.world_mut().spawn((Kbd::new("Esc"), ChildOf(root))).id();
    let pill = app
        .world_mut()
        .spawn((Pill::new("running"), ChildOf(root)))
        .id();
    let label = app
        .world_mut()
        .spawn((Label::regular("Row"), ChildOf(root)))
        .id();
    settle(&mut app);

    let text = |app: &App, e: Entity| app.world().get::<Text>(e).unwrap().0.clone();
    assert_eq!(access_of(&app, kbd).2.as_deref(), Some("Esc"));
    assert_eq!(access_of(&app, pill).1.as_deref(), Some("running"));

    app.world_mut().entity_mut(kbd).insert(Kbd::new("Tab"));
    app.world_mut()
        .entity_mut(pill)
        .insert(Pill::new("waiting").with_tone(PillTone::Warning));
    app.world_mut()
        .entity_mut(label)
        .insert(Label::secondary("Other"));
    settle(&mut app);
    assert_eq!(text(&app, kbd), "Tab");
    assert_eq!(access_of(&app, kbd).2.as_deref(), Some("Tab"));
    assert_eq!(text(&app, pill), "WAITING");
    assert_eq!(access_of(&app, pill).1.as_deref(), Some("waiting"));
    assert_eq!(
        app.world().get::<BackgroundColor>(pill).unwrap().0,
        DEFAULT_LIGHT_THEME_COLORS.wait_background
    );
    assert_eq!(text(&app, label), "Other");
    assert_eq!(
        app.world().get::<TextColor>(label).unwrap().0,
        DEFAULT_LIGHT_THEME_COLORS.text_secondary
    );

    // Empty text keeps the entity, its container layout and its accessibility node.
    app.world_mut().entity_mut(kbd).insert(Kbd::new(""));
    app.world_mut().entity_mut(pill).insert(Pill::new(""));
    app.world_mut().entity_mut(label).insert(Label::regular(""));
    settle(&mut app);
    for e in [kbd, pill, label] {
        assert_eq!(text(&app, e), "");
        assert_ne!(app.world().get::<Node>(e).unwrap().display, Display::None);
    }
    // The label's accessibility node is Bevy's, made by its accessibility plugin, which the
    // headless app lacks.
    assert_eq!(access_of(&app, kbd).2.as_deref(), Some(""));
    assert_eq!(access_of(&app, pill).1.as_deref(), Some(""));

    app.world_mut().entity_mut(kbd).insert(Kbd::new("Esc"));
    app.world_mut().entity_mut(pill).insert(Pill::new("back"));
    app.world_mut()
        .entity_mut(label)
        .insert(Label::regular("Back"));
    settle(&mut app);
    assert_eq!(text(&app, kbd), "Esc");
    assert_eq!(text(&app, pill), "BACK");
    assert_eq!(text(&app, label), "Back");
}

#[test]
fn replacing_a_badge_updates_text_tone_and_host_description_and_empty_text_clears_them() {
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
    let description = |app: &App| access_of(app, button).3;

    app.world_mut().entity_mut(button).insert(Badge::count(4));
    settle(&mut app);
    assert_eq!(description(&app).as_deref(), Some("4"));
    let (overlay, _) = badge_under(&app, button).unwrap();

    app.world_mut()
        .entity_mut(button)
        .insert(Badge::new("9+").with_tone(BadgeTone::Warning));
    settle(&mut app);
    assert_eq!(description(&app).as_deref(), Some("9+"));
    assert_eq!(badge_under(&app, button).unwrap().0, overlay);
    assert_eq!(
        app.world().get::<BackgroundColor>(overlay).unwrap().0,
        DEFAULT_LIGHT_THEME_COLORS.warning_text
    );

    app.world_mut().entity_mut(button).insert(Badge::new(""));
    settle(&mut app);
    assert_eq!(badge_under(&app, button), None);
    assert_eq!(description(&app), None);

    app.world_mut().entity_mut(button).insert(Badge::count(2));
    settle(&mut app);
    assert_eq!(badge_under(&app, button).unwrap().1, "2");
    assert_eq!(description(&app).as_deref(), Some("2"));
}

#[test]
fn replacing_a_toast_updates_its_text_and_label_and_keeps_one_entity_in_its_place() {
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    let mut app = headless_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        250,
    )));
    let root = app.world_mut().spawn(Node::default()).id();
    let first = app
        .world_mut()
        .spawn((Toast::new("One").with_duration(2.0), ChildOf(root)))
        .id();
    app.update();
    let second = app
        .world_mut()
        .spawn((Toast::new("Two").with_duration(60.0), ChildOf(root)))
        .id();
    settle(&mut app);
    let stack = app.world().get::<ChildOf>(first).unwrap().parent();
    let order =
        |app: &App| -> Vec<Entity> { app.world().get::<Children>(stack).unwrap().iter().collect() };
    assert_eq!(order(&app), [first, second]);
    for _ in 0..4 {
        app.update();
    }

    // Replacing the older toast changes it in place and restarts its timer; it is not
    // re-stacked.
    app.world_mut()
        .entity_mut(first)
        .insert(Toast::new("Uno").with_duration(2.0));
    app.update();
    assert_eq!(app.world().get::<Text>(first).unwrap().0, "Uno");
    assert_eq!(access_of(&app, first).1.as_deref(), Some("Uno"));
    assert_eq!(order(&app), [first, second]);
    assert_eq!(count_toasts(&mut app), 2);
    for _ in 0..6 {
        app.update();
    }
    assert!(app.world().get_entity(first).is_ok(), "timer not restarted");
    for _ in 0..4 {
        app.update();
    }
    assert!(app.world().get_entity(first).is_err());
    assert!(app.world().get_entity(second).is_ok());

    // Empty text keeps the card and the normal lifetime, and nonempty text comes back.
    app.world_mut()
        .entity_mut(second)
        .insert(Toast::new("").with_duration(1.0));
    app.update();
    assert_eq!(app.world().get::<Text>(second).unwrap().0, "");
    assert!(app.world().get::<Card>(second).is_some());
    assert_eq!(access_of(&app, second).1.as_deref(), Some(""));
    app.world_mut()
        .entity_mut(second)
        .insert(Toast::new("Back").with_duration(1.0));
    app.update();
    assert_eq!(app.world().get::<Text>(second).unwrap().0, "Back");
    for _ in 0..6 {
        app.update();
    }
    assert!(app.world().get_entity(second).is_err());
}

#[test]
fn a_badge_on_a_host_without_an_accessibility_node_describes_it_once_the_node_appears() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let host = app
        .world_mut()
        .spawn((Node::default(), Badge::count(5), ChildOf(root)))
        .id();
    settle(&mut app);
    assert!(app.world().get::<AccessibilityNode>(host).is_none());
    assert_eq!(badge_under(&app, host).unwrap().1, "5");

    app.world_mut()
        .entity_mut(host)
        .insert(AccessibilityNode::from(accesskit::Node::new(
            accesskit::Role::Button,
        )));
    settle(&mut app);
    assert_eq!(access_of(&app, host).3.as_deref(), Some("5"));

    // An empty badge then clears the description again.
    app.world_mut().entity_mut(host).insert(Badge::new(""));
    settle(&mut app);
    assert_eq!(access_of(&app, host).3, None);
}

#[test]
fn replacing_a_label_with_an_overline_uppercases_it_and_spaces_its_letters_and_back() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let label = app
        .world_mut()
        .spawn((Label::regular("Locomotives"), ChildOf(root)))
        .id();
    settle(&mut app);
    let state = |app: &App| {
        let e = app.world().entity(label);
        (
            e.get::<Text>().unwrap().0.clone(),
            *e.get::<LetterSpacing>().unwrap(),
            e.get::<TextFont>().unwrap().weight,
        )
    };
    assert_eq!(
        state(&app),
        (
            "Locomotives".to_string(),
            LetterSpacing::default(),
            FontWeight::default()
        )
    );

    app.world_mut()
        .entity_mut(label)
        .insert(Label::overline("Locomotives"));
    settle(&mut app);
    assert_eq!(
        state(&app),
        (
            "LOCOMOTIVES".to_string(),
            LetterSpacing::Px(0.5),
            FontWeight::SEMIBOLD
        )
    );

    app.world_mut()
        .entity_mut(label)
        .insert(Label::regular("Locomotives"));
    settle(&mut app);
    assert_eq!(
        state(&app),
        (
            "Locomotives".to_string(),
            LetterSpacing::default(),
            FontWeight::default()
        )
    );
}

/// The `Entity` in the reflected field `name` of the component `C` on `entity`.
fn reflected_entity<C: Component + Struct>(
    app: &App,
    entity: Entity,
    name: &str,
) -> Option<Entity> {
    app.world()
        .get::<C>(entity)?
        .field(name)?
        .try_downcast_ref::<Entity>()
        .copied()
}

#[test]
fn the_item_and_owner_of_a_selection_view_are_read_by_getters_and_by_reflection() {
    let mut app = headless_app();

    let control = spawn_segmented_control(&mut app);
    let segment = segments_of(&app, control)[0];
    let segment_ref = app.world().get::<Segment>(segment).unwrap();
    assert_eq!(
        reflected_entity::<Segment>(&app, segment, "item"),
        Some(segment_ref.item())
    );
    assert_eq!(
        reflected_entity::<Segment>(&app, segment, "owner"),
        Some(control)
    );
    assert_eq!(segment_ref.owner(), control);
    let indicator = indicator_of(&app, control);
    assert_eq!(
        reflected_entity::<SegmentIndicator>(&app, indicator, "owner"),
        Some(control)
    );
    assert_eq!(
        app.world()
            .get::<SegmentIndicator>(indicator)
            .unwrap()
            .owner(),
        control
    );

    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio"].map(Item::new)),
    );
    let tab = tabs_of(&app, bar)[0];
    let tab_ref = app.world().get::<TabBarTab>(tab).unwrap();
    assert_eq!(
        reflected_entity::<TabBarTab>(&app, tab, "item"),
        Some(tab_ref.item())
    );
    assert_eq!(reflected_entity::<TabBarTab>(&app, tab, "owner"), Some(bar));
    assert_eq!(tab_ref.owner(), bar);

    let (_, chips) = spawn_chips(&mut app, three_chips());
    let chip = chip_views(&app, chips)[0];
    let chip_ref = app.world().get::<ChipView>(chip).unwrap();
    assert_eq!(
        reflected_entity::<ChipView>(&app, chip, "item"),
        Some(chip_ref.item())
    );
    assert_eq!(
        reflected_entity::<ChipView>(&app, chip, "owner"),
        Some(chips)
    );
    assert_eq!(chip_ref.owner(), chips);

    let gallery = spawn_gallery(&mut app);
    let card = gallery_groups(&app, gallery)[0].1[0];
    let card_ref = app.world().get::<GalleryCardView>(card).unwrap();
    assert_eq!(
        reflected_entity::<GalleryCardView>(&app, card, "item"),
        Some(card_ref.item())
    );
    assert_eq!(
        reflected_entity::<GalleryCardView>(&app, card, "owner"),
        Some(gallery)
    );
    assert_eq!(card_ref.owner(), gallery);

    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let button = toolbar_buttons(&app, toolbar)[0];
    let button_ref = app.world().get::<ToolbarButton>(button).unwrap();
    assert_eq!(
        reflected_entity::<ToolbarButton>(&app, button, "item"),
        Some(button_ref.item())
    );
    assert_eq!(
        reflected_entity::<ToolbarButton>(&app, button, "owner"),
        Some(toolbar)
    );
    assert_eq!(button_ref.owner(), toolbar);
}

// ---- FilterCombo ----

const CITIES: [&str; 4] = ["Amsterdam", "Delft", "Den Haag", "Utrecht"];

/// A [`FilterCombo`] over [`CITIES`] (or other names) with its parts, in an app with a window
/// and a text input.
struct FilterApp {
    app: App,
    window: Entity,
    root: Entity,
    combo: Entity,
    refs: FilterComboRefs,
}

#[derive(Resource, Default)]
struct BubbledKeys(Vec<KeyCode>);

fn filter_app_with(
    names: &[&str],
    selection: impl FnOnce(SelectionBuilder) -> SelectionBuilderWithValue,
) -> FilterApp {
    let mut app = headless_app_with(|app| {
        app.add_plugins((bevy::text::TextPlugin, bevy::ui_widgets::TextInputPlugin));
    });
    app.init_resource::<BubbledKeys>();
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.world_mut().spawn(Camera2d);
    let root = app
        .world_mut()
        .spawn(Node::default())
        .observe(
            |key: On<bevy::input_focus::FocusedInput<bevy::input::keyboard::KeyboardInput>>,
             mut log: ResMut<BubbledKeys>| {
                log.0.push(key.event().input.key_code);
            },
        )
        .id();
    let items: Vec<Item<String>> = names.iter().map(|name| Item::new(*name)).collect();
    let combo = app
        .world_mut()
        .spawn((
            FilterCombo::default().with_name("City"),
            selection(Selection::builder()).build_with_items(items),
            ChildOf(root),
        ))
        .id();
    for _ in 0..4 {
        app.update();
    }
    let refs = *app.world().entity(combo).get::<FilterComboRefs>().unwrap();
    FilterApp {
        app,
        window,
        root,
        combo,
        refs,
    }
}

fn filter_app() -> FilterApp {
    filter_app_with(&CITIES, SelectionBuilder::zero_or_one)
}

impl FilterApp {
    fn settle(&mut self) {
        for _ in 0..4 {
            self.app.update();
        }
    }

    fn open(&mut self) {
        self.app
            .world_mut()
            .entity_mut(self.combo)
            .insert(ComboState::Open);
        self.settle();
    }

    fn state(&self) -> ComboState {
        *self
            .app
            .world()
            .entity(self.combo)
            .get::<ComboState>()
            .unwrap()
    }

    fn focus(&self) -> Option<Entity> {
        self.app.world().get_resource::<InputFocus>().unwrap().get()
    }

    fn key_with(
        &mut self,
        key_code: KeyCode,
        key: bevy::input::keyboard::Key,
        text: Option<&str>,
        repeat: bool,
    ) {
        self.app
            .world_mut()
            .write_message(bevy::input::keyboard::KeyboardInput {
                key_code,
                logical_key: key,
                state: bevy::input::ButtonState::Pressed,
                text: text.map(Into::into),
                repeat,
                window: self.window,
            });
        self.settle();
    }

    fn key(&mut self, key_code: KeyCode) {
        use bevy::input::keyboard::Key;
        let key = match key_code {
            KeyCode::ArrowDown => Key::ArrowDown,
            KeyCode::ArrowUp => Key::ArrowUp,
            KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,
            KeyCode::Escape => Key::Escape,
            KeyCode::Tab => Key::Tab,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Space => Key::Space,
            other => {
                return self.key_with(
                    other,
                    Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                    None,
                    false,
                );
            }
        };
        let text = (key_code == KeyCode::Space).then_some(" ");
        self.key_with(key_code, key, text, false);
    }

    /// Types `text` one character at a time.
    fn type_text(&mut self, text: &str) {
        for character in text.chars() {
            let s = character.to_string();
            let code = if character == ' ' {
                KeyCode::Space
            } else {
                KeyCode::KeyA
            };
            let key = if character == ' ' {
                bevy::input::keyboard::Key::Space
            } else {
                bevy::input::keyboard::Key::Character(s.as_str().into())
            };
            self.key_with(code, key, Some(&s), false);
        }
    }

    /// Deletes the last character of the query. The headless app has no font layout, which
    /// Bevy's own Backspace needs to find the character, so the text is set the way an edit
    /// that deleted would leave it; the edit system reports the change as it does for typing.
    fn backspace(&mut self) {
        let input = self.refs.input();
        let mut text = self.query_text();
        text.pop();
        let mut editable = self
            .app
            .world_mut()
            .entity_mut(input)
            .into_mut::<bevy::text::EditableText>()
            .unwrap();
        editable.pending_edits.clear();
        editable.editor.set_text(&text);
        editable.queue_edit(bevy::text::TextEdit::TextEnd(false));
        self.settle();
    }

    /// Pastes `text` into the search box as one edit.
    fn paste(&mut self, text: &str) {
        let input = self.refs.input();
        self.app
            .world_mut()
            .entity_mut(input)
            .get_mut::<bevy::text::EditableText>()
            .unwrap()
            .queue_edit(bevy::text::TextEdit::Insert(text.into()));
        self.settle();
    }

    fn query_text(&self) -> String {
        input_text(&self.app, self.refs.input())
    }

    /// The labels of the rows that show, in order.
    fn shown(&self) -> Vec<String> {
        self.rows()
            .into_iter()
            .filter(|(_, shown)| *shown)
            .map(|(label, _)| label)
            .collect()
    }

    fn rows(&self) -> Vec<(String, bool)> {
        let world = self.app.world();
        let list = world.entity(self.refs.list());
        let Some(children) = list.get::<Children>() else {
            return Vec::new();
        };
        children
            .iter()
            .filter_map(|row| {
                let view = world.entity(row).get::<ItemView>()?;
                let label = world
                    .entity(view.item_ref)
                    .get::<Item<String>>()?
                    .data
                    .clone();
                let shown = world.entity(row).get::<Node>()?.display != Display::None;
                Some((label, shown))
            })
            .collect()
    }

    fn row_of(&self, label: &str) -> Entity {
        let world = self.app.world();
        world
            .entity(self.refs.list())
            .get::<Children>()
            .unwrap()
            .iter()
            .find(|row| {
                world.entity(*row).get::<ItemView>().is_some_and(|view| {
                    world
                        .entity(view.item_ref)
                        .get::<Item<String>>()
                        .is_some_and(|item| item.data == label)
                })
            })
            .unwrap()
    }

    fn item_of(&self, label: &str) -> Entity {
        self.app
            .world()
            .entity(self.row_of(label))
            .get::<ItemView>()
            .unwrap()
            .item_ref
    }

    /// The label of the row marked active, if any.
    fn active(&mut self) -> Option<String> {
        let world = self.app.world_mut();
        let mut query = world.query::<(Entity, &ItemView, &Outline)>();
        let rows: Vec<(Entity, Entity)> = query
            .iter(world)
            .filter(|(row, ..)| {
                world
                    .entity(self.refs.list())
                    .get::<Children>()
                    .is_some_and(|children| children.contains(row))
            })
            .map(|(row, view, _)| (row, view.item_ref))
            .collect();
        assert!(rows.len() <= 1, "{} active rows", rows.len());
        rows.first().map(|(_, item)| {
            world
                .entity(*item)
                .get::<Item<String>>()
                .unwrap()
                .data
                .clone()
        })
    }

    fn selected(&self) -> Vec<String> {
        let world = self.app.world();
        let selection = world.entity(self.combo).get::<Selection>().unwrap();
        let value = selection.as_selection().unwrap();
        value
            .iter()
            .map(|item| {
                world
                    .entity(item)
                    .get::<Item<String>>()
                    .unwrap()
                    .data
                    .clone()
            })
            .collect()
    }

    fn button_text(&self) -> String {
        let world = self.app.world();
        let button = world
            .entity(self.refs.button())
            .get::<PushButtonRefs>()
            .unwrap()
            .label();
        world.entity(button).get::<Text>().unwrap().0.clone()
    }

    fn node(&self, entity: Entity) -> &AccessibilityNode {
        self.app
            .world()
            .entity(entity)
            .get::<AccessibilityNode>()
            .unwrap()
    }

    fn bubbled_keys(&self) -> Vec<KeyCode> {
        self.app
            .world()
            .get_resource::<BubbledKeys>()
            .unwrap()
            .0
            .clone()
    }
}

#[test]
fn a_filter_combo_opens_with_every_option_an_empty_query_and_the_focus_in_its_search_box() {
    let mut fa = filter_app();
    assert_eq!(fa.state(), ComboState::Closed);
    fa.open();
    assert_eq!(fa.state(), ComboState::Open);
    assert_eq!(fa.shown(), CITIES);
    assert_eq!(fa.query_text(), "");
    assert_eq!(fa.focus(), Some(fa.refs.input()));
    assert_eq!(fa.active().as_deref(), Some("Amsterdam"));
}

#[test]
fn typing_filters_the_options_by_case_insensitive_substring_in_source_order() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("DEL");
    assert_eq!(fa.shown(), ["Delft"]);
    fa.backspace();
    fa.backspace();
    fa.backspace();
    assert_eq!(fa.shown(), CITIES);
    fa.type_text("e");
    // Amst(e)rdam, D(e)lft, D(e)n Haag, Utr(e)cht.
    assert_eq!(fa.shown(), CITIES);
    fa.type_text("n");
    assert_eq!(fa.shown(), ["Den Haag"]);
}

#[test]
fn a_query_with_a_space_and_a_pasted_query_filter_too() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("n h");
    assert_eq!(fa.shown(), ["Den Haag"]);
    fa.backspace();
    fa.backspace();
    fa.backspace();
    assert_eq!(fa.shown(), CITIES);
    fa.paste("utr");
    assert_eq!(fa.query_text(), "utr");
    assert_eq!(fa.shown(), ["Utrecht"]);
    fa.paste("x");
    assert_eq!(fa.shown(), Vec::<String>::new());
}

#[test]
fn space_types_a_space_in_the_query_and_picks_nothing() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("den");
    fa.key(KeyCode::Space);
    assert_eq!(fa.query_text(), "den ");
    assert_eq!(fa.shown(), ["Den Haag"]);
    fa.key(KeyCode::Space);
    assert_eq!(fa.query_text(), "den  ");
    assert_eq!(fa.shown(), Vec::<String>::new());
    assert!(fa.selected().is_empty());
    assert_eq!(fa.state(), ComboState::Open);
}

#[test]
fn a_query_with_no_match_shows_the_empty_line_and_enter_does_nothing() {
    let mut fa = filter_app();
    fa.open();
    let empty = fa.refs.empty();
    let display = |fa: &FilterApp| fa.app.world().entity(empty).get::<Node>().unwrap().display;
    assert_eq!(display(&fa), Display::None);
    fa.type_text("zzz");
    assert_eq!(fa.shown(), Vec::<String>::new());
    assert_eq!(display(&fa), Display::Flex);
    assert_eq!(fa.active(), None);
    fa.key(KeyCode::Enter);
    assert_eq!(fa.state(), ComboState::Open);
    assert!(fa.selected().is_empty());
    assert_eq!(fa.focus(), Some(fa.refs.input()));
    // Clearing the query brings every option back.
    for _ in 0..3 {
        fa.backspace();
    }
    assert_eq!(fa.shown(), CITIES);
    assert_eq!(display(&fa), Display::None);
}

#[test]
fn arrow_keys_move_the_active_option_through_the_matches_only_and_wrap() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("e");
    // All four match "e"; narrow to Delft, Den Haag with "de".
    fa.type_text("d");
    // "ed" matches nothing; replace it.
    fa.backspace();
    fa.backspace();
    fa.type_text("de");
    assert_eq!(fa.shown(), ["Delft", "Den Haag"]);
    assert_eq!(fa.active().as_deref(), Some("Delft"));
    fa.key(KeyCode::ArrowDown);
    assert_eq!(fa.active().as_deref(), Some("Den Haag"));
    fa.key(KeyCode::ArrowDown);
    assert_eq!(fa.active().as_deref(), Some("Delft"));
    fa.key(KeyCode::ArrowUp);
    assert_eq!(fa.active().as_deref(), Some("Den Haag"));
    // Navigation alone selects nothing, and typing goes on after it.
    assert!(fa.selected().is_empty());
    fa.type_text("n");
    assert_eq!(fa.query_text(), "den");
    assert_eq!(fa.shown(), ["Den Haag"]);
}

#[test]
fn a_held_arrow_key_repeats_and_no_navigation_key_reaches_the_application() {
    let mut fa = filter_app();
    fa.open();
    for _ in 0..2 {
        fa.key_with(
            KeyCode::ArrowDown,
            bevy::input::keyboard::Key::ArrowDown,
            None,
            true,
        );
    }
    assert_eq!(fa.active().as_deref(), Some("Den Haag"));
    fa.key(KeyCode::ArrowUp);
    fa.key(KeyCode::Enter);
    let bubbled = fa.bubbled_keys();
    assert!(
        !bubbled
            .iter()
            .any(|key| matches!(key, KeyCode::ArrowDown | KeyCode::ArrowUp | KeyCode::Enter)),
        "{bubbled:?}"
    );
}

#[test]
fn enter_picks_the_original_item_of_a_single_selection_and_closes_with_the_focus_on_the_button() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("utr");
    let utrecht = fa.item_of("Utrecht");
    fa.key(KeyCode::Enter);
    assert_eq!(fa.selected(), ["Utrecht"]);
    let selection = fa.app.world().entity(fa.combo).get::<Selection>().unwrap();
    assert_eq!(selection.as_selection().unwrap().single(), Some(utrecht));
    assert_eq!(fa.state(), ComboState::Closed);
    assert_eq!(fa.focus(), Some(fa.refs.button()));
    assert_eq!(fa.button_text(), "Utrecht");
    // Reopening starts unfiltered, with the selected option active.
    fa.open();
    assert_eq!(fa.query_text(), "");
    assert_eq!(fa.shown(), CITIES);
    assert_eq!(fa.active().as_deref(), Some("Utrecht"));
}

#[test]
fn the_numpad_enter_picks_too() {
    let mut fa = filter_app();
    fa.open();
    fa.key_with(
        KeyCode::NumpadEnter,
        bevy::input::keyboard::Key::Enter,
        None,
        false,
    );
    assert_eq!(fa.selected(), ["Amsterdam"]);
    assert_eq!(fa.state(), ComboState::Closed);
}

#[test]
fn a_selection_of_many_stays_open_with_its_query_after_enter() {
    let mut fa = filter_app_with(&CITIES, SelectionBuilder::zero_or_more);
    fa.open();
    fa.type_text("de");
    fa.key(KeyCode::Enter);
    assert_eq!(fa.selected(), ["Delft"]);
    assert_eq!(fa.state(), ComboState::Open);
    assert_eq!(fa.query_text(), "de");
    fa.key(KeyCode::ArrowDown);
    fa.key(KeyCode::Enter);
    assert_eq!(fa.selected(), ["Delft", "Den Haag"]);
    assert_eq!(fa.state(), ComboState::Open);
    // Enter on a selected option deselects it, as a click does.
    fa.key(KeyCode::Enter);
    assert_eq!(fa.selected(), ["Delft"]);
}

#[test]
fn every_selection_cardinality_picks_through_the_filter() {
    for (name, multiple) in [
        ("one", false),
        ("zero_or_one", false),
        ("one_or_more", true),
        ("zero_or_more", true),
    ] {
        let mut fa = filter_app_with(&CITIES, |b| match name {
            "one" => b.one(),
            "zero_or_one" => b.zero_or_one(),
            "one_or_more" => b.one_or_more(),
            _ => b.zero_or_more(),
        });
        fa.open();
        fa.type_text("utr");
        fa.key(KeyCode::Enter);
        assert!(fa.selected().contains(&"Utrecht".to_string()), "{name}");
        assert_eq!(fa.state() == ComboState::Open, multiple, "{name}");
    }
}

#[test]
fn filtering_never_changes_the_selection_and_a_hidden_selected_option_stays_selected() {
    let mut fa = filter_app();
    let delft = fa.item_of("Delft");
    fa.app
        .world_mut()
        .entity_mut(fa.combo)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(delft))));
    fa.settle();
    assert_eq!(fa.selected(), ["Delft"]);
    fa.open();
    assert_eq!(fa.active().as_deref(), Some("Delft"));
    fa.type_text("utr");
    assert_eq!(fa.shown(), ["Utrecht"]);
    assert_eq!(fa.selected(), ["Delft"]);
    assert!(
        fa.app
            .world()
            .entity(fa.row_of("Delft"))
            .contains::<Selected>()
    );
    assert!(
        !fa.app
            .world()
            .entity(fa.row_of("Utrecht"))
            .contains::<Selected>()
    );
    // Arrow keys and clearing the query change nothing either.
    fa.key(KeyCode::ArrowDown);
    for _ in 0..3 {
        fa.backspace();
    }
    assert_eq!(fa.selected(), ["Delft"]);
    assert!(
        fa.app
            .world()
            .entity(fa.row_of("Delft"))
            .contains::<Selected>()
    );
    assert_eq!(fa.button_text(), "Delft");
}

#[test]
fn the_active_option_stays_while_it_matches_and_moves_to_the_first_match_when_it_does_not() {
    let mut fa = filter_app();
    fa.open();
    fa.key(KeyCode::ArrowDown);
    fa.key(KeyCode::ArrowDown);
    assert_eq!(fa.active().as_deref(), Some("Den Haag"));
    fa.type_text("e");
    assert_eq!(fa.active().as_deref(), Some("Den Haag"), "still matches");
    fa.type_text("n");
    assert_eq!(fa.active().as_deref(), Some("Den Haag"));
    fa.backspace();
    fa.backspace();
    fa.type_text("lf");
    assert_eq!(fa.shown(), ["Delft"]);
    assert_eq!(fa.active().as_deref(), Some("Delft"), "first match");
}

#[test]
fn escape_closes_without_picking_and_reopening_starts_unfiltered() {
    let mut fa = filter_app_with(&CITIES, SelectionBuilder::zero_or_more);
    fa.open();
    fa.type_text("del");
    fa.key(KeyCode::Enter);
    fa.type_text("f");
    fa.key(KeyCode::Escape);
    assert_eq!(fa.state(), ComboState::Closed);
    // The earlier pick stays; the unconfirmed active option was not applied.
    assert_eq!(fa.selected(), ["Delft"]);
    assert_eq!(fa.focus(), Some(fa.refs.button()));
    fa.open();
    assert_eq!(fa.query_text(), "");
    assert_eq!(fa.shown(), CITIES);
}

#[test]
fn tab_closes_without_picking_and_the_focus_goes_where_tab_sent_it() {
    let mut fa = filter_app();
    let other = fa
        .app
        .world_mut()
        .spawn((PushButton::from_label("After"), ChildOf(fa.root)))
        .id();
    fa.settle();
    fa.open();
    fa.type_text("utr");
    fa.key(KeyCode::Tab);
    assert_eq!(fa.state(), ComboState::Closed);
    assert!(fa.selected().is_empty());
    assert_ne!(fa.focus(), Some(fa.refs.input()));
    assert_ne!(fa.focus(), None);
    let _ = other;
}

#[test]
fn losing_the_focus_to_something_else_closes_the_popup_without_taking_the_focus_back() {
    let mut fa = filter_app();
    let other = fa
        .app
        .world_mut()
        .spawn((PushButton::from_label("Other"), ChildOf(fa.root)))
        .id();
    fa.settle();
    fa.open();
    fa.app
        .world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(other, bevy::input_focus::FocusCause::Navigated);
    fa.settle();
    assert_eq!(fa.state(), ComboState::Closed);
    assert_eq!(fa.focus(), Some(other));
    assert!(fa.selected().is_empty());
}

#[test]
fn closing_never_leaves_the_focus_on_the_hidden_search_box() {
    let mut fa = filter_app();
    fa.open();
    fa.app
        .world_mut()
        .entity_mut(fa.combo)
        .insert(ComboState::Closed);
    fa.settle();
    assert_eq!(fa.focus(), Some(fa.refs.button()));
}

#[test]
fn the_other_widgets_on_the_same_items_stay_unfiltered() {
    let mut fa = filter_app();
    let list = fa
        .app
        .world_mut()
        .spawn((List, Selection::reference(fa.combo), ChildOf(fa.root)))
        .id();
    fa.settle();
    fa.open();
    fa.type_text("utr");
    let rows = fa.app.world().entity(list).get::<Children>().unwrap().len();
    assert_eq!(rows, CITIES.len());
    assert!(
        fa.app
            .world()
            .entity(list)
            .get::<Children>()
            .unwrap()
            .iter()
            .all(|row| fa.app.world().entity(row).get::<Node>().unwrap().display != Display::None)
    );
}

#[test]
fn a_pending_one_selection_is_repaired_by_the_first_item_and_filtering_leaves_it_alone() {
    let mut fa = filter_app_with(&[], SelectionBuilder::one);
    fa.open();
    assert_eq!(fa.shown(), Vec::<String>::new());
    let empty = fa.refs.empty();
    assert_eq!(
        fa.app.world().entity(empty).get::<Node>().unwrap().display,
        Display::Flex
    );
    fa.key(KeyCode::Enter);
    assert_eq!(fa.state(), ComboState::Open);
    let item = fa
        .app
        .world_mut()
        .spawn((Item::new("Leiden"), ItemOf(fa.combo)))
        .id();
    fa.settle();
    assert_eq!(fa.shown(), ["Leiden"]);
    assert_eq!(fa.selected(), ["Leiden"]);
    assert_eq!(fa.active().as_deref(), Some("Leiden"));
    assert_eq!(
        fa.app.world().entity(empty).get::<Node>().unwrap().display,
        Display::None
    );
    let _ = item;
}

#[test]
fn added_removed_and_renamed_items_are_filtered_while_the_popup_is_open() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("de");
    assert_eq!(fa.shown(), ["Delft", "Den Haag"]);
    // A new item that matches shows; one that does not stays hidden.
    fa.app
        .world_mut()
        .spawn((Item::new("Deventer"), ItemOf(fa.combo)));
    fa.app
        .world_mut()
        .spawn((Item::new("Zwolle"), ItemOf(fa.combo)));
    fa.settle();
    assert_eq!(fa.shown(), ["Delft", "Den Haag", "Deventer"]);
    // A rename brings an item in and takes another out.
    let utrecht = fa.item_of("Utrecht");
    let delft = fa.item_of("Delft");
    fa.app
        .world_mut()
        .entity_mut(utrecht)
        .get_mut::<Item<String>>()
        .unwrap()
        .data = "Delden".to_string();
    fa.app
        .world_mut()
        .entity_mut(delft)
        .get_mut::<Item<String>>()
        .unwrap()
        .data = "Gouda".to_string();
    fa.settle();
    assert_eq!(fa.shown(), ["Den Haag", "Delden", "Deventer"]);
    // The active option going away moves it to the first match.
    assert_eq!(fa.active().as_deref(), Some("Den Haag"));
    let den_haag = fa.item_of("Den Haag");
    fa.app.world_mut().entity_mut(den_haag).despawn();
    fa.settle();
    assert_eq!(fa.shown(), ["Delden", "Deventer"]);
    assert_eq!(fa.active().as_deref(), Some("Delden"));
}

#[test]
fn renaming_an_item_of_another_source_does_not_rematch_an_open_filter_combo() {
    let mut fa = filter_app();
    let other = fa
        .app
        .world_mut()
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(vec![Item::new("Berlin"), Item::new("Bern")]),
        )
        .id();
    fa.settle();
    fa.open();
    // Hide a row by hand: only a rematch of this combo shows it again.
    let row = fa.row_of("Delft");
    fa.app.world_mut().get_mut::<Node>(row).unwrap().display = Display::None;
    let berlin = fa
        .app
        .world()
        .entity(other)
        .get::<ItemValues>()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    fa.app
        .world_mut()
        .get_mut::<Item<String>>(berlin)
        .unwrap()
        .data = "Bremen".to_string();
    fa.settle();
    assert_eq!(
        fa.app.world().get::<Node>(row).unwrap().display,
        Display::None,
        "a rename in an unrelated source rematched the combo"
    );
    // A rename in its own source does rematch it, keeping case-insensitive matching.
    let utrecht = fa.item_of("Utrecht");
    fa.app
        .world_mut()
        .get_mut::<Item<String>>(utrecht)
        .unwrap()
        .data = "UTRECHT-Zuid".to_string();
    fa.settle();
    assert_eq!(
        fa.app.world().get::<Node>(row).unwrap().display,
        Display::Flex
    );
    fa.type_text("zuid");
    assert_eq!(fa.shown(), ["UTRECHT-Zuid"]);
}

#[test]
fn pointing_the_combo_at_other_items_filters_the_new_ones() {
    let mut fa = filter_app();
    let other = fa
        .app
        .world_mut()
        .spawn(Selection::builder().zero_or_one().build_with_items(vec![
            Item::new("Berlin"),
            Item::new("Bern"),
            Item::new("Paris"),
        ]))
        .id();
    fa.open();
    fa.type_text("ber");
    fa.app
        .world_mut()
        .entity_mut(fa.combo)
        .insert(Selection::reference(other));
    fa.settle();
    assert_eq!(fa.shown(), ["Berlin", "Bern"]);
    assert_eq!(fa.active().as_deref(), Some("Berlin"));
    fa.key(KeyCode::ArrowDown);
    fa.key(KeyCode::Enter);
    let selection = fa.app.world().entity(other).get::<Selection>().unwrap();
    let picked = selection.as_selection().unwrap().single().unwrap();
    assert_eq!(
        fa.app
            .world()
            .entity(picked)
            .get::<Item<String>>()
            .unwrap()
            .data,
        "Bern"
    );
}

#[test]
fn a_clicked_option_picks_by_the_same_rules_and_a_click_outside_closes_without_picking() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("utr");
    let row = fa.row_of("Utrecht");
    click_row(&mut fa.app, row);
    fa.settle();
    assert_eq!(fa.selected(), ["Utrecht"]);

    let mut fa = filter_app();
    fa.open();
    fa.type_text("utr");
    // A press and release on nothing: the pointer is on neither the button nor the popup.
    for state in [
        bevy::input::ButtonState::Pressed,
        bevy::input::ButtonState::Released,
    ] {
        fa.app
            .world_mut()
            .write_message(bevy::input::mouse::MouseButtonInput {
                button: MouseButton::Left,
                state,
                window: fa.window,
            });
        fa.app.update();
    }
    fa.settle();
    assert_eq!(fa.state(), ComboState::Closed);
    assert!(fa.selected().is_empty());
}

#[test]
fn the_search_box_and_the_button_tell_a_screen_reader_how_they_belong_together() {
    let mut fa = filter_app();
    let (button, input, list) = (fa.refs.button(), fa.refs.input(), fa.refs.list());
    let id = |e: Entity| accesskit::NodeId(e.to_bits());
    // Closed: the button is a collapsed combo box with the selection as its value.
    assert_eq!(fa.node(button).role(), accesskit::Role::ComboBox);
    assert_eq!(fa.node(button).is_expanded(), Some(false));
    assert_eq!(fa.node(button).controls(), [id(list)]);
    assert_eq!(fa.node(button).label(), Some("City"));
    assert!(
        fa.node(input).is_hidden(),
        "the closed search box is out of the tree"
    );
    fa.open();
    assert_eq!(fa.node(button).is_expanded(), Some(true));
    assert!(!fa.node(input).is_hidden());
    assert_eq!(fa.node(input).role(), accesskit::Role::ComboBox);
    assert_eq!(fa.node(input).label(), Some("City"));
    assert_eq!(fa.node(input).value(), Some(""));
    assert_eq!(fa.node(input).controls(), [id(list)]);
    assert_eq!(fa.node(input).is_expanded(), Some(true));
    assert_eq!(
        fa.node(input).active_descendant(),
        Some(id(fa.row_of("Amsterdam")))
    );
    fa.type_text("del");
    assert_eq!(fa.node(input).value(), Some("del"));
    assert_eq!(
        fa.node(input).active_descendant(),
        Some(id(fa.row_of("Delft")))
    );
    // Excluded rows are out of the tree; the shown one is in it.
    assert!(fa.node(fa.row_of("Amsterdam")).is_hidden());
    assert!(!fa.node(fa.row_of("Delft")).is_hidden());
    // Zero results: no active descendant, and a status line in the tree.
    fa.type_text("x");
    assert_eq!(fa.node(input).active_descendant(), None);
    let empty = fa.refs.empty();
    assert!(!fa.node(empty).is_hidden());
    assert_eq!(fa.node(empty).role(), accesskit::Role::Status);
    assert_eq!(fa.node(empty).label(), Some("No matching items"));
    // Closed again: nothing points at a row.
    fa.key(KeyCode::Escape);
    assert_eq!(fa.node(input).active_descendant(), None);
    assert_eq!(fa.node(button).is_expanded(), Some(false));
    assert!(fa.node(input).is_hidden());
    assert!(fa.node(empty).is_hidden());
}

#[test]
fn the_selected_options_are_selected_for_a_screen_reader_and_the_button_says_which() {
    let mut fa = filter_app_with(&CITIES, SelectionBuilder::zero_or_more);
    fa.open();
    fa.type_text("de");
    fa.key(KeyCode::Enter);
    fa.key(KeyCode::ArrowDown);
    fa.key(KeyCode::Enter);
    assert_eq!(fa.node(fa.refs.button()).value(), Some("Delft, Den Haag"));
    assert!(
        fa.app
            .world()
            .entity(fa.row_of("Delft"))
            .contains::<Selected>()
    );
    assert!(
        fa.app
            .world()
            .entity(fa.row_of("Den Haag"))
            .contains::<Selected>()
    );
}

#[test]
fn the_popup_follows_the_theme_in_both_modes() {
    let mut fa = filter_app();
    fa.open();
    for dark in [false, true] {
        if dark {
            fa.app
                .world_mut()
                .get_resource_mut::<Theme>()
                .unwrap()
                .light_dark_toggle();
            fa.settle();
        }
        let colors = *fa.app.world().get_resource::<Theme>().unwrap().colors();
        let world = fa.app.world();
        let popup = world.entity(fa.refs.popup());
        assert_eq!(
            popup.get::<BackgroundColor>().unwrap().0,
            colors.background_primary
        );
        assert_eq!(popup.get::<BorderColor>().unwrap().top, colors.border);
        let outline = world
            .entity(fa.row_of("Amsterdam"))
            .get::<Outline>()
            .unwrap()
            .color;
        assert_eq!(outline, colors.focus_ring);
    }
}

#[test]
fn despawning_a_filter_combo_leaves_nothing_behind() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("de");
    let (combo, input) = (fa.combo, fa.refs.input());
    fa.app.world_mut().entity_mut(combo).despawn();
    fa.settle();
    assert!(fa.app.world().get_entity(input).is_err());
    // Keys with the old focus do nothing and do not panic.
    fa.key(KeyCode::ArrowDown);
    fa.key(KeyCode::Enter);
    let mut parts = fa.app.world_mut().query::<&FilterComboRefs>();
    assert_eq!(parts.iter(fa.app.world()).count(), 0);
}

#[test]
fn an_ordinary_combo_is_unchanged_next_to_a_filter_combo() {
    let mut fa = filter_app();
    let combo = fa
        .app
        .world_mut()
        .spawn((Combo, Selection::reference(fa.combo), ChildOf(fa.root)))
        .id();
    fa.settle();
    let list = fa
        .app
        .world()
        .entity(combo)
        .get::<ComboRefs>()
        .unwrap()
        .list();
    fa.app
        .world_mut()
        .entity_mut(combo)
        .insert(ComboState::Open);
    fa.settle();
    // Its focus is on its own list; the filter combo's search box is untouched.
    assert_eq!(fa.focus(), Some(list));
    fa.key(KeyCode::ArrowDown);
    fa.key(KeyCode::Enter);
    assert_eq!(fa.selected().len(), 1);
    assert_eq!(
        fa.app.world().entity(combo).get::<ComboState>(),
        Some(&ComboState::Closed)
    );
    assert_eq!(fa.state(), ComboState::Closed);
}

#[test]
fn spawning_a_filter_combo_does_not_move_the_keyboard_focus_into_itself() {
    let fa = filter_app();
    assert_ne!(fa.focus(), Some(fa.refs.button()));
    assert_ne!(fa.focus(), Some(fa.refs.input()));
}

#[test]
fn shift_tab_to_the_button_closes_the_popup_without_picking_but_a_press_on_it_does_not() {
    let mut fa = filter_app();
    fa.open();
    fa.type_text("utr");
    fa.app
        .world_mut()
        .get_resource_mut::<ButtonInput<KeyCode>>()
        .unwrap()
        .press(KeyCode::ShiftLeft);
    fa.key(KeyCode::Tab);
    fa.app
        .world_mut()
        .get_resource_mut::<ButtonInput<KeyCode>>()
        .unwrap()
        .release(KeyCode::ShiftLeft);
    assert_eq!(fa.state(), ComboState::Closed);
    assert!(fa.selected().is_empty());
    assert_eq!(fa.focus(), Some(fa.refs.button()));
}

// ---- ThemeEditor (the library widget) ----

use bevy_pluim::theme_editor::{
    ThemeEditor, ThemeEditorRefs, ThemeParameterEditor, theme_parameters,
};

#[test]
fn every_number_of_the_theme_has_a_row_in_the_theme_editor() {
    use bevy::reflect::structs::Struct as _;
    let theme = Theme::default();
    let names: Vec<&str> = theme_parameters().iter().map(|p| p.name).collect();
    for i in 0..theme.field_len() {
        let field = theme.name_at(i).unwrap();
        let value = theme.field_at(i).unwrap();
        if value.try_downcast_ref::<Val>().is_some() || value.try_downcast_ref::<f32>().is_some() {
            assert!(names.contains(&field), "`{field}` has no row");
        } else if value.try_downcast_ref::<UiRect>().is_some() {
            for side in ["left", "right", "top", "bottom"] {
                let name = format!("{field}.{side}");
                assert!(names.contains(&name.as_str()), "`{name}` has no row");
            }
        } else if value.try_downcast_ref::<ShadowStyle>().is_some() {
            assert!(
                names.iter().any(|n| n.starts_with(&format!("{field}."))),
                "`{field}` has no row"
            );
        }
    }
    // Names are unique.
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len());
}

#[test]
fn a_theme_parameter_reads_what_it_sets_and_stays_in_its_range() {
    for parameter in theme_parameters() {
        let mut theme = Theme::default();
        parameter.apply(&mut theme, parameter.max + 100.0);
        assert!(
            (parameter.value(&theme) - parameter.max).abs() < f32::EPSILON,
            "{}",
            parameter.name
        );
        parameter.apply(&mut theme, parameter.min - 100.0);
        assert!(
            (parameter.value(&theme) - parameter.min).abs() < f32::EPSILON,
            "{}",
            parameter.name
        );
        let middle = f32::midpoint(parameter.min, parameter.max).round();
        parameter.apply(&mut theme, middle);
        assert!((parameter.value(&theme) - middle).abs() < f32::EPSILON);
        // Only that number moved.
        let mut expected = Theme::default();
        parameter.apply(&mut expected, middle);
        assert_eq!(format!("{theme:?}"), format!("{expected:?}"));
    }
}

fn editor_stepper(app: &mut App, name: &str) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &ThemeParameterEditor)>()
        .iter(world)
        .find(|(_, editor)| editor.0 == name)
        .map(|(entity, _)| entity)
        .unwrap()
}

#[test]
fn the_theme_editor_has_a_stepper_for_each_number_showing_the_value_in_the_theme() {
    let mut app = theme_editor_app();
    let count = {
        let world = app.world_mut();
        world.query::<&ThemeParameterEditor>().iter(world).count()
    };
    assert_eq!(count, theme_parameters().len());
    let stepper = editor_stepper(&mut app, "control_radius");
    let value = app.world().entity(stepper).get::<Value>().unwrap();
    assert_eq!(value.as_f32(), Some(8.0));
}

#[test]
fn stepping_a_number_in_the_theme_editor_sets_it_in_the_theme_and_only_it() {
    let mut app = theme_editor_app();
    let before = app.world().get_resource::<Theme>().unwrap().clone();
    let stepper = editor_stepper(&mut app, "control_radius");
    app.world_mut()
        .trigger(bevy_pluim::numeric_stepper::ValueChange {
            entity: stepper,
            value: Value::new_f32(14.0, 0.0, 32.0, 1.0).unwrap(),
        });
    app.update();
    app.update();
    let theme = app.world().get_resource::<Theme>().unwrap();
    assert_eq!(theme.control_radius, Val::Px(14.0));
    let mut expected = before;
    expected.control_radius = Val::Px(14.0);
    assert_eq!(format!("{theme:?}"), format!("{expected:?}"));
}

#[test]
fn the_theme_editor_edits_the_padding_sides_and_the_shadow_geometry_too() {
    let mut app = theme_editor_app();
    for (name, check) in [
        (
            "item_padding.top",
            (|t: &Theme| t.item_padding.top) as fn(&Theme) -> Val,
        ),
        ("compact_button_padding.left", |t| {
            t.compact_button_padding.left
        }),
        ("color_button_padding.bottom", |t| {
            t.color_button_padding.bottom
        }),
        ("shadow_style.blur_radius", |t| t.large_shadow().blur_radius),
    ] {
        let stepper = editor_stepper(&mut app, name);
        app.world_mut()
            .trigger(bevy_pluim::numeric_stepper::ValueChange {
                entity: stepper,
                value: Value::new_f32(17.0, 0.0, 40.0, 1.0).unwrap(),
            });
        app.update();
        app.update();
        assert_eq!(
            check(app.world().get_resource::<Theme>().unwrap()),
            Val::Px(17.0),
            "{name}"
        );
    }
}

#[test]
fn a_change_to_the_theme_moves_the_steppers_of_the_theme_editor() {
    let mut app = theme_editor_app();
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .font_size_body = 21.0;
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .item_padding
        .left = Val::Px(9.0);
    for _ in 0..3 {
        app.update();
    }
    let body = editor_stepper(&mut app, "font_size_body");
    assert_eq!(
        app.world().entity(body).get::<Value>().unwrap().as_f32(),
        Some(21.0)
    );
    let left = editor_stepper(&mut app, "item_padding.left");
    assert_eq!(
        app.world().entity(left).get::<Value>().unwrap().as_f32(),
        Some(9.0)
    );
}

#[test]
fn the_theme_editor_says_which_half_it_edits_and_follows_the_toggle() {
    let mut app = theme_editor_app();
    let label_text = |app: &mut App| {
        let world = app.world_mut();
        let editor = world
            .query_filtered::<&ThemeEditorRefs, With<ThemeEditor>>()
            .single(world)
            .unwrap()
            .mode_label();
        world.entity(editor).get::<Text>().unwrap().0.clone()
    };
    assert_eq!(label_text(&mut app), "Editing the light theme");
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .light_dark_toggle();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(label_text(&mut app), "Editing the dark theme");
}

#[test]
fn the_theme_editor_is_a_library_widget_that_needs_no_example_code() {
    let mut app = headless_app();
    let editor = app.world_mut().spawn((ThemeEditor, Node::default())).id();
    for _ in 0..6 {
        app.update();
    }
    let refs = *app.world().entity(editor).get::<ThemeEditorRefs>().unwrap();
    assert!(app.world().get_entity(refs.scroll_area()).is_ok());
    let slots = {
        let world = app.world_mut();
        world
            .query::<&bevy_pluim::theme_editor::ThemeColorSlot>()
            .iter(world)
            .count()
    };
    assert_eq!(
        slots,
        bevy_pluim::theme_editor::editable_color_slots().len()
    );
    // Despawning it takes everything with it, the colours it chooses from included.
    app.world_mut().entity_mut(editor).despawn();
    for _ in 0..2 {
        app.update();
    }
    assert!(app.world().get_entity(refs.colors()).is_err());
    let left = {
        let world = app.world_mut();
        world.query::<&ThemeParameterEditor>().iter(world).count()
    };
    assert_eq!(left, 0);
}

#[test]
fn the_theme_editor_example_exports_the_numbers_as_a_theme_function() {
    let source = overview::theme_editor::theme_source(&Theme::default());
    assert!(source.contains("const LIGHT: ThemeColors"));
    assert!(source.contains("fn theme() -> Theme {"));
    assert!(source.contains("        control_radius: Val::Px(8.0),"));
    assert!(source.contains("        font_size_body: 15.0,"));
    assert!(source.contains("        item_padding: UiRect {\n            left: Val::Px(12.0),"));
    assert!(source.contains("    // shadow blur_radius: 8.0"));
    assert!(source.contains("..Theme::with_colors(LIGHT, DARK)"));
    assert_eq!(
        source.matches('{').count(),
        source.matches('}').count(),
        "balanced braces"
    );
}

// ---- ThemeEditor: values the rows cannot show as they are (#611) ----

/// Runs `frames` updates of `app` and returns the `warn!` messages logged meanwhile.
fn warnings_during(app: &mut App, frames: usize) -> Vec<String> {
    use bevy::log::tracing::{self, Level, field::Visit};
    use bevy::log::tracing_subscriber::{self, layer::SubscriberExt};
    use std::sync::{Arc, Mutex};

    struct Message(String);
    impl Visit for Message {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.0 = format!("{value:?}");
            }
        }
    }
    struct Warnings(Arc<Mutex<Vec<String>>>);
    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Warnings {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if *event.metadata().level() == Level::WARN
                && let Ok(mut all) = self.0.lock()
            {
                let mut message = Message(String::new());
                event.record(&mut message);
                all.push(message.0);
            }
        }
    }

    let seen = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Warnings(seen.clone()));
    // The subscriber is thread-local, so the systems must run on this thread.
    app.edit_schedule(Update, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });
    tracing::subscriber::with_default(subscriber, || {
        for _ in 0..frames {
            app.update();
        }
    });
    seen.lock().unwrap().clone()
}

/// An app whose `Theme` is edited by `edit` before a `ThemeEditor` is spawned, and the warnings
/// logged while the editor built and showed it.
fn theme_editor_over(edit: impl FnOnce(&mut Theme)) -> (App, Vec<String>) {
    let mut app = headless_app();
    edit(&mut app.world_mut().get_resource_mut::<Theme>().unwrap());
    app.world_mut().spawn((ThemeEditor, Node::default()));
    let warnings = warnings_during(&mut app, 6);
    (app, warnings)
}

fn step_editor_to(app: &mut App, name: &str, value: f32) {
    let stepper = editor_stepper(app, name);
    app.world_mut()
        .trigger(bevy_pluim::numeric_stepper::ValueChange {
            entity: stepper,
            value: Value::new_f32(value, 0.0, 100.0, 1.0).unwrap(),
        });
    app.update();
    app.update();
}

#[test]
fn a_theme_number_above_its_range_still_has_a_row_with_the_range_widened_to_it() {
    let (mut app, _) = theme_editor_over(|theme| {
        theme.control_radius = Val::Px(50.0);
        theme.font_size_body = 60.0;
        theme.scroll_bar_width = Val::Px(0.5);
    });
    let count = {
        let world = app.world_mut();
        world.query::<&ThemeParameterEditor>().iter(world).count()
    };
    assert_eq!(count, theme_parameters().len(), "a row is missing");
    for (name, shown) in [
        ("control_radius", 50.0),
        ("font_size_body", 60.0),
        ("scroll_bar_width", 0.5),
    ] {
        let stepper = editor_stepper(&mut app, name);
        let value = app.world().entity(stepper).get::<Value>().unwrap();
        assert_eq!(value.as_f32(), Some(shown), "{name}");
    }

    // Stepping stays inside the widened range: down to 49, and 50 is still the top.
    let stepper = editor_stepper(&mut app, "control_radius");
    assert!(
        !app.world()
            .entity(stepper)
            .get::<Value>()
            .unwrap()
            .can_increment()
    );
    assert!(
        app.world()
            .entity(stepper)
            .get::<Value>()
            .unwrap()
            .can_decrement()
    );
    step_editor_to(&mut app, "control_radius", 49.0);
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().control_radius,
        Val::Px(49.0)
    );
    // A later change of the theme to another out-of-range value widens the live row too.
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .control_radius = Val::Px(70.0);
    for _ in 0..3 {
        app.update();
    }
    let stepper = editor_stepper(&mut app, "control_radius");
    assert_eq!(
        app.world().entity(stepper).get::<Value>().unwrap().as_f32(),
        Some(70.0)
    );
}

#[test]
fn a_theme_number_that_is_nan_still_has_a_row_at_its_minimum_and_a_warning_names_it() {
    let (mut app, warnings) = theme_editor_over(|theme| {
        theme.font_size_body = f32::NAN;
        theme.panel_radius = Val::Px(f32::INFINITY);
    });
    let count = {
        let world = app.world_mut();
        world.query::<&ThemeParameterEditor>().iter(world).count()
    };
    assert_eq!(count, theme_parameters().len(), "a row is missing");
    for (name, minimum) in [("font_size_body", 6.0), ("panel_radius", 0.0)] {
        let stepper = editor_stepper(&mut app, name);
        assert_eq!(
            app.world().entity(stepper).get::<Value>().unwrap().as_f32(),
            Some(minimum),
            "{name}"
        );
        assert_eq!(
            warnings.iter().filter(|w| w.contains(name)).count(),
            1,
            "{name}: {warnings:?}"
        );
    }
    // The theme keeps the value until a step sets it.
    assert!(
        app.world()
            .get_resource::<Theme>()
            .unwrap()
            .font_size_body
            .is_nan()
    );
    step_editor_to(&mut app, "font_size_body", 16.0);
    assert!(
        (app.world().get_resource::<Theme>().unwrap().font_size_body - 16.0).abs() < f32::EPSILON
    );
}

#[test]
fn a_theme_number_that_is_not_in_pixels_shows_0_and_is_warned_about_once() {
    let (mut app, warnings) = theme_editor_over(|theme| {
        theme.border_thickness = Val::Percent(50.0);
        theme.item_padding.left = Val::Auto;
    });
    let count = {
        let world = app.world_mut();
        world.query::<&ThemeParameterEditor>().iter(world).count()
    };
    assert_eq!(count, theme_parameters().len(), "a row is missing");
    let stepper = editor_stepper(&mut app, "border_thickness");
    assert_eq!(
        app.world().entity(stepper).get::<Value>().unwrap().as_f32(),
        Some(0.0)
    );
    for name in ["border_thickness", "item_padding.left"] {
        assert_eq!(
            warnings.iter().filter(|w| w.contains(name)).count(),
            1,
            "{name}: {warnings:?}"
        );
    }
    // The warning is not repeated when the theme changes again ...
    app.world_mut()
        .get_resource_mut::<Theme>()
        .unwrap()
        .panel_radius = Val::Px(12.0);
    let again = warnings_during(&mut app, 3);
    assert!(again.is_empty(), "{again:?}");
    // ... and stepping turns the value into pixels.
    step_editor_to(&mut app, "border_thickness", 2.0);
    assert_eq!(
        app.world()
            .get_resource::<Theme>()
            .unwrap()
            .border_thickness,
        Val::Px(2.0)
    );
}

#[test]
fn the_buttons_of_a_stepper_keep_the_default_cursor_and_a_drag_holds_the_resize_cursor() {
    use bevy::picking::cursor::{EntityCursor, OverrideCursor};
    use bevy::window::SystemCursorIcon;

    let (mut app, _, stepper, label) = scrub_stepper_app(0);
    give_width(&mut app, stepper);
    let kids: Vec<Entity> = app
        .world()
        .entity(stepper)
        .get::<Children>()
        .unwrap()
        .iter()
        .collect();
    for button in [kids[1], kids[3]] {
        assert_eq!(
            app.world().entity(button).get::<EntityCursor>(),
            Some(&EntityCursor::default()),
            "hovering a button does not change the cursor"
        );
    }
    assert_eq!(
        app.world().get_resource::<OverrideCursor>().unwrap().0,
        None
    );
    app.world_mut()
        .get_resource_mut::<ButtonInput<MouseButton>>()
        .unwrap()
        .press(MouseButton::Left);
    press_and_drag(&mut app, label, PointerButton::Primary, 50.0);
    assert_eq!(
        app.world().get_resource::<OverrideCursor>().unwrap().0,
        Some(EntityCursor::System(SystemCursorIcon::ColResize)),
        "a drag shows the resize cursor wherever the pointer is"
    );
    app.world_mut()
        .get_resource_mut::<ButtonInput<MouseButton>>()
        .unwrap()
        .release(MouseButton::Left);
    app.update();
    app.update();
    assert_eq!(
        app.world().get_resource::<OverrideCursor>().unwrap().0,
        None,
        "the cursor is released with the drag"
    );
}

#[test]
fn the_scrub_track_runs_between_the_buttons_and_the_width_of_a_stepper_can_be_set() {
    let mut app = headless_app();
    let (stepper, _, _) = spawn_stepper(&mut app, 1);
    let children = app.world().entity(stepper).get::<Children>().unwrap();
    let (track, label) = (children[0], children[2]);
    let node = app.world().entity(track).get::<Node>().unwrap();
    assert_eq!((node.left, node.right), (Val::Px(30.0), Val::Px(30.0)));
    assert!(
        (app.world().entity(label).get::<Node>().unwrap().flex_grow - 1.0).abs() < f32::EPSILON
    );

    app.world_mut()
        .entity_mut(stepper)
        .get_mut::<Node>()
        .unwrap()
        .width = Val::Px(300.0);
    app.update();
    assert_eq!(
        app.world().entity(stepper).get::<Node>().unwrap().width,
        Val::Px(300.0),
        "the widget leaves the app's width alone"
    );
}

#[test]
fn a_tooltip_stays_hidden_while_a_numeric_stepper_is_scrubbed() {
    let (mut app, _, stepper, label) = scrub_stepper_app(0);
    give_width(&mut app, stepper);
    let increase = app
        .world_mut()
        .query_filtered::<(Entity, &Tooltip), With<IconButton>>()
        .iter(app.world())
        .find(|(_, tip)| tip.text == "Increase")
        .map(|(entity, _)| entity)
        .unwrap();
    // The button stays down, or `end_scrub` ends the drag in the first frame.
    app.world_mut()
        .get_resource_mut::<ButtonInput<MouseButton>>()
        .unwrap()
        .press(MouseButton::Left);
    press_and_drag(&mut app, label, PointerButton::Primary, 20.0);
    app.world_mut().entity_mut(increase).insert(Hovered(true));
    for _ in 0..8 {
        tick(&mut app);
    }
    assert!(
        tooltip_popups(&mut app).is_empty(),
        "a tooltip showed during a scrub"
    );
}

/// Replaces the node as Bevy's own resets do (a plain button with no label), through `DerefMut`.
fn reset_accessibility_node(app: &mut App, entity: Entity) {
    let mut node = app
        .world_mut()
        .get_mut::<AccessibilityNode>(entity)
        .unwrap();
    *node = AccessibilityNode(accesskit::Node::new(accesskit::Role::Button));
}

fn accessibility_node_of(app: &App, entity: Entity) -> &AccessibilityNode {
    app.world().get::<AccessibilityNode>(entity).unwrap()
}

#[test]
fn filter_chips_restore_role_label_and_toggle_after_a_reset_and_follow_the_selection() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    let node = accessibility_node_of(&app, views[0]);
    assert_eq!(node.role(), accesskit::Role::RadioButton);
    assert_eq!(node.label(), Some("All 4"));
    assert_eq!(node.toggled(), Some(accesskit::Toggled::True));

    reset_accessibility_node(&mut app, views[1]);
    app.update();
    let node = accessibility_node_of(&app, views[1]);
    assert_eq!(node.role(), accesskit::Role::RadioButton);
    assert_eq!(node.label(), Some("Running 2"));
    assert_eq!(node.toggled(), Some(accesskit::Toggled::False));

    activate(&mut app, views[1]);
    app.update();
    assert_eq!(
        accessibility_node_of(&app, views[0]).toggled(),
        Some(accesskit::Toggled::False)
    );
    assert_eq!(
        accessibility_node_of(&app, views[1]).toggled(),
        Some(accesskit::Toggled::True)
    );
}

#[test]
fn filter_chips_leave_their_accessibility_nodes_alone_on_idle_frames_and_for_other_chips() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let (_, other) = spawn_chips(&mut app, three_chips());
    let views = chip_views(&app, chips);
    let others = chip_views(&app, other);
    let all: Vec<_> = views.iter().chain(&others).copied().collect();
    let before: Vec<_> = all
        .iter()
        .map(|e| accessibility_changed_at(&app, *e))
        .collect();
    app.update();
    app.update();
    let now: Vec<_> = all
        .iter()
        .map(|e| accessibility_changed_at(&app, *e))
        .collect();
    assert_eq!(before, now, "idle frames wrote a node");

    activate(&mut app, views[2]);
    app.update();
    for e in &others {
        assert_eq!(
            accessibility_changed_at(&app, *e),
            before[all.iter().position(|a| a == e).unwrap()],
            "a chip of another widget was written"
        );
    }
    assert_eq!(
        accessibility_node_of(&app, views[2]).toggled(),
        Some(accesskit::Toggled::True)
    );
}

#[test]
fn a_segmented_control_keeps_its_accessibility_nodes_right_through_resets_and_selection_changes() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let other = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);
    let others = segments_of(&app, other);
    let node = accessibility_node_of(&app, segments[0]);
    assert_eq!(node.role(), accesskit::Role::RadioButton);
    assert_eq!(node.label(), Some("Auto"));
    assert_eq!(node.toggled(), Some(accesskit::Toggled::True));

    let before: Vec<_> = others
        .iter()
        .map(|e| accessibility_changed_at(&app, *e))
        .collect();
    app.update();
    app.update();
    for (e, tick) in others.iter().zip(&before) {
        assert_eq!(accessibility_changed_at(&app, *e), *tick, "idle write");
    }

    reset_accessibility_node(&mut app, segments[1]);
    app.update();
    let node = accessibility_node_of(&app, segments[1]);
    assert_eq!(node.role(), accesskit::Role::RadioButton);
    assert_eq!(node.label(), Some("Manual"));
    assert_eq!(node.toggled(), Some(accesskit::Toggled::False));

    select_by_text(&mut app, control, "Manual");
    assert_eq!(
        accessibility_node_of(&app, segments[0]).toggled(),
        Some(accesskit::Toggled::False)
    );
    assert_eq!(
        accessibility_node_of(&app, segments[1]).toggled(),
        Some(accesskit::Toggled::True)
    );
    for (e, tick) in others.iter().zip(&before) {
        assert_eq!(accessibility_changed_at(&app, *e), *tick, "other control");
    }
}

#[test]
fn a_segment_label_follows_a_change_of_its_items_text_in_the_accessibility_node() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);
    let item = app
        .world()
        .get::<Segment>(segments[2])
        .map(Segment::item)
        .unwrap();
    app.world_mut().get_mut::<Item<String>>(item).unwrap().data = "Never".into();
    app.update();
    app.update();
    assert_eq!(
        accessibility_node_of(&app, segments[2]).label(),
        Some("Never")
    );
}

#[test]
fn a_gallery_keeps_its_card_accessibility_nodes_right_through_resets_and_leaves_idle_frames_alone()
{
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let cards: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<GalleryCardView>>()
        .iter(app.world())
        .collect();
    assert_eq!(cards.len(), 5);
    let _ = gallery;
    let before: Vec<_> = cards
        .iter()
        .map(|e| accessibility_changed_at(&app, *e))
        .collect();
    app.update();
    app.update();
    for (e, tick) in cards.iter().zip(&before) {
        assert_eq!(accessibility_changed_at(&app, *e), *tick, "idle write");
    }
    let card = cards[3];
    reset_accessibility_node(&mut app, card);
    app.update();
    let node = accessibility_node_of(&app, card);
    assert_eq!(node.role(), accesskit::Role::RadioButton);
    assert!(node.label().is_some());
    assert_eq!(node.toggled(), Some(accesskit::Toggled::False));
}

#[test]
fn a_link_and_buttons_restore_their_accessibility_nodes_after_a_reset_and_idle_frames_write_nothing()
 {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let link = app
        .world_mut()
        .spawn((Link::new("Docs"), ChildOf(root)))
        .id();
    let push = app
        .world_mut()
        .spawn((
            PushButton::from_label("Go"),
            ToggleState(true),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let ticks = [
        accessibility_changed_at(&app, link),
        accessibility_changed_at(&app, push),
    ];
    app.update();
    app.update();
    assert_eq!(accessibility_changed_at(&app, link), ticks[0], "link idle");
    assert_eq!(
        accessibility_changed_at(&app, push),
        ticks[1],
        "button idle"
    );

    reset_accessibility_node(&mut app, link);
    reset_accessibility_node(&mut app, push);
    app.update();
    let node = accessibility_node_of(&app, link);
    assert_eq!(node.role(), accesskit::Role::Link);
    assert_eq!(node.label(), Some("Docs"));
    let node = accessibility_node_of(&app, push);
    assert_eq!(node.label(), Some("Go"));
    assert_eq!(node.toggled(), Some(accesskit::Toggled::True));

    app.world_mut().entity_mut(push).remove::<ToggleState>();
    app.update();
    assert_eq!(accessibility_node_of(&app, push).toggled(), None);
}

/// Spawns `controls` icon-only segmented controls of `segments` items each, and returns them.
fn spawn_many_icon_controls(app: &mut App, controls: usize, segments: usize) -> Vec<Entity> {
    let root = app.world_mut().spawn(Node::default()).id();
    let ids: Vec<Entity> = (0..controls)
        .map(|_| {
            app.world_mut()
                .spawn((
                    SegmentedControl,
                    SegmentLabels::Hidden,
                    Selection::builder()
                        .one()
                        .build_with_items((0..segments).map(|i| format!("S{i}")).map(Item::new)),
                    ChildOf(root),
                ))
                .id()
        })
        .collect();
    for _ in 0..3 {
        app.update();
    }
    ids
}

#[test]
fn many_icon_segmented_controls_keep_one_icon_and_hidden_label_per_segment() {
    let mut app = headless_app();
    let controls = spawn_many_icon_controls(&mut app, 30, 40);
    let items: Vec<Entity> = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<Item<String>>>()
            .iter(world)
            .collect()
    };
    for item in &items {
        app.world_mut()
            .entity_mut(*item)
            .insert(ItemIcon(Handle::default()));
    }
    for _ in 0..3 {
        app.update();
    }
    for _ in 0..20 {
        app.world_mut()
            .entity_mut(items[0])
            .insert(ItemIcon(Handle::default()));
        app.update();
    }
    for control in &controls {
        for segment in segments_of(&app, *control) {
            let icons = app
                .world()
                .entity(segment)
                .get::<Children>()
                .unwrap()
                .iter()
                .filter(|c| app.world().entity(*c).contains::<SegmentIcon>())
                .count();
            assert_eq!(icons, 1);
            assert_eq!(label_display(&app, segment), Display::None);
        }
    }
    for item in &items {
        app.world_mut().entity_mut(*item).remove::<ItemIcon>();
    }
    for _ in 0..3 {
        app.update();
    }
    for control in &controls {
        for segment in segments_of(&app, *control) {
            assert!(icon_of(&app, segment).is_none());
            assert_eq!(label_display(&app, segment), Display::Flex);
        }
    }
}

/// How many children of `track` are named `name`: the sliding indicators of a track.
fn indicators_named(app: &App, track: Entity, name: &str) -> Vec<Entity> {
    app.world()
        .entity(track)
        .get::<Children>()
        .map(|children| {
            children
                .iter()
                .filter(|c| {
                    app.world()
                        .get::<Name>(*c)
                        .is_some_and(|n| n.as_str() == name)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// One of each track widget that has a sliding indicator, with the indicator's name.
fn spawn_indicator_tracks(app: &mut App) -> [(Entity, &'static str); 3] {
    let root = app.world_mut().spawn(Node::default()).id();
    let items = || {
        Selection::builder()
            .one()
            .build_with_items(["A", "B"].map(Item::new))
    };
    let control = app
        .world_mut()
        .spawn((SegmentedControl, items(), ChildOf(root)))
        .id();
    let bar = app.world_mut().spawn((TabBar, items(), ChildOf(root))).id();
    let toggle = app.world_mut().spawn((LightDarkToggle, ChildOf(root))).id();
    for _ in 0..3 {
        app.update();
    }
    [
        (control, "SegmentIndicator"),
        (bar, "TabIndicator"),
        (toggle, "OptionIndicator"),
    ]
}

#[test]
fn every_track_has_exactly_one_indicator_after_reinsertions() {
    let mut app = headless_app();
    let tracks = spawn_indicator_tracks(&mut app);
    let first = tracks.map(|(t, n)| indicators_named(&app, t, n));
    for indicators in &first {
        assert_eq!(indicators.len(), 1);
    }
    for _ in 0..3 {
        app.world_mut()
            .entity_mut(tracks[0].0)
            .insert(SegmentedControl);
        app.world_mut().entity_mut(tracks[1].0).insert(TabBar);
        app.world_mut()
            .entity_mut(tracks[2].0)
            .insert(LightDarkToggle);
        app.update();
        app.update();
    }
    for ((track, name), before) in tracks.into_iter().zip(first) {
        assert_eq!(indicators_named(&app, track, name), before);
    }
}

#[test]
fn a_removed_indicator_is_made_again_once_and_other_tracks_keep_theirs() {
    let mut app = headless_app();
    let tracks = spawn_indicator_tracks(&mut app);
    let before = tracks.map(|(t, n)| indicators_named(&app, t, n)[0]);
    for (i, (track, name)) in tracks.into_iter().enumerate() {
        app.world_mut().despawn(before[i]);
        app.update();
        app.update();
        app.update();
        let now = indicators_named(&app, track, name);
        assert_eq!(now.len(), 1, "{name} made again exactly once");
        assert_ne!(now[0], before[i]);
        for (j, (other, other_name)) in tracks.into_iter().enumerate() {
            if j > i {
                assert_eq!(indicators_named(&app, other, other_name), vec![before[j]]);
            }
        }
    }
}

#[test]
fn despawning_a_track_leaves_the_indicators_of_the_others_alone() {
    let mut app = headless_app();
    let tracks = spawn_indicator_tracks(&mut app);
    let before = tracks.map(|(t, n)| indicators_named(&app, t, n)[0]);
    app.world_mut().entity_mut(tracks[0].0).despawn();
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().get_entity(before[0]).is_err());
    assert_eq!(
        indicators_named(&app, tracks[1].0, "TabIndicator"),
        vec![before[1]]
    );
    assert_eq!(
        indicators_named(&app, tracks[2].0, "OptionIndicator"),
        vec![before[2]]
    );
}

#[test]
fn a_track_that_gains_its_widget_later_gets_one_indicator() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let control = app
        .world_mut()
        .spawn((
            Node::default(),
            Selection::builder()
                .one()
                .build_with_items(["A"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    app.world_mut().entity_mut(control).insert(SegmentedControl);
    for _ in 0..3 {
        app.update();
    }
    let indicators = indicators_named(&app, control, "SegmentIndicator");
    assert_eq!(indicators.len(), 1);
    assert_eq!(
        app.world()
            .get::<SegmentIndicator>(indicators[0])
            .unwrap()
            .owner(),
        control
    );
}

/// The labels shown by the views of `list`, in the order of the list.
fn view_labels(app: &App, list: Entity) -> Vec<String> {
    item_views(app, list)
        .into_iter()
        .map(|view| item_label(app, app.world().get::<ItemView>(view).unwrap().item_ref))
        .collect()
}

#[test]
fn a_list_keeps_its_views_and_their_children_when_items_are_added_and_removed() {
    let mut app = headless_app();
    let (list, items) = spawn_custom_list(&mut app, Selection::builder().one());
    let before = item_views(&app, list);
    assert_eq!(before.len(), 2);
    let app_child = app.world_mut().spawn(Name::new("app child")).id();
    app.world_mut().entity_mut(before[0]).add_child(app_child);
    app.world_mut()
        .get_resource_mut::<CreatedViews>()
        .unwrap()
        .0
        .clear();

    app.world_mut().spawn((Item::new("c"), ItemOf(items)));
    app.update();
    app.update();
    let after = item_views(&app, list);
    assert_eq!(view_labels(&app, list), ["a", "b", "c"]);
    assert_eq!(
        after[..2],
        before[..],
        "existing views are the same entities"
    );
    assert!(app.world().get::<ChildOf>(app_child).is_some());
    let created = &app.world().get_resource::<CreatedViews>().unwrap().0;
    assert_eq!(created.len(), 1, "the creator runs for the new item only");
    assert_eq!(created[0].1, after[2]);

    let b = app.world().get::<ItemView>(after[1]).unwrap().item_ref;
    app.world_mut().entity_mut(b).despawn();
    app.update();
    app.update();
    assert_eq!(view_labels(&app, list), ["a", "c"]);
    assert!(app.world().get_entity(after[1]).is_err());
    assert_eq!(item_views(&app, list), [after[0], after[2]]);
    assert!(app.world().get::<ChildOf>(app_child).is_some());
}

#[test]
fn a_list_despawns_a_second_view_of_the_same_item() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let items = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["a", "b"].map(Item::new)),
        )
        .id();
    let list = app
        .world_mut()
        .spawn((List, Selection::reference(items), ChildOf(root)))
        .id();
    app.update();
    app.update();
    let views = item_views(&app, list);
    let item = app.world().get::<ItemView>(views[0]).unwrap().item_ref;
    let twin = app
        .world_mut()
        .spawn((
            ItemView {
                item_ref: item,
                list_ref: list,
            },
            ChildOf(list),
        ))
        .id();
    app.world_mut().spawn((Item::new("c"), ItemOf(items)));
    app.update();
    app.update();
    assert!(app.world().get_entity(twin).is_err());
    assert_eq!(view_labels(&app, list), ["a", "b", "c"]);
    assert_eq!(item_views(&app, list)[..2], views[..]);
}

#[test]
fn a_list_of_thousands_of_items_gets_one_view_each_and_marks_the_selected_one() {
    const COUNT: usize = 4_000;
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let items = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items((0..COUNT).map(|i| Item::new(format!("item {i}")))),
        )
        .id();
    let list = app
        .world_mut()
        .spawn((List, Selection::reference(items), ChildOf(root)))
        .id();
    app.update();
    app.update();
    let labels = view_labels(&app, list);
    assert_eq!(labels.len(), COUNT);
    assert_eq!(labels.first().map(String::as_str), Some("item 0"));
    assert_eq!(labels.last().map(String::as_str), Some("item 3999"));
    assert_eq!(selected_labels(&app, list), ["item 0"]);

    let first_views = item_views(&app, list);
    app.world_mut().spawn((Item::new("last"), ItemOf(items)));
    app.update();
    app.update();
    let views = item_views(&app, list);
    assert_eq!(views.len(), COUNT + 1);
    assert_eq!(views[..COUNT], first_views[..]);
    assert_eq!(selected_labels(&app, list), ["item 0"]);
}

/// A closed palette combo over `selection` (two colours): the combo and its palette.
fn closed_palette_combo_over(app: &mut App, selection: impl Bundle) -> (Entity, Entity) {
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, selection, ChildOf(root)))
        .id();
    app.update();
    app.update();
    let palette = single_entity_with::<ColorPalette>(app);
    (combo, palette)
}

#[test]
fn a_closed_color_palette_combo_builds_no_swatches_until_it_opens() {
    let mut app = headless_app();
    let (combo, palette) = closed_palette_combo_over(&mut app, two_color_selection());
    assert!(item_views(&app, palette).is_empty());

    let button = single_entity_with::<ColorPushButton>(&mut app);
    click(&mut app, button);
    assert_eq!(palette_combo_state(&app, combo), PaletteComboState::Open);
    assert_eq!(item_views(&app, palette).len(), 2);

    click(&mut app, button);
    app.update();
    assert_eq!(palette_combo_state(&app, combo), PaletteComboState::Closed);
    assert!(item_views(&app, palette).is_empty());

    click(&mut app, button);
    assert_eq!(item_views(&app, palette).len(), 2);
}

#[test]
fn a_reopened_color_palette_combo_shows_the_selection_changed_while_it_was_closed() {
    let mut app = headless_app();
    let (_, palette) = closed_palette_combo_over(&mut app, two_color_selection());
    let button = single_entity_with::<ColorPushButton>(&mut app);
    // Pick black while closed, through the palette's own selection.
    let black = {
        let world = app.world_mut();
        let mut items = world.query::<(Entity, &Item<Color>)>();
        items
            .iter(world)
            .find(|(_, item)| item.data == Color::BLACK)
            .map(|(entity, _)| entity)
            .unwrap()
    };
    let owner = single_entity_with::<ColorPaletteCombo>(&mut app);
    app.world_mut()
        .entity_mut(owner)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(black))));
    app.update();
    assert!(item_views(&app, palette).is_empty());

    click(&mut app, button);
    assert_eq!(selected_colors(&app, palette), [Color::BLACK]);
}

#[test]
fn items_added_while_a_color_palette_combo_is_closed_have_swatches_once_it_opens() {
    let mut app = headless_app();
    let (_, palette) = closed_palette_combo_over(&mut app, two_color_selection());
    let button = single_entity_with::<ColorPushButton>(&mut app);
    let items = app.world().get::<ItemsRef>(palette).map(|r| r.0);
    let owner = items.unwrap_or_else(|| single_entity_with::<ColorPaletteCombo>(&mut app));
    app.world_mut().spawn((Item { data: RED }, ItemOf(owner)));
    app.update();
    assert!(item_views(&app, palette).is_empty());

    click(&mut app, button);
    assert_eq!(item_views(&app, palette).len(), 3);

    // And while open the palette follows further changes.
    app.world_mut().spawn((Item { data: BLUE }, ItemOf(owner)));
    app.update();
    app.update();
    assert_eq!(item_views(&app, palette).len(), 4);
}

#[test]
fn an_open_color_palette_combo_popup_is_visible_and_takes_the_focus() {
    let mut app = headless_app();
    let (combo, palette, _) = open_palette_combo_over(&mut app, two_color_selection());
    app.update();
    let float = app
        .world()
        .get::<ColorPaletteComboRefs>(combo)
        .map(ColorPaletteComboRefs::popup)
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(float),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .get_resource::<InputFocus>()
            .and_then(InputFocus::get),
        Some(palette)
    );
}

#[test]
fn despawning_color_palette_combos_open_or_dormant_leaves_no_swatches_behind() {
    let mut app = headless_app();
    let (open, _, _) = open_palette_combo_over(&mut app, two_color_selection());
    app.world_mut().entity_mut(open).despawn();
    app.update();
    app.update();
    let mut views = app.world_mut().query::<&ItemView>();
    assert_eq!(views.iter(app.world()).count(), 0);

    let mut app = headless_app();
    let (closed, _) = closed_palette_combo_over(&mut app, two_color_selection());
    app.world_mut().entity_mut(closed).despawn();
    app.update();
    let mut palettes = app.world_mut().query::<&ColorPalette>();
    assert_eq!(palettes.iter(app.world()).count(), 0);
}

/// Removes the items called `names` from `source` and adds one for each of `added`, in one frame.
fn swap_items(app: &mut App, source: Entity, removed: &[&str], added: &[&str]) {
    let doomed: Vec<Entity> = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &Item<String>, &ItemOf)>();
        query
            .iter(world)
            .filter(|(_, item, of)| of.0 == source && removed.contains(&item.data.as_str()))
            .map(|(entity, ..)| entity)
            .collect()
    };
    for entity in doomed {
        app.world_mut().entity_mut(entity).despawn();
    }
    for name in added {
        app.world_mut().spawn((Item::new(*name), ItemOf(source)));
    }
    for _ in 0..3 {
        app.update();
    }
}

#[test]
fn a_segmented_control_keeps_the_segments_of_kept_items_and_a_child_of_the_app_through_a_swap() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let before = segments_of(&app, control);
    let own = app.world_mut().spawn(ChildOf(control)).id();
    app.update();
    swap_items(&mut app, control, &["Manual"], &["New"]);
    let after = segments_of(&app, control);
    assert_eq!(segment_texts(&app, control), ["Auto", "Off", "New"]);
    assert_eq!(after[..2], [before[0], before[2]]);
    assert!(app.world().get_entity(before[1]).is_err());
    assert!(app.world().get_entity(own).is_ok());
    assert_eq!(selected_segments(&app, control), ["Auto"]);
}

#[test]
fn a_tab_bar_keeps_the_tabs_of_kept_items_and_a_child_of_the_app_through_a_swap() {
    let mut app = headless_app();
    let bar = spawn_tab_bar(
        &mut app,
        Selection::builder()
            .one()
            .build_with_items(["General", "Audio", "Input"].map(Item::new)),
    );
    let before = tabs_of(&app, bar);
    let own = app.world_mut().spawn(ChildOf(bar)).id();
    app.update();
    swap_items(&mut app, bar, &["Audio"], &["New"]);
    let after = tabs_of(&app, bar);
    let texts: Vec<_> = after.iter().map(|t| segment_text(&app, *t)).collect();
    assert_eq!(texts, ["General", "Input", "New"]);
    assert_eq!(after[..2], [before[0], before[2]]);
    assert!(app.world().get_entity(before[1]).is_err());
    assert!(app.world().get_entity(own).is_ok());
    assert_eq!(selected_tabs(&app, bar), ["General"]);
}

#[test]
fn thousands_of_tabs_and_segments_are_built_and_reordered_in_item_order() {
    const COUNT: usize = 3_000;
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let names: Vec<String> = (0..COUNT).map(|i| format!("Item {i}")).collect();
    let bar = app
        .world_mut()
        .spawn((
            TabBar,
            Selection::builder()
                .one()
                .build_with_items(names.clone().into_iter().map(Item::new)),
            ChildOf(root),
        ))
        .id();
    let control = app
        .world_mut()
        .spawn((
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(names.clone().into_iter().map(Item::new)),
            ChildOf(root),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    let texts: Vec<_> = tabs_of(&app, bar)
        .iter()
        .map(|t| segment_text(&app, *t))
        .collect();
    assert_eq!(texts, names);
    assert_eq!(segment_texts(&app, control), names);
    // one more item at the front: every existing child stays, in place after it
    let before = tabs_of(&app, bar);
    let first = app
        .world()
        .entity(before[0])
        .get::<TabBarTab>()
        .unwrap()
        .item();
    let new = app.world_mut().spawn(Item::new("Front")).id();
    app.world_mut()
        .entity_mut(bar)
        .insert_related::<ItemOf>(0, &[new]);
    for _ in 0..3 {
        app.update();
    }
    let after = tabs_of(&app, bar);
    assert_eq!(after.len(), COUNT + 1);
    assert_eq!(segment_text(&app, after[0]), "Front");
    assert_eq!(after[1..], before[..]);
    assert_eq!(
        app.world()
            .entity(after[1])
            .get::<TabBarTab>()
            .unwrap()
            .item(),
        first
    );
}

#[test]
fn a_palette_despawns_a_second_swatch_of_one_item_and_keeps_the_others() {
    let mut app = headless_app();
    let root = app.world_mut().spawn(Node::default()).id();
    let colors = [Color::WHITE, Color::BLACK];
    let items = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(colors.map(|data| Item { data })),
        )
        .id();
    let palette = app
        .world_mut()
        .spawn((ColorPalette, Selection::reference(items), ChildOf(root)))
        .id();
    app.update();
    app.update();
    let views = item_views(&app, palette);
    assert_eq!(views.len(), 2);
    let item = app.world().get::<ItemView>(views[0]).unwrap().item_ref;
    let twin = app
        .world_mut()
        .spawn((
            ItemView {
                item_ref: item,
                list_ref: palette,
            },
            ChildOf(palette),
        ))
        .id();
    app.world_mut()
        .spawn((Item { data: Color::BLACK }, ItemOf(items)));
    app.update();
    app.update();
    assert!(app.world().get_entity(twin).is_err());
    let after = item_views(&app, palette);
    assert_eq!(after.len(), 3);
    assert_eq!(after[..2], views[..]);
}

/// Entities the pointers are on, for [`wheel_app`]: the headless app has no picking backend, so
/// the hover map is written by hand after picking has rebuilt it.
#[derive(Resource, Default)]
struct FakeHover(Vec<(PointerId, Entity)>);

fn write_fake_hover(fake: Res<FakeHover>, mut hover_map: ResMut<bevy::picking::hover::HoverMap>) {
    for (pointer, entity) in &fake.0 {
        hover_map
            .entry(*pointer)
            .or_default()
            .insert(*entity, HitData::new(Entity::PLACEHOLDER, 0.0, None, None));
    }
}

fn wheel_app() -> App {
    let mut app = headless_app_with(|app| {
        app.init_resource::<FakeHover>();
        app.add_systems(
            PreUpdate,
            write_fake_hover.after(bevy::picking::PickingSystems::Hover),
        );
    });
    app.update();
    app
}

fn hover(app: &mut App, pointers: &[(PointerId, Entity)]) {
    app.world_mut().get_resource_mut::<FakeHover>().unwrap().0 = pointers.to_vec();
}

fn wheel(app: &mut App, unit: bevy::input::mouse::MouseScrollUnit, x: f32, y: f32) {
    app.world_mut()
        .write_message(bevy::input::mouse::MouseWheel {
            unit,
            x,
            y,
            window: Entity::PLACEHOLDER,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
}

#[test]
fn many_wheel_messages_in_one_frame_are_applied_and_clamped_one_after_another() {
    use bevy::input::mouse::MouseScrollUnit::Pixel;
    // The area scrolls 300 px at most. Down 250, down 250 (clamped to 300), up 100: 200. A sum
    // (400 down) would give 300, so the order and the clamp in between show.
    let mut app = wheel_app();
    let area = scrolling_contents(&mut app, None, Overflow::scroll_y(), Vec2::ZERO, 1.0);
    hover(&mut app, &[(PointerId::Mouse, area)]);
    wheel(&mut app, Pixel, 0.0, -250.0);
    wheel(&mut app, Pixel, 0.0, -250.0);
    wheel(&mut app, Pixel, 0.0, 100.0);
    app.update();
    assert_eq!(scroll_of(&app, area), Vec2::new(0.0, 200.0));
}

#[test]
fn a_wheel_line_is_21_pixels_and_control_swaps_the_axes() {
    use bevy::input::mouse::MouseScrollUnit::Line;
    let mut app = wheel_app();
    let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, 1.0);
    hover(&mut app, &[(PointerId::Mouse, area)]);
    wheel(&mut app, Line, 0.0, -2.0);
    app.update();
    assert_eq!(scroll_of(&app, area), Vec2::new(0.0, 42.0));
    app.world_mut()
        .get_resource_mut::<ButtonInput<KeyCode>>()
        .unwrap()
        .press(KeyCode::ControlLeft);
    wheel(&mut app, Line, 0.0, -1.0);
    app.update();
    assert_eq!(scroll_of(&app, area), Vec2::new(21.0, 42.0));
}

#[test]
fn an_area_that_scrolls_sideways_alone_takes_a_vertical_turn_for_a_horizontal_one() {
    use bevy::input::mouse::MouseScrollUnit::Pixel;
    let mut app = wheel_app();
    let area = scrolling_contents(&mut app, None, SCROLL_X, Vec2::ZERO, 1.0);
    hover(&mut app, &[(PointerId::Mouse, area)]);
    wheel(&mut app, Pixel, 0.0, -30.0);
    wheel(&mut app, Pixel, 0.0, -30.0);
    app.update();
    assert_eq!(scroll_of(&app, area), Vec2::new(60.0, 0.0));
}

#[test]
fn only_the_innermost_hovered_scroll_area_takes_the_wheel_also_at_its_limit() {
    use bevy::input::mouse::MouseScrollUnit::Pixel;
    let mut app = wheel_app();
    let outer = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, 1.0);
    let inner = scrolling_contents(&mut app, Some(outer), SCROLL_BOTH, Vec2::ZERO, 1.0);
    let leaf = target_at(&mut app, Some(inner), Vec2::ZERO, 1.0);
    // Picking reports the leaf, the inner and the outer area, as it does for a blocking area.
    hover(
        &mut app,
        &[
            (PointerId::Mouse, leaf),
            (PointerId::Mouse, inner),
            (PointerId::Mouse, outer),
        ],
    );
    // 400 down: the inner area stops at 300, nothing is passed on to the outer one.
    wheel(&mut app, Pixel, 0.0, -200.0);
    wheel(&mut app, Pixel, 0.0, -200.0);
    app.update();
    assert_eq!(scroll_of(&app, inner), Vec2::new(0.0, 300.0));
    assert_eq!(scroll_of(&app, outer), Vec2::ZERO);
}

/// Counts the frames in which a `ScrollPosition` was marked changed.
#[derive(Resource, Default)]
struct ScrollChanges(usize);

fn count_scroll_changes(
    changed: Query<(), Changed<ScrollPosition>>,
    mut count: ResMut<ScrollChanges>,
) {
    count.0 += changed.iter().count();
}

fn scroll_changes_in(app: &mut App, wheels: &[(f32, f32)]) -> usize {
    use bevy::input::mouse::MouseScrollUnit::Pixel;
    app.update();
    app.world_mut()
        .get_resource_mut::<ScrollChanges>()
        .unwrap()
        .0 = 0;
    for &(x, y) in wheels {
        wheel(app, Pixel, x, y);
    }
    app.update();
    app.world().get_resource::<ScrollChanges>().unwrap().0
}

#[test]
fn a_wheel_that_cannot_move_a_scroll_area_does_not_mark_its_position_changed() {
    let mut app = wheel_app();
    app.init_resource::<ScrollChanges>();
    app.add_systems(Last, count_scroll_changes);
    let area = scrolling_contents(&mut app, None, SCROLL_BOTH, Vec2::ZERO, 1.0);
    hover(&mut app, &[(PointerId::Mouse, area)]);
    // Up and left at the top left corner: clamped to where it is.
    assert_eq!(scroll_changes_in(&mut app, &[(30.0, 30.0)]), 0);
    // No delta at all.
    assert_eq!(scroll_changes_in(&mut app, &[(0.0, 0.0)]), 0);
    // A real scroll still marks it, once, however many messages.
    assert_eq!(
        scroll_changes_in(&mut app, &[(0.0, -50.0), (0.0, -50.0)]),
        1
    );
    assert_eq!(scroll_of(&app, area), Vec2::new(0.0, 100.0));
    // At the far limit, further turns change nothing.
    assert_eq!(scroll_changes_in(&mut app, &[(0.0, -1000.0)]), 1);
    assert_eq!(scroll_changes_in(&mut app, &[(0.0, -1000.0)]), 0);
    assert_eq!(scroll_of(&app, area), Vec2::new(0.0, 300.0));
}

#[test]
fn a_wheel_at_the_limit_leaves_an_open_combo_in_the_scroll_area_open() {
    let mut app = wheel_app();
    let area = scrolling_contents(&mut app, None, Overflow::scroll_y(), Vec2::ZERO, 1.0);
    let combo = app
        .world_mut()
        .spawn((
            Combo,
            Selection::builder()
                .zero_or_one()
                .build_with_items(["a", "b"].map(|s| Item::new(s.to_string()))),
            ChildOf(area),
        ))
        .id();
    app.update();
    app.update();
    hover(&mut app, &[(PointerId::Mouse, area)]);
    app.world_mut().entity_mut(combo).insert(ComboState::Open);
    app.update();
    app.update();
    // Already at the top: scrolling up moves nothing, so the combo stays open.
    wheel(
        &mut app,
        bevy::input::mouse::MouseScrollUnit::Pixel,
        0.0,
        40.0,
    );
    app.update();
    assert_eq!(
        app.world().get::<ComboState>(combo),
        Some(&ComboState::Open)
    );
    // A scroll that moves closes it.
    wheel(
        &mut app,
        bevy::input::mouse::MouseScrollUnit::Pixel,
        0.0,
        -40.0,
    );
    app.update();
    app.update();
    assert_eq!(
        app.world().get::<ComboState>(combo),
        Some(&ComboState::Closed)
    );
}

#[test]
fn a_fixed_node_stops_the_search_for_a_scroll_area_above_it() {
    use bevy::input::mouse::MouseScrollUnit::Pixel;
    let mut app = wheel_app();
    let area = scrolling_contents(&mut app, None, Overflow::scroll_y(), Vec2::ZERO, 1.0);
    let popup = app
        .world_mut()
        .spawn((Node::default(), FixedNode, ChildOf(area)))
        .id();
    let item = target_at(&mut app, Some(popup), Vec2::ZERO, 1.0);
    hover(&mut app, &[(PointerId::Mouse, item)]);
    wheel(&mut app, Pixel, 0.0, -50.0);
    app.update();
    assert_eq!(scroll_of(&app, area), Vec2::ZERO);
}

#[test]
fn an_area_two_pointers_are_on_scrolls_once_for_each_pointer_and_message() {
    use bevy::input::mouse::MouseScrollUnit::Pixel;
    let mut app = wheel_app();
    let area = scrolling_contents(&mut app, None, Overflow::scroll_y(), Vec2::ZERO, 1.0);
    hover(
        &mut app,
        &[(PointerId::Mouse, area), (PointerId::Touch(0), area)],
    );
    wheel(&mut app, Pixel, 0.0, -10.0);
    wheel(&mut app, Pixel, 0.0, -10.0);
    app.update();
    assert_eq!(scroll_of(&app, area), Vec2::new(0.0, 40.0));
}

#[test]
fn reinserting_a_numeric_stepper_while_its_number_editor_is_open_keeps_app_children_and_leaves_no_stale_editor()
 {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};

    let mut app = headless_app_with(|app| {
        app.add_plugins((bevy::text::TextPlugin, bevy::ui_widgets::TextInputPlugin));
    });
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.world_mut().spawn(Camera2d);
    let root = app.world_mut().spawn(Node::default()).id();
    let stepper = app
        .world_mut()
        .spawn((
            NumericStepper::default(),
            Value::new_u8(1, 0, 3, 1).unwrap(),
            ChildOf(root),
        ))
        .id();
    let mine = app
        .world_mut()
        .spawn((Name::new("Mine"), ChildOf(stepper)))
        .id();
    app.update();
    app.update();
    let key = |app: &mut App, code: KeyCode, logical: Key| {
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: logical,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
        app.update();
    };

    // F2 opens the number editor, which takes the focus.
    app.world_mut()
        .insert_resource(InputFocus::from_entity(stepper));
    key(&mut app, KeyCode::F2, Key::F2);
    let editors = |app: &mut App| -> Vec<Entity> {
        let mut query = app.world_mut().query::<(Entity, &TextFieldRefs)>();
        query.iter(app.world()).map(|(entity, _)| entity).collect()
    };
    let opened = editors(&mut app);
    assert_eq!(opened.len(), 1, "F2 opens one editor");
    let input = app
        .world()
        .entity(opened[0])
        .get::<TextFieldRefs>()
        .unwrap()
        .input;
    assert_eq!(
        app.world()
            .get_resource::<InputFocus>()
            .and_then(InputFocus::get),
        Some(input)
    );

    // Replace the stepper while the editor is open and focused.
    app.world_mut()
        .entity_mut(stepper)
        .insert(NumericStepper::default());
    app.update();
    app.update();
    app.update();

    assert!(app.world().get_entity(mine).is_ok(), "the app child stays");
    assert!(
        editors(&mut app).is_empty(),
        "no editor of the stepper is left"
    );
    assert!(
        app.world().get_entity(opened[0]).is_err(),
        "the old editor is despawned"
    );
    let labels: Vec<Entity> = app
        .world()
        .entity(stepper)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|child| {
            app.world()
                .entity(*child)
                .get::<Name>()
                .is_some_and(|name| name.as_str() == "Value")
        })
        .collect();
    assert_eq!(labels.len(), 1, "exactly one value label");
    assert_eq!(
        app.world().entity(labels[0]).get::<Node>().unwrap().display,
        Display::Flex,
        "the label shows"
    );
    if let Some(focused) = app
        .world()
        .get_resource::<InputFocus>()
        .and_then(InputFocus::get)
    {
        assert!(
            app.world().get_entity(focused).is_ok(),
            "focus is not on a despawned entity"
        );
    }

    // The stepper is not stuck as "editor open": the arrow keys step it again.
    app.world_mut()
        .insert_resource(InputFocus::from_entity(stepper));
    key(&mut app, KeyCode::ArrowUp, Key::ArrowUp);
    assert_eq!(stepper_value(&app, stepper), 2);
    // And F2 opens a fresh editor.
    key(&mut app, KeyCode::F2, Key::F2);
    assert_eq!(editors(&mut app).len(), 1);
}

/// The text of the label of the row `view` (its first child with a `Text`).
fn row_text(app: &App, view: Entity) -> String {
    app.world()
        .get::<Children>(view)
        .unwrap()
        .iter()
        .find_map(|child| app.world().get::<Text>(child).map(|text| text.0.clone()))
        .unwrap()
}

/// The views of `list`, in order, with their entities.
fn views_of_list(app: &mut App, list: Entity) -> Vec<Entity> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &ItemView)>();
    query
        .iter(world)
        .filter(|(_, view)| view.list_ref == list)
        .map(|(entity, _)| entity)
        .collect()
}

fn rename(app: &mut App, item: Entity, text: &str) {
    app.world_mut().get_mut::<Item<String>>(item).unwrap().data = text.to_string();
    for _ in 0..4 {
        app.update();
    }
}

#[test]
fn renaming_an_item_updates_the_visible_and_accessible_text_of_every_list_that_shows_it() {
    let mut app = headless_app();
    app.world_mut().spawn((Camera2d, UiRoot));
    let first = app
        .world_mut()
        .spawn((
            Node::default(),
            List,
            Selection::builder()
                .one()
                .build_with_items(["Alpha", "Beta"].map(|s| Item::new(s.to_string()))),
        ))
        .id();
    let sharing = app
        .world_mut()
        .spawn((
            Node::default(),
            List,
            Selection::builder().one().build_with_items_ref(first),
        ))
        .id();
    let other = app
        .world_mut()
        .spawn((
            Node::default(),
            List,
            Selection::builder()
                .one()
                .build_with_items(["Alpha", "Gamma"].map(|s| Item::new(s.to_string()))),
        ))
        .id();
    for _ in 0..4 {
        app.update();
    }
    let before_first = views_of_list(&mut app, first);
    let before_sharing = views_of_list(&mut app, sharing);
    let before_other = views_of_list(&mut app, other);
    let by_text = |app: &App, views: &[Entity], text: &str| {
        *views
            .iter()
            .find(|view| row_text(app, **view) == text)
            .unwrap()
    };
    let alpha_first = by_text(&app, &before_first, "Alpha");
    let alpha_sharing = by_text(&app, &before_sharing, "Alpha");
    let beta_first = by_text(&app, &before_first, "Beta");
    let alpha_other = by_text(&app, &before_other, "Alpha");
    let item = app.world().get::<ItemView>(alpha_first).unwrap().item_ref;

    rename(&mut app, item, "Alfa");

    for (list, before) in [
        (first, &before_first),
        (sharing, &before_sharing),
        (other, &before_other),
    ] {
        let mut now = views_of_list(&mut app, list);
        let mut was = before.clone();
        now.sort();
        was.sort();
        assert_eq!(now, was, "rows were rebuilt");
    }
    for view in [alpha_first, alpha_sharing] {
        assert_eq!(row_text(&app, view), "Alfa");
        assert_eq!(accessible_label(&app, view).as_deref(), Some("Alfa"));
    }
    assert_eq!(row_text(&app, beta_first), "Beta");
    assert_eq!(row_text(&app, alpha_other), "Alpha");
}

#[test]
fn renaming_the_selected_item_updates_a_combos_button_and_popup_but_not_its_placeholder() {
    let mut app = headless_app();
    let (combo, button) = spawn_combo(&mut app);
    let list = app.world().get::<ComboRefs>(combo).unwrap().list();
    assert_eq!(combo_label(&app, button), "Select..");
    let views = views_of_list(&mut app, list);
    let a = app.world().get::<ItemView>(views[0]).unwrap().item_ref;
    let b = app.world().get::<ItemView>(views[1]).unwrap().item_ref;

    // Nothing selected: a rename shows in the popup and leaves the placeholder.
    rename(&mut app, a, "first");
    assert_eq!(combo_label(&app, button), "Select..");
    assert_eq!(row_text(&app, views[0]), "first");

    app.world_mut()
        .entity_mut(combo)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(a))));
    for _ in 0..4 {
        app.update();
    }
    assert_eq!(combo_label(&app, button), "first");

    rename(&mut app, a, "renamed");
    assert_eq!(combo_label(&app, button), "renamed");
    assert_eq!(accessible_value(&app, button).as_deref(), Some("renamed"));
    assert_eq!(row_text(&app, views[0]), "renamed");

    // An item that is not selected leaves the button alone.
    rename(&mut app, b, "other");
    assert_eq!(combo_label(&app, button), "renamed");
    assert_eq!(row_text(&app, views[1]), "other");
}

#[test]
fn renaming_an_item_updates_a_filter_combos_button_rows_and_matches() {
    let mut fa = filter_app();
    let delft = fa.item_of("Delft");
    fa.app
        .world_mut()
        .entity_mut(fa.combo)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(delft))));
    fa.settle();
    assert_eq!(fa.button_text(), "Delft");
    fa.open();
    fa.type_text("utr");
    assert_eq!(fa.shown(), ["Utrecht"]);
    let delft_row = fa.row_of("Delft");
    let amsterdam_row = fa.row_of("Amsterdam");

    // The selected item is renamed to match: button and row text follow, and it is shown.
    rename(&mut fa.app, delft, "Utrecht Zuid");
    assert_eq!(fa.button_text(), "Utrecht Zuid");
    assert_eq!(row_text(&fa.app, delft_row), "Utrecht Zuid");
    assert_eq!(fa.shown(), ["Utrecht Zuid", "Utrecht"]);
    assert_eq!(fa.row_of("Utrecht Zuid"), delft_row, "the row was rebuilt");
    assert_eq!(fa.row_of("Amsterdam"), amsterdam_row);

    // And renamed away again, it is filtered out.
    rename(&mut fa.app, delft, "Delft");
    assert_eq!(fa.shown(), ["Utrecht"]);
    assert_eq!(fa.button_text(), "Delft");
}

#[test]
fn editing_an_item_colour_in_place_updates_its_swatch_highlight_combo_preview_and_name() {
    use bevy::color::palettes::tailwind::{BLUE_500, GREEN_500, RED_500, YELLOW_300};
    let mut app = headless_app();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let root = app.world_mut().spawn(Node::default()).id();
    let combo = app
        .world_mut()
        .spawn((
            ColorPaletteCombo,
            Selection::builder().one().build_with_items(three_colors()),
            ChildOf(root),
        ))
        .id();
    app.update();
    app.update();
    let button = single_entity_with::<ColorPushButton>(&mut app);
    click(&mut app, button);
    app.update();
    let palette = single_entity_with::<ColorPalette>(&mut app);
    let swatches = item_views(&app, palette);
    assert_eq!(swatches.len(), 3);
    let items = items_of(&app, combo);
    let border = |app: &App, swatch: Entity| app.world().get::<BorderColor>(swatch).unwrap().top;
    let before_border = border(&app, swatches[0]);
    let other_border = border(&app, swatches[1]);
    assert_eq!(combo_square(&app, button), Color::from(RED_500));

    let edited: Color = YELLOW_300.into();
    app.world_mut()
        .entity_mut(items[0])
        .get_mut::<Item<Color>>()
        .unwrap()
        .data = edited;
    app.update();
    app.update();

    assert_eq!(swatch_color(&app, swatches[0]), edited);
    assert_eq!(swatch_color(&app, swatches[1]), Color::from(BLUE_500));
    assert_eq!(swatch_color(&app, swatches[2]), Color::from(GREEN_500));
    assert_ne!(border(&app, swatches[0]), before_border, "highlight");
    assert_eq!(border(&app, swatches[1]), other_border, "unrelated row");
    assert_eq!(combo_square(&app, button), edited, "combo preview");
    assert_eq!(combo_value(&app, button), "yellow-300");
    assert_eq!(
        app.world()
            .get::<AccessibilityNode>(swatches[0])
            .unwrap()
            .label(),
        Some("yellow-300")
    );
    // The swatches were edited in place, not rebuilt.
    assert_eq!(item_views(&app, palette), swatches);
}

#[test]
fn a_structural_change_to_chips_keeps_the_views_and_the_focus_of_the_items_that_stay() {
    let mut app = headless_app();
    let (_, chips) = spawn_chips(&mut app, three_chips());
    let before = chip_views(&app, chips);
    let item_of =
        |app: &App, view: Entity| app.world().entity(view).get::<ChipView>().unwrap().item();
    let removed = item_of(&app, before[1]);
    app.world_mut()
        .insert_resource(InputFocus::from_entity(before[2]));

    app.world_mut().entity_mut(removed).despawn();
    app.world_mut().spawn((
        Item {
            data: Chip::new("New"),
        },
        ItemOf(chips),
    ));
    for _ in 0..3 {
        app.update();
    }

    let after = chip_views(&app, chips);
    assert_eq!(after.len(), 3);
    assert_eq!(after[0], before[0]);
    assert_eq!(after[1], before[2]);
    assert!(!after.contains(&before[1]));
    assert_eq!(chip_texts(&app, after[2]).0, "New");
    assert_eq!(
        app.world().get_resource::<InputFocus>().unwrap().get(),
        Some(before[2])
    );
}

#[test]
fn a_structural_change_to_a_toolbar_keeps_its_buttons_and_regroups_the_dividers() {
    let mut app = headless_app();
    let toolbar = spawn_toolbar(&mut app, ToolbarStyle::Filled, Selection::builder().one());
    let before = toolbar_buttons(&app, toolbar);
    let item_of = |app: &App, button: Entity| {
        app.world()
            .entity(button)
            .get::<ToolbarButton>()
            .unwrap()
            .item()
    };
    let dividers = |app: &App| {
        toolbar_children(app, toolbar)
            .into_iter()
            .filter(|child| app.world().entity(*child).contains::<Divider>())
            .count()
    };
    assert_eq!(dividers(&app), 2);
    let mine = app
        .world_mut()
        .spawn((AppOwnedChild, Node::default(), ChildOf(toolbar)))
        .id();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(before[3]));

    // Dropping "Rail" leaves "Signal" alone in its group: both dividers stay, order holds.
    let rail = item_of(&app, before[1]);
    app.world_mut().entity_mut(rail).despawn();
    for _ in 0..3 {
        app.update();
    }
    let after = toolbar_buttons(&app, toolbar);
    assert_eq!(after, [before[0], before[2], before[3]]);
    assert_eq!(dividers(&app), 2);
    assert!(toolbar_children(&app, toolbar).contains(&mine));
    assert_eq!(
        app.world().get_resource::<InputFocus>().unwrap().get(),
        Some(before[3])
    );

    // Dropping "Signal" merges nothing but removes one divider.
    let signal = item_of(&app, before[2]);
    app.world_mut().entity_mut(signal).despawn();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(toolbar_buttons(&app, toolbar), [before[0], before[3]]);
    assert_eq!(dividers(&app), 1);
    assert!(toolbar_children(&app, toolbar).contains(&mine));
}

#[test]
fn a_structural_change_to_a_gallery_keeps_the_cards_of_the_items_that_stay() {
    let mut app = headless_app();
    let gallery = spawn_gallery(&mut app);
    let cards_before = gallery_groups(&app, gallery);
    let flat = |groups: &[(String, Vec<Entity>)]| {
        groups
            .iter()
            .flat_map(|(_, cards)| cards.clone())
            .collect::<Vec<_>>()
    };
    let before = flat(&cards_before);
    assert_eq!(before.len(), 5);
    app.world_mut()
        .insert_resource(InputFocus::from_entity(before[4]));
    let removed = app
        .world()
        .entity(before[1])
        .get::<GalleryCardView>()
        .unwrap()
        .item();

    app.world_mut().entity_mut(removed).despawn();
    app.world_mut().spawn((
        Item {
            data: GalleryCard::new(Handle::default(), "Tram", "Light rail"),
        },
        ItemOf(gallery),
    ));
    for _ in 0..3 {
        app.update();
    }

    let groups = gallery_groups(&app, gallery);
    assert_eq!(
        groups
            .iter()
            .map(|(title, _)| title.as_str())
            .collect::<Vec<_>>(),
        ["LOCOMOTIVES", "WAGONS", "LIGHT RAIL"]
    );
    let after = flat(&groups);
    assert_eq!(after.len(), 5);
    assert_eq!(after[..4], [before[0], before[2], before[3], before[4]]);
    assert!(!after.contains(&before[1]));
    assert_eq!(
        app.world().get_resource::<InputFocus>().unwrap().get(),
        Some(before[4])
    );
}

#[test]
fn two_inserts_of_a_setting_row_in_one_command_batch_build_one_set_of_parts_and_the_last_wins() {
    let mut app = headless_app();
    let row = app.world_mut().spawn_empty().id();
    let app_child = app.world_mut().spawn((Node::default(), ChildOf(row))).id();
    app.world_mut().commands().queue(move |world: &mut World| {
        // Both inserts happen before the hooks' own commands run.
        world.insert_batch([
            (row, SettingRow::new("A", "first")),
            (row, SettingRow::new("B", "second")),
        ]);
    });
    app.world_mut().flush();
    app.update();
    app.update();

    let refs = *app.world().get::<SettingRowRefs>(row).unwrap();
    // The app's child, the content and the divider, nothing twice.
    assert_eq!(app.world().get::<Children>(row).unwrap().len(), 3);
    assert!(
        app.world()
            .get::<Children>(row)
            .unwrap()
            .contains(&app_child)
    );
    assert_eq!(app.world().get::<Text>(refs.title_id).unwrap().0, "B");
    assert_eq!(
        app.world().get::<Text>(refs.description_id).unwrap().0,
        "second"
    );
    let node = app.world().get::<AccessibilityNode>(row).unwrap();
    assert_eq!(node.label(), Some("B"));
}

#[test]
fn two_inserts_of_a_panel_in_one_command_batch_build_one_set_of_parts_and_one_close_observer() {
    let mut app = headless_app();
    app.init_resource::<PanelsClosed>();
    app.add_observer(|t: On<PanelClosed>, mut closed: ResMut<PanelsClosed>| {
        closed.0.push(t.event().event_target());
    });
    let panel = app.world_mut().spawn_empty().id();
    let app_child = app
        .world_mut()
        .spawn((Node::default(), ChildOf(panel)))
        .id();
    app.world_mut().commands().queue(move |world: &mut World| {
        // Both inserts happen before the hooks' own commands run.
        world.insert_batch([
            (
                panel,
                Panel::new("First").with_close_icon(Handle::default()),
            ),
            (
                panel,
                Panel::new("Second").with_close_icon(Handle::default()),
            ),
        ]);
    });
    app.world_mut().flush();
    app.update();
    app.update();

    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    // The app's child, header, strip, two dividers, scroll area and footer.
    assert_eq!(app.world().get::<Children>(panel).unwrap().len(), 7);
    assert!(
        app.world()
            .get::<Children>(panel)
            .unwrap()
            .contains(&app_child)
    );
    assert_eq!(app.world().get::<Text>(refs.title()).unwrap().0, "Second");
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
fn two_inserts_of_an_icon_button_in_one_command_batch_build_one_icon_and_one_set_of_observers() {
    let mut app = headless_app();
    let button = app.world_mut().spawn(ToggleState(false)).id();
    let app_child = app
        .world_mut()
        .spawn((Node::default(), ChildOf(button)))
        .id();
    app.world_mut().commands().queue(move |world: &mut World| {
        // Both inserts happen before the hooks' own commands run.
        world.insert_batch([
            (
                button,
                IconButton::new(Handle::default(), "First", IconButtonSize::Regular),
            ),
            (
                button,
                IconButton::new(Handle::default(), "Second", IconButtonSize::Large),
            ),
        ]);
    });
    app.world_mut().flush();
    app.update();
    app.update();

    let children = app.world().get::<Children>(button).unwrap();
    let icons = children
        .iter()
        .filter(|child| app.world().get::<Icon>(*child).is_some())
        .count();
    assert_eq!(icons, 1, "one icon");
    assert!(children.contains(&app_child));
    assert_eq!(app.world().get::<Tooltip>(button).unwrap().text, "Second");
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert!(
        app.world().get::<ToggleState>(button).unwrap().0,
        "one toggle observer flips the state once"
    );
}
