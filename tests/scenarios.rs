//! Scenario tests: `bevy_stagecraft` drives this crate's widgets with
//! scripted pointer input in a headless app, and checks what they did.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests may abort loudly with a case-specific diagnostic"
)]

mod common;
mod widgets_check;
mod widgets_setup;

use bevy::color::palettes::tailwind::{BLUE_500, GREEN_500, RED_500};
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel, SingleThreadedExecutor};
use bevy::ecs::system::RunSystemOnce;
use std::path::Path;

use bevy::a11y::AccessibilityNode;
use bevy::camera::{CameraUpdateSystems, NormalizedRenderTarget, RenderTarget};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input_focus::directional_navigation::{AutoNavigationConfig, DirectionalNavigationMap};
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{FocusCause, FocusedInput, InputFocus, InputFocusVisible};
use bevy::math::CompassOctant;
use bevy::picking::input::PointerInputSettings;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::ui::auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator};
use bevy::ui::{InteractionDisabled, IsDefaultUiCamera, Selected, UiSystems};
use bevy::ui_widgets::{Activate, ActiveDescendant, Button, Tab};
use bevy::window::PrimaryWindow;
use bevy_pluim::badge::Badge;
use bevy_pluim::breadcrumbs::BreadCrumbs;
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::combo::{Combo, ComboButton, ComboState};
use bevy_pluim::filter_chips::{Chip, ChipView, FilterChips};
use bevy_pluim::gallery::{Gallery, GalleryCard};
use bevy_pluim::icon_button::{IconButton, IconButtonSize};
use bevy_pluim::kbd::Kbd;
use bevy_pluim::label::Label;
use bevy_pluim::light_dark_toggle::{LightDarkToggle, ThemePreference};
use bevy_pluim::list::{ItemViewCreator, ItemViewSpawner, List};
use bevy_pluim::numeric_stepper::{NumericStepper, Value};
use bevy_pluim::panel::{Panel, PanelRefs};
use bevy_pluim::pill::Pill;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::scroll_area::{ScrollArea, ScrollContents};
use bevy_pluim::segmented_control::SegmentedControl;
use bevy_pluim::select::{Item, ItemOf, ItemValues, Selection, SelectionParam, SelectionValue};
use bevy_pluim::slider::Slider;
use bevy_pluim::tab_bar::{TabBar, TabBarTab};
use bevy_pluim::text_field::{TextField, TextFieldChange, TextFieldCommit, TextFieldRefs};
use bevy_pluim::theme::{
    DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme, ThemeColors, ThemeMode,
};
use bevy_pluim::toast::{Toast, ToastStack};
use bevy_pluim::toolbar::{Tool, Toolbar, ToolbarStyle};
use bevy_pluim::virtual_list::{
    MIN_ROW_HEIGHT, RowHeight, RowHeightError, ShownItem, VirtualList, VirtualListRow,
};
use bevy_stagecraft::{
    AppCheck, AppSetup, Check, CheckResult, DEFAULT_WINDOW_SIZE, NoiseFilter, Probe, Registry,
    Scenario, ScenarioReport, Step, SyntheticInput, Target, WidgetRole, WorldSnapshot,
    load_ron_file, resolve, suggest_checks,
};

use common::{spawn_camera_and_root, widgets_app};
use widgets_check::{AccessibleState, WidgetsCheck};
use widgets_setup::{Clicks, WidgetsSetup};

fn add_button() -> Target {
    Target::role(WidgetRole::Button, "Add")
}

/// The background colour of the "Add" button is this colour.
fn background_is(color: Color) -> Check {
    Check::component::<BackgroundColor>(add_button(), ".0").eq(color)
}

fn play(scenario: &Scenario) -> ScenarioReport {
    scenario.run_in(&mut widgets_app())
}

/// The scenario that follows the pointer over, onto, off and away from "Add",
/// expecting `colors` to give the background at each point.
fn hover_and_click(colors: &ThemeColors) -> Scenario {
    Scenario::new("a push button follows the theme under the pointer")
        .app_setup(WidgetsSetup::Buttons)
        .steps([
            Step::check(background_is(colors.background_primary)),
            Step::move_to(add_button()),
            Step::check(background_is(colors.hovered_item)),
            Step::press(),
            Step::check(background_is(colors.selected_hovered_item)),
            Step::check(Check::resource::<Clicks>(".0").eq(0_u32)),
            Step::release(),
            // Let go with the pointer still on the button: it is hovered, not selected.
            Step::check(background_is(colors.hovered_item)),
            Step::check(Check::resource::<Clicks>(".0").eq(1_u32)),
            Step::move_to(Target::role(WidgetRole::Button, "Other")),
            Step::check(background_is(colors.background_primary)),
        ])
        .expect_check(Check::resource::<Clicks>(".0").eq(1_u32))
}

#[test]
fn a_push_button_follows_the_theme_under_the_pointer_and_reports_the_click() {
    let report = play(&hover_and_click(&DEFAULT_LIGHT_THEME_COLORS));
    assert!(report.passed(), "{:#?}", report.failures());
}

#[test]
fn a_scenario_expecting_the_wrong_theme_colours_fails() {
    // The app runs the light theme, so the dark slots must not match; this
    // proves the colour checks above are not vacuous.
    let report = play(&hover_and_click(&DEFAULT_DARK_THEME_COLORS));
    assert!(!report.passed());
    assert!(
        report
            .failures()
            .iter()
            .any(|failure| failure.contains("BackgroundColor")),
        "{:#?}",
        report.failures()
    );
}

/// A scenario that taps Tab `taps` times on the "Buttons" screen, with "Other" disabled
/// when `disable_other`, and expects the focus on `expected` at the end.
fn tab_to(taps: usize, disable_other: bool, expected: &str) -> Scenario {
    let button = |label: &str| Target::role(WidgetRole::Button, label);
    let mut steps = vec![Step::wait(2)];
    steps.extend((0..taps).map(|_| Step::tap(KeyCode::Tab)));
    steps.push(Step::check(Check::app(WidgetsCheck::Focused {
        of: button(expected),
    })));
    Scenario::new("Tab moves the focus through the buttons in spawn order")
        .setup(move |world| {
            let root = spawn_camera_and_root(world);
            world.entity_mut(root).with_children(|root| {
                root.spawn(PushButton::from_label("Add"));
                let mut other = root.spawn(PushButton::from_label("Other"));
                if disable_other {
                    other.insert(InteractionDisabled);
                }
                root.spawn(PushButton::from_label("Last"));
            });
        })
        .steps(steps)
}

#[test]
fn tab_focuses_the_first_button_then_the_next_in_spawn_order() {
    for (taps, expected) in [(1, "Add"), (2, "Other"), (3, "Last"), (4, "Add")] {
        let report = play(&tab_to(taps, false, expected));
        assert!(report.passed(), "{taps} taps: {:#?}", report.failures());
    }
}

#[test]
fn tab_skips_a_disabled_button() {
    let report = play(&tab_to(2, true, "Last"));
    assert!(report.passed(), "{:#?}", report.failures());
}

#[test]
fn a_scenario_expecting_the_wrong_focus_fails() {
    // The second Tab lands on "Other", so expecting "Last" there must fail.
    let report = play(&tab_to(2, false, "Last"));
    assert!(!report.passed());
}

#[test]
fn tab_skips_a_tab_bar_in_a_hidden_stack_panel() {
    // A `Stack` hides a panel with `Display::None`; the bar in it must not take the focus.
    let button = |label: &str| Target::role(WidgetRole::Button, label);
    let scenario = Scenario::new("Tab skips a tab bar in a hidden panel")
        .setup(|world| {
            let root = spawn_camera_and_root(world);
            world.entity_mut(root).with_children(|root| {
                root.spawn(PushButton::from_label("Before"));
                root.spawn(Node {
                    display: Display::None,
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn((
                        TabBar,
                        Selection::builder()
                            .one()
                            .build_with_items(["A", "B"].map(Item::new)),
                    ));
                });
                root.spawn(PushButton::from_label("After"));
            });
        })
        .steps([
            Step::wait(2),
            Step::tap(KeyCode::Tab),
            focused(button("Before")),
            Step::tap(KeyCode::Tab),
            focused(button("After")),
        ]);
    let report = play(&scenario);
    assert!(report.passed(), "{:#?}", report.failures());
}

/// Moves the focus with the arrow keys, as an app that turns directional navigation on
/// does: an observer on the window, so it gets only the keys no focused widget kept.
fn navigate_with_arrow_keys(
    input: On<FocusedInput<KeyboardInput>>,
    mut navigator: AutoDirectionalNavigator,
    mut visible: ResMut<InputFocusVisible>,
) {
    let key = &input.event().input;
    if key.state != ButtonState::Pressed {
        return;
    }
    let direction = match key.key_code {
        KeyCode::ArrowUp => CompassOctant::North,
        KeyCode::ArrowDown => CompassOctant::South,
        KeyCode::ArrowLeft => CompassOctant::West,
        KeyCode::ArrowRight => CompassOctant::East,
        _ => return,
    };
    if navigator.navigate(direction).is_ok() {
        visible.0 = true;
    }
}

/// An app with directional navigation turned on: the resources of Bevy's
/// `DirectionalNavigationPlugin` (the app is finished, so it takes no plugin), and every
/// button taking part.
fn directional_navigation_app() -> App {
    let mut app = widgets_app();
    app.init_resource::<DirectionalNavigationMap>()
        .init_resource::<AutoNavigationConfig>();
    app.add_observer(|added: On<Add<Button>>, mut commands: Commands| {
        commands
            .entity(added.entity)
            .insert(AutoDirectionalNavigation::default());
    });
    app
}

/// "Left" and "Right" side by side, the `nav list` (A, B, C, A selected) under them and the
/// `nav stepper` (10, from 5 to 15) under that; the list takes part in directional
/// navigation, and the arrow keys move the focus through [`navigate_with_arrow_keys`].
fn directional_navigation_screen(world: &mut World) {
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn(Node {
            column_gap: px(20),
            ..default()
        })
        .with_children(|row| {
            row.spawn(PushButton::from_label("Left"));
            row.spawn(PushButton::from_label("Right"));
        });
        root.spawn((
            Name::new("nav list"),
            List,
            AutoDirectionalNavigation::default(),
            Selection::builder()
                .one()
                .build_with_items(["A", "B", "C"].map(Item::new)),
        ));
        root.spawn((
            Name::new("nav stepper"),
            NumericStepper::default(),
            Value::new_u16(10, 5, 15, 1).unwrap(),
        ));
    });
    let window = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world)
        .unwrap();
    world.entity_mut(window).observe(navigate_with_arrow_keys);
}

fn nav_row(label: &str) -> Target {
    Target::within(
        Target::name("nav list"),
        Target::role(WidgetRole::ListItem, label),
    )
}

fn increase_button() -> Target {
    Target::within(Target::name("nav stepper"), Target::name("Increase"))
}

fn focused(of: Target) -> Step {
    Step::check(Check::app(WidgetsCheck::Focused { of }))
}

#[test]
fn an_arrow_key_moves_the_focus_between_buttons_and_the_ring_follows() {
    let button = |label: &str| Target::role(WidgetRole::Button, label);
    let scenario = Scenario::new("an arrow key moves the focus between buttons")
        .setup(directional_navigation_screen)
        .steps([
            Step::wait(2),
            Step::tap(KeyCode::Tab),
            focused(button("Left")),
            Step::tap(KeyCode::ArrowRight),
            focused(button("Right")),
            Step::wait(1),
            Step::check(Check::component::<Outline>(button("Right"), ".width").eq(px(2))),
            Step::tap(KeyCode::ArrowLeft),
            focused(button("Left")),
        ]);
    let report = scenario.run_in(&mut directional_navigation_app());
    assert!(report.passed(), "{:#?}", report.failures());
}

#[test]
fn a_focused_list_keeps_the_arrow_keys_from_directional_navigation() {
    let list = || Target::name("nav list");
    let scenario = Scenario::new("a focused list keeps the arrow keys")
        .setup(directional_navigation_screen)
        .steps([
            Step::wait(2),
            Step::tap(KeyCode::Tab),
            Step::tap(KeyCode::Tab),
            Step::tap(KeyCode::Tab),
            focused(list()),
            Step::tap(KeyCode::ArrowDown),
            focused(list()),
            Step::check(Check::app(WidgetsCheck::Active {
                of: list(),
                is: nav_row("B"),
            })),
            Step::tap(KeyCode::ArrowUp),
            Step::tap(KeyCode::ArrowUp),
            focused(list()),
            Step::check(Check::app(WidgetsCheck::Active {
                of: list(),
                is: nav_row("C"),
            })),
        ]);
    let report = scenario.run_in(&mut directional_navigation_app());
    assert!(report.passed(), "{:#?}", report.failures());
}

#[test]
fn a_focused_stepper_button_keeps_the_arrow_keys_from_directional_navigation() {
    let scenario = Scenario::new("a focused stepper button keeps the arrow keys")
        .setup(directional_navigation_screen)
        .steps([
            Step::wait(2),
            Step::tap(KeyCode::Tab),
            Step::tap(KeyCode::Tab),
            Step::tap(KeyCode::Tab),
            Step::tap(KeyCode::Tab),
            Step::tap(KeyCode::Tab),
            focused(increase_button()),
            Step::tap(KeyCode::ArrowUp),
            Step::check(Check::shows("11")),
            focused(increase_button()),
            Step::tap(KeyCode::ArrowDown),
            Step::tap(KeyCode::ArrowDown),
            Step::check(Check::shows("9")),
            focused(increase_button()),
        ]);
    let report = scenario.run_in(&mut directional_navigation_app());
    assert!(report.passed(), "{:#?}", report.failures());
}

/// A custom item view holding a button named after its item ("In row a").
fn row_with_button(
    spawner: In<ItemViewSpawner>,
    items: Query<&Item<String>>,
    mut commands: Commands,
) {
    let Ok(item) = items.get(spawner.item) else {
        return;
    };
    let label = format!("In row {}", item.data);
    commands.entity(spawner.view).with_children(|view| {
        view.spawn((Name::new(label.clone()), PushButton::from_label(label)));
    });
}

/// A `row list` of custom views (a, b, c), each holding a button, with a "Beside" button to
/// its right, the app's directional navigation turned on and the focus on the button in row
/// a. Returns the app and the list.
fn button_in_a_list_row_app() -> (App, Entity) {
    let mut app = directional_navigation_app();
    let create = app.world_mut().register_system(row_with_button);
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn(Node {
            column_gap: px(20),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Name::new("row list"),
                List,
                ItemViewCreator::new(create),
                Selection::builder()
                    .one()
                    .build_with_items(["a", "b", "c"].map(Item::new)),
            ));
            row.spawn((Name::new("Beside"), PushButton::from_label("Beside")));
        });
    });
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .entity_mut(window)
        .observe(navigate_with_arrow_keys);
    app.update();
    app.update();
    let in_row = named(&mut app, "In row a");
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(in_row, FocusCause::Navigated);
    app.update();
    let list = named(&mut app, "row list");
    (app, list)
}

fn active_row(app: &App, list: Entity) -> Option<Entity> {
    app.world()
        .get::<ActiveDescendant>(list)
        .and_then(|active| active.0)
}

/// A held arrow key's repeat on a button inside a list row is not the list's: it reaches the
/// app's navigation.
#[test]
fn a_held_arrow_key_moves_the_focus_out_of_a_button_inside_a_list_row() {
    let (mut app, _) = button_in_a_list_row_app();
    let beside = named(&mut app, "Beside");

    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, true);

    assert_eq!(focus_of(&app), Some(beside));
}

/// A first press of an arrow key on a button inside a list row is not the list's either: the
/// `ListBox` would take it as its own, so it is passed on past the list to the app's
/// navigation.
#[test]
fn a_first_arrow_press_moves_the_focus_out_of_a_button_inside_a_list_row() {
    let (mut app, _) = button_in_a_list_row_app();
    let beside = named(&mut app, "Beside");

    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, false);

    assert_eq!(focus_of(&app), Some(beside));
}

/// A first press of Home or End on a button inside a list row does not move the list's active
/// row, and the focus stays on the button.
#[test]
fn home_and_end_on_a_button_inside_a_list_row_leave_the_active_row_alone() {
    let (mut app, list) = button_in_a_list_row_app();
    let in_row = named(&mut app, "In row a");
    let active = active_row(&app, list);

    send_key(&mut app, KeyCode::End, ButtonState::Pressed, false);
    assert_eq!(active_row(&app, list), active);
    send_key(&mut app, KeyCode::End, ButtonState::Released, false);
    send_key(&mut app, KeyCode::Home, ButtonState::Pressed, false);
    assert_eq!(active_row(&app, list), active);

    assert_eq!(focus_of(&app), Some(in_row));
}

