//! `List`: renders the items of a [`Selection`] as views and keeps them in sync.

use std::sync::Arc;

use bevy::a11y::AccessibilityNode;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemId;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusedInput, InputFocus};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::{
    ActiveDescendant, Button, ListBox, ListBoxMultiSelect, ListItem, ReselectListRow, ValueChange,
};
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::combo::Combo;
use crate::filter_combo::FilterCombo;
use crate::focus::{AccessibilitySync, TabStop, sync_label};
use crate::select::{Item, ItemsChanged, Selection, SelectionParam, SelectionValue};
use crate::theme::{Theme, ThemeColors, contrast_ratio};
use crate::{MouseEvent, MouseKind};

/// Padding around a [`CornerStyle::Separated`] list.
pub(crate) const SEPARATED_LIST_PADDING: Val = Val::Px(6.0);
/// Gap between the rows of a [`CornerStyle::Separated`] list.
pub(crate) const SEPARATED_ROW_GAP: Val = Val::Px(2.0);
/// Width of the ring on the selected row of a [`CornerStyle::Separated`] list.
pub(crate) const SEPARATED_RING_WIDTH: Val = Val::Px(1.0);
/// Padding inside a row of a [`CornerStyle::Separated`] list: 10 pixels by 12.
pub(crate) const SEPARATED_ITEM_PADDING: UiRect = UiRect::axes(Val::Px(12.0), Val::Px(10.0));

pub(crate) struct ListPlugin;

impl Plugin for ListPlugin {
    fn build(&self, app: &mut App) {
        // Bevy's own type; registered so a check can read the row the keyboard is on.
        app.register_type::<ActiveDescendant>()
            .add_systems(
                Update,
                (
                    (|| None::<Entity>)
                        .pipe(theme_items)
                        .run_if(resource_changed::<Theme>),
                    (|| None::<Entity>)
                        .pipe(theme_lists)
                        .run_if(resource_changed::<Theme>),
                    (|| None::<Entity>)
                        .pipe(theme_item_labels)
                        .run_if(resource_changed::<Theme>),
                    selection_change,
                    custom_view_hover,
                    sync_item_notes.run_if(item_notes_changed),
                    sync_item_label_text,
                    note_ink_follows_row.after(selection_change),
                ),
            )
            .add_systems(PostUpdate, sync_item_labels.in_set(AccessibilitySync))
            .add_observer(on_insert_selection)
            .add_observer(on_insert_list)
            .add_observer(on_value_change)
            .add_observer(on_reselect_row)
            .add_observer(keep_navigation_keys)
            .add_observer(pass_first_presses_from_descendants)
            .add_observer(handle_custom_view_mouse_events)
            .register_type::<ItemView>()
            .register_type::<ItemNote>()
            .register_type::<NoteTone>()
            .register_type::<ListLayout>()
            .register_type::<CornerStyle>();
    }
}

/// Renders the items of a [`Selection`] as views, one per item, and keeps them in sync.
///
/// It is a [`ListBox`] and its item views are [`ListItem`]s, so an item of the
/// selection carries [`Selected`]. The [`Selection`] stays the one source of truth:
/// [`Selected`] follows it, and [`ListBoxMultiSelect`] is on the list while the selection
/// holds more than one item (`OneOrMore`, `ZeroOrMore`).
///
/// Use [`ListLayout`] and [`CornerStyle`] to change how they are laid out.
///
/// A list does not scroll: it is as large as its items. To scroll a list that
/// does not fit, put it in a [`ScrollArea`](crate::scroll_area::ScrollArea) (see
/// [`ScrollArea::bundle`](crate::scroll_area::ScrollArea::bundle)).
///
/// To cap its height, give that area a `max_height` (not a `height`): the area grows with the
/// list up to the maximum and then scrolls, as a [`Panel`](crate::panel::Panel) body does. A
/// list has no maximum height of its own, since a `ScrollArea` already scrolls it, keeps a
/// focused row in view and draws the scroll bar.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("List"),
    ListBox,
    TabStop,
    Node {
        position_type: PositionType::Relative,
        display: Display::Grid,
        ..default()
    },
    ListLayout::Vertical,
    CornerStyle::FollowLayout,
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub struct List;

/// How the item views of a [`List`] are arranged.
///
/// Changing it on a live list updates the grid of its [`Node`] (flow and columns) in place.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
pub enum ListLayout {
    /// One item per row.
    Vertical,
    /// One item per column.
    Horizontal,
    /// A grid that fills row by row.
    VerticalGrid {
        /// The number of columns. `0` has no meaning: the list is laid out as one column
        /// (`1`) and an `error!` is logged when the layout is inserted.
        columns: u16,
    },
}

impl ListLayout {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        if matches!(
            world.entity(context.entity).get::<Self>(),
            Some(Self::VerticalGrid { columns: 0 })
        ) {
            error!(
                "ListLayout::VerticalGrid {{ columns: 0 }} on {}: laying out as one column",
                context.entity
            );
        }
        apply_layout(&mut world, context.entity);
    }
}

/// Which corners of a [`List`] and its item views are rounded.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub enum CornerStyle {
    /// Round the outer corners of the list.
    FollowLayout,
    /// Round the corners on the outside of a tab bar only.
    Tabs,
    /// Square corners.
    None,
    /// Rows that stand apart on the surface they lie on: no border or background on the
    /// list, a little padding around it and a gap between the rows, each row rounded on
    /// all corners. A row under the pointer gets the hover background; the selected row
    /// gets the strong hover background and a ring in the selection colour, not a fill.
    Separated,
}

