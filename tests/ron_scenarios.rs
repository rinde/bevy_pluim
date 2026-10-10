//! Scenario files: every `.ron` file in `tests/ron` is one test, named after
//! the file, played in a headless app with [`WidgetsPlugin`]. The files start
//! from a screen of [`widgets_setup::WidgetsSetup`] (`setup: App(Buttons)`).
//!
//! This target has its own `main` (`harness = false` in `Cargo.toml`), so
//! `cargo test --test ron_scenarios push_button` runs the files whose name
//! contains `push_button`.

mod common;
mod widgets_check;
mod widgets_setup;

use std::process::ExitCode;

use rin_bevy_ui_tester::{Registry, run_ron_tests};

use common::widgets_app;

fn main() -> ExitCode {
    // The default registry holds every type that derives `Reflect`, which is
    // what a file's `App(..)` values and reflected checks are loaded against.
    let registry = Registry::default();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ron");
    run_ron_tests(dir, &registry, |scenario| {
        let mut app = widgets_app();
        registry.install(app.world_mut());
        scenario.run_in(&mut app)
    })
}
