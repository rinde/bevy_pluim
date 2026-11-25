//! Records every scenario in `tests/ron` offscreen. Needs a GPU, so it is built
//! only with the `record` feature; `RECORD_DIR` and `RECORD_CHECK_ONLY` steer
//! the output.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may abort loudly"
)]

#[expect(dead_code, reason = "the recorder builds its own app")]
mod common;
mod widgets_check;
mod widgets_setup;

use std::path::{Path, PathBuf};

use bevy_pluim::WidgetsPlugin;
use bevy_stagecraft::{Recorder, Registry, load_ron_file};

/// Scenarios expected to fail only when recorded; remove a name once it passes.
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
