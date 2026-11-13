//! `SegmentedControl`: a track of segments, one per item of a [`Selection`], that picks one.
//! A segment shows the item's text, an icon, or both.

use std::sync::Arc;

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::{Activate, Button};

use crate::focus::{
    AccessibilitySync, RovingGroup, RovingItem, TabStop, sync_label, sync_role, sync_toggled,
};
use crate::indicator::{self, Indicator};
use crate::list::sync_selected;
use crate::select::{Item, ItemsObserver, Selection, SelectionParam, items_in_order, watch_items};
use crate::theme::{Theme, ThemeColors};
use crate::tooltip::{OwnedTooltip, Tooltip};

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
/// The side of a segment's icon, in logical pixels.
const SEGMENT_ICON_SIZE: f32 = 16.0;
/// The gap between a segment's icon and its text.
const SEGMENT_ICON_GAP: f32 = 6.0;
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
                sync_icons,
                sync_label_text,
                hover_change,
                indicator::spawn::<SegmentIndicator, SegmentedControl>,
                indicator::animate::<SegmentIndicator, SegmentedControl, Segment>,
            )
                .chain(),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
        .add_observer(on_insert_selection)
        .register_type::<SegmentedControl>()
        .register_type::<Segment>()
        .register_type::<SegmentIndicator>()
        .register_type::<SegmentIcon>()
        .register_type::<SegmentLabels>()
        .register_type::<ItemIcon>();
    }
}

/// A track of segments that picks one item, like a small tab bar or a switch.
///
/// The track and its segments are rounded by [`Theme::control_radius`], like the other controls,
/// not by radii of their own.
///
/// Give it a [`Selection`] (own, with its [`Item<String>`]s, or a
/// [`Selection::Ref`] to one that other widgets share) of the kind
/// [`SelectionValue::One`](crate::select::SelectionValue::One); it shows one [`Segment`] per
/// item, with the selected one highlighted. A click on a segment (or Enter / Space on a focused
/// one) selects it, and a selection changed from elsewhere restyles the segments.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_pluim::segmented_control::SegmentedControl;
/// # use bevy_pluim::select::{Item, Selection};
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
/// A segment shows its item's text. An item that also carries an [`ItemIcon`] shows that icon,
/// 16 px, before the text; with [`SegmentLabels::Hidden`] on the control the icon is all it
/// shows, and a [`Tooltip`] with the text. An item without an icon always shows its text.
///
/// The selected segment is marked by a [`SegmentIndicator`] behind the segments, which slides to
/// a newly selected segment instead of jumping.
///
/// Each segment is a [`Button`] and carries [`Selected`] while it is the selected one. As a
/// whole the control is one tab stop, at its selected item (the first when none is selected).
/// Left and up move the focus to the previous item and right and down to the next, wrapping in
/// the order shown; Home and End go to the first and last. For a selection of at most one item
/// the arrows also pick the item they land on (never toggling it off); for a selection of many
/// they move the focus only, and Enter or Space toggle. Disabled and hidden items are skipped.
///
/// To a screen reader the control is a radio group and each segment a radio button, checked
/// while it is selected. The library gives the group no label; an app can insert Bevy's
/// `AccessibleLabel` on the control.
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
    SegmentLabels,
    RovingGroup = RovingGroup::radio(),
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::RadioGroup)),
)]
pub struct SegmentedControl;

impl SegmentedControl {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_segmented_controls, Some(context.entity));
    }
}

/// An icon for the segment of an item of a [`SegmentedControl`], put on the item entity next to
/// its [`Item<String>`] (like [`ItemNote`](crate::list::ItemNote) on a list item).
///
/// The image is white on transparent, like those of [`Icon`](crate::icon::Icon); the segment
/// tints it with the colour of its text. Inserting, changing or removing it updates the segment
/// in place.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ItemIcon(pub Handle<Image>);