/// Sends `key` as the window's keyboard would, pressed (perhaps as a repeat) or released.
fn send_key(app: &mut App, key_code: KeyCode, state: ButtonState, repeat: bool) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat,
        window,
    });
    app.update();
}

#[test]
fn a_held_arrow_key_does_not_move_the_focus_out_of_a_focused_list() {
    let mut app = directional_navigation_app();
    directional_navigation_screen(app.world_mut());
    app.update();
    app.update();
    let list = named(&mut app, "nav list");
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(list, FocusCause::Navigated);
    app.update();

    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Pressed, false);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Released, false);

    assert_eq!(
        app.world().get_resource::<InputFocus>().unwrap().get(),
        Some(list)
    );
}

/// Spawns a tab bar (A, B) with a "Beside" button to its right, its tabs taking part in the
/// app's directional navigation, and returns its tabs.
fn tab_bar_beside_a_button(app: &mut App) -> [Entity; 2] {
    app.add_observer(|added: On<Add<Tab>>, mut commands: Commands| {
        commands
            .entity(added.entity)
            .insert(AutoDirectionalNavigation::default());
    });
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn(Node {
            column_gap: px(20),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Name::new("nav tab bar"),
                TabBar,
                Selection::builder()
                    .one()
                    .build_with_items(["A", "B"].map(Item::new)),
            ));
            row.spawn((Name::new("Beside"), PushButton::from_label("Beside")));
        });
    });
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .entity_mut(window)
        .observe(navigate_with_arrow_keys);
    app.update();
    app.update();
    let bar = named(app, "nav tab bar");
    // The bar's children are its tabs and the sliding indicator.
    app.world()
        .entity(bar)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|c| app.world().entity(*c).contains::<TabBarTab>())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn focus_of(app: &App) -> Option<Entity> {
    app.world().get_resource::<InputFocus>().unwrap().get()
}

#[test]
fn a_held_arrow_key_does_not_move_the_focus_out_of_a_tab_bar() {
    let mut app = directional_navigation_app();
    let [a, b] = tab_bar_beside_a_button(&mut app);
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(b, FocusCause::Navigated);
    app.update();

    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, false);
    assert_eq!(focus_of(&app), Some(a), "the first press wraps round");
    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Released, false);

    assert_eq!(focus_of(&app), Some(a));
}

// ---- Composite widgets: one tab stop and arrow keys ----

/// A column of "Before", the `seg` segmented control (A, B, C), "Between", the `chips` (X, Y, Z),
/// the `gallery` (P, Q in one group, R in another), the `bar` toolbar (Select, Build, Bulldoze)
/// and "After", with the arrow keys moving the focus between widgets as an app's directional
/// navigation would.
fn composite_screen(world: &mut World) {
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((Name::new("Before"), PushButton::from_label("Before")));
        root.spawn((
            Name::new("seg"),
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["A", "B", "C"].map(Item::new)),
        ));
        root.spawn((Name::new("Between"), PushButton::from_label("Between")));
        root.spawn((
            Name::new("chips"),
            FilterChips,
            Selection::builder()
                .one()
                .build_with_items(["X", "Y", "Z"].map(|label| Item {
                    data: Chip::new(label),
                })),
        ));
        root.spawn((
            Name::new("gallery"),
            Gallery,
            Selection::builder().one().build_with_items(
                [("P", "one"), ("Q", "one"), ("R", "two")].map(|(label, group)| Item {
                    data: GalleryCard::new(Handle::default(), label, group),
                }),
            ),
        ));
        root.spawn((
            Name::new("bar"),
            Toolbar::new(ToolbarStyle::Filled),
            Selection::builder()
                .one()
                .build_with_items(["Select", "Build", "Bulldoze"].map(|label| Item {
                    data: Tool::new(Handle::default(), label),
                })),
        ));
        root.spawn((Name::new("After"), PushButton::from_label("After")));
    });
    let window = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world)
        .unwrap();
    world.entity_mut(window).observe(navigate_with_arrow_keys);
}

fn composite_app() -> App {
    let mut app = directional_navigation_app();
    composite_screen(app.world_mut());
    settle(&mut app);
    app
}

fn settle(app: &mut App) {
    for _ in 0..4 {
        app.update();
    }
}

/// The item buttons of the widget `owner`, in the order they are shown.
fn item_buttons(app: &App, owner: Entity) -> Vec<Entity> {
    fn walk(world: &World, entity: Entity, found: &mut Vec<Entity>) {
        let Some(children) = world.get::<Children>(entity) else {
            return;
        };
        for child in children {
            if world.get::<Button>(*child).is_some() {
                found.push(*child);
            }
            walk(world, *child, found);
        }
    }
    let mut found = Vec::new();
    walk(app.world(), owner, &mut found);
    found
}

/// The item buttons of `owner` that Tab can reach (a `TabIndex` of 0 or more).
fn tab_stops(app: &App, owner: Entity) -> Vec<Entity> {
    let buttons = item_buttons(app, owner);
    buttons
        .into_iter()
        .filter(|button| {
            app.world()
                .get::<TabIndex>(*button)
                .is_some_and(|index| index.0 >= 0)
        })
        .collect()
}

fn focus_on(app: &mut App, entity: Entity) {
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(entity, FocusCause::Navigated);
    settle(app);
}

fn press(app: &mut App, key: KeyCode) {
    send_key(app, key, ButtonState::Pressed, false);
    send_key(app, key, ButtonState::Released, false);
}

fn selected_texts(app: &App, owner: Entity) -> Vec<String> {
    let selection = app.world().get::<Selection>(owner).unwrap();
    let Selection::Value(value) = selection else {
        return Vec::new();
    };
    value
        .iter()
        .filter_map(|item| {
            let world = app.world();
            world
                .get::<Item<String>>(item)
                .map(|i| i.data.clone())
                .or_else(|| world.get::<Item<Chip>>(item).map(|i| i.data.label.clone()))
                .or_else(|| world.get::<Item<Tool>>(item).map(|i| i.data.label.clone()))
                .or_else(|| {
                    world
                        .get::<Item<GalleryCard>>(item)
                        .map(|i| i.data.label.clone())
                })
        })
        .collect()
}

#[test]
fn each_composite_widget_is_one_tab_stop_at_its_selected_item_or_its_first_tool() {
    let mut app = composite_app();
    for (name, selected_at) in [("seg", 0), ("chips", 0), ("gallery", 0), ("bar", 0)] {
        let owner = named(&mut app, name);
        let buttons = item_buttons(&app, owner);
        assert_eq!(buttons.len(), 3, "{name}");
        assert_eq!(tab_stops(&app, owner), vec![buttons[selected_at]], "{name}");
    }
    // The selected item, not the first, is the stop of a radio group.
    let seg = named(&mut app, "seg");
    let buttons = item_buttons(&app, seg);
    press_on(&mut app, buttons[2]);
    assert_eq!(tab_stops(&app, seg), vec![buttons[2]]);
}

/// Clicks `entity` as the pointer would: its `Activate`, with the focus on it.
fn press_on(app: &mut App, entity: Entity) {
    focus_on(app, entity);
    app.world_mut().trigger(Activate { entity });
    settle(app);
}

#[test]
fn arrows_move_wrap_and_pick_in_a_segmented_control_and_home_and_end_go_to_the_ends() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let buttons = item_buttons(&app, seg);
    focus_on(&mut app, buttons[0]);

    press(&mut app, KeyCode::ArrowLeft);
    assert_eq!(focus_of(&app), Some(buttons[2]), "left wraps to the last");
    assert_eq!(selected_texts(&app, seg), ["C"]);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(buttons[0]), "right wraps to the first");
    assert_eq!(selected_texts(&app, seg), ["A"]);
    press(&mut app, KeyCode::ArrowDown);
    assert_eq!(focus_of(&app), Some(buttons[1]));
    press(&mut app, KeyCode::ArrowUp);
    assert_eq!(focus_of(&app), Some(buttons[0]));
    press(&mut app, KeyCode::End);
    assert_eq!(focus_of(&app), Some(buttons[2]));
    assert_eq!(selected_texts(&app, seg), ["C"]);
    press(&mut app, KeyCode::Home);
    assert_eq!(focus_of(&app), Some(buttons[0]));
    assert_eq!(selected_texts(&app, seg), ["A"]);
    assert_eq!(tab_stops(&app, seg), vec![buttons[0]]);
}

#[test]
fn the_arrows_of_a_group_with_several_selected_items_move_the_focus_only() {
    let mut app = directional_navigation_app();
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new("multi"),
            FilterChips,
            Selection::builder()
                .zero_or_more()
                .build_with_items(["X", "Y", "Z"].map(|label| Item {
                    data: Chip::new(label),
                })),
        ));
    });
    settle(&mut app);
    let multi = named(&mut app, "multi");
    let buttons = item_buttons(&app, multi);
    focus_on(&mut app, buttons[0]);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(buttons[1]));
    press(&mut app, KeyCode::ArrowLeft);
    press(&mut app, KeyCode::ArrowLeft);
    assert_eq!(focus_of(&app), Some(buttons[2]));
    assert!(selected_texts(&app, multi).is_empty(), "no pick by moving");
    press(&mut app, KeyCode::Space);
    assert_eq!(selected_texts(&app, multi), ["Z"], "Space still toggles");
}

#[test]
fn a_held_arrow_never_leaves_a_composite_widget_but_a_toolbars_up_and_down_reach_the_app() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let bar = named(&mut app, "bar");
    let between = named(&mut app, "Between");
    let after = named(&mut app, "After");
    let seg_buttons = item_buttons(&app, seg);
    let bar_buttons = item_buttons(&app, bar);

    // A held arrow (a press, repeats, a release) stays in the control.
    focus_on(&mut app, seg_buttons[2]);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Pressed, false);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowDown, ButtonState::Released, false);
    assert!(seg_buttons.contains(&focus_of(&app).unwrap()));
    assert_ne!(focus_of(&app), Some(between));

    // The toolbar keeps left and right, Home and End ...
    focus_on(&mut app, bar_buttons[2]);
    for key in [
        KeyCode::ArrowRight,
        KeyCode::ArrowLeft,
        KeyCode::Home,
        KeyCode::End,
    ] {
        send_key(&mut app, key, ButtonState::Pressed, false);
        send_key(&mut app, key, ButtonState::Pressed, true);
        send_key(&mut app, key, ButtonState::Released, false);
        assert!(bar_buttons.contains(&focus_of(&app).unwrap()), "{key:?}");
    }
    // ... and focus alone: a tool is picked by Enter or Space, not by arriving.
    assert_eq!(selected_texts(&app, bar), ["Select"]);

    // Up and down go to the app, whose navigation moves the focus off the toolbar.
    focus_on(&mut app, bar_buttons[2]);
    press(&mut app, KeyCode::ArrowDown);
    assert_eq!(focus_of(&app), Some(after));
    press(&mut app, KeyCode::ArrowUp);
    assert!(bar_buttons.contains(&focus_of(&app).unwrap()));
}

#[test]
fn disabled_and_hidden_items_are_skipped_and_a_group_without_an_eligible_item_has_no_stop() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let [a, b, c] = item_buttons(&app, seg).try_into().unwrap();
    app.world_mut().entity_mut(b).insert(InteractionDisabled);
    settle(&mut app);
    assert_eq!(tab_stops(&app, seg), vec![a]);
    focus_on(&mut app, a);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(c), "the disabled segment is skipped");
    press(&mut app, KeyCode::ArrowLeft);
    assert_eq!(focus_of(&app), Some(a));

    // The entry item may not be the stop when it is hidden or disabled itself.
    app.world_mut().entity_mut(a).insert(InteractionDisabled);
    settle(&mut app);
    assert_eq!(
        tab_stops(&app, seg),
        vec![c],
        "A went, so the stop moved on"
    );
    app.world_mut().entity_mut(c).insert(InteractionDisabled);
    settle(&mut app);
    assert!(tab_stops(&app, seg).is_empty(), "nothing eligible, no stop");

    // Hidden items: the whole toolbar under a `Display::None`.
    let bar = named(&mut app, "bar");
    assert_eq!(tab_stops(&app, bar).len(), 1);
    app.world_mut()
        .entity_mut(bar)
        .get_mut::<Node>()
        .unwrap()
        .display = Display::None;
    settle(&mut app);
    assert!(
        tab_stops(&app, bar).is_empty(),
        "a hidden toolbar has no stop"
    );
    app.world_mut()
        .entity_mut(bar)
        .get_mut::<Node>()
        .unwrap()
        .display = Display::Flex;
    settle(&mut app);
    assert_eq!(
        tab_stops(&app, bar).len(),
        1,
        "and has one again when shown"
    );
}

#[test]
fn an_empty_composite_widget_contributes_no_stop_and_one_with_items_added_later_does() {
    let mut app = directional_navigation_app();
    let root = spawn_camera_and_root(app.world_mut());
    let items = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(Vec::<Item<String>>::new()),
        )
        .id();
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new("empty"),
            SegmentedControl,
            Selection::reference(items),
        ));
    });
    settle(&mut app);
    let empty = named(&mut app, "empty");
    assert!(item_buttons(&app, empty).is_empty());
    assert!(tab_stops(&app, empty).is_empty());

    app.world_mut().spawn((Item::new("A"), ItemOf(items)));
    app.world_mut().spawn((Item::new("B"), ItemOf(items)));
    settle(&mut app);
    let buttons = item_buttons(&app, empty);
    assert_eq!(buttons.len(), 2);
    assert_eq!(tab_stops(&app, empty), vec![buttons[0]]);
}

#[test]
fn focus_on_a_removed_item_moves_to_the_next_then_the_previous_and_is_cleared_when_none_is_left() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let items = items_of(&app, seg);
    let buttons = item_buttons(&app, seg);
    focus_on(&mut app, buttons[1]);
    app.world_mut().entity_mut(items[1]).despawn();
    settle(&mut app);
    let after_first = item_buttons(&app, seg);
    assert_eq!(after_first.len(), 2);
    assert_eq!(
        focus_of(&app),
        Some(after_first[1]),
        "the next one takes the focus"
    );
    assert_eq!(tab_stops(&app, seg), vec![after_first[1]]);

    // The last one went: the previous takes it.
    app.world_mut().entity_mut(items[2]).despawn();
    settle(&mut app);
    let after_second = item_buttons(&app, seg);
    assert_eq!(after_second.len(), 1);
    assert_eq!(focus_of(&app), Some(after_second[0]));

    // None eligible: the stale focus is cleared.
    app.world_mut()
        .entity_mut(after_second[0])
        .insert(InteractionDisabled);
    settle(&mut app);
    assert_eq!(focus_of(&app), None);
    assert!(tab_stops(&app, seg).is_empty());
}

#[test]
fn removing_an_item_or_changing_a_selection_does_not_take_the_focus_from_another_widget() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let between = named(&mut app, "Between");
    let items = items_of(&app, seg);
    let buttons = item_buttons(&app, seg);
    focus_on(&mut app, buttons[1]);
    focus_on(&mut app, between);

    app.world_mut().entity_mut(items[1]).despawn();
    settle(&mut app);
    assert_eq!(focus_of(&app), Some(between));

    let remaining = item_buttons(&app, seg);
    app.world_mut()
        .entity_mut(seg)
        .insert(Selection::Value(SelectionValue::One(items[2])));
    settle(&mut app);
    assert_eq!(focus_of(&app), Some(between));
    assert_eq!(
        tab_stops(&app, seg),
        vec![remaining[1]],
        "the stop follows the selection"
    );
}

#[test]
fn shared_selections_keep_one_stop_per_control_at_the_shared_selected_item() {
    let mut app = directional_navigation_app();
    let root = spawn_camera_and_root(app.world_mut());
    let shared = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["Dark", "Light"].map(Item::new)),
        )
        .id();
    app.world_mut().entity_mut(root).with_children(|root| {
        for name in ["one", "two"] {
            root.spawn((
                Name::new(name),
                SegmentedControl,
                Selection::reference(shared),
            ));
        }
    });
    settle(&mut app);
    let one = named(&mut app, "one");
    let two = named(&mut app, "two");
    let one_buttons = item_buttons(&app, one);
    let two_buttons = item_buttons(&app, two);
    assert_eq!(tab_stops(&app, one), vec![one_buttons[0]]);
    assert_eq!(tab_stops(&app, two), vec![two_buttons[0]]);

    focus_on(&mut app, one_buttons[0]);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(selected_texts(&app, shared), ["Light"]);
    assert_eq!(tab_stops(&app, one), vec![one_buttons[1]]);
    assert_eq!(
        tab_stops(&app, two),
        vec![two_buttons[1]],
        "the other control follows"
    );
    assert_eq!(focus_of(&app), Some(one_buttons[1]));
}

