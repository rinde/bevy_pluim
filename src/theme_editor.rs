//! `ThemeEditor`: a card that edits the [`Theme`] in use, colors and numbers, while the widgets follow.
//!
//! Fonts and the translucent color slots (the shadows) are not editable.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::reflect::structs::Struct;

use crate::card::Card;
use crate::color_palette_combo::ColorPaletteCombo;
use crate::label::Label;
use crate::list::ListLayout;
use crate::numeric_stepper::{NumericStepper, Value, ValueChange};
use crate::scroll_area::ScrollArea;
use crate::select::{Item, Selection, SelectionParam, SelectionValue};
use crate::tailwind::COLORS;
use crate::theme::{
    DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS, Theme, ThemeColors, ThemeMode,
};

/// Registers the systems of [`ThemeEditor`].
#[derive(Debug, Clone, Copy)]
pub struct ThemeEditorPlugin;

impl Plugin for ThemeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pick)
            .add_observer(on_step)
            .add_systems(Update, show_theme.run_if(theme_or_rows_changed));
    }
}

fn theme_or_rows_changed(
    theme: Res<Theme>,
    new: Query<(), Or<(Added<ThemeColorSwatch>, Added<ThemeParameterEditor>)>>,
) -> bool {
    theme.is_changed() || !new.is_empty()
}

const MAX_CONTENT_HEIGHT: f32 = 520.0;

/// A card that edits the [`Theme`]: see the [module](self).
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
#[require(
    Card = Card,
    Name::new("ThemeEditor"),
    Node {
        width: Val::Px(440.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.0),
        padding: UiRect::all(Val::Px(12.0)),
        ..default()
    },
)]
pub struct ThemeEditor;

impl ThemeEditor {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let editor = context.entity;
        let snapshot = world.get_resource::<Theme>().cloned().unwrap_or_default();
        let mut commands = world.commands();
        let colors = commands
            .spawn((
                Name::new("ThemeEditorColors"),
                Selection::builder()
                    .zero_or_one()
                    .build_with_items(COLORS.iter().map(|c| Item { data: c.color })),
                ChildOf(editor),
            ))
            .id();
        let mode_label = commands
            .spawn((Label::secondary(""), ThemeEditorModeLabel, ChildOf(editor)))
            .id();
        let scroll_area = commands
            .spawn((
                ScrollArea::bundle(
                    move |rows: &mut ChildSpawner| {
                        rows.spawn(rows_column()).with_children(|column| {
                            spawn_rows(column, colors, &snapshot);
                        });
                    },
                    true,
                    false,
                ),
                Node {
                    max_height: Val::Px(MAX_CONTENT_HEIGHT),
                    ..default()
                },
                ChildOf(editor),
            ))
            .id();
        commands.entity(editor).insert(ThemeEditorRefs {
            mode_label,
            scroll_area,
            colors,
        });
    }
}

/// The parts of a [`ThemeEditor`], on the editor itself. Only the widget builds it.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ThemeEditorRefs {
    mode_label: Entity,
    scroll_area: Entity,
    colors: Entity,
}

impl ThemeEditorRefs {
    /// The label that says which mode is being edited.
    #[must_use]
    pub const fn mode_label(&self) -> Entity {
        self.mode_label
    }

    /// The scroll area that holds the rows.
    #[must_use]
    pub const fn scroll_area(&self) -> Entity {
        self.scroll_area
    }

    /// The entity holding the Tailwind colors the palette combos choose from.
    #[must_use]
    pub const fn colors(&self) -> Entity {
        self.colors
    }
}

/// On a palette combo of a [`ThemeEditor`]: the [`ThemeColors`] field its pick sets.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ThemeColorSlot(pub String);

/// On a swatch of a [`ThemeEditor`]: the [`ThemeColors`] field whose current color it shows.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ThemeColorSwatch(pub String);

/// On a stepper of a [`ThemeEditor`]: the [`ThemeParameter::name`] whose number it sets.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ThemeParameterEditor(pub String);

/// On the label saying which mode is being edited.
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
struct ThemeEditorModeLabel;

/// A number of the [`Theme`] that the editor can change.
#[derive(Debug, Clone, Copy)]
pub struct ThemeParameter {
    /// The unique field path in the [`Theme`], such as `item_padding.left`.
    pub name: &'static str,
    /// The text of its row.
    pub label: &'static str,
    /// The heading of the group of rows it is in.
    pub section: &'static str,
    /// The least value the stepper goes to.
    pub min: f32,
    /// The greatest value the stepper goes to.
    pub max: f32,
    get: fn(&Theme) -> f32,
    set: fn(&mut Theme, f32),
    /// The [`Val`] this number is read from, `None` for an `f32`.
    raw: fn(&Theme) -> Option<Val>,
}

