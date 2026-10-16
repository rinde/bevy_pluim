//! The widget scenarios of `tests/ron/`, played in a real window.
//!
//! The widgets are in the primary window; the tester's **Scenarios** window
//! beside it lists every scenario file, one group per widget. Pick one to watch
//! it play, pause, step, loop or take over with the real mouse, or **Record** a
//! session by hand. The app starts on the buttons screen, which like every
//! screen has a "Change theme" button.
//!
//! ```text
//! cargo run --example gallery
//! ```
//!
//! With `--run-all` every scenario plays once, unattended, and the app closes
//! itself; the exit code is non-zero if one failed (`just gallery-test`). Words
//! that are not flags keep only the files whose name contains one of them, as
//! with `cargo test`: `just example gallery -- --run-all stack`.
//!
//! The screens and checks the files use are the ones of the tests, shared
//! through `#[path]`, so the library stays free of the tester.

#[path = "../tests/common/mod.rs"]
#[expect(
    dead_code,
    reason = "the headless app of `common` is only for the tests; the gallery needs its root node"
)]
mod common;
#[path = "../tests/widgets_check/mod.rs"]
mod widgets_check;
#[path = "../tests/widgets_setup/mod.rs"]
#[expect(dead_code, reason = "the screens' `Clicks` are only read by the tests")]
mod widgets_setup;

use std::fs;
use std::path::Path;

use bevy::app::AppExit;
use bevy::camera::Camera2d;
use bevy::prelude::*;
use rin_bevy_ui::select::{ItemOf, Selection};
use rin_bevy_ui::theme::Theme;
use rin_bevy_ui::{UiRoot, WidgetsPlugin};
use rin_bevy_ui_tester::{
    AppSetup, Gallery, GalleryEntry, GalleryPlugin, NotUnderTest, Registry, Scenario,
};
use widgets_setup::{Clicks, CurrentScreen, WidgetsSetup};

/// Where the scenario files are.
const SCENARIOS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ron");

/// Where recordings are written. Not in `tests/ron`: a recording is a draft
/// until someone moves it there.
const RECORDINGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recordings");

/// The group of a scenario file, by the start of its name. The first prefix
/// that matches wins, so a longer one comes before the shorter it starts with.
const GROUPS: [(&str, &str); 24] = [
    ("push_button", "PushButton"),
    ("link", "Link"),
    ("list", "List"),
    ("combo", "Combo"),
    ("color_palette_combo", "ColorPaletteCombo"),
    ("color_palette", "ColorPalette"),
    ("color_push_button", "ColorPushButton"),
    ("numeric_stepper", "NumericStepper"),
    ("slider", "Slider"),
    ("scroll_area", "ScrollArea"),
    ("segmented_control", "SegmentedControl"),
    ("filter_chips", "FilterChips"),
    ("breadcrumbs", "BreadCrumbs"),
    ("stack", "Stack"),
    ("tabs", "Stack"),
    ("tab_bar", "TabBar"),
    ("panel", "Panel"),
    ("setting_row", "SettingRow"),
    ("badge", "Badge"),
    ("icon_button", "IconButton"),
    ("tooltip", "Tooltip"),
    ("theme", "Theme"),
    ("accessibility", "Accessibility"),
    ("accessible", "Accessibility"),
];

/// The group for files that match no prefix of [`GROUPS`] (`selected.ron`).
const OTHER_GROUP: &str = "Selection";

/// Builds the screen the app starts on.
fn start_screen(world: &mut World) {
    if let Err(error) = WidgetsSetup::Buttons.build(world) {
        error!("cannot build the first screen: {error}");
    }
}

/// Puts the app back for a scenario: removes its UI, cameras and selections,
/// but not the gallery's, and resets the theme. The scenario's setup builds the screen.
fn reset_app(world: &mut World) {
    let mut doomed: Vec<Entity> = world
        .query_filtered::<Entity, (With<UiRoot>, Without<NotUnderTest>)>()
        .iter(world)
        .collect();
    doomed.extend(
        world
            .query_filtered::<Entity, (With<Camera2d>, Without<NotUnderTest>)>()
            .iter(world),
    );
    // Selections and their items are spawned beside the UI, not under it.
    doomed.extend(
        world
            .query_filtered::<Entity, With<Selection>>()
            .iter(world),
    );
    doomed.extend(world.query_filtered::<Entity, With<ItemOf>>().iter(world));
    for entity in doomed {
        // A root's descendants go with it; one may already be gone.
        if let Ok(entity) = world.get_entity_mut(entity) {
            entity.despawn();
        }
    }
    world.insert_resource(Theme::default());
    world.remove_resource::<CurrentScreen>();
    world.remove_resource::<Clicks>();
}

/// The text of the `//` comment at the top of a scenario file, as one line.
fn description(text: &str) -> String {
    text.lines()
        .map_while(|line| line.strip_prefix("//"))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ")
}

/// The catalogue: every file in `dir`, in name order, grouped by widget.
/// Only files whose name contains one of `filters` are in it, if there are any. A file that does not
/// load is skipped with a warning.
fn catalogue(dir: &Path, registry: &Registry, filters: &[String]) -> Gallery {
    let mut paths: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|file| file.ok().map(|file| file.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "ron"))
        .filter(|path| {
            let name = path.to_string_lossy();
            filters.is_empty() || filters.iter().any(|filter| name.contains(filter.as_str()))
        })
        .collect();
    paths.sort();
    let mut groups: Vec<(&str, Vec<GalleryEntry>)> = Vec::new();
    for path in paths {
        let loaded = fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|text| {
                Scenario::from_ron_with(&text, registry)
                    .map(|scenario| (scenario, description(&text)))
                    .map_err(|error| error.to_string())
            });
        let (scenario, described) = match loaded {
            Ok(loaded) => loaded,
            Err(error) => {
                warn!("skipping {}: {error}", path.display());
                continue;
            }
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let group = GROUPS
            .iter()
            .find(|(prefix, _)| stem.starts_with(prefix))
            .map_or(OTHER_GROUP, |(_, group)| group);
        let entry = GalleryEntry::new(scenario).described(described);
        match groups.iter_mut().find(|(name, _)| *name == group) {
            Some((_, entries)) => entries.push(entry),
            None => groups.push((group, vec![entry])),
        }
    }
    groups
        .into_iter()
        .fold(Gallery::new(), |gallery, (name, entries)| {
            gallery.group(name, entries)
        })
}

fn main() -> AppExit {
    let registry = Registry::default();
    // As with `cargo test`: a word that is not a flag filters by file name.
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| !argument.starts_with('-'))
        .collect();
    let mut plugin = GalleryPlugin::new(catalogue(Path::new(SCENARIOS), &registry, &filters))
        .with_reset(reset_app)
        .with_export_dir(RECORDINGS);
    if std::env::args().any(|arg| arg == "--run-all") {
        plugin = plugin.with_run_all();
    }
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rin-bevy-ui".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WidgetsPlugin)
        .add_plugins(plugin)
        .add_systems(Startup, start_screen)
        .run()
}