#[test]
fn a_gallery_moves_linearly_through_its_cards_across_groups_and_wraps() {
    let mut app = composite_app();
    let gallery = named(&mut app, "gallery");
    let cards = item_buttons(&app, gallery);
    assert_eq!(cards.len(), 3);
    focus_on(&mut app, cards[1]);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(cards[2]), "on into the next group");
    assert_eq!(selected_texts(&app, gallery), ["R"]);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(cards[0]), "round to the first");
    press(&mut app, KeyCode::End);
    assert_eq!(focus_of(&app), Some(cards[2]));
}

#[test]
fn a_toolbar_re_enters_at_the_tool_that_had_the_focus_last() {
    let mut app = composite_app();
    let bar = named(&mut app, "bar");
    let after = named(&mut app, "After");
    let tools = item_buttons(&app, bar);
    focus_on(&mut app, tools[0]);
    press(&mut app, KeyCode::ArrowRight);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(tools[2]));
    assert_eq!(
        selected_texts(&app, bar),
        ["Select"],
        "moving picks nothing"
    );
    focus_on(&mut app, after);
    assert_eq!(
        tab_stops(&app, bar),
        vec![tools[2]],
        "Tab comes back to Bulldoze"
    );
    press(&mut app, KeyCode::ArrowLeft);
    focus_on(&mut app, tools[2]);
    press(&mut app, KeyCode::Enter);
    assert_eq!(selected_texts(&app, bar), ["Bulldoze"]);
}

#[test]
fn an_arrow_that_lands_on_the_selected_item_keeps_it_selected() {
    // With the other segments disabled the right arrow wraps round to the item that is picked
    // already, and a toggle would clear a `ZeroOrOne` selection (or be refused by a `One`).
    let mut app = directional_navigation_app();
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new("one"),
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["A", "B", "C"].map(Item::new)),
        ));
        root.spawn((
            Name::new("optional"),
            FilterChips,
            Selection::builder()
                .zero_or_one()
                .build_with_items(["X", "Y"].map(|label| Item {
                    data: Chip::new(label),
                })),
        ));
        root.spawn((
            Name::new("single"),
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["Only"].map(Item::new)),
        ));
    });
    settle(&mut app);
    let one = named(&mut app, "one");
    let optional = named(&mut app, "optional");
    let single = named(&mut app, "single");

    let one_buttons = item_buttons(&app, one);
    for button in &one_buttons[1..] {
        app.world_mut()
            .entity_mut(*button)
            .insert(InteractionDisabled);
    }
    settle(&mut app);
    focus_on(&mut app, one_buttons[0]);
    for key in [
        KeyCode::ArrowRight,
        KeyCode::ArrowLeft,
        KeyCode::Home,
        KeyCode::End,
    ] {
        press(&mut app, key);
        assert_eq!(selected_texts(&app, one), ["A"], "{key:?}");
        assert_eq!(focus_of(&app), Some(one_buttons[0]), "{key:?}");
    }

    // A `ZeroOrOne` chip row: pick X, disable Y, and the arrows land on X again.
    let chips = item_buttons(&app, optional);
    let items = items_of_two(&app, optional);
    app.world_mut()
        .entity_mut(optional)
        .insert(Selection::Value(SelectionValue::ZeroOrOne(Some(items[0]))));
    app.world_mut()
        .entity_mut(chips[1])
        .insert(InteractionDisabled);
    settle(&mut app);
    focus_on(&mut app, chips[0]);
    for key in [KeyCode::ArrowRight, KeyCode::ArrowLeft, KeyCode::End] {
        press(&mut app, key);
        assert_eq!(selected_texts(&app, optional), ["X"], "{key:?}");
    }

    // A control with one item at all.
    let only = item_buttons(&app, single);
    focus_on(&mut app, only[0]);
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(selected_texts(&app, single), ["Only"]);
    assert_eq!(focus_of(&app), Some(only[0]));
}

/// The two item entities `selection` chooses from, in order.
fn items_of_two(app: &App, selection: Entity) -> [Entity; 2] {
    let items: Vec<Entity> = app
        .world()
        .get::<ItemValues>(selection)
        .unwrap()
        .iter()
        .collect();
    items.try_into().unwrap()
}

/// The accessible labels of `buttons`, in order.
fn labels_of(app: &App, buttons: &[Entity]) -> Vec<String> {
    buttons
        .iter()
        .map(|button| {
            app.world()
                .get::<AccessibilityNode>(*button)
                .and_then(|node| node.label().map(str::to_owned))
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn after_a_reorder_the_one_stop_stays_valid_and_the_arrows_follow_the_new_order() {
    // A segmented control reorders its segments in place; chips, galleries and toolbars
    // respawn their buttons. Either way the order shown is the order the keys follow.
    let mut app = composite_app();
    for (name, expected_after) in [
        ("seg", ["C", "A", "B"]),
        ("chips", ["Z", "X", "Y"]),
        ("bar", ["Bulldoze", "Select", "Build"]),
    ] {
        let owner = named(&mut app, name);
        let before = item_buttons(&app, owner);
        let last_item = items_of(&app, owner)[2];
        app.world_mut()
            .entity_mut(owner)
            .insert_related::<ItemOf>(0, &[last_item]);
        settle(&mut app);

        let buttons = item_buttons(&app, owner);
        assert_eq!(buttons.len(), 3, "{name}");
        let labels = labels_of(&app, &buttons);
        let trimmed: Vec<&str> = labels.iter().map(|l| l.trim()).collect();
        assert_eq!(
            trimmed, expected_after,
            "{name}: the buttons follow the items"
        );
        assert_eq!(tab_stops(&app, owner).len(), 1, "{name}: one stop");

        focus_on(&mut app, buttons[0]);
        press(&mut app, KeyCode::ArrowRight);
        assert_eq!(focus_of(&app), Some(buttons[1]), "{name}: right");
        press(&mut app, KeyCode::End);
        assert_eq!(focus_of(&app), Some(buttons[2]), "{name}: End");
        press(&mut app, KeyCode::ArrowRight);
        assert_eq!(focus_of(&app), Some(buttons[0]), "{name}: wrap");
        press(&mut app, KeyCode::ArrowLeft);
        assert_eq!(focus_of(&app), Some(buttons[2]), "{name}: left wraps back");
        press(&mut app, KeyCode::Home);
        assert_eq!(focus_of(&app), Some(buttons[0]), "{name}: Home");
        assert_eq!(tab_stops(&app, owner).len(), 1, "{name}: still one stop");
        let _ = before;
    }
}

#[test]
fn a_focus_cleared_elsewhere_in_the_frame_an_item_goes_is_not_put_back() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let items = items_of(&app, seg);
    let buttons = item_buttons(&app, seg);
    focus_on(&mut app, buttons[1]);

    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .clear();
    app.world_mut().entity_mut(items[1]).despawn();
    settle(&mut app);

    assert_eq!(focus_of(&app), None, "nobody asked for the focus back");
    assert_eq!(
        tab_stops(&app, seg).len(),
        1,
        "the control still has its stop"
    );
}

#[test]
fn keys_pressed_with_the_focus_outside_a_composite_widget_are_not_taken_by_it() {
    let mut app = composite_app();
    let seg = named(&mut app, "seg");
    let between = named(&mut app, "Between");
    focus_on(&mut app, between);
    for key in [
        KeyCode::Home,
        KeyCode::End,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ] {
        press(&mut app, key);
    }
    // Home and End reach nobody; the arrows are the app's, which moves the focus about.
    assert_eq!(selected_texts(&app, seg), ["A"], "no segment was picked");
}

#[test]
fn every_widgets_setup_screen_builds_and_shows_its_theme_button() {
    // The screens the scenario files start from must build without a window
    // and lay out inside it. A screen added to `WidgetsSetup` goes in here.
    let screens = [
        WidgetsSetup::Buttons,
        WidgetsSetup::Combos,
        WidgetsSetup::Lists,
        WidgetsSetup::ScrollAreas,
        WidgetsSetup::BreadCrumbs,
        WidgetsSetup::Tabs,
        WidgetsSetup::ColorPalette,
        WidgetsSetup::NumericStepper,
        WidgetsSetup::Slider,
        WidgetsSetup::SegmentedControls,
        WidgetsSetup::LightDarkToggle,
        WidgetsSetup::FilterChips,
        WidgetsSetup::Panel,
        WidgetsSetup::SettingRows,
        WidgetsSetup::Badge,
        WidgetsSetup::IconButtons,
        WidgetsSetup::Gallery,
        WidgetsSetup::Toolbar,
        WidgetsSetup::Tooltip,
        WidgetsSetup::Toast,
        WidgetsSetup::TextFields,
        WidgetsSetup::ButtonNames,
        WidgetsSetup::Themed,
        WidgetsSetup::VirtualList,
        WidgetsSetup::SeparatedVirtualList,
        WidgetsSetup::ComboRows,
        WidgetsSetup::FilterComboRows,
        WidgetsSetup::FilterCombos,
        WidgetsSetup::ThemeEditor,
    ];
    for screen in screens {
        let scenario = Scenario::new(format!("{screen:?}"))
            .app_setup(screen)
            .steps([Step::wait(2), Step::check(Check::shows("Change theme"))]);
        let report = play(&scenario);
        assert!(report.passed(), "{screen:?}: {:#?}", report.failures());
    }
}

#[test]
fn a_widgets_setup_captures_as_the_screen_it_built() {
    // What a recording started from the app as it is writes as its setup: the
    // screen the app was last built from, and nothing before one was built.
    let mut world = World::new();
    assert_eq!(
        WidgetsSetup::capture(&mut world, Path::new("."), "x"),
        Ok(None)
    );
    for screen in [WidgetsSetup::Lists, WidgetsSetup::Themed] {
        screen.build(&mut world).unwrap();
        assert_eq!(
            WidgetsSetup::capture(&mut world, Path::new("."), "x"),
            Ok(Some(screen))
        );
    }
}

// ---- `WidgetsCheck::Selected` ----

/// The labels of the items every list of [`selection_app`] chooses from.
const LETTERS: [&str; 3] = ["a", "b", "c"];

/// What [`selection_app`] spawned that a test changes.
struct Selections {
    /// The selection entity behind "ref list", "combo" and "palette".
    shared: Entity,
    /// The selection entity behind "trail".
    trail: Entity,
    /// The selection entity behind "palette".
    colors: Entity,
}

/// An app with one list per `SelectionValue` variant ("one", "zero or one",
/// "one or more", "zero or more", over `a`, `b` and `c`), a "ref list" and a
/// "combo" sharing a bare selection entity, a "trail" of breadcrumbs and a
/// "palette" of three colours, all laid out.
fn selection_app() -> (App, Selections) {
    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).insert(Name::new("root"));
    let items = || LETTERS.map(Item::new);
    let shared = world
        .spawn(
            Selection::builder()
                .zero_or_more()
                .build_with_items(items()),
        )
        .id();
    let trail = world
        .spawn(Selection::builder().one_or_more().build_with_items(items()))
        .id();
    let colors = world
        .spawn(Selection::builder().zero_or_one().build_with_items(
            [BLUE_500, RED_500, GREEN_500].map(|c| Item {
                data: Color::Srgba(c),
            }),
        ))
        .id();
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            List,
            Name::new("one"),
            Selection::builder().one().build_with_items(items()),
        ));
        root.spawn((
            List,
            Name::new("zero or one"),
            Selection::builder().zero_or_one().build_with_items(items()),
        ));
        root.spawn((
            List,
            Name::new("one or more"),
            Selection::builder().one_or_more().build_with_items(items()),
        ));
        root.spawn((
            List,
            Name::new("zero or more"),
            Selection::builder()
                .zero_or_more()
                .build_with_items(items()),
        ));
        root.spawn((List, Name::new("ref list"), Selection::reference(shared)));
        root.spawn((Combo, Name::new("combo"), Selection::reference(shared)));
        root.spawn((BreadCrumbs, Name::new("trail"), Selection::reference(trail)));
        root.spawn((
            ColorPalette,
            Name::new("palette"),
            Selection::reference(colors),
        ));
    });
    for _ in 0..3 {
        app.update();
    }
    (
        app,
        Selections {
            shared,
            trail,
            colors,
        },
    )
}

/// The entity with the name `name`.
fn named(app: &mut App, name: &str) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &Name)>();
    query
        .iter(app.world())
        .find_map(|(entity, n)| (n.as_str() == name).then_some(entity))
        .unwrap()
}

/// The three item entities `selection` chooses from, in order.
fn items_of(app: &App, selection: Entity) -> [Entity; 3] {
    let items: Vec<Entity> = app
        .world()
        .get::<ItemValues>(selection)
        .unwrap()
        .iter()
        .collect();
    items.try_into().unwrap()
}

/// Changes the selection on `entity` (following a reference) through
/// [`SelectionParam`], as the widgets do, and lets the world settle.
fn change(
    app: &mut App,
    entity: Entity,
    update: impl Fn(&SelectionValue) -> Option<SelectionValue> + Send + Sync + 'static,
) {
    let changed = app
        .world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            selections.update_selection(entity, &update)
        })
        .unwrap();
    assert!(changed, "the selection did not change");
    app.update();
}

/// Runs `Selected { of, items }` on the app as it is.
fn selected(app: &App, of: &str, items: &[&str]) -> CheckResult {
    WidgetsCheck::Selected {
        of: Target::name(of),
        items: items.iter().map(ToString::to_string).collect(),
    }
    .run(&Probe::new(app.world()))
}

