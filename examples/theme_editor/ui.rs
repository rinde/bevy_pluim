//! The theme editor example: the library's [`ThemeEditor`] in a window, with "Change theme"
//! to switch the half of the colour pair being edited and "Export", which writes the edited
//! theme as Rust text ([`theme_source`]) to the log and shows a toast. The export is the
//! example's: the library only edits.
//!
//! Shared by the `theme_editor` example and the headless tests in `tests/headless.rs`: add
//! [`ThemeEditorExamplePlugin`] to an app with `WidgetsPlugin`.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private module"
)]

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_pluim::UiRoot;
use bevy_pluim::label::Label;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::theme::{Theme, ThemeColors, ThemeMode};
use bevy_pluim::theme_editor::{ThemeEditor, color_slot, theme_parameters};
use bevy_pluim::toast::Toast;
use std::fmt::Write as _;

/// Spawns the editor and its buttons at startup.
#[derive(Debug, Clone, Copy)]
pub struct ThemeEditorExamplePlugin;

impl Plugin for ThemeEditorExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_editor);
    }
}

/// `colors` as a Rust expression `const NAME: ThemeColors = ThemeColors { .. };`, one
/// `field: Color::srgba(..)` line for each slot, ready to paste into an app.
#[must_use]
pub fn colors_source(name: &str, colors: &ThemeColors) -> String {
    use bevy::reflect::structs::Struct as _;
    let mut source = format!("const {name}: ThemeColors = ThemeColors {{\n");
    for i in 0..colors.field_len() {
        let Some(field) = colors.name_at(i) else {
            continue;
        };
        let Some(color) = color_slot(colors, field) else {
            continue;
        };
        let c = color.to_srgba();
        let _ = writeln!(
            source,
            "    {field}: Color::srgba({:?}, {:?}, {:?}, {:?}),",
            c.red, c.green, c.blue, c.alpha
        );
    }
    source.push_str("};\n");
    source
}

/// Both halves of `theme` as Rust text, for `Theme::with_colors(LIGHT, DARK)`.
#[must_use]
pub fn pair_source(theme: &Theme) -> String {
    format!(
        "{}\n{}",
        colors_source("LIGHT", &theme.colors_of(ThemeMode::Light)),
        colors_source("DARK", &theme.colors_of(ThemeMode::Dark)),
    )
}

/// The numbers of `theme` as a function building a `Theme` from the colour pair: the public
/// fields as struct fields (a padding as a `UiRect`), the private shadow geometry as comments.
#[must_use]
pub fn numbers_source(theme: &Theme) -> String {
    let mut fields = String::new();
    let mut comments = String::new();
    // The padding group (`UiRect`) whose fields are being written, if any.
    let mut open_group: Option<&str> = None;
    for parameter in theme_parameters() {
        let value = parameter.value(theme);
        let (group, side) = parameter
            .name
            .split_once('.')
            .unwrap_or((parameter.name, ""));
        if open_group.is_some_and(|open| open != group) {
            fields.push_str("        },\n");
            open_group = None;
        }
        if group == "shadow_style" {
            let _ = writeln!(comments, "    // shadow {side}: {value:?}");
        } else if side.is_empty() {
            let literal = if parameter.name.starts_with("font_size") {
                format!("{value:?}")
            } else {
                format!("Val::Px({value:?})")
            };
            let _ = writeln!(fields, "        {group}: {literal},");
        } else {
            if open_group.is_none() {
                let _ = writeln!(fields, "        {group}: UiRect {{");
                open_group = Some(group);
            }
            let _ = writeln!(fields, "            {side}: Val::Px({value:?}),");
        }
    }
    if open_group.is_some() {
        fields.push_str("        },\n");
    }
    format!(
        "fn theme() -> Theme {{\n{comments}    Theme {{\n{fields}        ..Theme::with_colors(LIGHT, DARK)\n    }}\n}}\n"
    )
}

/// The edited theme as Rust text: the colour pair and the function that builds the theme.
#[must_use]
pub fn theme_source(theme: &Theme) -> String {
    format!("{}\n{}", pair_source(theme), numbers_source(theme))
}

fn spawn_editor(mut commands: Commands) {
    commands.spawn(Camera2d);
    let title = commands.spawn(Label::title("Theme editor")).id();
    let change_theme = commands
        .spawn(PushButton::from_label("Change theme"))
        .observe(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        .id();
    let export = commands
        .spawn(PushButton::from_label("Export"))
        .observe(export)
        .id();
    let buttons = commands
        .spawn(Node {
            column_gap: px(8.0),
            ..default()
        })
        .add_children(&[change_theme, export])
        .id();
    let editor = commands.spawn(ThemeEditor).id();
    let column = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            ..default()
        })
        .add_children(&[title, buttons, editor])
        .id();
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::all(px(16.0)),
            ..default()
        })
        .add_child(column);
}

/// Writes the edited theme as Rust text to the log and says so in a toast.
fn export(
    _: On<Activate>,
    theme: Res<Theme>,
    roots: Query<Entity, With<UiRoot>>,
    mut commands: Commands,
) {
    info!("\n{}", theme_source(&theme));
    if let Some(root) = roots.iter().next() {
        commands.spawn((Toast::new("Exported the theme to the log"), ChildOf(root)));
    }
}