/// Whether the segments of a [`SegmentedControl`] show their text next to their icon.
///
/// Insert a new value on the control to switch; segments are not respawned. It only affects
/// items with an [`ItemIcon`]: an item without one keeps its text, so no segment is empty. A
/// segment whose text is hidden gets a [`Tooltip`] with it, and its accessible name stays the
/// text.
#[derive(Component, Debug, Copy, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum SegmentLabels {
    /// The text is shown, after the icon if there is one.
    #[default]
    Shown,
    /// Segments with an icon show the icon alone.
    Hidden,
}

/// The icon of a [`Segment`] whose item has an [`ItemIcon`]: a 16 px image, tinted with the
/// segment's text colour.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[require(Name::new("SegmentIcon"))]
pub struct SegmentIcon;

/// One segment of a [`SegmentedControl`]: a button showing one item.
///
/// Only the widget builds one, and the component is immutable: the item and the control are
/// fixed for the life of the segment and are read with [`item`](Self::item) and
/// [`owner`](Self::owner). A segment's accessible name is its item's text, so callers supply
/// meaningful item labels.
///
/// ```compile_fail
/// use bevy_pluim::segmented_control::Segment;
/// # let (item, control) = (bevy::prelude::Entity::PLACEHOLDER, bevy::prelude::Entity::PLACEHOLDER);
/// let _ = Segment { item, owner: control };
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::segmented_control::Segment;
/// fn change(mut segments: Query<&mut Segment>) {}
/// bevy::ecs::system::assert_is_system(change);
/// ```
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(
    Name::new("Segment"),
    Button,
    Hovered,
    TabStop,
    BackgroundColor::default()
)]
pub struct Segment {
    item: Entity,
    owner: Entity,
}

impl Segment {
    pub(crate) const fn new(item: Entity, owner: Entity) -> Self {
        Self { item, owner }
    }

    /// The item this segment shows and selects.
    #[must_use]
    pub const fn item(&self) -> Entity {
        self.item
    }

    /// The [`SegmentedControl`] this segment belongs to.
    #[must_use]
    pub const fn owner(&self) -> Entity {
        self.owner
    }
}

/// The highlight behind the selected [`Segment`] of a [`SegmentedControl`].
///
/// It is a child of the control, placed absolutely behind the segments, and slides to the
/// selected segment when the selection changes; the segments themselves are transparent.
///
/// Only the widget builds one, and the component is immutable: the control is fixed for the life
/// of the indicator and is read with [`owner`](Self::owner).
///
/// ```compile_fail
/// use bevy_pluim::segmented_control::SegmentIndicator;
/// # let control = bevy::prelude::Entity::PLACEHOLDER;
/// let _ = SegmentIndicator { owner: control };
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::segmented_control::SegmentIndicator;
/// fn change(mut indicators: Query<&mut SegmentIndicator>) {}
/// bevy::ecs::system::assert_is_system(change);
/// ```
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(Name::new("SegmentIndicator"), Node, BackgroundColor::default())]
pub struct SegmentIndicator {
    owner: Entity,
}

impl SegmentIndicator {
    /// The [`SegmentedControl`] this indicator belongs to.
    #[must_use]
    pub const fn owner(&self) -> Entity {
        self.owner
    }
}

impl Indicator for SegmentIndicator {
    fn of(owner: Entity) -> Self {
        Self { owner }
    }

    fn owner(&self) -> Entity {
        self.owner
    }
}

/// Marks the text of a [`Segment`].
#[derive(Component, Debug, Copy, Clone)]
struct SegmentLabel;

/// The background and text colour of a segment from its state.
pub(crate) const fn segment_colors(
    colors: &ThemeColors,
    selected: bool,
    hovered: bool,
) -> (Color, Color) {
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
    selections.update_selection(segment.owner(), |selection| {
        selection.set_selection([segment.item()])
    });
}

