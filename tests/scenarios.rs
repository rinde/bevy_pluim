//! Scenario tests: `rin_bevy_ui_tester` drives this crate's widgets with
//! scripted pointer input in a headless app, and checks what they did.

#![expect(clippy::unwrap_used, reason = "tests may abort loudly")]

mod common;
mod widgets_check;
mod widgets_setup;

use bevy::color::palettes::tailwind::{BLUE_500, GREEN_500, RED_500};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::combo::Combo;
use rin_bevy_ui::list::List;
use rin_bevy_ui::select::{Item, ItemValues, Selection, SelectionParam, SelectionValue};
use rin_bevy_ui::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, ThemeColors};
use rin_bevy_ui_tester::{
    AppCheck, Check, CheckResult, NoiseFilter, Probe, Scenario, ScenarioReport, Step, Target,
    WidgetRole, WorldSnapshot, suggest_checks,
};

use common::{spawn_camera_and_root, widgets_app};
use widgets_check::WidgetsCheck;
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
            Step::check(background_is(colors.selected_item)),
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
    ];
    for screen in screens {
        let scenario = Scenario::new(format!("{screen:?}"))
            .app_setup(screen)
            .steps([Step::wait(2), Step::check(Check::shows("Change theme"))]);
        let report = play(&scenario);
        assert!(report.passed(), "{screen:?}: {:#?}", report.failures());
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
            Target::component::<rin_bevy_ui::list::ItemView>(),
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
