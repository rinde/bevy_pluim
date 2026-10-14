//! `FilterChips`: a wrapping row of pill-shaped options, one of them active, each with an
//! optional count.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::focus::{AccessibilitySync, TabStop, sync_label, sync_role, sync_toggled};
use crate::select::{Item, Selection, SelectionParam};
use crate::theme::{Theme, ThemeColors};

/// The height of a chip.
const CHIP_HEIGHT: f32 = 30.0;
/// The padding left and right of the text of a chip.
const CHIP_PADDING: f32 = 11.0;
/// The gap between chips, along and across the row.
const CHIP_GAP: f32 = 6.0;
/// The gap between the label and the count of a chip.
const COUNT_GAP: f32 = 5.0;
/// The opacity of a count, against the label.
const COUNT_ALPHA: f32 = 0.6;

/// Registers the systems and observers of [`FilterChips`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct FilterChipsPlugin;

impl Plugin for FilterChipsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item<Chip>>()
            .register_type::<Chip>()
            .register_type::<ChipView>()
            .add_systems(
                Update,
                (
                    sync_chips,
                    (|| None::<Entity>)
                        .pipe(theme_chips)
                        .run_if(resource_changed::<Theme>),
                    restyle_changed_chips,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
            .add_observer(on_activate);
    }
}

/// A wrapping row of pill-shaped options over a [`Selection`] of [`Item<Chip>`]s, with the
/// active one filled.
///
/// Put it on an entity together with a selection that has [`Chip`] items, such as
/// `Selection::builder().one().build_with_items([Item { data: Chip::with_count("All", 4) }])`,
/// or with a [`Selection::Ref`] to one. The selection should be a `One`: exactly one chip is
/// active, and activating the active chip changes nothing. Which chips there are is the
/// application's: add and remove the items to change them. To change a count, change the
/// [`Chip`] in its [`Item`]; the chip is kept and its text changes.
///
/// A chip is a [`Button`] and a tab stop: Enter and Space on a focused chip pick it, like a
/// click. The chips wrap onto further lines when the row is too narrow.
///
/// There is no arrow-key navigation between chips: each chip is a tab stop of its own.
///
/// To a screen reader the row is a radio group and each chip a radio button named after its
/// label and count (`Waiting 3`), checked while it is active. The library gives the group no
/// label; an app can insert Bevy's `AccessibleLabel` on the row.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("FilterChips"),
    Node {
        flex_wrap: FlexWrap::Wrap,
        align_content: AlignContent::FlexStart,
        column_gap: Val::Px(CHIP_GAP),
        row_gap: Val::Px(CHIP_GAP),
        ..default()
    },
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::RadioGroup)),
)]
pub struct FilterChips;

/// What a chip of [`FilterChips`] shows: a label and an optional count.
#[derive(Debug, Clone, Eq, PartialEq, Reflect)]
pub struct Chip {
    /// The text of the chip.
    pub label: String,
    /// The number shown after the label, if any.
    pub count: Option<usize>,
}

impl Chip {
    /// A chip with only a label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            count: None,
        }
    }

    /// A chip with a label and a count.
    pub fn with_count(label: impl Into<String>, count: usize) -> Self {
        Self {
            label: label.into(),
            count: Some(count),
        }
    }
}

/// One chip of a [`FilterChips`]: a pill that picks its item when it is activated.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
pub struct ChipView {
    /// The [`Item<Chip>`] this chip shows.
    pub item: Entity,
    /// The [`FilterChips`] this chip belongs to.
    pub chips: Entity,
}

impl ChipView {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_chips, Some(context.entity));
    }
}

/// The text children of a [`ChipView`].
#[derive(Component, Debug, Copy, Clone)]
pub struct ChipRefs {
    /// The label, marked [`ChipLabel`].
    pub label: Entity,
    /// The count, marked [`ChipCount`]; its text is empty when the chip has no count.
    pub count: Entity,
}

/// Marks the label text of a chip.
#[derive(Component, Debug, Copy, Clone)]
pub struct ChipLabel;

/// Marks the count text of a chip.
#[derive(Component, Debug, Copy, Clone)]
pub struct ChipCount;