/// Makes a [`List`] build the contents of its item views with a system of the app's own,
/// instead of the default text.
///
/// The list still owns each [`ItemView`] node: it spawns one per item, keeps them in step
/// with [`ItemsChanged`], and handles hover, press and selection exactly as for the default
/// views. The creator runs once per view, right after it is spawned, and adds the
/// contents as children of [`ItemViewSpawner::view`]. The items need not be `Item<String>`.
///
/// A nested [`Button`], [`List`], [`Combo`] or [`FilterCombo`] handles its own pointer: a
/// press on it does not restyle the row and a click on it does not change the selection.
/// Other contents pass the pointer on to the row.
///
/// Add it with the [`List`], before the first update; the creator is a system registered
/// with `World::register_system`.
#[derive(Component, Debug, Copy, Clone)]
pub struct ItemViewCreator {
    /// The system that spawns the contents of a view, through the [`ItemViewSpawner`] it is given.
    pub create: SystemId<In<ItemViewSpawner>>,
}

impl ItemViewCreator {
    /// A list whose views are filled by `create`.
    #[must_use]
    pub const fn new(create: SystemId<In<ItemViewSpawner>>) -> Self {
        Self { create }
    }
}

/// Given to the creator system of an [`ItemViewCreator`]: the item to show and the (empty)
/// view to fill with children.
#[derive(Debug, Copy, Clone)]
pub struct ItemViewSpawner {
    /// The item the view shows.
    pub item: Entity,
    /// The [`ItemView`] node to spawn the contents under.
    pub view: Entity,
}

/// Marks an [`ItemView`] whose contents come from an [`ItemViewCreator`]. Such a view has
/// children that take the pointer, so it reads hover from [`Hovered`] and the pointer events
/// of its descendants instead of observing its own.
#[derive(Component, Debug, Copy, Clone)]
struct CustomView;

/// Opt out of the default item views, for a [`List`] that renders its items itself.
#[derive(Component, Copy, Clone, Default, Debug)]
pub struct NoDefaultItems;

impl List {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        apply_layout(&mut world, context.entity);
        world
            .commands()
            .run_system_cached_with(theme_lists, Some(context.entity));
    }
}

/// Sets the grid fields of the [`Node`] of `entity` from its [`ListLayout`], in place: the
/// fields a [`List`] owns (`display`, `position_type`, `grid_auto_flow` and
/// `grid_template_columns`) and nothing else, so a width or margin of the app survives.
/// Does nothing for an entity that is not a [`List`] or has no [`Node`] yet.
fn apply_layout(world: &mut DeferredWorld, entity: Entity) {
    let Some(layout) = world.entity(entity).get::<ListLayout>().copied() else {
        return;
    };
    if !world.entity(entity).contains::<List>() {
        return;
    }
    let grid_auto_flow = match layout {
        ListLayout::Vertical | ListLayout::VerticalGrid { columns: _ } => GridAutoFlow::RowDense,
        ListLayout::Horizontal => GridAutoFlow::ColumnDense,
    };
    let grid_template_columns = match layout {
        ListLayout::Vertical | ListLayout::Horizontal => vec![],
        ListLayout::VerticalGrid { columns } => vec![RepeatedGridTrack::auto(columns.max(1))],
    };
    let Some(mut node) = world.get_mut::<Node>(entity) else {
        return;
    };
    node.position_type = PositionType::Relative;
    node.display = Display::Grid;
    node.grid_auto_flow = grid_auto_flow;
    node.grid_template_columns = grid_template_columns;
}

/// One item of a [`List`] (or [`ColorPalette`](crate::color_palette::ColorPalette)).
///
/// It is a [`ListItem`]: it carries [`Selected`] while its item is selected. Its
/// [`Hovered`] says whether the pointer is on it, so that a change of the selection
/// keeps the hover colour of the view under the pointer.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Hovered, ListItem)]
#[component(on_insert = ItemView::on_insert)]
pub struct ItemView {
    /// The item this view shows.
    pub item_ref: Entity,
    /// The list this view belongs to.
    pub list_ref: Entity,
}

impl ItemView {
    /// Records the view on its item, so a change to the item's note finds the views to look at.
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(item) = world.get::<Self>(context.entity).map(|view| view.item_ref) else {
            return;
        };
        world
            .commands()
            .entity(context.entity)
            .insert(NoteViewOf(item));
    }
}

/// Private relationship from a view to the item it shows: the item's [`NoteViews`] lists the
/// views a change of its [`ItemNote`] concerns, whichever lists they belong to.
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = NoteViews)]
struct NoteViewOf(Entity);

/// The views that show an item.
#[derive(Component, Debug)]
#[relationship_target(relationship = NoteViewOf)]
struct NoteViews(Vec<Entity>);

/// A note shown after the label of an item.
///
/// It goes on the item entity next to its [`Item<String>`] and shows in the default view of a
/// [`List`] (so also in a [`Combo`]'s popup), in the caption size and the
/// colour of its [`NoteTone`].
///
/// A [`Combo`]'s button shows the label of the selected item only, never
/// the note. To a screen reader the row's name is the label followed by the note
/// (`Delft < unreachable >`). Inserting a new note, or removing it, updates the row in place; an
/// empty note shows nothing.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ItemNote {
    /// The text of the note.
    pub text: String,
    /// The colour the note is shown in.
    pub tone: NoteTone,
}

impl ItemNote {
    /// A note in the secondary text colour.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: NoteTone::Secondary,
        }
    }

    /// A note in the warning text colour.
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: NoteTone::Warning,
        }
    }
}