#[test]
fn selected_follows_a_one_selection() {
    let (mut app, _) = selection_app();
    let list = named(&mut app, "one");
    let [_, b, _] = items_of(&app, list);
    assert_eq!(selected(&app, "one", &["a"]), Ok(()));

    change(&mut app, list, move |s| s.toggle_selected(b));

    assert_eq!(selected(&app, "one", &["b"]), Ok(()));
    let failure = selected(&app, "one", &["a"]).unwrap_err();
    assert!(
        failure.contains(r#"["b"] selected, not ["a"]"#),
        "{failure}"
    );
}

#[test]
fn selected_follows_a_zero_or_one_selection() {
    let (mut app, _) = selection_app();
    let list = named(&mut app, "zero or one");
    let [_, b, c] = items_of(&app, list);
    assert_eq!(selected(&app, "zero or one", &[]), Ok(()));

    change(&mut app, list, move |s| s.toggle_selected(b));
    assert_eq!(selected(&app, "zero or one", &["b"]), Ok(()));

    change(&mut app, list, move |s| s.set_selection([c]));
    assert_eq!(selected(&app, "zero or one", &["c"]), Ok(()));

    change(&mut app, list, SelectionValue::clear);
    assert_eq!(selected(&app, "zero or one", &[]), Ok(()));
    let failure = selected(&app, "zero or one", &["c"]).unwrap_err();
    assert!(failure.contains(r#"[] selected, not ["c"]"#), "{failure}");
}

#[test]
fn selected_follows_a_one_or_more_selection() {
    let (mut app, _) = selection_app();
    let list = named(&mut app, "one or more");
    let [_, b, c] = items_of(&app, list);
    assert_eq!(selected(&app, "one or more", &["a"]), Ok(()));

    change(&mut app, list, move |s| s.push(c));
    change(&mut app, list, move |s| s.push(b));
    assert_eq!(selected(&app, "one or more", &["a", "c", "b"]), Ok(()));

    change(&mut app, list, SelectionValue::pop);
    assert_eq!(selected(&app, "one or more", &["a", "c"]), Ok(()));
    let failure = selected(&app, "one or more", &["a", "b"]).unwrap_err();
    assert!(
        failure.contains(r#"["a", "c"] selected, not ["a", "b"]"#),
        "{failure}"
    );
}

#[test]
fn selected_follows_a_zero_or_more_selection_and_its_order() {
    let (mut app, _) = selection_app();
    let list = named(&mut app, "zero or more");
    let [a, _, c] = items_of(&app, list);
    assert_eq!(selected(&app, "zero or more", &[]), Ok(()));

    change(&mut app, list, move |s| s.toggle_selected(c));
    change(&mut app, list, move |s| s.toggle_selected(a));
    assert_eq!(selected(&app, "zero or more", &["c", "a"]), Ok(()));
    // The order is part of the claim: it is the order they were selected in.
    assert!(selected(&app, "zero or more", &["a", "c"]).is_err());

    change(&mut app, list, move |s| s.trim_to_length(1));
    assert_eq!(selected(&app, "zero or more", &["c"]), Ok(()));

    change(&mut app, list, SelectionValue::clear);
    assert_eq!(selected(&app, "zero or more", &[]), Ok(()));
}

#[test]
fn selected_follows_a_selection_ref_from_every_widget_that_shares_it() {
    let (mut app, Selections { shared, .. }) = selection_app();
    let [a, _, c] = items_of(&app, shared);
    assert_eq!(selected(&app, "ref list", &[]), Ok(()));
    assert_eq!(selected(&app, "combo", &[]), Ok(()));

    // Through the list, which follows its reference to the bare selection.
    let list = named(&mut app, "ref list");
    change(&mut app, list, move |s| s.toggle_selected(c));
    assert_eq!(selected(&app, "ref list", &["c"]), Ok(()));
    assert_eq!(selected(&app, "combo", &["c"]), Ok(()));

    // And through the selection entity itself.
    change(&mut app, shared, move |s| s.toggle_selected(a));
    assert_eq!(selected(&app, "ref list", &["c", "a"]), Ok(()));
    assert_eq!(selected(&app, "combo", &["c", "a"]), Ok(()));
    assert!(selected(&app, "combo", &["c"]).is_err());
}

#[test]
fn selected_reads_the_trail_of_breadcrumbs() {
    let (mut app, Selections { trail, .. }) = selection_app();
    let [_, b, _] = items_of(&app, trail);
    assert_eq!(selected(&app, "trail", &["a"]), Ok(()));

    change(&mut app, trail, move |s| s.push(b));

    assert_eq!(selected(&app, "trail", &["a", "b"]), Ok(()));
}

#[test]
fn selected_names_a_colour_by_its_tailwind_class() {
    let (mut app, Selections { colors, .. }) = selection_app();
    let [_, red, _] = items_of(&app, colors);
    assert_eq!(selected(&app, "palette", &[]), Ok(()));

    change(&mut app, colors, move |s| s.toggle_selected(red));

    assert_eq!(selected(&app, "palette", &["red-500"]), Ok(()));
    let failure = selected(&app, "palette", &["blue-500"]).unwrap_err();
    assert!(
        failure.contains(r#"["red-500"] selected, not ["blue-500"]"#),
        "{failure}"
    );
}

#[test]
fn selected_fails_when_the_target_is_missing_or_holds_no_selection() {
    let (app, _) = selection_app();

    let missing = selected(&app, "nothing of this name", &[]).unwrap_err();
    assert!(missing.contains("nothing of this name"), "{missing}");

    // The root has no selection on it or above it.
    let none = selected(&app, "root", &[]).unwrap_err();
    assert!(none.contains("no selection"), "{none}");
}

#[test]
fn the_facts_propose_a_selected_for_every_widget_with_a_selection_and_all_hold() {
    let (app, _) = selection_app();
    let probe = Probe::new(app.world());

    let facts = WidgetsCheck::facts(&probe);

    for fact in &facts {
        assert_eq!(fact.run(&probe), Ok(()), "{fact:?}");
    }
    // The five lists come first, in the order of the UI, then the
    // combo, the trail and the palette, each by the one target that leads
    // back to it.
    let of_lists = |index| Target::nth(index, Target::component::<List>());
    let wanted = [
        (of_lists(0), vec!["a"]),
        (of_lists(1), vec![]),
        (of_lists(2), vec!["a"]),
        (of_lists(3), vec![]),
        (of_lists(4), vec![]),
        (Target::component::<Combo>(), vec![]),
        (Target::component::<BreadCrumbs>(), vec!["a"]),
        (Target::component::<ColorPalette>(), vec![]),
    ];
    let wanted: Vec<_> = wanted
        .into_iter()
        .map(|(of, items)| WidgetsCheck::Selected {
            of,
            items: items.into_iter().map(str::to_owned).collect(),
        })
        .collect();
    assert_eq!(facts, wanted);
}

#[test]
fn a_recording_suggests_selected_for_the_selection_that_changed() {
    let (mut app, _) = selection_app();
    let list = named(&mut app, "one or more");
    let [_, b, _] = items_of(&app, list);
    let filter = NoiseFilter::default();
    let before = WorldSnapshot::take(app.world(), &filter);

    change(&mut app, list, move |s| s.push(b));

    let after = WorldSnapshot::take(app.world(), &filter);
    let suggested = suggest_checks(&before, &after, app.world());
    let wanted = Check::app(WidgetsCheck::Selected {
        of: Target::nth(2, Target::component::<List>()),
        items: vec!["a".to_owned(), "b".to_owned()],
    });
    assert!(suggested.contains(&wanted), "{suggested:#?}");
    // The lists that did not change are not suggested.
    let unchanged = Check::app(WidgetsCheck::Selected {
        of: Target::nth(0, Target::component::<List>()),
        items: vec!["a".to_owned()],
    });
    assert!(!suggested.contains(&unchanged), "{suggested:#?}");
}

#[test]
fn clicking_items_changes_what_selected_reports_in_a_scenario() {
    let first_list = || Target::nth(0, Target::component::<List>());
    let second_item_of_first_list = Target::nth(
        1,
        Target::within(
            first_list(),
            Target::component::<bevy_pluim::list::ItemView>(),
        ),
    );
    let selected = |of: Target, items: &[&str]| {
        Step::check(Check::app(WidgetsCheck::Selected {
            of,
            items: items.iter().map(ToString::to_string).collect(),
        }))
    };
    let scenario = Scenario::new("clicking an item selects it")
        .app_setup(WidgetsSetup::Lists)
        .steps([
            Step::wait(2),
            selected(first_list(), &["Netherlands"]),
            Step::move_to(second_item_of_first_list),
            Step::click(),
            selected(first_list(), &["Netherlands", "Canada"]),
            // The third list shares the first one's selection by reference.
            selected(
                Target::nth(2, Target::component::<List>()),
                &["Netherlands", "Canada"],
            ),
            // The second list has a selection of its own.
            selected(Target::nth(1, Target::component::<List>()), &["NL"]),
        ]);
    let report = play(&scenario);
    assert!(report.passed(), "{:#?}", report.failures());

    // Proves the checks are not vacuous: the wrong claim fails.
    let wrong = Scenario::new("a wrong claim fails")
        .app_setup(WidgetsSetup::Lists)
        .steps([Step::wait(2), selected(first_list(), &["Canada"])]);
    let report = play(&wrong);
    assert!(!report.passed());
    assert!(
        report
            .failures()
            .iter()
            .any(|failure| failure.contains(r#"["Netherlands"] selected, not ["Canada"]"#)),
        "{:#?}",
        report.failures()
    );
}

/// The vertical scroll area's thumb of the [`WidgetsSetup::ScrollAreas`]
/// screen.
fn vertical_thumb() -> Target {
    Target::within(
        Target::name("vertical scroll area"),
        Target::name("ScrollThumb"),
    )
}

/// The contents of the vertical scroll area of the
/// [`WidgetsSetup::ScrollAreas`] screen.
fn vertical_contents() -> Target {
    Target::within(
        Target::name("vertical scroll area"),
        Target::component::<ScrollContents>(),
    )
}

#[test]
fn several_drag_moves_in_one_frame_scroll_by_the_sum_of_their_deltas() {
    // The scenario ends with the left button held on the thumb, so what
    // follows is a drag in progress.
    let grab = Scenario::new("grab the thumb")
        .app_setup(WidgetsSetup::ScrollAreas)
        .steps([
            Step::wait(2),
            Step::move_to(vertical_thumb()),
            Step::press(),
        ]);
    let mut app = widgets_app();
    let report = grab.run_in(&mut app);
    assert!(report.passed(), "{:#?}", report.failures());

    let thumb = resolve(app.world(), &vertical_thumb()).unwrap().position;
    let contents = resolve(app.world(), &vertical_contents())
        .unwrap()
        .widget
        .unwrap();
    let scroll_y = |app: &App| app.world().get::<ScrollPosition>(contents).unwrap().0.y;
    let before = scroll_y(&app);

    // The `bevy_ui_widgets` `Scrollbar` moves the contents by `contents / bar`
    // times the distance the pointer dragged, `bar` being the length of the
    // scroll bar, which runs the full height of the contents here.
    let node = app.world().get::<ComputedNode>(contents).unwrap();
    let scale = node.inverse_scale_factor();
    let bar = node.size().y * scale;
    let total = node.content_size().y * scale;
    let per_pixel = total / bar;

    // Three moves in one frame, as a high-rate mouse delivers them. Each
    // must count, not only the last.
    let step = 5.0;
    let mut input = app
        .world_mut()
        .get_resource_mut::<SyntheticInput>()
        .unwrap();
    for moves in 1..=3_u8 {
        input.move_cursor(thumb + Vec2::new(0.0, step * f32::from(moves)));
    }
    app.update();

    let scrolled = scroll_y(&app) - before;
    let wanted = 3.0 * step * per_pixel;
    assert!(
        (scrolled - wanted).abs() < 1.0,
        "scrolled {scrolled}, wanted {wanted} (three moves of {step} px at {per_pixel} per pixel)"
    );
}

// ---- `WidgetsCheck::Accessible` ----

/// An app with a node named "described" whose `AccessibilityNode` has role
/// `Unknown`, label "Answer", value "42" and is expanded, a "plain" node
/// without an `AccessibilityNode`, and a disabled push button "Off", all laid
/// out.
fn accessible_app() -> App {
    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    let mut described = AccessibilityNode::default();
    described.set_label("Answer");
    described.set_value("42");
    described.set_expanded(true);
    world.entity_mut(root).with_children(|root| {
        root.spawn((Node::default(), Name::new("described"), described));
        root.spawn((Node::default(), Name::new("plain")));
        root.spawn((PushButton::from_label("Off"), InteractionDisabled));
    });
    for _ in 0..3 {
        app.update();
    }
    app
}

/// Runs `Accessible` on `of` on the app as it is.
fn accessible(
    app: &App,
    of: Target,
    role: Option<&str>,
    label: Option<&str>,
    value: Option<&str>,
    states: &[AccessibleState],
) -> CheckResult {
    WidgetsCheck::Accessible {
        of,
        role: role.map(ToString::to_string),
        label: label.map(ToString::to_string),
        value: value.map(ToString::to_string),
        states: states.to_vec(),
    }
    .run(&Probe::new(app.world()))
}

#[test]
fn accessible_passes_when_role_label_value_and_states_match() {
    let app = accessible_app();
    assert_eq!(
        accessible(
            &app,
            Target::name("described"),
            Some("Unknown"),
            Some("Answer"),
            Some("42"),
            &[AccessibleState::Expanded, AccessibleState::Enabled],
        ),
        Ok(())
    );
}

#[test]
fn accessible_does_not_check_what_it_is_not_given() {
    let app = accessible_app();
    assert_eq!(
        accessible(&app, Target::name("described"), None, None, None, &[]),
        Ok(())
    );
}

#[test]
fn accessible_names_a_different_role() {
    let app = accessible_app();
    assert_eq!(
        accessible(
            &app,
            Target::name("described"),
            Some("ComboBox"),
            None,
            None,
            &[]
        ),
        Err(r#"Name("described") has role Unknown, not ComboBox"#.to_owned())
    );
}

#[test]
fn accessible_names_a_different_label() {
    let app = accessible_app();
    assert_eq!(
        accessible(
            &app,
            Target::name("described"),
            None,
            Some("Question"),
            None,
            &[]
        ),
        Err(r#"Name("described") has label "Answer", not "Question""#.to_owned())
    );
}

#[test]
fn accessible_names_a_different_value() {
    let app = accessible_app();
    assert_eq!(
        accessible(&app, Target::name("described"), None, None, Some("43"), &[]),
        Err(r#"Name("described") has value "42", not "43""#.to_owned())
    );
}

#[test]
fn accessible_names_a_state_that_does_not_hold() {
    let app = accessible_app();
    assert_eq!(
        accessible(
            &app,
            Target::name("described"),
            None,
            None,
            None,
            &[AccessibleState::Collapsed]
        ),
        Err(r#"Name("described") is expanded, not Collapsed"#.to_owned())
    );
}

#[test]
fn accessible_names_every_difference() {
    let app = accessible_app();
    assert_eq!(
        accessible(
            &app,
            Target::name("described"),
            Some("Button"),
            None,
            None,
            &[AccessibleState::Disabled, AccessibleState::Hidden]
        ),
        Err(
            r#"Name("described") has role Unknown, not Button, is enabled, not Disabled, is shown, not Hidden"#
                .to_owned()
        )
    );
}

#[test]
fn accessible_asks_a_negative_state_to_be_set() {
    // The node can be neither selected nor toggled, so it is not `NotSelected`
    // or `NotToggled` either.
    let app = accessible_app();
    assert_eq!(
        accessible(
            &app,
            Target::name("described"),
            None,
            None,
            None,
            &[AccessibleState::NotSelected, AccessibleState::NotToggled]
        ),
        Err(
            r#"Name("described") is neither selected nor not selected, not NotSelected, is neither toggled nor not toggled, not NotToggled"#
                .to_owned()
        )
    );
}

#[test]
fn accessible_reads_a_disabled_push_button() {
    let app = accessible_app();
    let button = || Target::role(WidgetRole::Button, "Off");
    assert_eq!(
        accessible(
            &app,
            button(),
            Some("Button"),
            None,
            None,
            &[AccessibleState::Disabled]
        ),
        Ok(())
    );
    assert_eq!(
        accessible(
            &app,
            button(),
            None,
            None,
            None,
            &[AccessibleState::Enabled]
        ),
        Err(r#"Role(role: Button, label: "Off") is disabled, not Enabled"#.to_owned())
    );
}

#[test]
fn accessible_fails_on_a_node_without_an_accessibility_node() {
    let app = accessible_app();
    assert_eq!(
        accessible(&app, Target::name("plain"), None, None, None, &[]),
        Err(r#"Name("plain") has no AccessibilityNode"#.to_owned())
    );
}

// ---- The UI drawn to an image (#100) ----

/// The image the cameras of [`image_target_app`] draw to.
#[derive(Resource)]
struct TargetImage(Handle<Image>);

/// A [`widgets_app`] whose cameras draw to an image instead of the window, as
/// the recorder's do, with the pointer aimed at that image.
///
/// Nothing is drawn: there is no GPU. What matters is that no camera renders to
/// a window, so Bevy's `ui_focus_system` sees no cursor, while picking still
/// finds what is under the pointer in the image.
fn image_target_app() -> App {
    let mut app = widgets_app();
    let (width, height) = DEFAULT_WINDOW_SIZE;
    let image = Image::new_target_texture(width, height, TextureFormat::Rgba8UnormSrgb, None);
    let handle = app
        .world_mut()
        .get_resource_mut::<Assets<Image>>()
        .unwrap()
        .add(image);
    app.world_mut()
        .get_resource_mut::<SyntheticInput>()
        .unwrap()
        .set_target(Some(NormalizedRenderTarget::Image(handle.clone().into())));
    // The synthetic pointer writes its own `PointerInput`, aimed at the image;
    // Bevy's mouse input would add one aimed at the window.
    app.insert_resource(PointerInputSettings {
        is_mouse_enabled: false,
        is_touch_enabled: false,
    })
    .insert_resource(TargetImage(handle))
    .add_systems(
        PostUpdate,
        draw_to_the_image
            .before(CameraUpdateSystems)
            .before(UiSystems::Prepare),
    );
    app
}

/// Points every camera that renders to a window at the [`TargetImage`], and
/// makes it the default UI camera, which a camera rendering to an image
/// otherwise never is.
fn draw_to_the_image(
    image: Res<TargetImage>,
    mut cameras: Query<(Entity, &mut RenderTarget), With<Camera>>,
    mut commands: Commands,
) {
    for (camera, mut target) in &mut cameras {
        if matches!(*target, RenderTarget::Window(_)) {
            *target = RenderTarget::Image(image.0.clone().into());
            commands.entity(camera).insert(IsDefaultUiCamera);
        }
    }
}

fn run_image_case(path: &str) {
    // The recorder draws the UI to an image. A widget that asks the window
    // where the cursor is (`RelativeCursorPosition`) finds nothing there, so a
    // combo closed on the release that opened it and hovered items lost their
    // hover colour (#100). Picking works for any render target.
    let registry = Registry::default();
    let scenario = load_ron_file(path, &registry).unwrap();
    if scenario.disabled().is_some() {
        return;
    }
    let label = format!("{path} [drawn to image]");
    let mut app = image_target_app();
    registry.install(app.world_mut());
    let report = scenario.run_in(&mut app);
    assert_eq!(
        report.passed(),
        scenario.should_fail_reason().is_none(),
        "{label}: {}\n{:#?}",
        scenario
            .should_fail_reason()
            .map_or("scenario failed", |reason| reason),
        report.failures(),
    );
}

include!(concat!(env!("OUT_DIR"), "/image_scenario_tests.rs"));

#[test]
fn the_tooltip_scenario_passes_when_frames_are_shorter_than_a_sixtieth_of_a_second() {
    // The gallery plays in a real window whose frames are as long as the display
    // makes them, often shorter than 1/60 s, so a fixed number of frames can be
    // less than the tooltip's half second (#381).
    let registry = Registry::default();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/ron/tooltip_shows_after_hover_and_hides_on_leave.ron"
    );
    let scenario = load_ron_file(path, &registry).unwrap();
    let mut app = widgets_app();
    registry.install(app.world_mut());
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_micros(4_167),
    ));
    let report = scenario.run_in(&mut app);
    assert!(report.passed(), "{:#?}", report.failures());
}

#[test]
fn the_toast_scenario_passes_when_frames_are_shorter_than_a_sixtieth_of_a_second() {
    // The gallery plays in a real window whose frames are as long as the display
    // makes them, often shorter than 1/60 s, so a fixed number of frames can be
    // less than a toast's second (#399).
    let registry = Registry::default();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/ron/toast_shows_stacks_and_goes.ron"
    );
    let scenario = load_ron_file(path, &registry).unwrap();
    let mut app = widgets_app();
    registry.install(app.world_mut());
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_micros(4_167),
    ));
    let report = scenario.run_in(&mut app);
    assert!(report.passed(), "{:#?}", report.failures());
}

/// Fixed seeds for shuffling a schedule: never the clock, so a failure
/// reproduces from the seed it names.
/// The schedules this crate adds systems to, by name.
fn shuffled_schedules() -> [(&'static str, InternedScheduleLabel); 4] {
    [
        ("PreUpdate", PreUpdate.intern()),
        ("Update", Update.intern()),
        ("PostUpdate", PostUpdate.intern()),
        ("Last", Last.intern()),
    ]
}

/// Has `schedule` run its systems in an order shuffled by `seed` wherever
/// nothing orders them, on the single-threaded executor so the order is the
/// seed's alone.
fn shuffle(app: &mut App, schedule: InternedScheduleLabel, seed: u64) {
    app.edit_schedule(schedule, |schedule| {
        let mut settings = schedule.get_build_settings();
        settings.shuffle_seed = Some(seed);
        schedule.set_build_settings(settings);
        schedule.set_executor(SingleThreadedExecutor::new());
    });
}

fn run_shuffled_case(path: &str, schedule_name: &str, seed: u64) {
    // A system that runs at the right moment only because nothing happened to
    // run it earlier passes under one seed and fails under another. A failure
    // names the file, the schedule and the seed, and replays from them.
    let registry = Registry::default();
    let scenario = load_ron_file(path, &registry).unwrap();
    if scenario.disabled().is_some() {
        return;
    }
    let label = format!("{path} [schedule {schedule_name}, seed {seed}]");
    let mut app = widgets_app();
    registry.install(app.world_mut());
    let (_, schedule) = shuffled_schedules()
        .into_iter()
        .find(|(name, _)| *name == schedule_name)
        .expect("generated schedule must exist");
    shuffle(&mut app, schedule, seed);
    let report = scenario.run_in(&mut app);
    assert_eq!(
        report.passed(),
        scenario.should_fail_reason().is_none(),
        "{label}: {}\n{:#?}",
        scenario
            .should_fail_reason()
            .map_or("scenario failed", |reason| reason),
        report.failures(),
    );
}

include!(concat!(env!("OUT_DIR"), "/shuffled_scenario_tests.rs"));

/// A combo over a selection that may hold several items: picks in its list
/// leave it open, and a click elsewhere closes it.
fn picking_several_in_a_combo() -> Scenario {
    let combo_is =
        |state| Check::component::<ComboState>(Target::name("multi combo"), "").eq(state);
    let item = |label: &str| {
        Target::within(
            Target::name("multi combo"),
            Target::role(WidgetRole::ListItem, label),
        )
    };
    Scenario::new("picks in a combo over several items leave it open")
        .setup(|world| {
            let root = spawn_camera_and_root(world);
            world.spawn((
                Combo,
                Name::new("multi combo"),
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(LETTERS.map(Item::new)),
                ChildOf(root),
            ));
        })
        .steps([
            Step::wait(2),
            Step::move_to(Target::within(
                Target::name("multi combo"),
                Target::component::<ComboButton>(),
            )),
            Step::click(),
            Step::check(combo_is(ComboState::Open)),
            Step::move_to(item("a")),
            Step::click(),
            Step::move_to(item("c")),
            Step::click(),
            Step::check(combo_is(ComboState::Open)),
            Step::check(Check::app(WidgetsCheck::Selected {
                of: Target::name("multi combo"),
                items: vec!["a".to_owned(), "c".to_owned()],
            })),
            Step::move_to(Target::screen(Vec2::new(900.0, 500.0))),
            Step::click(),
            Step::check(combo_is(ComboState::Closed)),
        ])
}

#[test]
fn a_swatch_picked_under_the_pointer_keeps_its_hover_highlight_in_a_window_and_in_an_image() {
    // The first swatch of `palette` is slate-50, a light colour on the light
    // theme, so its square's border is the colour darkened: by 0.8 while it is
    // selected and hovered, by 0.6 once the pointer has left.
    let swatch = || {
        Target::nth(
            0,
            Target::within(
                Target::name("palette"),
                Target::component::<bevy_pluim::list::ItemView>(),
            ),
        )
    };
    let scenario = Scenario::new("pick the first swatch")
        .app_setup(WidgetsSetup::ColorPalette)
        .steps([Step::wait(2), Step::move_to(swatch()), Step::click()]);
    for mut app in [widgets_app(), image_target_app()] {
        let report = scenario.run_in(&mut app);
        assert!(report.passed(), "{:#?}", report.failures());

        let world = app.world();
        let view = resolve(world, &swatch()).unwrap().widget.unwrap();
        let item = world
            .get::<bevy_pluim::list::ItemView>(view)
            .unwrap()
            .item_ref;
        let color = world.get::<Item<Color>>(item).unwrap().data;
        let square = *world.get::<Children>(view).unwrap().first().unwrap();
        let border = world.get::<BorderColor>(square).unwrap().top;
        assert_eq!(border, color.darker(0.8), "selected and hovered");
    }
}

#[test]
fn picks_in_a_combo_over_several_items_leave_it_open_in_a_window_and_in_an_image() {
    let report = play(&picking_several_in_a_combo());
    assert!(report.passed(), "{:#?}", report.failures());
    let report = picking_several_in_a_combo().run_in(&mut image_target_app());
    assert!(report.passed(), "{:#?}", report.failures());
}

// ---- Filter chips wrap ----

/// The distinct vertical positions (whole pixels) of the chips of a `FilterChips` that sits in
/// a box `width` pixels wide, i.e. how many lines the chips are on.
fn chip_lines(width: f32) -> usize {
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    let wrapper = app
        .world_mut()
        .spawn((
            Node {
                width: px(width),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    let chips = app
        .world_mut()
        .spawn((
            FilterChips,
            Selection::builder().one().build_with_items(
                [
                    Chip::with_count("All", 4),
                    Chip::with_count("Running", 2),
                    Chip::with_count("At stop", 1),
                    Chip::with_count("Problems", 1),
                ]
                .map(|data| Item { data }),
            ),
            ChildOf(wrapper),
        ))
        .id();
    for _ in 0..6 {
        app.update();
    }
    let children = app.world().entity(chips).get::<Children>().unwrap();
    let mut lines = children
        .iter()
        .filter(|c| app.world().entity(*c).contains::<ChipView>())
        .map(|c| {
            app.world()
                .get::<UiGlobalTransform>(c)
                .unwrap()
                .translation
                .y
                .round() as i32
        })
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 4, "four chips");
    lines.sort_unstable();
    lines.dedup();
    lines.len()
}

#[test]
fn filter_chips_wrap_onto_more_lines_in_a_narrower_row() {
    assert_eq!(
        chip_lines(2000.0),
        1,
        "a wide row holds the chips on one line"
    );
    assert!(
        chip_lines(100.0) > 1,
        "a row narrower than the chips wraps them"
    );
}

/// The height, in logical pixels, a panel's body area is laid out at with
/// `rows` rows of 50 px in it and a maximum of 200.
fn panel_body_height(rows: usize) -> (f32, f32) {
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    let panel = app
        .world_mut()
        .spawn((
            Panel::new("Trains").with_max_body_height(200.0).unwrap(),
            ChildOf(root),
        ))
        .id();
    let refs = *app.world().get::<PanelRefs>(panel).unwrap();
    for _ in 0..rows {
        app.world_mut().spawn((
            Node {
                height: px(50),
                width: px(100),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(refs.body()),
        ));
    }
    for _ in 0..6 {
        app.update();
    }
    let height = |e: Entity| {
        let computed = app.world().get::<ComputedNode>(e).unwrap();
        computed.size().y * computed.inverse_scale_factor()
    };
    let overflow = {
        let contents = *app
            .world()
            .get::<Children>(refs.scroll_area())
            .unwrap()
            .first()
            .unwrap();
        let computed = app.world().get::<ComputedNode>(contents).unwrap();
        (computed.content_size().y - computed.size().y) * computed.inverse_scale_factor()
    };
    (height(refs.scroll_area()), overflow)
}

#[test]
fn a_panel_body_grows_with_its_content_up_to_its_maximum_and_then_scrolls() {
    let (short, short_overflow) = panel_body_height(2);
    // The content (100) plus the scroll contents' 3 px of padding on each side, for focus rings.
    assert!((short - 106.0).abs() < 1.5, "grows to its content: {short}");
    assert!(short_overflow < 1.0, "nothing to scroll: {short_overflow}");

    let (tall, tall_overflow) = panel_body_height(10);
    assert!((tall - 200.0).abs() < 1.5, "stops at the maximum: {tall}");
    assert!(
        (tall_overflow - 306.0).abs() < 1.5,
        "the rest scrolls: {tall_overflow}"
    );
}

#[test]
fn a_panel_body_with_control_groups_scrolls_past_its_maximum_height() {
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    let panel = app
        .world_mut()
        .spawn((
            Panel::new("Controls").with_max_body_height(100.0).unwrap(),
            ChildOf(root),
        ))
        .id();
    let body = app.world().get::<PanelRefs>(panel).unwrap().body();
    for index in 0..12 {
        let group = app
            .world_mut()
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    flex_shrink: 0.0,
                    ..default()
                },
                ChildOf(body),
            ))
            .id();
        let row = app
            .world_mut()
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ChildOf(group),
            ))
            .id();
        app.world_mut().spawn((
            PushButton::from_label(format!("Action {}", index + 1)),
            ChildOf(row),
        ));
        app.world_mut().spawn((
            Combo,
            Selection::builder()
                .zero_or_one()
                .build_with_items(["Red", "White", "Blue", "Yellow"].map(Item::new)),
            ChildOf(row),
        ));
        app.world_mut()
            .spawn((Slider::new(0.5, 0.0, 1.0).unwrap(), ChildOf(group)));
        app.world_mut().spawn((
            NumericStepper::default(),
            Value::new_u8(index, 0, 20, 1).unwrap(),
            ChildOf(group),
        ));
    }
    for _ in 0..6 {
        app.update();
    }
    let refs = app.world().get::<PanelRefs>(panel).unwrap();
    let area = refs.scroll_area();
    let contents = *app.world().get::<Children>(area).unwrap().first().unwrap();
    let computed = app.world().get::<ComputedNode>(contents).unwrap();
    let overflow =
        (computed.content_size().y - computed.size().y) * computed.inverse_scale_factor();
    assert!(
        overflow > 1.0,
        "control panel should scroll, overflow is {overflow}"
    );
}

/// The height of a scroll area with a `max_height` of 200 around a list of `rows` rows, and how
/// far its contents overflow it (logical pixels).
fn list_scroll_area_height(rows: usize) -> (f32, f32) {
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    let area = app
        .world_mut()
        .spawn((
            Node {
                width: px(200),
                max_height: px(200),
                ..default()
            },
            ScrollArea::bundle(
                move |parent: &mut ChildSpawner| {
                    parent.spawn((
                        List,
                        Selection::builder()
                            .one()
                            .build_with_items((1..=rows).map(|n| Item::new(format!("Row {n}")))),
                    ));
                },
                true,
                false,
            ),
            ChildOf(root),
        ))
        .id();
    for _ in 0..6 {
        app.update();
    }
    let height = {
        let computed = app.world().get::<ComputedNode>(area).unwrap();
        computed.size().y * computed.inverse_scale_factor()
    };
    let contents = *app.world().get::<Children>(area).unwrap().first().unwrap();
    let computed = app.world().get::<ComputedNode>(contents).unwrap();
    let overflow =
        (computed.content_size().y - computed.size().y) * computed.inverse_scale_factor();
    (height, overflow)
}

#[test]
fn a_list_in_a_scroll_area_with_a_maximum_height_grows_with_its_rows_and_then_scrolls() {
    let (short, short_overflow) = list_scroll_area_height(2);
    assert!(short < 200.0, "grows to its rows: {short}");
    assert!(short_overflow < 1.0, "nothing to scroll: {short_overflow}");

    let (tall, tall_overflow) = list_scroll_area_height(30);
    assert!((tall - 200.0).abs() < 1.5, "stops at the maximum: {tall}");
    assert!(tall_overflow > 100.0, "the rest scrolls: {tall_overflow}");
}

// ---- Text widgets in the accessibility tree ----

/// An app with a title, a pill, a key cap and an icon button with a badge of 3, all named.
fn text_widgets_app() -> App {
    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((Label::title("Trains"), Name::new("title")));
        root.spawn((Pill::new("Running"), Name::new("pill")));
        root.spawn((Kbd::new("Esc"), Name::new("kbd")));
        root.spawn((
            IconButton::new(Handle::default(), "Warnings", IconButtonSize::Regular),
            Badge::count(3),
            Name::new("warnings"),
        ));
    });
    for _ in 0..4 {
        app.update();
    }
    app
}

fn accessibility_node(app: &mut App, name: &str) -> AccessibilityNode {
    let entity = named(app, name);
    app.world()
        .get::<AccessibilityNode>(entity)
        .unwrap()
        .clone()
}

#[test]
fn a_label_is_a_label_with_its_text_as_value_that_follows_a_new_label() {
    let mut app = text_widgets_app();
    let node = accessibility_node(&mut app, "title");
    assert_eq!(format!("{:?}", node.role()), "Label");
    assert_eq!(node.value(), Some("Trains"));

    let entity = named(&mut app, "title");
    app.world_mut()
        .entity_mut(entity)
        .insert(Label::title("Stops"));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(accessibility_node(&mut app, "title").value(), Some("Stops"));
}

#[test]
fn a_pill_is_a_status_named_by_its_text_as_given() {
    let mut app = text_widgets_app();
    let node = accessibility_node(&mut app, "pill");
    assert_eq!(format!("{:?}", node.role()), "Status");
    assert_eq!(node.label(), Some("Running"));
}

#[test]
fn a_toast_is_a_live_status_that_ignores_the_pointer_and_stacks_newest_last() {
    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    world.spawn((Toast::new("First"), Name::new("first"), ChildOf(root)));
    world.spawn((Toast::new("Second"), Name::new("second"), ChildOf(root)));
    for _ in 0..4 {
        app.update();
    }
    let node = accessibility_node(&mut app, "first");
    assert_eq!(format!("{:?}", node.role()), "Status");
    assert_eq!(node.label(), Some("First"));
    assert_eq!(format!("{:?}", node.live()), "Some(Polite)");

    let first = named(&mut app, "first");
    let second = named(&mut app, "second");
    let stack = app
        .world_mut()
        .query_filtered::<Entity, With<ToastStack>>()
        .single(app.world())
        .unwrap();
    let children: Vec<Entity> = app.world().get::<Children>(stack).unwrap().to_vec();
    assert_eq!(children, [first, second]);
    let pickable = app.world().get::<Pickable>(first).unwrap();
    assert!(!pickable.should_block_lower && !pickable.is_hoverable);
}

#[test]
fn a_kbd_is_a_label_with_the_key_as_value() {
    let mut app = text_widgets_app();
    let node = accessibility_node(&mut app, "kbd");
    assert_eq!(format!("{:?}", node.role()), "Label");
    assert_eq!(node.value(), Some("Esc"));
}

#[test]
fn a_badge_describes_its_button_and_is_not_in_the_tree_itself() {
    let mut app = text_widgets_app();
    let node = accessibility_node(&mut app, "warnings");
    assert_eq!(node.label(), Some("Warnings"));
    assert_eq!(node.description(), Some("3"));
    let badge = named(&mut app, "Badge");
    assert!(app.world().get::<AccessibilityNode>(badge).is_none());

    let entity = named(&mut app, "warnings");
    app.world_mut().entity_mut(entity).insert(Badge::count(4));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        accessibility_node(&mut app, "warnings").description(),
        Some("4")
    );

    app.world_mut().entity_mut(entity).remove::<Badge>();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(accessibility_node(&mut app, "warnings").description(), None);
}

// ---- Radio and toggle roles ----

/// A step that checks the role, label and states of the accessibility node of `of`.
fn accessible_step(
    of: Target,
    role: &str,
    label: Option<&str>,
    states: &[AccessibleState],
) -> Step {
    Step::check(Check::app(WidgetsCheck::Accessible {
        of,
        role: Some(role.to_owned()),
        label: label.map(ToString::to_string),
        value: None,
        states: states.to_vec(),
    }))
}

#[test]
fn a_gallery_is_a_radio_group_whose_toggle_follows_the_picked_card() {
    let card = |label: &str| {
        Target::within(
            Target::name("vehicles"),
            Target::role(WidgetRole::Button, label),
        )
    };
    let scenario = Scenario::new("a gallery is a radio group")
        .setup(|world| {
            let root = spawn_camera_and_root(world);
            world.spawn((
                Gallery,
                Name::new("vehicles"),
                AccessibleLabel::new("Vehicles"),
                Selection::builder().one().build_with_items(
                    [
                        ("Steam", "Locomotives"),
                        ("Diesel", "Locomotives"),
                        ("Boxcar", "Wagons"),
                    ]
                    .map(|(label, group)| Item {
                        data: GalleryCard::new(Handle::default(), label, group),
                    }),
                ),
                ChildOf(root),
            ));
        })
        .steps([
            Step::wait(2),
            accessible_step(
                Target::name("vehicles"),
                "RadioGroup",
                Some("Vehicles"),
                &[],
            ),
            accessible_step(
                card("Steam"),
                "RadioButton",
                Some("Steam"),
                &[AccessibleState::Toggled],
            ),
            accessible_step(
                card("Boxcar"),
                "RadioButton",
                Some("Boxcar"),
                &[AccessibleState::NotToggled],
            ),
            Step::move_to(card("Boxcar")),
            Step::click(),
            Step::wait(1),
            accessible_step(
                card("Boxcar"),
                "RadioButton",
                Some("Boxcar"),
                &[AccessibleState::Toggled],
            ),
            accessible_step(
                card("Steam"),
                "RadioButton",
                Some("Steam"),
                &[AccessibleState::NotToggled],
            ),
        ]);
    let report = play(&scenario);
    assert!(report.passed(), "{:?}", report.failures());
}

#[test]
fn a_toolbar_is_a_toolbar_of_buttons_toggled_while_their_tool_is_active() {
    let tool = |label: &str| {
        Target::within(
            Target::name("tools"),
            Target::role(WidgetRole::Button, label),
        )
    };
    let scenario = Scenario::new("a toolbar is a toolbar of toggle buttons")
        .setup(|world| {
            let root = spawn_camera_and_root(world);
            world.spawn((
                Toolbar::new(ToolbarStyle::Filled),
                Name::new("tools"),
                Selection::builder()
                    .zero_or_one()
                    .build_with_items(["Rail", "Road"].map(|label| Item {
                        data: Tool::new(Handle::default(), label),
                    })),
                ChildOf(root),
            ));
        })
        .steps([
            Step::wait(2),
            accessible_step(Target::name("tools"), "Toolbar", None, &[]),
            accessible_step(
                tool("Rail"),
                "Button",
                Some("Rail"),
                &[AccessibleState::NotToggled],
            ),
            Step::move_to(tool("Rail")),
            Step::click(),
            Step::wait(1),
            accessible_step(
                tool("Rail"),
                "Button",
                Some("Rail"),
                &[AccessibleState::Toggled],
            ),
            accessible_step(
                tool("Road"),
                "Button",
                Some("Road"),
                &[AccessibleState::NotToggled],
            ),
            Step::click(),
            Step::wait(1),
            accessible_step(
                tool("Rail"),
                "Button",
                Some("Rail"),
                &[AccessibleState::NotToggled],
            ),
        ]);
    let report = play(&scenario);
    assert!(report.passed(), "{:?}", report.failures());
}

#[test]
fn the_cursor_overlays_roots_get_no_ui_root() {
    let mut app = App::new();
    app.add_plugins((
        bevy_stagecraft::HeadlessPlugins::default(),
        bevy_stagecraft::overlay::CursorOverlayPlugin,
    ));
    app.init_resource::<ClearColor>();
    app.add_plugins(bevy_pluim::WidgetsPlugin);
    app.finish();
    app.cleanup();
    spawn_camera_and_root(app.world_mut());
    app.update();
    app.update();

    let roots = app
        .world_mut()
        .query_filtered::<Entity, With<bevy_pluim::UiRoot>>()
        .iter(app.world())
        .count();
    assert_eq!(roots, 1, "only the app's own root is a UiRoot");
}

// ---- `VirtualList` ----

/// The entity of the only `VirtualList`, and of the scroll contents around it.
fn virtual_list_and_contents(app: &mut App) -> (Entity, Entity) {
    let world = app.world_mut();
    let list = world
        .query_filtered::<Entity, With<VirtualList>>()
        .single(world)
        .unwrap();
    let contents = world
        .query_filtered::<Entity, With<ScrollContents>>()
        .single(world)
        .unwrap();
    (list, contents)
}

/// How many rows the `VirtualList`s have, and how many entities the world has.
fn rows_and_entities(app: &mut App) -> (usize, usize) {
    let world = app.world_mut();
    let rows = world.query::<&VirtualListRow>().iter(world).count();
    let entities = world.query::<Entity>().iter(world).count();
    (rows, entities)
}

/// The text of the rows that are shown, in the order of their items.
fn shown_rows(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let mut rows: Vec<(usize, String)> = world
        .query::<(&VirtualListRow, &Node, &Children)>()
        .iter(world)
        .filter(|(_, node, _)| node.display != Display::None)
        .filter_map(|(row, _, children)| {
            let text = children.iter().find_map(|child| world.get::<Text>(child))?;
            Some((row.shown?.index, text.0.clone()))
        })
        .collect();
    rows.sort_unstable();
    rows.into_iter().map(|(_, text)| text).collect()
}

#[test]
fn a_virtual_list_of_ten_thousand_rows_keeps_its_entities_bounded_at_any_scroll_position() {
    let mut app = widgets_app();
    WidgetsSetup::VirtualList.build(app.world_mut()).unwrap();
    for _ in 0..4 {
        app.update();
    }
    let (list, contents) = virtual_list_and_contents(&mut app);
    let (rows, entities) = rows_and_entities(&mut app);
    // 200 pixels of 28 pixel rows: 8 in view, one more for a part row and two beyond each edge.
    assert_eq!(rows, 13);
    assert_eq!(
        shown_rows(&mut app).first().map(String::as_str),
        Some("Row 1")
    );

    for scroll in [1_000.0, 140_000.0, 279_000.0, 50_000.0, 1_000_000.0] {
        app.world_mut()
            .entity_mut(contents)
            .insert(ScrollPosition(Vec2::new(0.0, scroll)));
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(rows_and_entities(&mut app), (rows, entities), "at {scroll}");
    }
    // The last scroll went past the end, which the layout stops at the bottom.
    let shown = shown_rows(&mut app);
    assert_eq!(shown.last().map(String::as_str), Some("Row 10000"));
    assert_eq!(shown.len(), rows);

    // A train added at the bottom: one entity more, the item, and no row.
    // The list's selection holds its own items.
    let items = list;
    app.world_mut()
        .spawn((Item::new("Row 10001"), ItemOf(items)));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(rows_and_entities(&mut app), (rows, entities + 1));
    assert_eq!(
        app.world().get::<ItemValues>(items).map(ItemValues::len),
        Some(10_001)
    );
}

/// For each shown row of the `VirtualList`s, in the order of their items: the item's index
/// and the `position_in_set` and `size_of_set` of the row's accessibility node.
fn shown_row_positions(app: &mut App) -> Vec<(usize, Option<usize>, Option<usize>)> {
    let world = app.world_mut();
    let mut rows: Vec<_> = world
        .query::<(&VirtualListRow, &Node, &AccessibilityNode)>()
        .iter(world)
        .filter(|(_, node, _)| node.display != Display::None)
        .filter_map(|(row, _, node)| {
            Some((row.shown?.index, node.position_in_set(), node.size_of_set()))
        })
        .collect();
    rows.sort_unstable();
    rows
}

#[test]
fn a_virtual_list_tells_a_screen_reader_where_each_row_is_among_all_its_items() {
    // A screen reader reads a pooled row as item 5,001 of 10,000, not as one of 13 rows.
    let mut app = widgets_app();
    WidgetsSetup::VirtualList.build(app.world_mut()).unwrap();
    for _ in 0..4 {
        app.update();
    }
    let (list, contents) = virtual_list_and_contents(&mut app);
    assert_eq!(
        app.world()
            .get::<AccessibilityNode>(list)
            .and_then(|node| node.size_of_set()),
        Some(10_000)
    );
    let first = shown_row_positions(&mut app);
    assert_eq!(first.first(), Some(&(0, Some(1), Some(10_000))));

    // Row 5001 at the top of the view.
    app.world_mut()
        .entity_mut(contents)
        .insert(ScrollPosition(Vec2::new(0.0, 5_000.0 * 28.0)));
    for _ in 0..3 {
        app.update();
    }
    let shown = shown_row_positions(&mut app);
    assert_eq!(shown.len(), 13);
    assert!(shown.iter().any(|(index, ..)| *index == 5_000));
    for (index, position, size) in shown {
        assert_eq!(
            (position, size),
            (Some(index + 1), Some(10_000)),
            "row of item {index}"
        );
    }
}

#[test]
fn a_separated_virtual_list_scrolled_by_gapped_rows_shows_the_rows_in_view_with_their_positions() {
    // 44 pixel rows with 2 pixel gaps: the row at the top of the view after a scroll is found
    // by the 46 pixel pitch (plus the 6 pixel padding), not by the row height, and a screen
    // reader still reads its place among the items.
    let mut app = widgets_app();
    WidgetsSetup::SeparatedVirtualList
        .build(app.world_mut())
        .unwrap();
    for _ in 0..4 {
        app.update();
    }
    let (_, contents) = virtual_list_and_contents(&mut app);
    // Item 5001 (index 5000) starts at 6 + 5000 * 46 in the list, 3 pixels more in the
    // contents, so index 4999 is the top row in view; two more lie beyond it.
    app.world_mut()
        .entity_mut(contents)
        .insert(ScrollPosition(Vec2::new(
            0.0,
            5_000.0f32.mul_add(46.0, 6.0),
        )));
    for _ in 0..3 {
        app.update();
    }
    let shown = shown_row_positions(&mut app);
    assert_eq!(shown.len(), 10);
    assert_eq!(shown.first().map(|(index, ..)| *index), Some(4_997));
    for (index, position, size) in shown {
        assert_eq!(
            (position, size),
            (Some(index + 1), Some(10_000)),
            "row of item {index}"
        );
    }
}

#[test]
fn a_virtual_list_row_that_shows_no_item_has_no_position_among_the_items() {
    // Scrolled to the bottom, then all but three items removed: most of the pool's rows
    // show nothing, and a screen reader must not read them as item 9,999 of 3.
    let mut app = widgets_app();
    WidgetsSetup::VirtualList.build(app.world_mut()).unwrap();
    for _ in 0..4 {
        app.update();
    }
    let (list, contents) = virtual_list_and_contents(&mut app);
    app.world_mut()
        .entity_mut(contents)
        .insert(ScrollPosition(Vec2::new(0.0, 1_000_000.0)));
    for _ in 0..3 {
        app.update();
    }
    let world = app.world_mut();
    let items: Vec<Entity> = world
        .query::<(Entity, &ItemOf)>()
        .iter(world)
        .filter(|(_, item_of)| item_of.0 == list)
        .map(|(item, _)| item)
        .collect();
    assert_eq!(items.len(), 10_000);
    for &item in items.iter().skip(3) {
        world.despawn(item);
    }
    for _ in 0..3 {
        app.update();
    }

    let world = app.world_mut();
    let rows: Vec<_> = world
        .query::<(&VirtualListRow, &AccessibilityNode)>()
        .iter(world)
        .map(|(row, node)| {
            (
                row.shown.map(|shown| shown.index),
                node.position_in_set(),
                node.size_of_set(),
            )
        })
        .collect();
    assert_eq!(rows.len(), 13);
    let empty = rows.iter().filter(|(index, ..)| index.is_none()).count();
    assert_eq!(empty, 10, "rows showing no item: {rows:?}");
    for (index, position, size) in rows {
        match index {
            Some(index) => assert_eq!((position, size), (Some(index + 1), Some(3)), "{index}"),
            None => assert_eq!((position, size), (None, None), "a row showing no item"),
        }
    }
}

#[test]
fn a_first_page_down_on_a_virtual_list_without_an_active_row_lands_on_the_first_item() {
    // Focused while it has no items, the list has no active row and nothing selected when
    // its items arrive; Page Down then starts where Arrow Down does, on the first item.
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    let mut list = Entity::PLACEHOLDER;
    app.world_mut().entity_mut(root).with_children(|root| {
        list = root
            .spawn((
                VirtualList::new(RowHeight::new(28.0).unwrap()),
                Selection::builder()
                    .zero_or_more()
                    .build_with_items(Vec::<Item<String>>::new()),
            ))
            .id();
    });
    for _ in 0..3 {
        app.update();
    }
    app.world_mut()
        .get_resource_mut::<InputFocus>()
        .unwrap()
        .set(list, FocusCause::Navigated);
    app.update();
    let first = app
        .world_mut()
        .spawn((Item::new("Row 1".to_string()), ItemOf(list)))
        .id();
    for index in 2..=100 {
        app.world_mut()
            .spawn((Item::new(format!("Row {index}")), ItemOf(list)));
    }
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(active_row(&app, list), None);

    send_key(&mut app, KeyCode::PageDown, ButtonState::Pressed, false);
    app.update();

    let row = active_row(&app, list).unwrap();
    assert_eq!(
        app.world()
            .get::<VirtualListRow>(row)
            .and_then(|row| row.shown.map(|shown| shown.item)),
        Some(first)
    );
}

/// Fills a `VirtualList` row with a button named after the row's item.
fn virtual_row_with_button(
    spawner: In<ItemViewSpawner>,
    items: Query<&Item<String>>,
    mut commands: Commands,
) {
    let Ok(item) = items.get(spawner.item) else {
        return;
    };
    let label = format!("In row {}", item.data);
    commands.entity(spawner.view).with_children(|view| {
        view.spawn((
            Name::new(label.clone()),
            PushButton::from_label(label),
            Node {
                width: px(90),
                ..default()
            },
        ));
    });
}

/// Spawns a `creator list` of a hundred rows (`Row 1` first selected), each filled by
/// [`virtual_row_with_button`], in a 200 pixel high scroll area.
fn spawn_virtual_list_with_creator(world: &mut World) {
    let create = world.register_system(virtual_row_with_button);
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn((
            Name::new("creator scroll area"),
            Node {
                width: px(240),
                height: px(200),
                ..default()
            },
            ScrollArea::bundle(
                move |parent: &mut ChildSpawner| {
                    parent.spawn((
                        Name::new("creator list"),
                        VirtualList::new(RowHeight::new(48.0).unwrap()),
                        ItemViewCreator::new(create),
                        Selection::builder()
                            .one()
                            .build_with_items((1..=100).map(|row| Item::new(format!("Row {row}")))),
                    ));
                },
                true,
                false,
            ),
        ));
    });
}

/// For each shown row of the `VirtualList`s, in the order of their items: the item's text and
/// the names of the row's children.
fn shown_rows_with_children(app: &mut App) -> Vec<(String, Vec<String>)> {
    let world = app.world_mut();
    let mut rows: Vec<(usize, String, Vec<String>)> = world
        .query::<(&VirtualListRow, &Node, Option<&Children>)>()
        .iter(world)
        .filter(|(_, node, _)| node.display != Display::None)
        .filter_map(|(row, _, children)| {
            let item = world.get::<Item<String>>(row.shown?.item)?.data.clone();
            let names = children
                .into_iter()
                .flatten()
                .filter_map(|child| world.get::<Name>(*child))
                .map(|name| name.as_str().to_owned())
                .collect();
            Some((row.shown?.index, item, names))
        })
        .collect();
    rows.sort_unstable();
    rows.into_iter()
        .map(|(_, item, names)| (item, names))
        .collect()
}

/// The `shown` of every `VirtualListRow` insert an observer saw.
#[derive(Resource, Default)]
struct InsertedRows(Vec<Option<ShownItem>>);

#[test]
fn a_virtual_list_row_shows_an_item_with_its_index_or_nothing_and_an_insert_observer_gets_both() {
    let mut app = widgets_app();
    app.init_resource::<InsertedRows>();
    app.add_observer(
        |t: On<Insert<VirtualListRow>>,
         rows: Query<&VirtualListRow>,
         mut seen: ResMut<InsertedRows>| {
            if let Ok(row) = rows.get(t.event_target()) {
                seen.0.push(row.shown);
            }
        },
    );
    spawn_virtual_list_with_creator(app.world_mut());
    for _ in 0..4 {
        app.update();
    }

    // Every shown row holds the index and the item together, and the item is the one at
    // that index of the list's items.
    let world = app.world_mut();
    let shown: Vec<_> = world
        .query::<&VirtualListRow>()
        .iter(world)
        .filter_map(|row| row.shown)
        .collect();
    assert!(!shown.is_empty());
    for ShownItem { index, item } in shown {
        let text = world.get::<Item<String>>(item).unwrap().data.clone();
        assert_eq!(text, format!("Row {}", index + 1));
    }

    // The observer of the insert got a shown row with both, for the creator's rows.
    let seen = &app.world().get_resource::<InsertedRows>().unwrap().0;
    assert!(seen.iter().any(Option::is_some), "no insert showed an item");
    assert!(seen.iter().flatten().all(|shown| shown.index < 100));
}

#[test]
fn a_virtual_list_runs_its_creator_again_for_a_row_handed_to_another_item() {
    let mut app = widgets_app();
    spawn_virtual_list_with_creator(app.world_mut());
    for _ in 0..4 {
        app.update();
    }
    let assert_each_row_holds_one_button_for_its_item = |app: &mut App| {
        let rows = shown_rows_with_children(app);
        assert!(!rows.is_empty());
        for (item, children) in rows {
            assert_eq!(children, [format!("In row {item}")], "row of {item}");
        }
    };
    assert_each_row_holds_one_button_for_its_item(&mut app);
    let (rows, entities) = rows_and_entities(&mut app);
    let first = shown_rows_with_children(&mut app);
    assert_eq!(first.first().map(|(item, _)| item.as_str()), Some("Row 1"));

    // Far enough down that every row is handed on.
    let contents = app
        .world_mut()
        .query_filtered::<Entity, With<ScrollContents>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .entity_mut(contents)
        .insert(ScrollPosition(Vec2::new(0.0, 2_400.0)));
    for _ in 0..3 {
        app.update();
    }

    assert_each_row_holds_one_button_for_its_item(&mut app);
    let scrolled = shown_rows_with_children(&mut app);
    assert!(
        scrolled.iter().all(|row| !first.contains(row)),
        "{scrolled:?} still shows a row of {first:?}"
    );
    assert_eq!(rows_and_entities(&mut app), (rows, entities));
    let world = app.world_mut();
    let row_one_buttons = world
        .query::<&Name>()
        .iter(world)
        .filter(|name| name.as_str() == "In row Row 1")
        .count();
    assert_eq!(row_one_buttons, 0, "the old children are gone");
}

#[test]
fn a_click_on_a_button_in_a_virtual_list_row_leaves_the_selection_alone() {
    let selected = |items: &[&str]| {
        Step::check(Check::app(WidgetsCheck::Selected {
            of: Target::name("creator list"),
            items: items.iter().map(ToString::to_string).collect(),
        }))
    };
    let scenario = Scenario::new("a click on a button in a row is the button's")
        .setup(spawn_virtual_list_with_creator)
        .steps([
            Step::wait(4),
            selected(&["Row 1"]),
            Step::move_to(Target::name("In row Row 3")),
            Step::click(),
            selected(&["Row 1"]),
            // A click on the row beside its button picks the row's item.
            Step::move_to(Target::nth(3, Target::component::<VirtualListRow>())),
            Step::click(),
            selected(&["Row 4"]),
        ]);
    let report = play(&scenario);
    assert!(report.passed(), "{:#?}", report.failures());
}

#[test]
fn a_virtual_list_row_follows_its_items_text_changing_in_place() {
    let mut app = widgets_app();
    WidgetsSetup::VirtualList.build(app.world_mut()).unwrap();
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    let row_one = world
        .query::<(Entity, &Item<String>)>()
        .iter(world)
        .find_map(|(entity, item)| (item.data == "Row 1").then_some(entity))
        .unwrap();
    world.entity_mut(row_one).insert(Item::new("Changed"));
    app.update();

    assert_eq!(
        shown_rows(&mut app).first().map(String::as_str),
        Some("Changed")
    );
}

#[test]
fn a_virtual_list_over_a_selection_reference_shows_and_marks_the_referenced_selection() {
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    let shared = app
        .world_mut()
        .spawn(
            Selection::builder()
                .one()
                .build_with_items(["a", "b", "c"].map(Item::new)),
        )
        .id();
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((
            VirtualList::new(RowHeight::new(28.0).unwrap()),
            Selection::reference(shared),
        ));
    });
    for _ in 0..4 {
        app.update();
    }
    assert_eq!(shown_rows(&mut app), ["a", "b", "c"]);

    let [_, b, _] = items_of(&app, shared);
    change(&mut app, shared, move |selection| {
        selection.set_selection([b])
    });
    app.update();

    let world = app.world_mut();
    let selected: Vec<Option<Entity>> = world
        .query_filtered::<&VirtualListRow, With<Selected>>()
        .iter(world)
        .map(|row| row.shown.map(|shown| shown.item))
        .collect();
    assert_eq!(selected, [Some(b)]);
}