/// Spawns a chip for every item of a [`FilterChips`] that has none, and drops the chips of
/// items that are gone, keeping the order of the items.
fn sync_chips(
    chips: Query<(Entity, Option<&Children>), With<FilterChips>>,
    views: Query<&ChipView>,
    items: Query<&Item<Chip>>,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for (chips_ref, children) in &chips {
        let Some((_, values)) = selections.get_item_values(chips_ref) else {
            continue;
        };
        let wanted = values
            .iter()
            .filter(|item| items.contains(*item))
            .collect::<Vec<_>>();
        let existing = children
            .into_iter()
            .flat_map(RelationshipTarget::iter)
            .filter_map(|child| Some((child, views.get(child).ok()?.item)))
            .collect::<Vec<_>>();
        if existing
            .iter()
            .map(|(_, item)| *item)
            .eq(wanted.iter().copied())
        {
            continue;
        }
        for (view, _) in existing {
            commands.entity(view).despawn();
        }
        for item in wanted {
            spawn_chip(&mut commands, &theme, chips_ref, item);
        }
    }
}

fn chip_font(theme: &Theme) -> TextFont {
    TextFont {
        font: theme.body_font.clone().into(),
        font_size: FontSize::Px(theme.font_size_chip),
        weight: FontWeight::MEDIUM,
        ..default()
    }
}

