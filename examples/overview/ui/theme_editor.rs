//! The theme editor page of the overview: the library's [`ThemeEditor`] with "Export", which
//! writes the edited theme as Rust text ([`theme_source`]) to the log and shows a toast. The
//! export is the overview's own: the library only edits.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private module"
)]

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;
use bevy_pluim::theme::{Theme, ThemeColors, ThemeMode};
use bevy_pluim::theme_editor::{ThemeEditor, color_slot, theme_parameters};
use std::fmt::Write as _;

pub(super) fn theme_editor(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Theme editor")}
                --
                ~{groups()}
                Children [
                    @variant("Export as Rust text", bsn! {
                        ~{PushButton::from_label("Export")}
                        on(export)
                    })
                    --
                    @variant("Editor for the theme in use", bsn! {
                        ~{ThemeEditor}
                    })
                ]
            ]
        },
    );
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