#[test]
fn invalid_virtual_list_row_heights_are_rejected_and_the_minimum_keeps_the_pool_bounded() {
    for row_height in [0.0, -1.0, f32::NAN, f32::INFINITY, 1e-30] {
        assert!(matches!(
            RowHeight::new(row_height),
            Err(RowHeightError::NotFinite | RowHeightError::BelowMinimum)
        ));
    }

    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((
            VirtualList::new(RowHeight::new(MIN_ROW_HEIGHT).unwrap()),
            Selection::builder()
                .one()
                .build_with_items((0..10_000).map(|row| Item::new(row.to_string()))),
        ));
    });
    for _ in 0..4 {
        app.update();
    }
    assert!(rows_and_entities(&mut app).0 < 100);
}

#[test]
fn separated_virtual_list_rejects_a_row_height_at_or_below_its_chrome() {
    for row_height in [10.0, 22.0] {
        let height = RowHeight::new(row_height).unwrap();
        assert!(matches!(
            VirtualList::separated(height),
            Err(RowHeightError::SeparatedTooShort)
        ));
    }
}

#[test]
fn a_separated_virtual_list_with_a_row_height_just_above_its_padding_and_ring_shows_rows() {
    let mut app = widgets_app();
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((
            VirtualList::separated(RowHeight::new(23.0).unwrap()).unwrap(),
            Selection::builder()
                .one()
                .build_with_items(["a", "b", "c"].map(Item::new)),
        ));
    });
    for _ in 0..4 {
        app.update();
    }

    assert!(rows_and_entities(&mut app).0 > 0);
}

