//! Scenario tests: `rin_bevy_ui_tester` drives this crate's widgets with
//! scripted pointer input in a headless app, and checks what they did.

mod common;

use bevy::prelude::*;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, ThemeColors};
use rin_bevy_ui_tester::{Check, Scenario, ScenarioReport, Step, Target, WidgetRole};

use common::{spawn_camera_and_root, widgets_app};

/// How many times the "Add" button was clicked.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct Clicks(u32);

/// Spawns a camera and a root holding an "Add" and an "Other" button, and
/// counts the clicks on "Add" in [`Clicks`].
fn spawn_two_buttons(world: &mut World) {
    world.init_resource::<Clicks>();
    let root = spawn_camera_and_root(world);
    world.entity_mut(root).with_children(|root| {
        root.spawn(PushButton::from_label("Add"))
            .observe(|_: On<Pointer<Click>>, mut clicks: ResMut<Clicks>| clicks.0 += 1);
        root.spawn(PushButton::from_label("Other"));
    });
}

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
        .setup(spawn_two_buttons)
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
