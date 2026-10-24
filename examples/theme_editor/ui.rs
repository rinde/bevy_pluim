//! The theme editor: a row for each colour slot of `ThemeColors` with a swatch of its current
//! colour and a Tailwind palette combo to change it, for the theme in use.
//!
//! Shared by the `theme_editor` example and the headless tests in `tests/headless.rs`: add
//! [`ThemeEditorPlugin`] to an app with `WidgetsPlugin`. Picking a colour sets that slot of the
//! light or dark half of the `Theme` that is showing ([`Theme::set_colors`]), so every widget on
//! screen follows. Export writes the pair as Rust text ([`pair_source`]) to the log and shows a
//! toast.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private module"
)]

use bevy::prelude::*;
use bevy::reflect::structs::Struct;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::UiRoot;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette_combo::ColorPaletteCombo;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::list::ListLayout;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::scroll_area::ScrollArea;
use rin_bevy_ui::select::{Item, Selection, SelectionParam, SelectionValue};
use rin_bevy_ui::tailwind::COLORS;
use rin_bevy_ui::theme::{
    DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme, ThemeColors, ThemeMode,
};
use rin_bevy_ui::toast::Toast;
use std::fmt::Write as _;

/// Spawns the editor at startup and wires the pickers to the `Theme`.
#[derive(Debug, Clone, Copy)]
pub struct ThemeEditorPlugin;

impl Plugin for ThemeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pick)
            .add_systems(Startup, spawn_editor)
            .add_systems(Update, show_theme.run_if(resource_changed::<Theme>));
    }
}

/// On a palette combo: the `ThemeColors` field its pick sets.
#[derive(Component, Debug, Clone)]
pub struct ThemeSlot(pub String);

/// On a swatch: the `ThemeColors` field whose current colour it shows.
#[derive(Component, Debug, Clone)]
pub struct SlotSwatch(pub String);

/// On the label saying which half of the pair is being edited.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ModeLabel;

/// The slots the editor offers: those opaque in both default themes, since a palette colour has
/// no alpha (shadows and translucent overlays keep their defaults), in field order.
#[must_use]
pub fn editable_slots() -> Vec<String> {
    (0..DEFAULT_LIGHT_THEME_COLORS.field_len())
        .filter_map(|i| {
            let name = DEFAULT_LIGHT_THEME_COLORS.name_at(i)?;
            let opaque = |colors: &ThemeColors| {
                slot_color(colors, name).is_some_and(|c| (c.alpha() - 1.0).abs() < f32::EPSILON)
            };
            (opaque(&DEFAULT_LIGHT_THEME_COLORS) && opaque(&DEFAULT_DARK_THEME_COLORS))
                .then(|| name.to_owned())
        })
        .collect()
}

/// The colour of the slot `name`, `None` for a name that is not a colour field.
#[must_use]
pub fn slot_color(colors: &ThemeColors, name: &str) -> Option<Color> {
    colors.field(name)?.try_downcast_ref::<Color>().copied()
}

/// Sets the slot `name` of `colors`; `false` (and no change) for a name that is not a colour field.
pub fn set_slot_color(colors: &mut ThemeColors, name: &str, color: Color) -> bool {
    colors
        .field_mut(name)
        .and_then(|field| field.try_downcast_mut::<Color>())
        .map(|slot| *slot = color)
        .is_some()
}

/// `colors` as a Rust expression `const NAME: ThemeColors = ThemeColors { .. };`, one
/// `field: Color::srgba(..)` line for each slot, ready to paste into an app.
#[must_use]
pub fn colors_source(name: &str, colors: &ThemeColors) -> String {
    let mut source = format!("const {name}: ThemeColors = ThemeColors {{\n");
    for i in 0..colors.field_len() {
        let Some(field) = colors.name_at(i) else {
            continue;
        };
        let Some(color) = slot_color(colors, field) else {
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

fn spawn_editor(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2d);
    let items = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(COLORS.iter().map(|c| Item { data: c.color })),
        )
        .id();
    let current = *theme.colors();
    let rows = move |rows: &mut ChildSpawner| {
        for slot in editable_slots() {
            rows.spawn(row(items, &slot, &current));
        }
    };
    // Plain spawns, not scenes: the rows are built from the slots the theme has.
    let title = commands.spawn(Label::title("Theme editor")).id();
    let mode_label = commands.spawn((Label::secondary(""), ModeLabel)).id();
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
    let area = commands
        .spawn((
            ScrollArea::bundle(rows, true, false),
            Node {
                height: px(520.0),
                ..default()
            },
        ))
        .id();
    let card = commands
        .spawn((
            Card,
            Node {
                width: px(440.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                padding: UiRect::all(px(12.0)),
                ..default()
            },
        ))
        .add_children(&[title, mode_label, buttons, area])
        .id();
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::all(px(16.0)),
            ..default()
        })
        .add_child(card);
}

/// A row: a swatch of the slot's colour, its name and the palette combo that sets it.
fn row(items: Entity, slot: &str, colors: &ThemeColors) -> impl Bundle {
    let color = slot_color(colors, slot).unwrap_or(Color::NONE);
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            ..default()
        },
        children![
            (
                Node {
                    width: px(24.0),
                    height: px(24.0),
                    ..default()
                },
                BackgroundColor(color),
                SlotSwatch(slot.to_owned()),
            ),
            (
                Label::regular(slot),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ),
            (
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(items),
                ThemeSlot(slot.to_owned()),
            ),
        ],
    )
}

/// A pick on a slot's combo sets that slot of the half of the pair in use.
fn on_pick(
    insert: On<Insert<Selection>>,
    slots: Query<&ThemeSlot>,
    selections: SelectionParam,
    colors: Query<&Item<Color>>,
    mut theme: ResMut<Theme>,
) {
    let entity = insert.event_target();
    let Ok(slot) = slots.get(entity) else {
        return;
    };
    let Some(picked) = selections.get(entity).and_then(SelectionValue::single) else {
        return;
    };
    let Ok(color) = colors.get(picked) else {
        return;
    };
    let mode = theme.mode();
    let mut set = theme.colors_of(mode);
    if set_slot_color(&mut set, &slot.0, color.data) {
        theme.set_colors(mode, set);
    }
}

/// Keeps the swatches and the mode label in step with the `Theme`.
fn show_theme(
    theme: Res<Theme>,
    mut swatches: Query<(&SlotSwatch, &mut BackgroundColor)>,
    mut labels: Query<&mut Text, With<ModeLabel>>,
) {
    for (swatch, mut background) in &mut swatches {
        if let Some(color) = slot_color(theme.colors(), &swatch.0) {
            background.0 = color;
        }
    }
    let text = match theme.mode() {
        ThemeMode::Light => "Editing the light theme",
        ThemeMode::Dark => "Editing the dark theme",
    };
    for mut label in &mut labels {
        text.clone_into(&mut label.0);
    }
}

/// Writes the pair as Rust text to the log and says so in a toast.
fn export(
    _: On<Activate>,
    theme: Res<Theme>,
    roots: Query<Entity, With<UiRoot>>,
    mut commands: Commands,
) {
    info!("\n{}", pair_source(&theme));
    if let Some(root) = roots.iter().next() {
        commands.spawn((Toast::new("Exported the theme to the log"), ChildOf(root)));
    }
}