/// The `left` of the segment indicator of `control`, in logical pixels, and the left edge of
/// `segment` relative to `control`, as the layout has it.
fn indicator_left_and_goal(app: &App, control: Entity, segment: Entity) -> (f32, f32) {
    use bevy_pluim::segmented_control::SegmentIndicator;
    let world = app.world();
    let indicator = world
        .entity(control)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|c| world.entity(*c).contains::<SegmentIndicator>())
        .unwrap();
    let Val::Px(left) = world.get::<Node>(indicator).unwrap().left else {
        return (f32::NAN, f32::NAN);
    };
    let edge = |e: Entity| {
        let node = world.get::<ComputedNode>(e).unwrap();
        (world.get::<UiGlobalTransform>(e).unwrap().translation.x - node.size().x / 2.0)
            * node.inverse_scale_factor()
    };
    (left, edge(segment) - edge(control))
}

#[test]
fn a_segmented_control_slides_its_indicator_to_a_newly_selected_segment() {
    use bevy_pluim::segmented_control::{Segment, SegmentedControl};
    use bevy_pluim::select::{Item, Selection, SelectionParam};

    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    let control = world
        .spawn((
            SegmentedControl,
            Selection::builder()
                .one()
                .build_with_items(["Auto", "Manual", "Off"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    for _ in 0..5 {
        app.update();
    }
    let [first, _, last]: [Entity; 3] = app
        .world()
        .entity(control)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|c| app.world().entity(*c).contains::<Segment>())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    // At rest it lies behind the selected segment.
    let (left, goal) = indicator_left_and_goal(&app, control, first);
    assert!((left - goal).abs() < 0.6, "{left} vs {goal}");

    let (start, _) = indicator_left_and_goal(&app, control, first);
    app.world_mut()
        .run_system_once(move |mut selections: SelectionParam| {
            let item = selections
                .get_item_values(control)
                .unwrap()
                .1
                .iter()
                .nth(2)
                .unwrap();
            selections.update_selection(control, |s| s.set_selection([item]));
        })
        .unwrap();
    // A few frames later it is on its way, neither at the start nor at the goal ...
    for _ in 0..3 {
        app.update();
    }
    let (left, goal) = indicator_left_and_goal(&app, control, last);
    assert!(
        left > start + 1.0 && left < goal - 1.0,
        "{start} {left} {goal}"
    );

    // ... and a second later it has arrived.
    for _ in 0..60 {
        app.update();
    }
    let (left, goal) = indicator_left_and_goal(&app, control, last);
    assert!((left - goal).abs() < 0.6, "{left} vs {goal}");
}

/// The `left` of the child of `track` named `indicator`, in logical pixels, and the left edge of
/// `item` relative to `track`, as the layout has it.
fn named_indicator_left_and_goal(
    app: &App,
    track: Entity,
    indicator: &str,
    item: Entity,
) -> (f32, f32) {
    let world = app.world();
    let indicator = world
        .entity(track)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|c| {
            world
                .get::<Name>(*c)
                .is_some_and(|n| n.as_str() == indicator)
        })
        .unwrap();
    let Val::Px(left) = world.get::<Node>(indicator).unwrap().left else {
        return (f32::NAN, f32::NAN);
    };
    let edge = |e: Entity| {
        let node = world.get::<ComputedNode>(e).unwrap();
        (world.get::<UiGlobalTransform>(e).unwrap().translation.x - node.size().x / 2.0)
            * node.inverse_scale_factor()
    };
    (left, edge(item) - edge(track))
}

/// Checks that the indicator named `indicator` of `track` rests behind `first`, slides when
/// `change` runs and rests behind `last` a second later.
fn assert_indicator_slides(
    app: &mut App,
    track: Entity,
    indicator: &str,
    first: Entity,
    last: Entity,
    change: impl FnOnce(&mut App),
) {
    let (left, goal) = named_indicator_left_and_goal(app, track, indicator, first);
    assert!((left - goal).abs() < 0.6, "{left} vs {goal}");

    let start = left;
    change(app);
    for _ in 0..3 {
        app.update();
    }
    let (left, goal) = named_indicator_left_and_goal(app, track, indicator, last);
    assert!(
        left > start + 1.0 && left < goal - 1.0,
        "{start} {left} {goal}"
    );

    for _ in 0..60 {
        app.update();
    }
    let (left, goal) = named_indicator_left_and_goal(app, track, indicator, last);
    assert!((left - goal).abs() < 0.6, "{left} vs {goal}");
}

#[test]
fn a_tab_bar_slides_its_indicator_to_a_newly_selected_tab() {
    use bevy_pluim::select::{Item, Selection, SelectionParam};
    use bevy_pluim::tab_bar::{TabBar, TabBarTab};

    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    let bar = world
        .spawn((
            TabBar,
            Selection::builder()
                .one()
                .build_with_items(["General", "Audio", "Input"].map(Item::new)),
            ChildOf(root),
        ))
        .id();
    for _ in 0..5 {
        app.update();
    }
    let [first, _, last]: [Entity; 3] = app
        .world()
        .entity(bar)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|c| app.world().entity(*c).contains::<TabBarTab>())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    assert_indicator_slides(&mut app, bar, "TabIndicator", first, last, |app| {
        app.world_mut()
            .run_system_once(move |mut selections: SelectionParam| {
                let item = selections
                    .get_item_values(bar)
                    .unwrap()
                    .1
                    .iter()
                    .nth(2)
                    .unwrap();
                selections.update_selection(bar, |s| s.set_selection([item]));
            })
            .unwrap();
    });
}

#[test]
fn a_light_dark_toggle_slides_its_indicator_to_a_newly_chosen_option() {
    use bevy_pluim::light_dark_toggle::LightDarkOption;

    let mut app = widgets_app();
    let world = app.world_mut();
    let root = spawn_camera_and_root(world);
    world.insert_resource(ThemePreference::Light);
    let toggle = world.spawn((LightDarkToggle, ChildOf(root))).id();
    for _ in 0..5 {
        app.update();
    }
    let [first, _, last]: [Entity; 3] = app
        .world()
        .entity(toggle)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter(|c| app.world().entity(*c).contains::<LightDarkOption>())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    assert_indicator_slides(&mut app, toggle, "OptionIndicator", first, last, |app| {
        app.world_mut().insert_resource(ThemePreference::Dark);
    });
}

// ---- `TextField` width constraints are layout units (#472) ----

/// The width, in logical pixels, a `TextField` built by `field` is laid out at in a parent
/// of `parent` (`Val::Auto` for none), whose children are not stretched
/// unless `stretch` says so.
fn text_field_width(field: TextField, parent_width: Val, stretch: bool) -> f32 {
    let mut app = widgets_app();
    app.world_mut().spawn(Camera2d);
    let parent = app
        .world_mut()
        .spawn(Node {
            width: parent_width,
            flex_direction: FlexDirection::Column,
            align_items: if stretch {
                AlignItems::Stretch
            } else {
                AlignItems::FlexStart
            },
            ..default()
        })
        .id();
    let field = app.world_mut().spawn((field, ChildOf(parent))).id();
    for _ in 0..6 {
        app.update();
    }
    logical_width(&app, field)
}

fn logical_width(app: &App, entity: Entity) -> f32 {
    let computed = app.world().get::<ComputedNode>(entity).unwrap();
    computed.size().x * computed.inverse_scale_factor()
}

fn assert_width(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1.0,
        "width {actual}, expected {expected}"
    );
}

