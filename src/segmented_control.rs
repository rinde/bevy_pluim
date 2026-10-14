//! `SegmentedControl`: a track of text segments, one per item of a [`Selection`], that picks one.

use std::collections::HashMap;

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::{Activate, Button};

use crate::focus::TabStop;
use crate::list::sync_selected;
use crate::select::{Item, ItemsChanged, Selection, SelectionParam};
use crate::theme::{Theme, ThemeColors};

/// The height of a segment, in logical pixels.
const SEGMENT_HEIGHT: f32 = 28.0;
/// The least width of a segment.
const SEGMENT_MIN_WIDTH: f32 = 40.0;
/// The padding between a segment's text and its sides.
const SEGMENT_SIDE_PADDING: f32 = 10.0;
/// The padding of the track around its segments.
const TRACK_PADDING: f32 = 3.0;
/// The gap between two segments.
const SEGMENT_GAP: f32 = 2.0;
/// The size of a segment's text.
const SEGMENT_FONT_SIZE: f32 = 13.0;

/// Registers the systems and observers of [`SegmentedControl`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct SegmentedControlPlugin;

impl Plugin for SegmentedControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_segmented_controls)
                    .run_if(resource_changed::<Theme>),
                selection_change,
                hover_change,
            ),
        )
        .add_observer(on_insert_selection)
        .register_type::<SegmentedControl>()
        .register_type::<Segment>();
    }
}

/// A track of text segments that picks one item, like a small tab bar or a switch.
///
/// Give it a [`Selection`] (own, with its [`Item<String>`]s, or a
/// [`Selection::Ref`] to one that other widgets share) of the kind
/// [`SelectionValue::One`](crate::select::SelectionValue::One); it shows one [`Segment`] per
/// item, with the selected one highlighted. A click on a segment (or Enter / Space on a focused
/// one) selects it, and a selection changed from elsewhere restyles the segments.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use rin_bevy_ui::segmented_control::SegmentedControl;
/// # use rin_bevy_ui::select::{Item, Selection};
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         SegmentedControl,
///         Selection::builder()
///             .one()
///             .build_with_items(["Auto", "Manual"].map(Item::new)),
///     ));
/// }
/// ```
///
/// Each segment is a [`Button`] and a tab stop, and carries [`Selected`] while it is the
/// selected one.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("SegmentedControl"),
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        align_self: AlignSelf::FlexStart,
        padding: UiRect::all(Val::Px(TRACK_PADDING)),
        column_gap: Val::Px(SEGMENT_GAP),
        ..default()
    },
    BackgroundColor::default(),
)]
pub struct SegmentedControl;

impl SegmentedControl {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_segmented_controls, Some(context.entity));
    }
}

/// One segment of a [`SegmentedControl`]: a button showing one item.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("Segment"),
    Button,
    Hovered,
    TabStop,
    BackgroundColor::default()
)]
pub struct Segment {
    /// The item this segment shows and selects.
    pub item_ref: Entity,
    /// The control this segment belongs to.
    pub control_ref: Entity,
}

/// Marks the text of a [`Segment`].
#[derive(Component, Debug, Copy, Clone)]
struct SegmentLabel;

/// The background and text colour of a segment from its state.
const fn segment_colors(colors: &ThemeColors, selected: bool, hovered: bool) -> (Color, Color) {
    if selected {
        (colors.segmented_selected, colors.segmented_selected_text)
    } else if hovered {
        (colors.segmented_track, colors.text_primary)
    } else {
        (colors.segmented_track, colors.text_secondary)
    }
}

/// Selects the item of the activated segment.
fn on_activate(t: On<Activate>, segments: Query<&Segment>, mut selections: SelectionParam) {
    let Ok(segment) = segments.get(t.event().entity) else {
        return;
    };
    selections.update_selection(segment.control_ref, |selection| {
        selection.set_selection([segment.item_ref])
    });
}

