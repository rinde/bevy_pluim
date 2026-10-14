//! Headless integration tests for `WidgetsPlugin`: a plain `App` built from
//! `MinimalPlugins`, with no window or GPU.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests may abort loudly"
)]

use bevy::MinimalPlugins;
use bevy::a11y::AccessibilityNode;
use bevy::camera::ClearColor;
use bevy::camera::NormalizedRenderTarget;
use bevy::ecs::system::RunSystemOnce;
use bevy::input::InputPlugin;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::picking::DefaultPickingPlugins;
use bevy::picking::backend::HitData;
use bevy::picking::events::{Click, Pointer, Press, Release};
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed, Selected};
use bevy::ui_widgets::{Activate, Button, ListBox, ListBoxMultiSelect, ListItem, ValueChange};
use bevy::window::{PrimaryWindow, WindowPlugin};
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::{ColorPaletteCombo, State as PaletteComboState};
use rin_bevy_ui::color_push_button::ColorPushButton;
use rin_bevy_ui::combo::{Combo, ComboState, ComboSubRefs};
use rin_bevy_ui::divider::Divider;
use rin_bevy_ui::filter_chips::{Chip, ChipRefs, ChipView, FilterChips};
use rin_bevy_ui::gallery::{Gallery, GalleryCard, GalleryCardRefs, GalleryCardView};
use rin_bevy_ui::icon::{Icon, IconColor};
use rin_bevy_ui::icon_button::{IconButton, IconButtonRefs, IconButtonSize, ToggleState, Tone};
use rin_bevy_ui::kbd::Kbd;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::link::{Link, LinkState};
use rin_bevy_ui::list::{CornerStyle, ItemView, ItemViewCreator, ItemViewSpawner, List};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::panel::{Panel, PanelClosed, PanelRefs};
use rin_bevy_ui::pill::{Pill, PillTone};
use rin_bevy_ui::push_button::{ButtonSize, PushButton, PushButtonRefs};
use rin_bevy_ui::segmented_control::{Segment, SegmentedControl};
use rin_bevy_ui::select::{
    Item, ItemOf, ItemValues, Selection, SelectionBuilderWithValue, SelectionParam, SelectionValue,
};
use rin_bevy_ui::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme};
use rin_bevy_ui::toolbar::{
    Tool, Toolbar, ToolbarButton, ToolbarButtonRefs, ToolbarLabels, ToolbarStyle,
};
use rin_bevy_ui::tooltip::{Tooltip, TooltipPopup};

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
    app.finish();
    app.cleanup();
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

    let icon_of = |app: &App, e: Entity| app.world().get::<IconButtonRefs>(e).unwrap().icon_id;
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
        let icon = app.world().get::<IconButtonRefs>(e).unwrap().icon_id;
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
        .label_id;
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
        assert_eq!(font.font_size, FontSize::Px(12.0));
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
            app.world().entity(popups[0]).get::<ChildOf>().map(|c| c.0),
            Some(root),
            "{popup:?} stays a child of the root"
        );
    }
}

#[test]
fn a_color_palette_combo_spawns_its_popup_under_the_topmost_ancestor_of_the_combo() {
    let mut app = headless_app();
    let first_root = app.world_mut().spawn(Node::default()).id();
    let second_root = app.world_mut().spawn(Node::default()).id();
    let panel = app
        .world_mut()
        .spawn((Node::default(), ChildOf(second_root)))
        .id();
    app.world_mut()
        .spawn((ColorPaletteCombo, two_color_selection(), ChildOf(panel)));

    app.update();
    app.update();

    let floats = palette_combo_floats(&mut app);
    assert_eq!(floats.len(), 1, "one popup, got {floats:?}");
    assert_eq!(
        app.world().entity(floats[0]).get::<ChildOf>().map(|c| c.0),
        Some(second_root),
        "under the combo's own root, not the first one ({first_root})"
    );
}

#[test]
fn a_color_palette_combo_without_a_parent_gets_no_popup() {
    let mut app = headless_app();
    let combo = app
        .world_mut()
        .spawn((ColorPaletteCombo, two_color_selection()))
        .id();

    app.update();
    app.update();

    assert!(
        app.world()
            .entity(combo)
            .get::<PaletteComboState>()
            .is_none()
    );
    assert!(palette_combo_floats(&mut app).is_empty());
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

    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Click {
            count: 1,
            button: PointerButton::Secondary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: std::time::Duration::ZERO,
        },
        swatches[1],
    ));
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
            Value::new_u8(0, 0, 14, 7),
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
        .label_id;
    assert_eq!(
        font_of(&app, button_label),
        (body.clone().into(), FontSize::Px(20.0))
    );
    assert_eq!(font_of(&app, regular), (body.into(), FontSize::Px(20.0)));
    assert_eq!(font_of(&app, header), (display.into(), FontSize::Px(30.0)));
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
            (c.text_primary, FontSize::Px(theme.font_size_title))
        );
        assert_eq!(
            look(app, regular),
            (c.text_primary, FontSize::Px(theme.font_size_secondary))
        );
        assert_eq!(
            look(app, secondary),
            (c.text_secondary, FontSize::Px(theme.font_size_secondary))
        );
        assert_eq!(
            look(app, caption),
            (c.text_tertiary, FontSize::Px(theme.font_size_small))
        );
        assert_eq!(
            look(app, overline),
            (c.text_secondary, FontSize::Px(theme.font_size_caption))
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
    }
    app.update();
    app.update();
    check(&app);
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
    assert_eq!(background(&app, segments[0]), colors.segmented_selected);
    assert_eq!(background(&app, segments[1]), colors.segmented_track);
}