fn theme_segmented_controls(
    control_id: In<Option<Entity>>,
    theme: Res<Theme>,
    selections: SelectionParam,
    mut controls: Query<
        (Entity, &mut Node, &mut BackgroundColor),
        (
            With<SegmentedControl>,
            Without<Segment>,
            Without<SegmentIndicator>,
        ),
    >,
    mut segments: Query<
        (&Segment, &mut Node, &mut BackgroundColor),
        (Without<SegmentedControl>, Without<SegmentIndicator>),
    >,
    mut indicators: Query<
        (&SegmentIndicator, &mut Node, &mut BackgroundColor),
        (Without<SegmentedControl>, Without<Segment>),
    >,
    mut labels: Query<(&ChildOf, &mut TextColor, &mut TextFont), With<SegmentLabel>>,
    mut icons: Query<(&ChildOf, &mut ImageNode), With<SegmentIcon>>,
    states: Query<(&Segment, &Hovered)>,
) {
    for (control, mut node, mut background) in &mut controls {
        if control_id.0.is_some_and(|id| id != control) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = theme.colors().segmented_track;
    }

    for (segment, mut node, mut background) in &mut segments {
        if control_id.0.is_some_and(|id| id != segment.owner()) {
            continue;
        }
        node.height = Val::Px(SEGMENT_HEIGHT);
        node.min_width = Val::Px(SEGMENT_MIN_WIDTH);
        node.padding = UiRect::axes(Val::Px(SEGMENT_SIDE_PADDING), Val::ZERO);
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        node.column_gap = Val::Px(SEGMENT_ICON_GAP);
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = Color::NONE;
    }

    for (indicator, mut node, mut background) in &mut indicators {
        if control_id.0.is_some_and(|id| id != indicator.owner()) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = theme.colors().segmented_selected;
    }

    for (child_of, mut text_color, mut text_font) in &mut labels {
        let Ok((segment, hovered)) = states.get(child_of.parent()) else {
            continue;
        };
        if control_id.0.is_some_and(|id| id != segment.owner()) {
            continue;
        }
        let selected = selections
            .get(segment.owner())
            .is_some_and(|selection| selection.is_selected(segment.item()));
        text_color.0 = segment_colors(theme.colors(), selected, hovered.get()).1;
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(SEGMENT_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }

    for (child_of, mut image) in &mut icons {
        let Ok((segment, hovered)) = states.get(child_of.parent()) else {
            continue;
        };
        if control_id.0.is_some_and(|id| id != segment.owner()) {
            continue;
        }
        let selected = selections
            .get(segment.owner())
            .is_some_and(|selection| selection.is_selected(segment.item()));
        let color = segment_colors(theme.colors(), selected, hovered.get()).1;
        if image.color != color {
            image.color = color;
        }
    }
}

/// Keeps each segment's icon, the visibility of its text and its tooltip in step with its item's
/// [`ItemIcon`] and its control's [`SegmentLabels`], in place.
fn sync_icons(
    changed_icons: Query<(), Changed<ItemIcon>>,
    mut removed_icons: RemovedComponents<ItemIcon>,
    changed_labels: Query<(), Changed<SegmentLabels>>,
    changed_items: Query<(), Changed<Item<String>>>,
    added: Query<(), Added<Segment>>,
    segments: Query<(
        Entity,
        &Segment,
        Option<&Tooltip>,
        Option<&OwnedTooltip>,
        Option<&Children>,
    )>,
    item_icons: Query<&ItemIcon>,
    items: Query<&Item<String>>,
    controls: Query<&SegmentLabels>,
    mut icons: Query<&mut ImageNode, With<SegmentIcon>>,
    mut labels: Query<&mut Node, With<SegmentLabel>>,
    mut commands: Commands,
) {
    let removed = removed_icons.read().count() > 0;
    if !removed
        && changed_icons.is_empty()
        && changed_labels.is_empty()
        && changed_items.is_empty()
        && added.is_empty()
    {
        return;
    }
    let mut restyle = Vec::new();
    for (segment_id, segment, tooltip, owned, children) in &segments {
        let children = children.map_or(&[][..], |c| &c[..]);
        let icon = item_icons.get(segment.item()).ok();
        let hidden = controls
            .get(segment.owner())
            .is_ok_and(|labels| *labels == SegmentLabels::Hidden)
            && icon.is_some();
        let existing_id = children.iter().copied().find(|c| icons.contains(*c));
        match (icon, existing_id) {
            (Some(icon), Some(id)) => {
                if let Ok(mut image) = icons.get_mut(id)
                    && image.image != icon.0
                {
                    image.image = icon.0.clone();
                }
            }
            (Some(icon), None) => {
                let id = commands
                    .spawn((
                        SegmentIcon,
                        ImageNode::new(icon.0.clone()),
                        Node {
                            width: Val::Px(SEGMENT_ICON_SIZE),
                            height: Val::Px(SEGMENT_ICON_SIZE),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .id();
                commands.entity(segment_id).insert_children(0, &[id]);
                restyle.push(segment.owner());
            }
            (None, Some(id)) => {
                commands.entity(id).despawn();
            }
            (None, None) => {}
        }
        let display = if hidden { Display::None } else { Display::Flex };
        for &child in children {
            if let Ok(mut node) = labels.get_mut(child)
                && node.display != display
            {
                node.display = display;
            }
        }
        // A tooltip whose text is not the one the widget added replaced it: it is the app's.
        let owned = match (owned, tooltip) {
            (Some(owned), Some(tooltip)) if owned.0 == tooltip.text => true,
            (Some(_), _) => {
                commands.entity(segment_id).remove::<OwnedTooltip>();
                false
            }
            (None, _) => false,
        };
        if hidden {
            if let Ok(item) = items.get(segment.item())
                && (tooltip.is_none() || owned)
                && tooltip.is_none_or(|t| t.text != item.data)
            {
                commands.entity(segment_id).insert((
                    Tooltip::new(item.data.clone()),
                    OwnedTooltip(item.data.clone()),
                ));
            }
        } else if owned {
            commands
                .entity(segment_id)
                .remove::<(Tooltip, OwnedTooltip)>();
        }
    }
    for control in restyle {
        commands.run_system_cached_with(theme_segmented_controls, Some(control));
    }
}

/// Shows the current text of each segment's item: a changed [`Item<String>`] updates its
/// segment's [`SegmentLabel`] in place.
fn sync_label_text(
    changed_items: Query<(), Changed<Item<String>>>,
    segments: Query<&Segment>,
    items: Query<&Item<String>>,
    mut labels: Query<(&ChildOf, &mut Text), With<SegmentLabel>>,
) {
    if changed_items.is_empty() {
        return;
    }
    for (child_of, mut text) in &mut labels {
        let Ok(segment) = segments.get(child_of.parent()) else {
            continue;
        };
        if let Ok(item) = items.get(segment.item())
            && text.0 != item.data
        {
            text.0.clone_from(&item.data);
        }
    }
}

/// Keeps the control a radio group and each segment a radio button named after its item's text,
/// toggled while it is [`Selected`]. Bevy makes a node a button again when its `Button` changes,
/// so this runs after it.
fn sync_accessibility(
    mut controls: Query<
        &mut AccessibilityNode,
        (
            With<SegmentedControl>,
            Without<Segment>,
            Changed<AccessibilityNode>,
        ),
    >,
    mut segments: ParamSet<(
        Query<
            (&Segment, &mut AccessibilityNode, Has<Selected>),
            Or<(
                Changed<AccessibilityNode>,
                Changed<Segment>,
                Added<Selected>,
            )>,
        >,
        Query<(&Segment, &mut AccessibilityNode, Has<Selected>)>,
    )>,
    items: Query<&Item<String>>,
    changed_items: Query<Entity, Changed<Item<String>>>,
    mut deselected: RemovedComponents<Selected>,
) {
    // The container is not a `Button`, so nothing of Bevy's resets its role; it is set once by
    // `#[require]` and re-synced only when its node changes (or is added).
    for mut node in &mut controls {
        sync_role(&mut node, Role::RadioGroup);
    }
    // A segment is looked at when Bevy reset its node, it is new, or it gained `Selected`; and
    // below when it lost `Selected` or its item's text changed. A quiet frame does none.
    for (segment, mut node, selected) in &mut segments.p0() {
        sync_segment(segment, &mut node, selected, &items);
    }
    let deselected: EntityHashSet = deselected.read().collect();
    let changed_items: EntityHashSet = changed_items.iter().collect();
    if deselected.is_empty() && changed_items.is_empty() {
        return;
    }
    let mut all = segments.p1();
    for (segment, mut node, selected) in &mut all {
        if changed_items.contains(&segment.item()) {
            sync_segment(segment, &mut node, selected, &items);
        }
    }
    for entity in deselected {
        if let Ok((segment, mut node, selected)) = all.get_mut(entity) {
            sync_segment(segment, &mut node, selected, &items);
        }
    }
}

/// Writes the role, label and toggled state of one segment's node, only where they differ.
fn sync_segment(
    segment: &Segment,
    node: &mut Mut<AccessibilityNode>,
    selected: bool,
    items: &Query<&Item<String>>,
) {
    if let Ok(item) = items.get(segment.item()) {
        sync_label(node, &item.data);
    }
    sync_role(node, Role::RadioButton);
    sync_toggled(node, Some(selected));
}

/// Restyles the segments of a control when a segment gains or loses the pointer.
fn hover_change(segments: Query<&Segment, Changed<Hovered>>, mut commands: Commands) {
    for segment in &segments {
        commands.run_system_cached_with(theme_segmented_controls, Some(segment.owner()));
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
                selection.is_selected(segment.item()),
            );
        }
        commands.run_system_cached_with(theme_segmented_controls, Some(control));
    }
}

struct ItemsChangedInput {
    control_ref: Entity,
    items: Arc<[Entity]>,
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

    // Linear reconciliation (see `List`): a segment stays when its item is wanted and no
    // earlier segment shows it; any other is despawned.
    let wanted: HashSet<Entity> = items.iter().copied().collect();
    let mut shown = HashSet::new();
    for (segment_id, segment) in children
        .into_iter()
        .flat_map(RelationshipTarget::iter)
        .filter_map(|c| segments.get(c).ok())
    {
        if !wanted.contains(&segment.item()) || !shown.insert(segment.item()) {
            commands.entity(segment_id).despawn();
        }
    }

    for &item_ref in items.iter().filter(|item_ref| !shown.contains(*item_ref)) {
        let Ok(item) = items_query.get(item_ref) else {
            continue;
        };
        let is_selected = selection.is_some_and(|selection| selection.is_selected(item_ref));
        commands.entity(*control_ref).with_children(|spawner| {
            let mut segment = spawner.spawn((
                Segment::new(item_ref, *control_ref),
                RovingItem::new(*control_ref, item_ref),
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
    commands.run_system_cached_with(
        order_segments,
        ItemsChangedInput {
            control_ref: *control_ref,
            items: Arc::clone(items),
        },
    );
    commands.run_system_cached_with(theme_segmented_controls, Some(*control_ref));
}

/// Puts the segments of a control in the order of its items, before any other child (the
/// indicator). Children are only reordered, and only when they differ.
fn order_segments(
    input: In<ItemsChangedInput>,
    controls: Query<&Children, With<SegmentedControl>>,
    segments: Query<&Segment>,
    mut commands: Commands,
) {
    let ItemsChangedInput { control_ref, items } = &*input;
    let Ok(children) = controls.get(*control_ref) else {
        return;
    };
    let wanted = items_in_order(children.iter(), items, |child| {
        segments.get(child).ok().map(Segment::item)
    });
    if !wanted.iter().copied().eq(children.iter()) {
        commands.entity(*control_ref).replace_children(&wanted);
    }
}

fn on_insert_selection(
    t: On<Insert<Selection>>,
    controls: Query<(), With<SegmentedControl>>,
    selections: SelectionParam,
    mut commands: Commands,
    watches: Query<&ItemsObserver>,
    observers: Query<(), With<Observer>>,
) {
    let control_ref = t.event().event_target();
    if controls.get(control_ref).is_err() {
        return;
    }
    let Some((values_ref, all_items)) = selections.get_items(control_ref) else {
        return;
    };
    let on_changed: fn(&mut Commands, Entity, Arc<[Entity]>) = |commands, control_ref, items| {
        commands.run_system_cached_with(on_items_changed, ItemsChangedInput { control_ref, items });
    };
    if !watch_items(
        &mut commands,
        &watches,
        &observers,
        control_ref,
        values_ref,
        on_changed,
    ) {
        return;
    }

    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            control_ref,
            items: all_items.iter().copied().collect(),
        },
    );
}