fn theme_segmented_controls(
    control_id: In<Option<Entity>>,
    theme: Res<Theme>,
    selections: SelectionParam,
    mut controls: Query<
        (Entity, &mut Node, &mut BackgroundColor),
        (With<SegmentedControl>, Without<Segment>),
    >,
    mut segments: Query<
        (&Segment, &Hovered, &mut Node, &mut BackgroundColor),
        Without<SegmentedControl>,
    >,
    mut labels: Query<(&ChildOf, &mut TextColor, &mut TextFont), With<SegmentLabel>>,
    states: Query<(&Segment, &Hovered)>,
) {
    for (control, mut node, mut background) in &mut controls {
        if control_id.0.is_some_and(|id| id != control) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = theme.colors.segmented_track;
    }

    for (segment, hovered, mut node, mut background) in &mut segments {
        if control_id.0.is_some_and(|id| id != segment.control_ref) {
            continue;
        }
        let selected = selections
            .get(segment.control_ref)
            .is_some_and(|selection| selection.is_selected(segment.item_ref));
        node.height = Val::Px(SEGMENT_HEIGHT);
        node.min_width = Val::Px(SEGMENT_MIN_WIDTH);
        node.padding = UiRect::axes(Val::Px(SEGMENT_SIDE_PADDING), Val::ZERO);
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = segment_colors(&theme.colors, selected, hovered.get()).0;
    }

    for (child_of, mut text_color, mut text_font) in &mut labels {
        let Ok((segment, hovered)) = states.get(child_of.parent()) else {
            continue;
        };
        if control_id.0.is_some_and(|id| id != segment.control_ref) {
            continue;
        }
        let selected = selections
            .get(segment.control_ref)
            .is_some_and(|selection| selection.is_selected(segment.item_ref));
        text_color.0 = segment_colors(&theme.colors, selected, hovered.get()).1;
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: FontSize::Px(SEGMENT_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }
}

/// Restyles the segments of a control when a segment gains or loses the pointer.
fn hover_change(segments: Query<&Segment, Changed<Hovered>>, mut commands: Commands) {
    for segment in &segments {
        commands.run_system_cached_with(theme_segmented_controls, Some(segment.control_ref));
    }
}

/// Keeps [`Selected`] and the colours of the segments in step with the selection, own or referenced.
fn selection_change(
    changed: Query<(), Changed<Selection>>,
    controls: Query<(Entity, Option<&Children>), With<SegmentedControl>>,
    segments: Query<(&Segment, Has<Selected>)>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    if changed.is_empty() {
        return;
    }
    for (control, children) in &controls {
        let Some(selection) = selections.get(control) else {
            continue;
        };
        for child in children.into_iter().flat_map(RelationshipTarget::iter) {
            let Ok((segment, was_selected)) = segments.get(child) else {
                continue;
            };
            sync_selected(
                &mut commands,
                child,
                was_selected,
                selection.is_selected(segment.item_ref),
            );
        }
        commands.run_system_cached_with(theme_segmented_controls, Some(control));
    }
}

struct ItemsChangedInput {
    control_ref: Entity,
    items: Vec<Entity>,
}

fn on_items_changed(
    input: In<ItemsChangedInput>,
    items_query: Query<&Item<String>>,
    controls: Query<Option<&Children>, With<SegmentedControl>>,
    segments: Query<(Entity, &Segment)>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let ItemsChangedInput { control_ref, items } = &*input;
    let Ok(children) = controls.get(*control_ref) else {
        return;
    };
    let selection = selections.get(*control_ref);

    let mut missing = items.clone();
    for (segment_id, segment) in children
        .into_iter()
        .flat_map(RelationshipTarget::iter)
        .filter_map(|c| segments.get(c).ok())
    {
        let before = missing.len();
        missing.retain(|item| *item != segment.item_ref);
        if before == missing.len() {
            // a segment without a corresponding item
            commands.entity(segment_id).despawn();
        }
    }

    for item_ref in missing {
        let Ok(item) = items_query.get(item_ref) else {
            continue;
        };
        let is_selected = selection.is_some_and(|selection| selection.is_selected(item_ref));
        commands.entity(*control_ref).with_children(|spawner| {
            let mut segment = spawner.spawn((
                Segment {
                    item_ref,
                    control_ref: *control_ref,
                },
                Node::default(),
            ));
            segment.observe(on_activate).with_children(|parent| {
                parent.spawn((Text::new(item.data.clone()), SegmentLabel, Pickable::IGNORE));
            });
            if is_selected {
                segment.insert(Selected);
            }
        });
    }
    commands.run_system_cached_with(theme_segmented_controls, Some(*control_ref));
}

fn on_insert_selection(
    t: On<Insert, Selection>,
    controls: Query<(), With<SegmentedControl>>,
    selections: SelectionParam,
    mut commands: Commands,
    mut existing_values_ref_map: Local<HashMap<Entity, Entity>>,
) {
    let control_ref = t.event().event_target();
    if controls.get(control_ref).is_err() {
        return;
    }
    let Some((values_ref, all_items)) = selections.get_item_values(control_ref) else {
        return;
    };
    if existing_values_ref_map.get(&control_ref) == Some(&values_ref) {
        return;
    }
    existing_values_ref_map.insert(control_ref, values_ref);

    commands
        .entity(values_ref)
        .observe(move |t: On<ItemsChanged>, mut commands: Commands| {
            commands.run_system_cached_with(
                on_items_changed,
                ItemsChangedInput {
                    control_ref,
                    items: t.event().items.iter().collect(),
                },
            );
        });

    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            control_ref,
            items: all_items.iter().collect(),
        },
    );
}