#[test]
fn a_text_field_with_a_minimum_above_its_maximum_is_as_wide_as_the_minimum() {
    // Parent without a width constraint: the field is as wide as its constraints make it.
    let default_max = TextField::new("").with_min_width(px(400));
    assert_width(text_field_width(default_max, Val::Auto, false), 400.0);

    let min_first = TextField::new("")
        .with_min_width(px(400))
        .with_max_width(px(320));
    assert_width(text_field_width(min_first, Val::Auto, false), 400.0);

    let max_first = TextField::new("")
        .with_max_width(px(320))
        .with_min_width(px(400));
    assert_width(text_field_width(max_first, Val::Auto, false), 400.0);

    // Also when the parent would stretch it, and the bounds are not clamped.
    let stretched = TextField::new("")
        .with_max_width(px(320))
        .with_min_width(px(400));
    assert_eq!(stretched.min_width, px(400));
    assert_eq!(stretched.max_width, px(320));
    assert_width(text_field_width(stretched, px(1000), true), 400.0);
}

#[test]
fn text_field_percentage_bounds_are_relative_to_the_parent() {
    // A 1000 px parent stretches the field, which stops at 60 %.
    let max = TextField::new("")
        .with_min_width(percent(30))
        .with_max_width(percent(60));
    assert_width(text_field_width(max, px(1000), true), 600.0);

    // Not stretched, an empty field is as narrow as its minimum, 50 % of 400 px.
    let min = TextField::new("")
        .with_min_width(percent(50))
        .with_max_width(Val::Auto);
    assert_width(text_field_width(min, px(400), false), 200.0);
}

#[test]
fn text_field_auto_bounds_do_not_constrain_the_width() {
    let auto = || {
        TextField::new("")
            .with_min_width(Val::Auto)
            .with_max_width(Val::Auto)
    };
    // The parent's width wins over the default 320 px maximum when stretched.
    assert_width(text_field_width(auto(), px(700), true), 700.0);
    // Not stretched, an empty field is as wide as its padding and border.
    let narrow = text_field_width(auto(), Val::Auto, false);
    assert!(narrow < 160.0, "no default minimum: {narrow}");
}

#[test]
fn a_text_field_viewport_bound_follows_the_window() {
    let (window_width, _) = DEFAULT_WINDOW_SIZE;
    let expected = window_width as f32 / 2.0;
    let min = TextField::new("")
        .with_min_width(vw(50))
        .with_max_width(Val::Auto);
    assert_width(text_field_width(min, Val::Auto, false), expected);
}