fn spawn_chip(commands: &mut Commands, theme: &Theme, chips: Entity, item: Entity) {
    // The children are spawned under the chip's reserved entity, so no node is ever parentless.
    let chip = commands.spawn_empty().id();
    let label = commands
        .spawn((
            Text::new(""),
            ChipLabel,
            chip_font(theme),
            Pickable::IGNORE,
            ChildOf(chip),
        ))
        .id();
    let count = commands
        .spawn((
            Text::new(""),
            ChipCount,
            chip_font(theme),
            Pickable::IGNORE,
            ChildOf(chip),
        ))
        .id();
    commands.entity(chip).insert((
        Name::new("Chip"),
        ChipView { item, chips },
        ChipRefs { label, count },
        Button,
        Hovered::default(),
        TabStop,
        Node {
            height: Val::Px(CHIP_HEIGHT),
            padding: UiRect::horizontal(Val::Px(CHIP_PADDING)),
            border: UiRect::all(theme.border_thickness),
            border_radius: BorderRadius::all(Val::Px(CHIP_HEIGHT / 2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(theme.colors.chip_border),
        ChildOf(chips),
    ));
}

/// How a chip looks.
#[derive(Debug, Copy, Clone, PartialEq)]
struct ChipStyle {
    background: Color,
    border: Color,
    text: Color,
}

/// The look of a chip from its state: active, hovered or at rest. Used by the system that
/// restyles chips on every change, so a theme change under a resting pointer keeps the hover.
const fn chip_style(colors: &ThemeColors, active: bool, hovered: bool) -> ChipStyle {
    if active {
        ChipStyle {
            background: colors.chip_on,
            border: colors.chip_on,
            text: colors.chip_on_ink,
        }
    } else {
        ChipStyle {
            background: Color::NONE,
            border: colors.chip_border,
            text: if hovered {
                colors.text_primary
            } else {
                colors.text_secondary
            },
        }
    }
}

/// Puts the text, colours and fonts of one chip, or of every chip when `chip` is `None`, in
/// step with its item, its selection, the pointer and the theme. Run for a new chip from its
/// hook, for all of them when the [`Theme`] changes and by [`restyle_changed_chips`].
fn theme_chips(
    chip: In<Option<Entity>>,
    theme: Res<Theme>,
    selections: SelectionParam,
    items: Query<&Item<Chip>>,
    mut views: Query<(
        Entity,
        &ChipView,
        &ChipRefs,
        &Hovered,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut texts: Query<(&mut Text, &mut TextColor, &mut TextFont, &mut Node)>,
) {
    for (entity, view, refs, hovered, mut background, mut border) in &mut views {
        if chip.0.is_some_and(|id| id != entity) {
            continue;
        }
        let Ok(item) = items.get(view.item) else {
            continue;
        };
        let active = selections
            .get(view.chips)
            .is_some_and(|selection| selection.is_selected(view.item));
        let style = chip_style(&theme.colors, active, hovered.get());
        background.set_if_neq(BackgroundColor(style.background));
        border.set_if_neq(BorderColor::all(style.border));

        let count_text = item
            .data
            .count
            .map_or_else(String::new, |count| count.to_string());
        let parts = [
            (refs.label, item.data.label.as_str(), style.text, Val::ZERO),
            (
                refs.count,
                count_text.as_str(),
                style.text.with_alpha(COUNT_ALPHA),
                if item.data.count.is_some() {
                    Val::Px(COUNT_GAP)
                } else {
                    Val::ZERO
                },
            ),
        ];
        for (entity, string, color, margin) in parts {
            let Ok((mut text, mut text_color, mut font, mut node)) = texts.get_mut(entity) else {
                continue;
            };
            if text.0 != string {
                string.clone_into(&mut text.0);
            }
            text_color.set_if_neq(TextColor(color));
            if node.margin.left != margin {
                node.margin.left = margin;
            }
            *font = chip_font(&theme);
        }
    }
}

/// Restyles the chips whose state changed: the pointer came or went, their [`Chip`] changed,
/// or a selection changed. Does nothing in a frame where none did.
fn restyle_changed_chips(
    hovered: Query<Entity, (With<ChipView>, Changed<Hovered>)>,
    views: Query<(Entity, &ChipView)>,
    changed_items: Query<Entity, Changed<Item<Chip>>>,
    changed_selections: Query<(), Changed<Selection>>,
    mut commands: Commands,
) {
    if !changed_selections.is_empty() {
        commands.run_system_cached_with(theme_chips, None);
        return;
    }
    for chip in &hovered {
        commands.run_system_cached_with(theme_chips, Some(chip));
    }
    for item in &changed_items {
        for (chip, _) in views.iter().filter(|(_, view)| view.item == item) {
            commands.run_system_cached_with(theme_chips, Some(chip));
        }
    }
}

/// The accessible name of a chip: its label, and its count after a space when it has one.
fn accessible_name(chip: &Chip) -> String {
    chip.count.map_or_else(
        || chip.label.clone(),
        |count| format!("{} {count}", chip.label),
    )
}

/// Keeps the row a radio group and each chip a radio button named after its label and count,
/// toggled while it is active. Bevy makes a node a button again when its `Button` changes, and
/// takes the name from only one of the chip's two texts, so this runs after it.
fn sync_accessibility(
    mut rows: Query<&mut AccessibilityNode, (With<FilterChips>, Without<ChipView>)>,
    mut chips: Query<(&ChipView, &mut AccessibilityNode)>,
    items: Query<&Item<Chip>>,
    selections: SelectionParam,
) {
    for mut node in &mut rows {
        sync_role(&mut node, Role::RadioGroup);
    }
    for (view, mut node) in &mut chips {
        let Ok(item) = items.get(view.item) else {
            continue;
        };
        let active = selections
            .get(view.chips)
            .is_some_and(|selection| selection.is_selected(view.item));
        sync_role(&mut node, Role::RadioButton);
        sync_label(&mut node, &accessible_name(&item.data));
        sync_toggled(&mut node, Some(active));
    }
}

/// Picks the chip that was used (a click, or Enter or Space on it).
///
/// This relies on `toggle_selected` doing nothing for the active chip of a `One` selection: the
/// selection stays as it is, so activating the active chip changes nothing.
fn on_activate(t: On<Activate>, views: Query<&ChipView>, mut selections: SelectionParam) {
    let Ok(view) = views.get(t.event().event_target()) else {
        return;
    };
    selections.update_selection(view.chips, |selection| selection.toggle_selected(view.item));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS};

    #[test]
    fn the_active_chip_is_filled_and_the_others_are_outlined() {
        for colors in [DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS] {
            let active = chip_style(&colors, true, false);
            assert_eq!(active.background, colors.chip_on);
            assert_eq!(active.text, colors.chip_on_ink);
            let rest = chip_style(&colors, false, false);
            assert_eq!(rest.background, Color::NONE);
            assert_eq!(rest.border, colors.chip_border);
            assert_eq!(rest.text, colors.text_secondary);
        }
    }

    #[test]
    fn hovering_a_chip_that_is_not_active_makes_its_text_primary() {
        let colors = DEFAULT_LIGHT_THEME_COLORS;
        assert_eq!(chip_style(&colors, false, true).text, colors.text_primary);
        assert_eq!(
            chip_style(&colors, true, true),
            chip_style(&colors, true, false)
        );
    }
}
