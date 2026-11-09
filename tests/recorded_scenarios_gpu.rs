//! Records every scenario in `tests/ron` offscreen for review: PNGs at step
//! boundaries, a filmstrip and a GIF per scenario, a `trace.html`, and one
//! `index.html` for each scenario.
//!
//! It needs a GPU (or lavapipe: `WGPU_BACKEND=vulkan`), so it is only built
//! with the `record` feature and its test is ignored. `just record` runs it and
//! writes to `target/recordings`; `RECORD_DIR` overrides the directory. Set
//! `RECORD_CHECK_ONLY` to run the rendered checks without recording artifacts.

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
use bevy_stagecraft::{Recorder, Registry, load_ron_file};

/// Scenarios that pass headless but fail in a recording, because of a bug in a
/// widget that only shows with UI drawn to an image (#100). The Card demo
/// scenario is listed for #103 (palette combos got no popup when the app had
/// more than one `UiRoot`), fixed in the library but not yet seen passing in a
/// recording. They are still recorded, and show as failures in the index. The
/// test fails when one of them passes, so remove each name when its issue is
/// fixed.
const FAILS_WHEN_RECORDED: [&str; 0] = [];

include!(concat!(env!("OUT_DIR"), "/record_scenario_tests.rs"));

fn run_record_case(path: &str) {
    let path = Path::new(path);
    let registry = Registry::default();
    let scenario = load_ron_file(path, &registry)
        .unwrap_or_else(|error| panic!("{} does not load: {error}", path.display()));
    let recorder = Recorder::new(|app| {
        app.add_plugins(WidgetsPlugin);
        Registry::default().install(app.world_mut());
    })
    .grouped(group_of(path), scenario);
    let (scenarios, output_hint) = if std::env::var_os("RECORD_CHECK_ONLY").is_some() {
        (
            recorder.check().expect("rendered checks work").scenarios,
            String::new(),
        )
    } else {
        let output = std::env::var_os("RECORD_DIR")
            .map_or_else(
                || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/recordings"),
                PathBuf::from,
            )
            .join(path.file_stem().expect("scenario file stem"));
        let recording = recorder.record(&output).expect("recording works");
        let hint = format!(" (see {})", recording.index.display());
        (recording.scenarios, hint)
    };
    for scenario in &scenarios {
        let expected_failure = FAILS_WHEN_RECORDED.contains(&scenario.name.as_str());
        assert_ne!(
            scenario.passed(),
            expected_failure,
            "unexpected recording result{output_hint}: {scenario:#?}"
        );
    }
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