#[test]
fn replacing_a_text_field_changes_its_widths_without_respawning_or_a_change_event() {
    #[derive(Resource, Default)]
    struct Changes(usize);

    let mut app = widgets_app();
    app.init_resource::<Changes>();
    app.world_mut().spawn(Camera2d);
    let parent = app
        .world_mut()
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .id();
    let field = app
        .world_mut()
        .spawn((TextField::new("Alpha"), ChildOf(parent)))
        .observe(|_: On<TextFieldChange>, mut changes: ResMut<Changes>| changes.0 += 1)
        .id();
    for _ in 0..6 {
        app.update();
    }
    let refs = *app.world().get::<TextFieldRefs>(field).unwrap();
    let children: Vec<Entity> = app.world().get::<Children>(field).unwrap().iter().collect();
    let before = logical_width(&app, field);
    assert_width(before, 160.0);

    app.world_mut()
        .entity_mut(field)
        .insert(TextField::new("Alpha").with_min_width(px(400)));
    for _ in 0..6 {
        app.update();
    }

    assert_width(logical_width(&app, field), 400.0);
    assert_eq!(
        app.world().get::<TextFieldRefs>(field).unwrap().input,
        refs.input
    );
    let after: Vec<Entity> = app.world().get::<Children>(field).unwrap().iter().collect();
    assert_eq!(after, children, "the parts are not respawned");
    assert_eq!(app.world().get_resource::<Changes>().unwrap().0, 0);
}

#[test]
fn reinserting_a_text_field_updates_value_placeholder_name_and_widths_without_events() {
    #[derive(Resource, Default)]
    struct Events(usize);

    let mut app = widgets_app();
    app.init_resource::<Events>();
    app.world_mut().spawn(Camera2d);
    let parent = app
        .world_mut()
        .spawn(Node {
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .id();
    let field = app
        .world_mut()
        .spawn((
            TextField::new("Alpha")
                .with_placeholder("Old")
                .with_name("Old name"),
            ChildOf(parent),
        ))
        .observe(|_: On<TextFieldChange>, mut events: ResMut<Events>| events.0 += 1)
        .observe(|_: On<TextFieldCommit>, mut events: ResMut<Events>| events.0 += 1)
        .id();
    for _ in 0..6 {
        app.update();
    }
    let refs = *app.world().get::<TextFieldRefs>(field).unwrap();

    app.world_mut().entity_mut(field).insert(
        TextField::new("")
            .with_placeholder("New")
            .with_name("New name")
            .with_min_width(px(300)),
    );
    for _ in 0..6 {
        app.update();
    }

    let world = app.world();
    assert_eq!(world.get::<TextFieldRefs>(field).unwrap().input, refs.input);
    assert_eq!(
        world
            .get::<bevy::text::EditableText>(refs.input)
            .unwrap()
            .value()
            .to_string(),
        ""
    );
    assert_eq!(world.get::<Text>(refs.placeholder).unwrap().0, "New");
    assert_eq!(
        world.get::<Node>(refs.placeholder).unwrap().display,
        Display::Flex
    );
    assert_eq!(world.get::<Node>(field).unwrap().min_width, px(300));
    let node = world.get::<AccessibilityNode>(refs.input).unwrap();
    assert_eq!(node.label(), Some("New name"));
    assert_eq!(
        world.get_resource::<Events>().unwrap().0,
        0,
        "no change or commit"
    );
}

// ---- LightDarkToggle: one tab stop whose arrows pick ----

/// A "Before" button, the `toggle` and an "After" button, with the arrow keys moving the focus
/// between widgets as an app's directional navigation would. `preference` is inserted first, as
/// an app that fixes it would.
fn toggle_app(preference: Option<ThemePreference>) -> App {
    let mut app = directional_navigation_app();
    if let Some(preference) = preference {
        app.insert_resource(preference);
    }
    let root = spawn_camera_and_root(app.world_mut());
    app.world_mut().entity_mut(root).with_children(|root| {
        root.spawn((Name::new("Before"), PushButton::from_label("Before")));
        root.spawn((Name::new("toggle"), LightDarkToggle));
        root.spawn((Name::new("After"), PushButton::from_label("After")));
    });
    settle(&mut app);
    app
}

fn preference_of(app: &App) -> ThemePreference {
    *app.world().get_resource::<ThemePreference>().unwrap()
}

#[test]
fn a_light_dark_toggle_is_one_tab_stop_at_the_option_of_the_preference() {
    for (index, preference) in [
        ThemePreference::Light,
        ThemePreference::System,
        ThemePreference::Dark,
    ]
    .into_iter()
    .enumerate()
    {
        let mut app = toggle_app(Some(preference));
        let toggle = named(&mut app, "toggle");
        let options = item_buttons(&app, toggle);
        assert_eq!(options.len(), 3);
        assert_eq!(
            tab_stops(&app, toggle),
            vec![options[index]],
            "{preference:?}"
        );
        assert!(
            app.world().get::<TabIndex>(toggle).is_none(),
            "no stop for the container"
        );
    }
}

#[test]
fn the_entry_option_of_a_light_dark_toggle_follows_the_preference_when_it_changes_elsewhere() {
    let mut app = toggle_app(None);
    let toggle = named(&mut app, "toggle");
    let options = item_buttons(&app, toggle);
    assert_eq!(tab_stops(&app, toggle), vec![options[1]]);
    app.insert_resource(ThemePreference::Dark);
    settle(&mut app);
    assert_eq!(tab_stops(&app, toggle), vec![options[2]]);
    app.insert_resource(ThemePreference::Light);
    settle(&mut app);
    assert_eq!(tab_stops(&app, toggle), vec![options[0]]);
}

#[test]
fn arrows_move_wrap_and_set_the_preference_in_a_light_dark_toggle() {
    let mut app = toggle_app(None);
    let toggle = named(&mut app, "toggle");
    let options = item_buttons(&app, toggle);
    focus_on(&mut app, options[1]);

    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(focus_of(&app), Some(options[2]));
    assert_eq!(preference_of(&app), ThemePreference::Dark);
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Dark
    );
    press(&mut app, KeyCode::ArrowDown);
    assert_eq!(focus_of(&app), Some(options[0]), "wraps to the first");
    assert_eq!(preference_of(&app), ThemePreference::Light);
    assert_eq!(
        app.world().get_resource::<Theme>().unwrap().mode(),
        ThemeMode::Light
    );
    press(&mut app, KeyCode::ArrowLeft);
    assert_eq!(focus_of(&app), Some(options[2]), "wraps to the last");
    press(&mut app, KeyCode::ArrowUp);
    assert_eq!(focus_of(&app), Some(options[1]));
    assert_eq!(preference_of(&app), ThemePreference::System);
    press(&mut app, KeyCode::End);
    assert_eq!(focus_of(&app), Some(options[2]));
    assert_eq!(preference_of(&app), ThemePreference::Dark);
    press(&mut app, KeyCode::Home);
    assert_eq!(focus_of(&app), Some(options[0]));
    assert_eq!(preference_of(&app), ThemePreference::Light);
    assert_eq!(tab_stops(&app, toggle), vec![options[0]]);

    // Enter and Space still pick the focused option.
    app.insert_resource(ThemePreference::Dark);
    settle(&mut app);
    focus_on(&mut app, options[0]);
    press(&mut app, KeyCode::Enter);
    assert_eq!(preference_of(&app), ThemePreference::Light);
}

#[test]
fn a_held_arrow_key_does_not_move_the_focus_out_of_a_light_dark_toggle() {
    let mut app = toggle_app(None);
    let toggle = named(&mut app, "toggle");
    let options = item_buttons(&app, toggle);
    focus_on(&mut app, options[2]);
    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, false);
    assert_eq!(
        focus_of(&app),
        Some(options[0]),
        "the first press wraps round"
    );
    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Pressed, true);
    send_key(&mut app, KeyCode::ArrowRight, ButtonState::Released, false);
    assert_eq!(
        focus_of(&app),
        Some(options[1]),
        "the repeat moves on inside the toggle and the release moves nothing"
    );
}

#[test]
fn a_light_dark_toggle_does_not_take_the_focus_from_another_widget() {
    let mut app = toggle_app(None);
    let after = named(&mut app, "After");
    focus_on(&mut app, after);
    app.insert_resource(ThemePreference::Dark);
    settle(&mut app);
    assert_eq!(focus_of(&app), Some(after));
    press(&mut app, KeyCode::ArrowRight);
    assert_eq!(
        preference_of(&app),
        ThemePreference::Dark,
        "an arrow elsewhere picks nothing"
    );
}

// ---- The overview example ----

#[path = "../examples/overview.rs"]
#[expect(
    dead_code,
    unreachable_pub,
    reason = "the example's own `main` is not run by the test, and its plugin is `pub` for it"
)]
mod overview;

/// The overview at the default window size (1280 by 720): the tab bar and the theme toggle on the
/// left, and on every page the same centred area with its variants inside it.
#[test]
fn every_page_of_the_overview_is_centred_in_the_same_area_and_fits_the_window() {
    let mut app = App::new();
    app.add_plugins(bevy_stagecraft::HeadlessPlugins::default());
    app.init_resource::<ClearColor>();
    app.add_plugins(bevy::scene::ScenePlugin);
    app.add_plugins((bevy_pluim::WidgetsPlugin, overview::OverviewPlugin));
    app.finish();
    app.cleanup();
    settle(&mut app);

    // The theme control is the toggle, not a button.
    let world = app.world_mut();
    assert_eq!(world.query::<&LightDarkToggle>().iter(world).count(), 1);
    assert!(
        world
            .query::<&Text>()
            .iter(world)
            .all(|text| text.0 != "Change theme"),
        "the \"Change theme\" button is gone"
    );

    let bar = world
        .query_filtered::<Entity, With<TabBar>>()
        .single(world)
        .unwrap();
    let toggle = world
        .query_filtered::<Entity, With<LightDarkToggle>>()
        .single(world)
        .unwrap();
    let rect = |entity| {
        let size = world.get::<ComputedNode>(entity).unwrap().size();
        let centre = world.get::<UiGlobalTransform>(entity).unwrap().translation;
        Rect::from_center_size(centre, size)
    };
    let toggle_rect = rect(toggle);
    let bar_rect = rect(bar);
    assert!(
        (toggle_rect.min.x - 10.0).abs() <= 1.0,
        "toggle at the left edge"
    );
    assert!(
        (toggle_rect.min.y - 10.0).abs() <= 1.0,
        "toggle at the top edge"
    );
    assert!(toggle_rect.max.y <= bar_rect.min.y, "toggle above the tabs");
    let tabs: Vec<String> = world
        .query::<&Item<String>>()
        .iter(world)
        .map(|item| item.data.clone())
        .collect();
    assert_eq!(tabs.len(), 20);
    let mut area = None;
    for tab in &tabs {
        let name = tab.clone();
        app.world_mut()
            .run_system_once(
                move |mut selections: SelectionParam, items: Query<(Entity, &Item<String>)>| {
                    let item = items.iter().find(|(_, i)| i.data == name).unwrap().0;
                    selections.update_selection(bar, |s| s.set_selection([item]));
                },
            )
            .unwrap();
        settle(&mut app);
        settle(&mut app);
        let world = app.world_mut();
        let mut shown = world.query::<(Entity, &Name, &Node, &ComputedNode, &UiGlobalTransform)>();
        let pages: Vec<_> = shown
            .iter(world)
            .filter(|(_, name, node, ..)| name.as_str() == "Page" && node.display != Display::None)
            .map(|(entity, _, _, computed, transform)| {
                (entity, computed.size(), transform.translation)
            })
            .collect();
        assert_eq!(pages.len(), 1, "{tab}: one page is shown");
        let (page, size, centre) = pages[0];
        // Every page is as wide as the first and has the same centre, so none shifts.
        let (first_size_x, first_centre_x) = *area.get_or_insert((size.x, centre.x));
        assert_eq!((size.x, centre.x), (first_size_x, first_centre_x), "{tab}");
        // A page fits the window height (the area around it scrolls past that).
        assert!(size.y <= 720.0 - 40.0, "{tab} is {} high", size.y);
        // The variants are inside the page and the page inside the window.
        assert!(centre.x + size.x / 2.0 <= 1280.0, "{tab}");
        let left = centre.x - size.x / 2.0;
        let right = centre.x + size.x / 2.0;
        let mut parts = vec![page];
        let mut checked = 0;
        // The page's children (header, groups) and the groups' children (the variants).
        for _depth in 0..2 {
            let mut next = Vec::new();
            for parent in parts {
                let Some(children) = world.get::<Children>(parent) else {
                    continue;
                };
                for child in children.iter() {
                    let (Some(computed), Some(transform)) = (
                        world.get::<ComputedNode>(child),
                        world.get::<UiGlobalTransform>(child),
                    ) else {
                        continue;
                    };
                    let half = computed.size().x / 2.0;
                    assert!(
                        transform.translation.x - half >= left - 1.0
                            && transform.translation.x + half <= right + 1.0,
                        "{tab}: a part sticks out of the page"
                    );
                    checked += 1;
                    next.push(child);
                }
            }
            parts = next;
        }
        assert!(
            checked >= 2,
            "{tab}: the header and the variants were looked at"
        );
        // The wrapping row of variants is centred in the page, and every line of variants is
        // centred in the row, none overlapping another. (The page centres the row itself, so
        // the lines are what `justify_content: Center` of the row decides.)
        let rows: Vec<Entity> = world
            .get::<Children>(page)
            .unwrap()
            .iter()
            .filter(|child| {
                world
                    .get::<Node>(*child)
                    .is_some_and(|node| node.flex_wrap == FlexWrap::Wrap)
            })
            .collect();
        assert_eq!(rows.len(), 1, "{tab}: one wrapping row of variants");
        let rect = |world: &World, entity: Entity| {
            let computed = world.get::<ComputedNode>(entity).unwrap();
            let centre = world.get::<UiGlobalTransform>(entity).unwrap().translation;
            let size = computed.size();
            Rect::from_center_size(centre, size)
        };
        let row = rect(world, rows[0]);
        let page_rect = rect(world, page);
        if row.width() < page_rect.width() {
            assert!(
                (row.center().x - page_rect.center().x).abs() <= 1.0,
                "{tab}: the row of variants is not centred in the page"
            );
        }
        let variants: Vec<Rect> = world
            .get::<Children>(rows[0])
            .unwrap()
            .iter()
            .map(|variant| rect(world, variant))
            .collect();
        assert!(!variants.is_empty(), "{tab}: the row has variants");
        // A line is the variants that start at the same height.
        let mut lines: Vec<(f32, f32, f32)> = Vec::new();
        for variant in &variants {
            match lines
                .iter_mut()
                .find(|(top, ..)| (top - variant.min.y).abs() < 1.0)
            {
                Some((_, left, right)) => {
                    *left = left.min(variant.min.x);
                    *right = right.max(variant.max.x);
                }
                None => lines.push((variant.min.y, variant.min.x, variant.max.x)),
            }
        }
        for (_, left, right) in &lines {
            assert!(
                ((left + right) / 2.0 - row.center().x).abs() <= 1.0,
                "{tab}: a line of variants ({left}..{right}) is not centred in the row {}..{}",
                row.min.x,
                row.max.x
            );
        }
        for (i, a) in variants.iter().enumerate() {
            for b in variants.iter().skip(i + 1) {
                let overlap = a.intersect(*b);
                assert!(
                    overlap.width() <= 0.0 || overlap.height() <= 0.0,
                    "{tab}: two variants overlap: {a:?} and {b:?}"
                );
            }
        }
    }
}

/// The lists on the overview's "Scroll area" page get their rows although a scene inserts their
/// `List` after the `Selection` and items, so the areas overflow and show their scroll bars.
#[test]
fn the_overview_scroll_area_page_has_rows_and_visible_scroll_bars() {
    let mut app = App::new();
    app.add_plugins(bevy_stagecraft::HeadlessPlugins::default());
    app.init_resource::<ClearColor>();
    app.add_plugins(bevy::scene::ScenePlugin);
    app.add_plugins((bevy_pluim::WidgetsPlugin, overview::OverviewPlugin));
    app.finish();
    app.cleanup();
    settle(&mut app);
    let bar = app
        .world_mut()
        .query_filtered::<Entity, With<TabBar>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .run_system_once(
            move |mut selections: SelectionParam, items: Query<(Entity, &Item<String>)>| {
                let item = items
                    .iter()
                    .find(|(_, i)| i.data == "Scroll area")
                    .unwrap()
                    .0;
                selections.update_selection(bar, |s| s.set_selection([item]));
            },
        )
        .unwrap();
    settle(&mut app);
    settle(&mut app);
    let world = app.world_mut();
    let views = world
        .query::<&bevy_pluim::list::ItemView>()
        .iter(world)
        .count();
    assert_eq!(views, 40, "two lists of 20 rows");
    let shown = world
        .query::<(&bevy_pluim::scroll_area::ScrollBar, &Visibility)>()
        .iter(world)
        .filter(|(_, v)| **v != Visibility::Hidden)
        .count();
    assert_eq!(
        shown, 2,
        "the vertical area's bar and the two-way area's vertical bar (the horizontal one needs text layout)"
    );
}
