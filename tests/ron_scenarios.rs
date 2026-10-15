//! Scenario files: every `.ron` file in `tests/ron` is one test, named after
//! the file, played in a headless app with [`WidgetsPlugin`]. The files start
//! from a screen of [`widgets_setup::WidgetsSetup`] (`setup: App(Buttons)`).
//!
//! This target has its own `main` (`harness = false` in `Cargo.toml`), so
//! `cargo test --test ron_scenarios push_button` runs the files whose name
//! contains `push_button`; each file is played at scale factors 1 and 2 (`name`
//! and `name [scale 2]`), and with the text 1.5 times its default size
//! (`name [text 1.5x]`).

mod common;
mod widgets_check;
mod widgets_setup;

use std::process::ExitCode;

use bevy::text::DEFAULT_REM_SIZE_PX;

use rin_bevy_ui_tester::{Registry, Variant, run_ron_tests_with_variants};

use common::widgets_app;

fn main() -> ExitCode {
    // The default registry holds every type that derives `Reflect`, which is
    // what a file's `App(..)` values and reflected checks are loaded against.
    let registry = Registry::default();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ron");
    // Every file plays as written (scale factor 1) and again on a window with a
    // scale factor of 2 and the same logical size, so no file depends on the
    // display it runs on. A third time it plays with Bevy's `RemSize`, the root
    // font size the widgets' text is sized in, at 1.5 times its default of 20
    // pixels, so the widgets hold and fit larger text.
    let variants = [
        Variant::default(),
        Variant::new("scale 2").scale_factor(2.0),
        Variant::new("text 1.5x").rem_size(DEFAULT_REM_SIZE_PX * 1.5),
    ];
    run_ron_tests_with_variants(dir, &registry, &variants, |scenario| {
        let mut app = widgets_app();
        registry.install(app.world_mut());
        scenario.run_in(&mut app)
    })
}
