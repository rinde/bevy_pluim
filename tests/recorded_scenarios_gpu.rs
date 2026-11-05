//! Records every scenario in `tests/ron` offscreen for review: PNGs at step
//! boundaries, a filmstrip and a GIF per scenario, a `trace.html`, and one
//! `index.html` with the failures first.
//!
//! It needs a GPU (or lavapipe: `WGPU_BACKEND=vulkan`), so it is only built
//! with the `record` feature and its test is ignored. `just record` runs it and
//! writes to `target/recordings`; `RECORD_DIR` overrides the directory.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may abort loudly"
)]

// `widgets_setup` builds its screens with `common::spawn_camera_and_root`; the
// recorder makes its own app, so `widgets_app` goes unused here.
#[expect(dead_code, reason = "the recorder builds its own app")]
mod common;
mod widgets_check;
mod widgets_setup;

use std::path::{Path, PathBuf};

use bevy_pluim::WidgetsPlugin;
use bevy_stagecraft::{Recorder, Registry, load_ron_file, ron_files};

/// Scenarios that pass headless but fail in a recording, because of a bug in a
/// widget that only shows with UI drawn to an image (#100). The Card demo
/// scenario is listed for #103 (palette combos got no popup when the app had
/// more than one `UiRoot`), fixed in the library but not yet seen passing in a
/// recording. They are still recorded, and show as failures in the index. The
/// test fails when one of them passes, so remove each name when its issue is
/// fixed.
const FAILS_WHEN_RECORDED: [&str; 0] = [];

#[test]
#[ignore = "needs a GPU: run by `just record`"]
fn gpu_records_every_ron_scenario() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ron");
    let registry = Registry::default();
    let mut recorder = Recorder::new(|app| {
        // The recorder's app renders for real, so `ClearColor` is already
        // there. The widgets, and the types a file's `App(..)` values name,
        // are what the scenarios need on top of it.
        app.add_plugins(WidgetsPlugin);
        Registry::default().install(app.world_mut());
    });
    for path in ron_files(dir).expect("tests/ron can be read") {
        let scenario = load_ron_file(&path, &registry)
            .unwrap_or_else(|error| panic!("{} does not load: {error}", path.display()));
        recorder = recorder.grouped(group_of(&path), scenario);
    }

    let output = std::env::var_os("RECORD_DIR").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/recordings"),
        PathBuf::from,
    );
    let recording = recorder.record(&output).expect("recording works");

    let failed: Vec<_> = recording
        .scenarios
        .iter()
        .filter(|scenario| !scenario.passed())
        .filter(|scenario| !FAILS_WHEN_RECORDED.contains(&scenario.name.as_str()))
        .map(|scenario| scenario.name.clone())
        .collect();
    let fixed: Vec<_> = recording
        .scenarios
        .iter()
        .filter(|scenario| scenario.passed())
        .filter(|scenario| FAILS_WHEN_RECORDED.contains(&scenario.name.as_str()))
        .map(|scenario| scenario.name.clone())
        .collect();
    assert!(
        fixed.is_empty(),
        "these scenarios pass when recorded now: remove them from FAILS_WHEN_RECORDED: {fixed:?}"
    );
    assert!(
        failed.is_empty(),
        "scenarios failed that did not fail before (see {}): {failed:?}",
        recording.index.display()
    );
}

/// The index group of a file: the widget its name starts with.
fn group_of(path: &Path) -> String {
    let stem = path.file_stem().unwrap().to_string_lossy();
    let widget = [
        "breadcrumbs",
        "color_palette",
        "color_push_button",
        "combo",
        "link",
        "list",
        "numeric_stepper",
        "push_button",
        "scroll_area",
        "stack",
        "tabs",
        "theme",
    ]
    .into_iter()
    .find(|widget| stem.starts_with(widget));
    widget.unwrap_or("other").replace('_', " ")
}