impl ThemeParameter {
    /// The number in `theme`; a [`Val`] that is not in pixels reads as 0.
    #[must_use]
    pub fn value(&self, theme: &Theme) -> f32 {
        (self.get)(theme)
    }

    /// Sets the number in `theme`, in pixels for a [`Val`], clamped to `min..=max`.
    pub fn apply(&self, theme: &mut Theme, value: f32) {
        (self.set)(theme, value.clamp(self.min, self.max));
    }

    /// The range of a row's stepper: `min..=max`, widened to include the current value.
    fn range_in(&self, theme: &Theme) -> (f32, f32) {
        let value = self.value(theme);
        if value.is_finite() {
            (self.min.min(value), self.max.max(value))
        } else {
            (self.min, self.max)
        }
    }

    /// The number a row shows: the value, or `min` when it is not finite.
    fn shown_in(&self, theme: &Theme) -> f32 {
        let value = self.value(theme);
        if value.is_finite() { value } else { self.min }
    }

    fn stepper_value(&self, theme: &Theme) -> Option<Value> {
        let (min, max) = self.range_in(theme);
        Value::new_f32(self.shown_in(theme), min, max, 1.0).ok()
    }

    fn issue_in(&self, theme: &Theme) -> Option<ThemeParameterIssue> {
        if !self.value(theme).is_finite() {
            Some(ThemeParameterIssue::NotFinite)
        } else if (self.raw)(theme).is_some_and(|val| !matches!(val, Val::Px(_))) {
            Some(ThemeParameterIssue::NotPixels)
        } else {
            None
        }
    }
}

/// Why the editor cannot show a number of the theme as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ThemeParameterIssue {
    NotFinite,
    /// A [`Val`] that is not in pixels.
    NotPixels,
}

impl ThemeParameterIssue {
    fn warn(self, parameter: &ThemeParameter) {
        match self {
            Self::NotFinite => warn!(
                "the theme editor shows {} at its minimum {}: the theme's value is not finite",
                parameter.name, parameter.min
            ),
            Self::NotPixels => warn!(
                "the theme editor reads {} as 0: the theme's value is not in pixels, and a step sets it in pixels",
                parameter.name
            ),
        }
    }
}

/// The pixels of `val`, 0 for any other unit.
const fn pixels(val: Val) -> f32 {
    match val {
        Val::Px(px) => px,
        _ => 0.0,
    }
}

macro_rules! parameter {
    ($name:expr, $label:expr, $section:expr, $min:expr, $max:expr, px $field:ident) => {
        ThemeParameter {
            name: $name,
            label: $label,
            section: $section,
            min: $min,
            max: $max,
            get: |theme| pixels(theme.$field),
            set: |theme, value| theme.$field = Val::Px(value),
            raw: |theme| Some(theme.$field),
        }
    };
    ($name:expr, $label:expr, $section:expr, $min:expr, $max:expr, px $field:ident . $side:ident) => {
        ThemeParameter {
            name: $name,
            label: $label,
            section: $section,
            min: $min,
            max: $max,
            get: |theme| pixels(theme.$field.$side),
            set: |theme, value| theme.$field.$side = Val::Px(value),
            raw: |theme| Some(theme.$field.$side),
        }
    };
    ($name:expr, $label:expr, $section:expr, $min:expr, $max:expr, size $field:ident) => {
        ThemeParameter {
            name: $name,
            label: $label,
            section: $section,
            min: $min,
            max: $max,
            get: |theme| theme.$field,
            set: |theme, value| theme.$field = value,
            raw: |_| None,
        }
    };
}

macro_rules! padding {
    ($field:ident, $label:expr) => {
        [
            parameter!(
                concat!(stringify!($field), ".left"),
                concat!($label, " left"),
                "Padding",
                0.0,
                40.0,
                px $field . left
            ),
            parameter!(
                concat!(stringify!($field), ".right"),
                concat!($label, " right"),
                "Padding",
                0.0,
                40.0,
                px $field . right
            ),
            parameter!(
                concat!(stringify!($field), ".top"),
                concat!($label, " top"),
                "Padding",
                0.0,
                40.0,
                px $field . top
            ),
            parameter!(
                concat!(stringify!($field), ".bottom"),
                concat!($label, " bottom"),
                "Padding",
                0.0,
                40.0,
                px $field . bottom
            ),
        ]
    };
}

const SHAPES: &str = "Shapes";
const TEXT: &str = "Text sizes";
const SHADOW: &str = "Shadow";

const ITEM_PADDING: [ThemeParameter; 4] = padding!(item_padding, "List item padding");
const COMPACT_PADDING: [ThemeParameter; 4] =
    padding!(compact_button_padding, "Compact button padding");
