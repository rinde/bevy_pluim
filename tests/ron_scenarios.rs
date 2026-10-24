//! Scenario files: every `.ron` file and supported variant in `tests/ron` is
//! an individual libtest case, named after the file and variant. The files start
//! from a screen of [`widgets_setup::WidgetsSetup`] (`setup: App(Buttons)`).
//!
//! `cargo test --test ron_scenarios push_button` filters individual generated
//! tests by their file name. Build-time generation lets libtest schedule these
//! cases in parallel.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "scenario test failures should stop with a case-specific diagnostic"
)]

mod common;
mod widgets_check;
mod widgets_setup;

use bevy::text::DEFAULT_REM_SIZE_PX;
use rin_bevy_ui_tester::{Registry, Variant, load_ron_file};

use common::widgets_app;

fn run_ron_case(path: &str, variant_name: &str) {
    let registry = Registry::default();
    let scenario = load_ron_file(path, &registry).unwrap();
    if scenario.disabled().is_some() {
        return;
    }
    let variant = match variant_name {
        "default" => Variant::default(),
        "scale_2" => Variant::new("scale 2").scale_factor(2.0),
        "text_1_5x" => Variant::new("text 1.5x").rem_size(DEFAULT_REM_SIZE_PX * 1.5),
        _ => panic!("unknown generated scenario variant: {variant_name}"),
    };
    if variant.skips(&scenario) {
        return;
    }
    let label = format!("{path} [{variant_name}]");
    let varied = variant
        .apply(&scenario)
        .unwrap_or_else(|error| panic!("{label}: {error}"));
    let mut app = widgets_app();
    registry.install(app.world_mut());
    let report = varied.run_in(&mut app);
    let passed = report.passed();
    let should_fail = scenario.should_fail_reason();
    assert_eq!(
        passed,
        should_fail.is_none(),
        "{label}: {}\n{:#?}",
        should_fail.map_or_else(
            || "scenario failed".to_owned(),
            |reason| format!("expected failure ({reason}) but passed")
        ),
        report.failures(),
    );
}

include!(concat!(env!("OUT_DIR"), "/ron_scenario_tests.rs"));