/// The colour of an [`ItemNote`], a slot of the theme.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum NoteTone {
    /// `text_secondary`.
    #[default]
    Secondary,
    /// `warning_text`.
    Warning,
}

/// Marks the text of an [`ItemNote`] inside a default [`ItemView`].
#[derive(Component, Debug, Clone, Copy)]
struct ItemNoteView(NoteTone);

/// The ink of a note of `tone` on a row drawn on `background`: the tone's own colour when
/// it reads at 4.5:1 there, else `text_primary`, which does on every row background. A
/// transparent row sits on the surface, where the tone's colour is meant to be read.
pub(crate) fn note_ink(colors: &ThemeColors, tone: NoteTone, background: Color) -> Color {
    let own = match tone {
        NoteTone::Secondary => colors.text_secondary,
        NoteTone::Warning => colors.warning_text,
    };
    // A transparent (alpha 0) row has no colour of its own to measure against, so a
    // note on it always keeps its own colour.
    if background.alpha() == 0.0 || contrast_ratio(own, background) >= 4.5 {
        own
    } else {
        colors.text_primary
    }
}

/// Keeps the note of a row readable when the row's background changes (hover, selection,
/// theme).
fn note_ink_follows_row(
    rows: Query<(&BackgroundColor, &Children), (With<ItemView>, Changed<BackgroundColor>)>,
    mut notes: Query<(&ItemNoteView, &mut TextColor)>,
    theme: Res<Theme>,
) {
    for (background, children) in &rows {
        for child in children {
            if let Ok((ItemNoteView(tone), mut color)) = notes.get_mut(*child) {
                color.set_if_neq(TextColor(note_ink(theme.colors(), *tone, background.0)));
            }
        }
    }
}

/// Whether a note, or a view that may show one, changed since the last run.
fn item_notes_changed(
    changed: Query<(), Or<(Changed<ItemNote>, Added<ItemView>)>>,
    removed: RemovedComponents<ItemNote>,
    theme: Res<Theme>,
) -> bool {
    !changed.is_empty() || !removed.is_empty() || theme.is_changed()
}