const COLOR_PADDING: [ThemeParameter; 4] = padding!(color_button_padding, "Colour button padding");

/// All the numbers the editor changes, in row order, grouped by `section`.
#[must_use]
pub fn theme_parameters() -> &'static [ThemeParameter] {
    static ALL: std::sync::LazyLock<Vec<ThemeParameter>> = std::sync::LazyLock::new(|| {
        let mut all = vec![
            parameter!("control_radius", "Control radius", SHAPES, 0.0, 32.0, px control_radius),
            parameter!("panel_radius", "Panel radius", SHAPES, 0.0, 40.0, px panel_radius),
            parameter!("border_thickness", "Border thickness", SHAPES, 0.0, 8.0, px border_thickness),
            parameter!("scroll_bar_width", "Scroll bar width", SHAPES, 2.0, 24.0, px scroll_bar_width),
            parameter!(
                "compact_button_height",
                "Compact button height",
                SHAPES,
                16.0,
                64.0,
                px compact_button_height
            ),
            parameter!(
                "compact_button_radius",
                "Compact button radius",
                SHAPES,
                0.0,
                32.0,
                px compact_button_radius
            ),
            parameter!("font_size_title", "Title", TEXT, 6.0, 48.0, size font_size_title),
            parameter!("font_size_body", "Body", TEXT, 6.0, 48.0, size font_size_body),
            parameter!("font_size_secondary", "Secondary", TEXT, 6.0, 48.0, size font_size_secondary),
            parameter!("font_size_small", "Small", TEXT, 6.0, 48.0, size font_size_small),
            parameter!("font_size_caption", "Caption", TEXT, 6.0, 48.0, size font_size_caption),
            parameter!("font_size_overline", "Overline", TEXT, 6.0, 48.0, size font_size_overline),
            parameter!("font_size_chip", "Chip", TEXT, 6.0, 48.0, size font_size_chip),
        ];
        all.extend(ITEM_PADDING);
        all.extend(COMPACT_PADDING);
        all.extend(COLOR_PADDING);
        all.extend([
            ThemeParameter {
                name: "shadow_style.x_offset",
                label: "Shadow x offset",
                section: SHADOW,
                min: -20.0,
                max: 20.0,
                get: |theme| pixels(theme.shadow_style().x_offset),
                set: |theme, value| theme.shadow_style_mut().x_offset = Val::Px(value),
                raw: |theme| Some(theme.shadow_style().x_offset),
            },
            ThemeParameter {
                name: "shadow_style.y_offset",
                label: "Shadow y offset",
                section: SHADOW,
                min: -20.0,
                max: 20.0,
                get: |theme| pixels(theme.shadow_style().y_offset),
                set: |theme, value| theme.shadow_style_mut().y_offset = Val::Px(value),
                raw: |theme| Some(theme.shadow_style().y_offset),
            },
            ThemeParameter {
                name: "shadow_style.spread_radius",
                label: "Shadow spread",
                section: SHADOW,
                min: -10.0,
                max: 20.0,
                get: |theme| pixels(theme.shadow_style().spread_radius),
                set: |theme, value| theme.shadow_style_mut().spread_radius = Val::Px(value),
                raw: |theme| Some(theme.shadow_style().spread_radius),
            },
            ThemeParameter {
                name: "shadow_style.blur_radius",
                label: "Shadow blur",
                section: SHADOW,
                min: 0.0,
                max: 40.0,
                get: |theme| pixels(theme.shadow_style().blur_radius),
                set: |theme, value| theme.shadow_style_mut().blur_radius = Val::Px(value),
                raw: |theme| Some(theme.shadow_style().blur_radius),
            },
        ]);
        all
    });
    &ALL
}

/// The color slots the editor offers: those opaque in both default themes, in field order.
#[must_use]
pub fn editable_color_slots() -> Vec<String> {
    (0..DEFAULT_LIGHT_THEME_COLORS.field_len())
        .filter_map(|i| {
            let name = DEFAULT_LIGHT_THEME_COLORS.name_at(i)?;
            let opaque = |colors: &ThemeColors| {
                color_slot(colors, name).is_some_and(|c| (c.alpha() - 1.0).abs() < f32::EPSILON)
            };
            (opaque(&DEFAULT_LIGHT_THEME_COLORS) && opaque(&DEFAULT_DARK_THEME_COLORS))
                .then(|| name.to_owned())
        })
        .collect()
}

/// The color of the slot `name`.
#[must_use]
pub fn color_slot(colors: &ThemeColors, name: &str) -> Option<Color> {
    colors.field(name)?.try_downcast_ref::<Color>().copied()
}