#[test]
fn clicking_a_segment_selects_it() {
    let mut app = headless_app();
    let control = spawn_segmented_control(&mut app);
    let segments = segments_of(&app, control);

    activate(&mut app, segments[1]);

    assert_eq!(selected_segments(&app, control), ["Manual"]);
    let colors = DEFAULT_LIGHT_THEME_COLORS;
    assert_eq!(background(&app, segments[0]), colors.segmented_track);
    assert_eq!(background(&app, segments[1]), colors.segmented_selected);

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
    assert_eq!(background(&app, segments[0]), colors.segmented_track);
    assert_eq!(background(&app, segments[2]), colors.segmented_selected);
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
    assert_eq!(
        background(&app, segments[0]),
        DEFAULT_LIGHT_THEME_COLORS.segmented_selected
    );
    assert_eq!(
        background(&app, segments[1]),
        DEFAULT_LIGHT_THEME_COLORS.segmented_track
    );
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
    assert_eq!(background(&app, segments[0]), colors.segmented_selected);
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
    assert_eq!(text(&app, refs.title), "Trains");
    assert_eq!(text(&app, refs.subtitle), "4 on the network");
    assert_eq!(display(&app, refs.subtitle), Display::Flex);
    assert_eq!(display(&app, refs.close), Display::Flex);
    let close_icon = app
        .world()
        .get::<IconButtonRefs>(refs.close)
        .unwrap()
        .icon_id;
    assert_eq!(
        app.world().get::<ImageNode>(close_icon).unwrap().image,
        icon
    );
    assert_eq!(display(&app, refs.strip), Display::None);
    assert_eq!(display(&app, refs.footer), Display::None);
    assert_eq!(
        app.world()
            .get::<Node>(refs.scroll_area)
            .unwrap()
            .max_height,
        Val::Px(620.0)
    );

    let footer_text = app
        .world_mut()
        .spawn((Label::caption("Pick a train."), ChildOf(refs.footer)))
        .id();
    app.update();
    assert_eq!(display(&app, refs.footer), Display::Flex);
    app.world_mut().despawn(footer_text);
    app.update();
    assert_eq!(display(&app, refs.footer), Display::None);

    let plain = app.world_mut().spawn(Panel::new("Stops")).id();
    app.update();
    app.update();
    let plain_refs = *app.world().get::<PanelRefs>(plain).unwrap();
    assert_eq!(display(&app, plain_refs.subtitle), Display::None);
    assert_eq!(display(&app, plain_refs.close), Display::None);
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

    let close = app.world().get::<PanelRefs>(panel).unwrap().close;
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
    assert_eq!(text(&app, refs.subtitle), "3 open");

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Alerts").with_subtitle("2 open"));
    app.update();
    app.update();
    assert_eq!(text(&app, refs.subtitle), "2 open");
    assert_eq!(text(&app, refs.title), "Alerts");
    assert_eq!(
        app.world().get::<Children>(refs.header).unwrap().len(),
        3,
        "the parts are spawned once"
    );

    app.world_mut()
        .entity_mut(panel)
        .insert(Panel::new("Alerts"));
    app.update();
    app.update();
    assert_eq!(
        app.world().get::<Node>(refs.subtitle).unwrap().display,
        Display::None
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
            .hint
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
        let e = app.world().entity(refs.label.unwrap());
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
    assert_eq!(node_display(&app, refs.bar.unwrap()), Display::Flex);

    activate(&mut app, buttons[1]);
    assert_eq!(selected_label(&app, toolbar), None);
    assert_eq!(background(&app, buttons[1]), Color::NONE);
    assert_eq!(node_display(&app, refs.bar.unwrap()), Display::None);
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
        .item;

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
        let label = world.entity(refs.label.unwrap()).get::<Node>().unwrap();
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
    let icon = app.world().get::<IconButtonRefs>(button).unwrap().icon_id;
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
            let icon = app.world().get::<ToolbarButtonRefs>(b).unwrap().icon;
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
        if let Some(label) = refs.label {
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