/// Shows, updates and removes the note text in each default view to match the item's
/// [`ItemNote`].
fn sync_item_notes(
    views: Query<(Entity, &ItemView, Option<&Children>, &BackgroundColor), Without<CustomView>>,
    changed_notes: Query<Entity, Changed<ItemNote>>,
    mut removed_notes: RemovedComponents<ItemNote>,
    new_views: Query<Entity, Added<ItemView>>,
    views_of_item: Query<&NoteViews>,
    notes: Query<&ItemNote>,
    mut note_texts: Query<(&mut Text, &mut TextColor, &mut TextFont, &mut ItemNoteView)>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    // A theme change restyles every note; otherwise only the new views and the views of the
    // items whose note changed or went are looked at, so an unrelated list costs nothing.
    let affected: Vec<Entity> = if theme.is_changed() {
        views.iter().map(|(view, ..)| view).collect()
    } else {
        let mut affected: Vec<Entity> = new_views.iter().collect();
        for item in changed_notes.iter().chain(removed_notes.read()) {
            if let Ok(of_item) = views_of_item.get(item) {
                affected.extend(of_item.0.iter().copied());
            }
        }
        affected
    };
    // Drain the removals when the theme branch skipped them, or they would be read again.
    removed_notes.clear();
    for view in affected {
        let Ok((view, ItemView { item_ref, .. }, children, background)) = views.get(view) else {
            continue;
        };
        let wanted = notes
            .get(*item_ref)
            .ok()
            .filter(|note| !note.text.is_empty());
        let existing = children
            .into_iter()
            .flatten()
            .copied()
            .find(|child| note_texts.contains(*child));
        let color = |note: &ItemNote| note_ink(theme.colors(), note.tone, background.0);
        let font = || TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(theme.font_size_caption),
            ..default()
        };
        match (wanted, existing) {
            (Some(note), Some(child)) => {
                if let Ok((mut text, mut text_color, mut text_font, mut view_tone)) =
                    note_texts.get_mut(child)
                {
                    if view_tone.0 != note.tone {
                        view_tone.0 = note.tone;
                    }
                    if text.0 != note.text {
                        text.0.clone_from(&note.text);
                    }
                    text_color.set_if_neq(TextColor(color(note)));
                    text_font.set_if_neq(font());
                }
            }
            (Some(note), None) => {
                commands.entity(view).with_children(|parent| {
                    parent.spawn((
                        Text(note.text.clone()),
                        ItemNoteView(note.tone),
                        TextColor(color(note)),
                        font(),
                        Node {
                            margin: UiRect::left(Val::Px(8.0)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
            }
            (None, Some(child)) => commands.entity(child).despawn(),
            (None, None) => {}
        }
    }
}

/// Shows the new text of a renamed `Item<String>` in the label of each default row of it,
/// leaving every other row and custom views alone.
fn sync_item_label_text(
    renamed: Query<(&Item<String>, &NoteViews), Changed<Item<String>>>,
    views: Query<&Children, (With<ItemView>, Without<CustomView>)>,
    mut labels: Query<&mut Text, With<ItemLabel>>,
) {
    for (item, of_item) in &renamed {
        for view in &of_item.0 {
            let Ok(children) = views.get(*view) else {
                continue;
            };
            for child in children {
                if let Ok(mut text) = labels.get_mut(*child)
                    && text.0 != item.data
                {
                    text.0.clone_from(&item.data);
                }
            }
        }
    }
}

/// Names each default row to assistive technology: its label, followed by its note if it has
/// one. Bevy names a node from one of its texts, which would be the note alone.
fn sync_item_labels(
    mut views: Query<(&ItemView, &mut AccessibilityNode), Without<CustomView>>,
    items: Query<&Item<String>>,
    notes: Query<&ItemNote>,
) {
    for (ItemView { item_ref, .. }, mut node) in &mut views {
        let Ok(item) = items.get(*item_ref) else {
            continue;
        };
        match notes
            .get(*item_ref)
            .ok()
            .filter(|note| !note.text.is_empty())
        {
            Some(note) => sync_label(&mut node, &format!("{} {}", item.data, note.text)),
            None => sync_label(&mut node, &item.data),
        }
    }
}

/// The text of a default item view, themed when it is inserted and when the [`Theme`] changes.
#[derive(Component, Debug, Clone, Copy)]
#[component(on_insert=Self::on_insert)]
pub(crate) struct ItemLabel;

impl ItemLabel {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_item_labels, Some(context.entity));
    }
}

fn handle_mouse_events(
    t: On<MouseEvent>,
    selections: SelectionParam,
    mut item_views: Query<(&ItemView, &mut BackgroundColor)>,
    styles: Query<&CornerStyle>,
    theme: Res<Theme>,
) {
    // A click and a drag say nothing new about the pointer being on the view: the over,
    // press, release and out events around them set the colours. A drag in particular can
    // arrive after the pointer left the view, so it must not put it back under the pointer.
    if matches!(t.event().kind, MouseKind::Click(_) | MouseKind::Drag) {
        return;
    }
    let Ok((
        ItemView {
            item_ref: target_item_entity,
            list_ref,
        },
        mut background_color,
    )) = item_views.get_mut(t.event().event_target())
    else {
        return;
    };
    let Some(selection) = selections.get(*list_ref) else {
        return;
    };
    let is_selected = selection.is_selected(*target_item_entity);

    // Selecting and deselecting are the `ListBox`'s events, see `on_value_change` and
    // `on_reselect_row`; a release only recolours.

    // The pointer is on the view for every event but `Out`; a press looks selected.
    let hovered = !matches!(t.event().kind, MouseKind::Out);
    let pressed = matches!(t.event().kind, MouseKind::Pressed);
    let style = styles.get(*list_ref).copied().unwrap_or(CornerStyle::None);
    background_color.0 = item_background(theme.colors(), style, is_selected || pressed, hovered);
}

/// The [`ItemView`] a pointer event on `target` belongs to, if `target` is a custom view or
/// lies under one without a nested button, list, combo or filter combo in between.
fn custom_view_of(
    target: Entity,
    parents: &Query<&ChildOf>,
    views: &Query<(), With<CustomView>>,
    nested: &Query<(), Or<(With<Button>, With<List>, With<Combo>, With<FilterCombo>)>>,
) -> Option<Entity> {
    let mut current = target;
    loop {
        if views.contains(current) {
            return Some(current);
        }
        if nested.contains(current) {
            return None;
        }
        current = parents.get(current).ok()?.parent();
    }
}

/// Stops a click that began inside a button, list, combo or filter combo in a custom view from
/// reaching the outer list, which would take it as a pick of the row.
///
/// The `ListBox` searches the ancestors of a click's original target for a row, whatever
/// the target is, so a click on a combo inside a row (its root node or its popup, which let
/// the pointer through) selects the row. A click whose original target is the row or lies
/// under it without such a widget in between is the row's and goes on.
fn keep_nested_clicks_from_row(
    mut click: On<PointerClick>,
    parents: Query<&ChildOf>,
    custom_views: Query<(), With<CustomView>>,
    nested: Query<(), Or<(With<Button>, With<List>, With<Combo>, With<FilterCombo>)>>,
) {
    let row = click.entity;
    if custom_view_of(
        click.original_event_target(),
        &parents,
        &custom_views,
        &nested,
    ) != Some(row)
    {
        click.propagate(false);
    }
}

fn handle_custom_view_mouse_events(
    t: On<MouseEvent>,
    parents: Query<&ChildOf>,
    custom_views: Query<(), With<CustomView>>,
    nested: Query<(), Or<(With<Button>, With<List>, With<Combo>, With<FilterCombo>)>>,
    selections: SelectionParam,
    mut item_views: Query<(&ItemView, &Hovered, &mut BackgroundColor)>,
    styles: Query<&CornerStyle>,
    theme: Res<Theme>,
) {
    let kind = t.event().kind;
    if !matches!(kind, MouseKind::Pressed | MouseKind::Released) {
        return;
    }
    let Some(view) = custom_view_of(t.event().event_target(), &parents, &custom_views, &nested)
    else {
        return;
    };
    let Ok((item_view, hovered, mut background_color)) = item_views.get_mut(view) else {
        return;
    };
    let Some(is_selected) = selections
        .get(item_view.list_ref)
        .map(|selection| selection.is_selected(item_view.item_ref))
    else {
        return;
    };
    let style = styles
        .get(item_view.list_ref)
        .copied()
        .unwrap_or(CornerStyle::None);
    if matches!(kind, MouseKind::Pressed) {
        background_color.0 = item_background(theme.colors(), style, true, hovered.get());
        return;
    }
    // Deselecting is the `ListBox`'s event, see `on_reselect_row`.
    background_color.0 = item_background(theme.colors(), style, is_selected, hovered.get());
}

/// Colours a custom view when the pointer enters or leaves it or anything inside it.
fn custom_view_hover(
    mut item_views: Query<
        (&ItemView, &Hovered, &mut BackgroundColor),
        (With<CustomView>, Changed<Hovered>),
    >,
    selections: SelectionParam,
    styles: Query<&CornerStyle>,
    theme: Res<Theme>,
) {
    for (item_view, hovered, mut background_color) in &mut item_views {
        let style = styles
            .get(item_view.list_ref)
            .copied()
            .unwrap_or(CornerStyle::None);
        let is_selected = selections
            .get(item_view.list_ref)
            .is_some_and(|selection| selection.is_selected(item_view.item_ref));
        background_color.0 = item_background(theme.colors(), style, is_selected, hovered.get());
    }
}

/// The background of an [`ItemView`] from its state: selected and hovered, selected,
/// hovered, or at rest.
pub(crate) const fn item_background(
    colors: &ThemeColors,
    style: CornerStyle,
    selected: bool,
    hovered: bool,
) -> Color {
    if matches!(style, CornerStyle::Separated) {
        // The selected row is told apart by its ring; the rest sits on the surface.
        return match (selected, hovered) {
            (true, _) => colors.hovered_selected_item,
            (false, true) => colors.hovered_item,
            (false, false) => Color::NONE,
        };
    }
    match (selected, hovered) {
        (true, true) => colors.selected_hovered_item,
        (true, false) => colors.selected_item,
        (false, true) => colors.hovered_item,
        (false, false) => colors.background_primary,
    }
}

/// The border colour of an [`ItemView`]: the selection ring on a selected row of a
/// [`CornerStyle::Separated`] list (transparent on the others), the plain and highlight
/// borders elsewhere.
pub(crate) const fn item_border(colors: &ThemeColors, style: CornerStyle, selected: bool) -> Color {
    match (style, selected) {
        (CornerStyle::Separated, true) => colors.selection_ring,
        (CornerStyle::Separated, false) => Color::NONE,
        (_, true) => colors.border_highlight,
        (_, false) => colors.border,
    }
}

/// Selects the item whose row the [`ListBox`] picked (a click, or Enter or Space on the
/// active row), as far as the [`Selection`] allows.
///
/// The `ListBox` sends a [`ValueChange`] for a row that is not [`Selected`], and
/// does nothing more: `listbox_update_selection`, which would insert and remove
/// [`Selected`] by itself, past the rules of the [`Selection`], is not registered.
/// [`Selected`] is put on the rows by `selection_change` once the selection has changed.
///
/// The `ListBox` sends it for a click on any row but the first selected one, selected rows
/// included, so a click on a selected row deselects it here, as far as the selection allows.
fn on_value_change(
    t: On<ValueChange<Entity>>,
    lists: Query<(), With<List>>,
    item_views: Query<&ItemView>,
    mut selections: SelectionParam,
) {
    let ValueChange { source, value, .. } = *t.event();
    if lists.get(source).is_err() {
        return;
    }
    let Ok(item_view) = item_views.get(value) else {
        return;
    };
    // A nested list (a combo's popup in a custom view) can report a row of the list around
    // it: `listbox_on_row_click` searches the ancestors of a click on the nested list itself.
    if item_view.list_ref != source {
        return;
    }
    selections.update_selection(source, |selection| {
        selection.toggle_selected(item_view.item_ref)
    });
}

/// Toggles the already-selected row the `ListBox` reports as chosen again: Space or Enter on
/// the active row, or a click on the first selected row. `toggle_selected` is `None` for `One`,
/// so that selection keeps its item.
fn on_reselect_row(
    t: On<ReselectListRow>,
    lists: Query<(), With<List>>,
    item_views: Query<&ItemView>,
    mut selections: SelectionParam,
) {
    let ReselectListRow { source, row } = *t.event();
    if lists.get(source).is_err() {
        return;
    }
    let Ok(item_view) = item_views.get(row) else {
        return;
    };
    if item_view.list_ref != source {
        return;
    }
    selections.update_selection(source, |selection| {
        selection.toggle_selected(item_view.item_ref)
    });
}

/// Keeps the keys a focused [`List`] navigates with from bubbling past it.
///
/// The `ListBox` stops a first press of an arrow key, Home or End itself, but lets a held
/// key's repeats and the release go on up to the window, where an app's directional
/// navigation would move the focus out of the list. The list owns those keys while it has
/// the focus, so none of them goes further. Keys that bubble up from a focused descendant
/// (a button in a custom item view) are not the list's: those go on past it, see
/// [`pass_first_presses_from_descendants`].
fn keep_navigation_keys(
    mut input: On<FocusedInput<KeyboardInput>>,
    lists: Query<(), With<List>>,
    focus: Res<InputFocus>,
) {
    let list = input.event_target();
    if focus.get() != Some(list) || !lists.contains(list) {
        return;
    }
    if is_navigation_key(input.event().input.key_code) {
        input.propagate(false);
    }
}

/// Lets a first press of an arrow key, Home or End from a focused descendant of a [`List`]
/// (a button in a custom item view) go on past the list.
///
/// Unlike the repeats and the release of that key, which bubble through the [`List`] entity
/// on their way up, the first press skips that one hop: an observer on the list sees a held
/// arrow's repeats and its release, but not its first press.
///
/// The `ListBox` takes any first press of those keys that reaches it as its own, because the
/// `focused_entity` it checks is the entity the bubbling event has reached, not the focus.
/// So on the child of the list that the event comes through, the press is stopped before
/// it reaches the list and sent on from the list's parent (the window, for a list without
/// one): the active row stays where it is and an app's directional navigation gets the key.
fn pass_first_presses_from_descendants(
    mut input: On<FocusedInput<KeyboardInput>>,
    parents: Query<&ChildOf>,
    lists: Query<Option<&ChildOf>, With<List>>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    let key = &input.event().input;
    if key.state != ButtonState::Pressed || key.repeat || !is_navigation_key(key.key_code) {
        return;
    }
    let Ok(child_of) = parents.get(input.event_target()) else {
        return;
    };
    let Ok(list_parent) = lists.get(child_of.parent()) else {
        return;
    };
    let Some(next) = list_parent
        .map(ChildOf::parent)
        // The primary window Bevy's `dispatch_focused_input` sent the event with: it only
        // dispatches while there is exactly one, so the first is that one.
        .or_else(|| windows.iter().next())
    else {
        return;
    };
    input.propagate(false);
    let mut passed = input.event().clone();
    passed.focused_entity = next;
    commands.trigger(passed);
}

/// Whether `key_code` is one a `ListBox` moves its active row with on a first press.
const fn is_navigation_key(key_code: KeyCode) -> bool {
    matches!(
        key_code,
        KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::Home
            | KeyCode::End
    )
}

/// Puts [`Selected`] on a view, or takes it off, to match its item.
pub(crate) fn sync_selected(
    commands: &mut Commands,
    view: Entity,
    was_selected: bool,
    is_selected: bool,
) {
    match (was_selected, is_selected) {
        (false, true) => {
            commands.entity(view).insert(Selected);
        }
        (true, false) => {
            commands.entity(view).remove::<Selected>();
        }
        (false, false) | (true, true) => {}
    }
}

/// Puts [`ListBoxMultiSelect`] on a list, or takes it off, to match its selection.
pub(crate) fn sync_multi_select(
    commands: &mut Commands,
    list: Entity,
    was_multi: bool,
    selection: &SelectionValue,
) {
    match (was_multi, selection.allow_multiple()) {
        (false, true) => {
            commands.entity(list).insert(ListBoxMultiSelect);
        }
        (true, false) => {
            commands.entity(list).remove::<ListBoxMultiSelect>();
        }
        (false, false) | (true, true) => {}
    }
}

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    selections: SelectionParam,
    lists: Query<
        (
            Entity,
            Option<&Children>,
            &Selection,
            &CornerStyle,
            Has<ListBoxMultiSelect>,
        ),
        (With<List>, Without<NoDefaultItems>),
    >,
    mut item_views: Query<(
        &ItemView,
        &Hovered,
        &mut BackgroundColor,
        &mut BorderColor,
        Has<Selected>,
    )>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    // One pass over the lists however many selections changed, not one pass per change.
    let changed: EntityHashSet = q.iter().collect();
    if changed.is_empty() {
        return;
    }
    for (list_ref, children, list_selection, style, is_multi) in lists {
        // The selection of the list that changed: its own entity, or the one it refers to.
        let changed_selection_ref = match list_selection {
            Selection::Ref(entity) if changed.contains(entity) => *entity,
            _ if changed.contains(&list_ref) => list_ref,
            _ => continue,
        };
        let Some(selection) = selections.get(changed_selection_ref) else {
            continue;
        };
        sync_multi_select(&mut commands, list_ref, is_multi, selection);

        for item_view_id in children.map(RelationshipTarget::iter).unwrap_or_default() {
            let Ok((item_view, hovered, mut bg, mut border_color, was_selected)) =
                item_views.get_mut(item_view_id)
            else {
                continue;
            };

            let is_selected = selection.is_selected(item_view.item_ref);
            sync_selected(&mut commands, item_view_id, was_selected, is_selected);
            bg.0 = item_background(theme.colors(), *style, is_selected, hovered.get());
            border_color.set_all(item_border(theme.colors(), *style, is_selected));
        }
    }
}

pub(crate) struct ItemsChangedInput {
    pub(crate) list_ref: Entity,
    pub(crate) items: Arc<[Entity]>,
}

fn on_items_changed(
    input: In<ItemsChangedInput>,
    items_query: Query<&Item<String>>,
    mut commands: Commands,
    lists: Query<
        (Option<&Children>, Option<&ItemViewCreator>),
        (With<List>, Without<NoDefaultItems>),
    >,
    item_views: Query<(Entity, &ItemView)>,
    selections: SelectionParam,
) {
    let ItemsChangedInput { list_ref, items } = &*input;
    let Ok((children, creator)) = lists.get(*list_ref) else {
        return;
    };
    let selection = selections.get(*list_ref);

    let existing = children
        .map(|c| {
            c.iter()
                .filter_map(|c| item_views.get(c).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut commands_run = Vec::new();
    // Linear reconciliation: one set of the wanted items, one of the items
    // that already have a view. A view is kept when its item is wanted and no
    // earlier view shows it; any other view is despawned.
    let wanted: HashSet<Entity> = items.iter().copied().collect();
    let mut shown = HashSet::with_capacity(existing.len());
    for (view_ref, ItemView { item_ref, .. }) in existing {
        if !wanted.contains(item_ref) || !shown.insert(*item_ref) {
            // a view without a corresponding item, or a second view of one
            commands.entity(view_ref).despawn();
        }
    }

    for &item_ref in items.iter().filter(|item_ref| !shown.contains(*item_ref)) {
        let item = items_query.get(item_ref).ok();
        if item.is_none() && creator.is_none() {
            continue;
        }
        let is_selected = selection.is_some_and(|selection| selection.is_selected(item_ref));
        commands.entity(*list_ref).with_children(|spawner| {
            let mut view = spawner.spawn((
                Name::new("ItemView"),
                // Allow scrolling
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
                Node {
                    display: Display::Flex,
                    justify_self: JustifySelf::Stretch,
                    // flex_grow: 1.0,
                    justify_content: JustifyContent::Stretch,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::ZERO,
                    ..default()
                },
                ItemView {
                    item_ref,
                    list_ref: *list_ref,
                },
            ));
            if is_selected {
                view.insert(Selected);
            }
            if let Some(creator) = creator {
                let spawner = ItemViewSpawner {
                    item: item_ref,
                    view: view.id(),
                };
                view.insert(CustomView).observe(keep_nested_clicks_from_row);
                let create = creator.create;
                commands_run.push((create, spawner));
            } else if let Some(item) = item {
                view.observe(handle_mouse_events).with_children(|parent| {
                    parent.spawn((Text(item.data.clone()), ItemLabel, Pickable::IGNORE));
                });
            }
        });
    }

    for (create, spawner) in commands_run {
        commands.run_system_with(create, spawner);
    }
    commands.run_system_cached_with(theme_items, Some(*list_ref));
}

/// The `ItemsChanged` observer that keeps a list or palette in step with the
/// items entity its selection draws from.
///
/// Kept on the list, so it goes with it and a list pointed at other items can
/// drop the observer of the old ones.
#[derive(Component, Debug, Clone, Copy)]
#[component(immutable)]
pub(crate) struct ItemsWatch {
    /// The items entity being watched.
    values: Entity,
    /// The observer on `values`.
    observer: Entity,
}

fn on_insert_selection(t: On<Insert<Selection>>, mut commands: Commands) {
    commands.run_system_cached_with(watch_list, t.event().event_target());
}

/// A list inserted after its selection and items (a scene inserts its components in the order
/// of the patches, not of the list's own requirements) starts watching them now.
fn on_insert_list(t: On<Insert<List>>, mut commands: Commands) {
    commands.run_system_cached_with(watch_list, t.event().event_target());
}

/// Starts watching the items of `list_ref`. Run as a deferred system by both observers, so the
/// second of two in one frame sees the `ItemsWatch` the first inserted and adds no second observer.
fn watch_list(
    list: In<Entity>,
    lists: Query<Option<&ItemsWatch>, (With<List>, With<Selection>, Without<NoDefaultItems>)>,
    selections: SelectionParam,
    commands: Commands,
) {
    let Ok(watch) = lists.get(list.0) else {
        return;
    };
    watch_items(list.0, watch, &selections, commands, on_items_changed);
}

/// Makes `on_items_changed` run for `list_ref` whenever the items of its
/// selection change, and once now. Used by [`List`] and the colour palette, so
/// their copies cannot drift.
///
/// Does nothing when the list already watches those items. When it was
/// pointed at other items, the observer on the old ones is despawned.
pub(crate) fn watch_items<M: 'static>(
    list_ref: Entity,
    watch: Option<&ItemsWatch>,
    selections: &SelectionParam,
    mut commands: Commands,
    on_items_changed: impl IntoSystem<In<ItemsChangedInput>, (), M> + Copy + Send + Sync + 'static,
) {
    let Some((values_ref, all_items)) = selections.get_items(list_ref) else {
        return;
    };
    if watch.is_some_and(|watch| watch.values == values_ref) {
        // no change in items ref
        return;
    }
    if let Some(old) = watch {
        commands.entity(old.observer).despawn();
    }

    // register observer for items change event
    let observer = commands
        .spawn(
            Observer::new(move |t: On<ItemsChanged>, mut commands: Commands| {
                commands.run_system_cached_with(
                    on_items_changed,
                    ItemsChangedInput {
                        list_ref,
                        items: Arc::clone(&t.event().items),
                    },
                );
            })
            .with_entity(values_ref),
        )
        .id();
    commands.entity(list_ref).insert(ItemsWatch {
        values: values_ref,
        observer,
    });

    // update all items now
    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            list_ref,
            items: all_items.iter().copied().collect(),
        },
    );
}

fn theme_items(
    list_id: In<Option<Entity>>,
    lists: Query<
        (Entity, Option<&Children>, &ListLayout, &CornerStyle),
        (With<List>, Without<NoDefaultItems>),
    >,
    theme: Res<Theme>,
    mut item_views: Query<(
        &ItemView,
        &Hovered,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    selections: SelectionParam,
) {
    let iter = list_id.0.map_or_else(
        || Either::Right(lists.into_iter()),
        |list_id| Either::Left(lists.get(list_id).into_iter()),
    );
    let no_selection = SelectionValue::default();

    for (list_ref, children, layout, corner_style) in iter {
        let Some(children) = children else {
            continue;
        };
        // A popup list also holds its scroll bar, which is no row.
        let views = children
            .iter()
            .filter(|child| item_views.contains(*child))
            .collect::<Vec<_>>();
        let Some(last) = views.len().checked_sub(1) else {
            continue;
        };

        let selection = selections.get(list_ref).unwrap_or(&no_selection);

        for (i, item_view_id) in views.into_iter().enumerate() {
            let Ok((ItemView { item_ref, .. }, hovered, mut node, mut bg, mut border_color)) =
                item_views.get_mut(item_view_id)
            else {
                continue;
            };

            let is_selected = selection.is_selected(*item_ref);

            bg.0 = item_background(theme.colors(), *corner_style, is_selected, hovered.get());
            border_color.set_all(item_border(theme.colors(), *corner_style, is_selected));

            node.padding = theme.item_padding;
            theme_item_border(&mut node, *corner_style, *layout, i, last, &theme);
        }
    }
}

fn theme_item_labels(
    item_label: In<Option<Entity>>,
    mut item_labels: Query<(&mut TextColor, &mut TextFont), With<ItemLabel>>,
    theme: Res<Theme>,
) {
    let it = if let Some(item_label_ref) = item_label.as_ref() {
        Either::Left(item_labels.get_mut(*item_label_ref).into_iter())
    } else {
        Either::Right(item_labels.iter_mut())
    };

    for (mut text_color, mut text_font) in it {
        text_color.0 = theme.colors().text_primary;
        text_font.font = theme.body_font.clone().into();
        text_font.font_size = Theme::font_size(theme.font_size_secondary);
    }
}

fn theme_lists(
    list: In<Option<Entity>>,
    theme: Res<Theme>,
    mut lists: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            &ListLayout,
            &CornerStyle,
        ),
        With<List>,
    >,
) {
    let theme_list = |node: &mut Node,
                      bg: &mut BackgroundColor,
                      bc: &mut BorderColor,
                      layout: ListLayout,
                      corner_style: CornerStyle| {
        node.border = match corner_style {
            CornerStyle::FollowLayout | CornerStyle::None => UiRect::all(theme.border_thickness),
            CornerStyle::Tabs | CornerStyle::Separated => UiRect::ZERO,
        };
        if matches!(corner_style, CornerStyle::Separated) {
            bg.0 = Color::NONE;
            node.padding = UiRect::all(SEPARATED_LIST_PADDING);
            node.row_gap = SEPARATED_ROW_GAP;
            node.column_gap = SEPARATED_ROW_GAP;
        } else {
            bg.0 = theme.colors().background_primary;
        }
        bc.set_all(theme.colors().border);

        let radius = match corner_style {
            CornerStyle::FollowLayout => BorderRadius::all(theme.control_radius),
            CornerStyle::Tabs => match layout {
                ListLayout::Vertical | ListLayout::VerticalGrid { .. } => {
                    BorderRadius::left(theme.control_radius)
                }
                ListLayout::Horizontal => BorderRadius::top(theme.control_radius),
            },
            CornerStyle::None | CornerStyle::Separated => BorderRadius::all(Val::Px(0.0)),
        };
        node.border_radius = radius;
    };

    if let Some(list_id) = *list {
        if let Ok((mut node, mut bg, mut bc, layout, corner_style)) = lists.get_mut(list_id) {
            theme_list(&mut node, &mut bg, &mut bc, *layout, *corner_style);
        }
    } else {
        for (mut node, mut bg, mut bc, layout, corner_style) in &mut lists {
            theme_list(&mut node, &mut bg, &mut bc, *layout, *corner_style);
        }
    }
}

fn theme_item_border(
    node: &mut Node,
    corner_style: CornerStyle,
    layout: ListLayout,
    i: usize,
    last: usize,
    theme: &Theme,
) {
    match corner_style {
        CornerStyle::Tabs => {
            node.border_radius = match layout {
                ListLayout::Vertical | ListLayout::VerticalGrid { .. } => {
                    BorderRadius::left(theme.control_radius)
                }
                ListLayout::Horizontal => BorderRadius::top(theme.control_radius),
            };
            node.border = UiRect::all(Val::Px(1.0));
        }
        CornerStyle::FollowLayout => {
            node.border = UiRect::ZERO;
            match layout {
                ListLayout::Vertical => {
                    node.border_radius = match i {
                        0 => BorderRadius::top(theme.control_radius),
                        n if n == last => BorderRadius::bottom(theme.control_radius),
                        _ => BorderRadius::ZERO,
                    }
                }
                ListLayout::Horizontal => {
                    node.border_radius = match i {
                        0 => BorderRadius::left(theme.control_radius),
                        n if n == last => BorderRadius::right(theme.control_radius),
                        _ => BorderRadius::ZERO,
                    }
                }
                ListLayout::VerticalGrid { columns } => {
                    let columns = usize::from(columns.max(1));
                    node.border_radius = match i {
                        0 => BorderRadius::top_left(theme.control_radius),
                        n if n == columns - 1 => BorderRadius::top_right(theme.control_radius),
                        n if n % columns == 0 && last.abs_diff(n) < columns => {
                            BorderRadius::bottom_left(theme.control_radius)
                        }
                        n if n == last && last.is_multiple_of(columns) => {
                            BorderRadius::bottom_right(theme.control_radius)
                        }
                        _ => BorderRadius::ZERO,
                    }
                }
            }
        }
        CornerStyle::None => {
            node.border = UiRect::ZERO;
            node.border_radius = BorderRadius::ZERO;
        }
        CornerStyle::Separated => {
            // The border is the ring of a selected row, transparent otherwise, so that
            // selecting a row does not move its content.
            node.border = UiRect::all(SEPARATED_RING_WIDTH);
            node.border_radius = BorderRadius::all(theme.control_radius);
            node.padding = SEPARATED_ITEM_PADDING;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS};

    #[test]
    fn a_note_keeps_its_tone_on_a_resting_row_and_turns_primary_on_a_selected_one() {
        for c in [DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS] {
            assert_eq!(note_ink(&c, NoteTone::Warning, Color::NONE), c.warning_text);
            assert_eq!(
                note_ink(&c, NoteTone::Secondary, c.background_primary),
                c.text_secondary
            );
            assert_eq!(
                note_ink(&c, NoteTone::Secondary, c.selected_item),
                c.text_primary
            );
            assert_eq!(
                note_ink(&c, NoteTone::Warning, c.selected_hovered_item),
                c.text_primary
            );
        }
    }
}