/// Sets the slot `name` of `colors`; `false` if there is no such slot.
pub fn set_color_slot(colors: &mut ThemeColors, name: &str, color: Color) -> bool {
    colors
        .field_mut(name)
        .and_then(|field| field.try_downcast_mut::<Color>())
        .map(|slot| *slot = color)
        .is_some()
}

/// Lets the pointer through to the scroll area below, so the wheel scrolls it over a row.
const fn see_through() -> Pickable {
    Pickable {
        should_block_lower: false,
        is_hoverable: true,
    }
}

/// The column of rows; the contents of a scroll area lay out in a row.
fn rows_column() -> impl Bundle {
    (
        Name::new("ThemeEditorRows"),
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        see_through(),
    )
}

fn spawn_rows(column: &mut ChildSpawner, colors: Entity, theme: &Theme) {
    column.spawn((Label::overline("Colours"), see_through()));
    for slot in editable_color_slots() {
        column.spawn(color_row(colors, &slot, theme.colors()));
    }
    let mut section = "";
    for parameter in theme_parameters() {
        if parameter.section != section {
            section = parameter.section;
            column.spawn((Label::overline(section), see_through()));
        }
        if let Some(row) = parameter_row(parameter, theme) {
            column.spawn(row);
        }
    }
}

fn color_row(colors: Entity, slot: &str, current: &ThemeColors) -> impl Bundle {
    let color = color_slot(current, slot).unwrap_or(Color::NONE);
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        see_through(),
        children![
            (
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    ..default()
                },
                BackgroundColor(color),
                ThemeColorSwatch(slot.to_owned()),
                see_through(),
            ),
            (
                Label::regular(slot),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                see_through(),
            ),
            (
                ColorPaletteCombo,
                ListLayout::VerticalGrid { columns: 11 },
                Selection::builder()
                    .zero_or_one()
                    .build_with_items_ref(colors),
                ThemeColorSlot(slot.to_owned()),
            ),
        ],
    )
}

fn parameter_row(parameter: &ThemeParameter, theme: &Theme) -> Option<impl Bundle> {
    let Some(value) = parameter.stepper_value(theme) else {
        error!("the theme editor has no row for {}", parameter.name);
        return None;
    };
    Some((
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        see_through(),
        children![
            (
                Label::regular(parameter.label),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                see_through(),
            ),
            (
                NumericStepper {
                    name: Some(parameter.label.to_owned()),
                    ..default()
                },
                value,
                ThemeParameterEditor(parameter.name.to_owned()),
            ),
        ],
    ))
}

fn on_pick(
    insert: On<Insert<Selection>>,
    slots: Query<&ThemeColorSlot>,
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
    if set_color_slot(&mut set, &slot.0, color.data) {
        theme.set_colors(mode, set);
    }
}

fn on_step(
    change: On<ValueChange>,
    parameters: Query<&ThemeParameterEditor>,
    mut theme: ResMut<Theme>,
) {
    let Ok(editor) = parameters.get(change.entity) else {
        return;
    };
    let Some(value) = change.value.as_f32() else {
        return;
    };
    if let Some(parameter) = theme_parameters().iter().find(|p| p.name == editor.0) {
        let (min, max) = parameter.range_in(&theme);
        (parameter.set)(&mut theme, value.clamp(min, max));
    }
}

fn show_theme(
    theme: Res<Theme>,
    mut swatches: Query<(&ThemeColorSwatch, &mut BackgroundColor)>,
    steppers: Query<(Entity, &ThemeParameterEditor, &Value)>,
    mut labels: Query<&mut Text, With<ThemeEditorModeLabel>>,
    mut warned: Local<std::collections::HashSet<(&'static str, ThemeParameterIssue)>>,
    mut commands: Commands,
) {
    for (swatch, mut background) in &mut swatches {
        if let Some(color) = color_slot(theme.colors(), &swatch.0)
            && background.0 != color
        {
            background.0 = color;
        }
    }
    for (stepper, editor, value) in &steppers {
        let Some(parameter) = theme_parameters().iter().find(|p| p.name == editor.0) else {
            continue;
        };
        if let Some(issue) = parameter.issue_in(&theme)
            && warned.insert((parameter.name, issue))
        {
            issue.warn(parameter);
        }
        let wanted = parameter.shown_in(&theme);
        if value
            .as_f32()
            .is_none_or(|shown| (shown - wanted).abs() > f32::EPSILON)
            && let Some(value) = parameter.stepper_value(&theme)
        {
            commands.entity(stepper).insert(value);
        }
    }
    let text = match theme.mode() {
        ThemeMode::Light => "Editing the light theme",
        ThemeMode::Dark => "Editing the dark theme",
    };
    for mut label in &mut labels {
        if label.0 != text {
            text.clone_into(&mut label.0);
        }
    }
}
