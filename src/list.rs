//! `List`: renders the items of a [`Selection`] as views and keeps them in sync.

use std::sync::Arc;

use accesskit::Role;
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
use bevy::ui::{Selectable, Selected};
use bevy::ui_widgets::{
    ActiveDescendant, Button, ListBox, ListBoxMultiSelect, ListItem, ReselectListRow, ValueChange,
};
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::combo::Combo;
use crate::filter_combo::FilterCombo;
use crate::focus::{AccessibilitySync, TabStop, sync_label, sync_role};
use crate::select::{Item, ItemsChanged, ItemsRef, Selection, SelectionParam, SelectionValue};
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
                    clear_dangling_lists,
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
            .add_observer(handle_mouse_events)
            .register_type::<ItemView>()
            .register_type::<ItemNote>()
            .register_type::<ItemHeader>()
            .register_type::<NoteTone>()
            .register_type::<ListLayout>()
            .register_type::<CornerStyle>();
    }
}

/// Renders the items of a [`Selection`] as views, one per item, and keeps them in sync.
///
/// It is a [`ListBox`] of [`ListItem`]s. It does not scroll: wrap it in a
/// [`ScrollArea`](crate::scroll_area::ScrollArea) with a `max_height`. A view costs several
/// entities per item, so use a [`VirtualList`](crate::virtual_list::VirtualList) for thousands.
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
        /// The number of columns; `0` is laid out as `1` and logs an `error!`.
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
    /// Rows that stand apart, each rounded on all corners, with a ring on the selected row.
    Separated,
}

/// Makes a [`List`] build the contents of its item views with a system of the app's own.
///
/// The creator runs once per view and adds children to [`ItemViewSpawner::view`]. A nested
/// [`Button`], [`List`], [`Combo`] or [`FilterCombo`] handles its own pointer. Add it with the
/// [`List`], before the first update.
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

/// Given to the creator system of an [`ItemViewCreator`].
#[derive(Debug, Copy, Clone)]
pub struct ItemViewSpawner {
    /// The item the view shows.
    pub item: Entity,
    /// The [`ItemView`] node to spawn the contents under.
    pub view: Entity,
}

/// Marks an [`ItemView`] whose contents come from an [`ItemViewCreator`].
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

/// Sets the fields of the [`Node`] of `entity` that its [`ListLayout`] owns, and no others.
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
    node.grid_auto_flow = grid_auto_flow;
    node.grid_template_columns = grid_template_columns;
    match layout {
        // A flex row or column lays out as a grid does and is far cheaper in Taffy.
        ListLayout::Vertical => {
            node.display = Display::Flex;
            node.flex_direction = FlexDirection::Column;
        }
        ListLayout::Horizontal => {
            node.display = Display::Flex;
            node.flex_direction = FlexDirection::Row;
        }
        ListLayout::VerticalGrid { columns: _ } => node.display = Display::Grid,
    }
}

/// One item of a [`List`] (or [`ColorPalette`](crate::color_palette::ColorPalette)).
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

/// Private relationship from a view to the item it shows.
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = NoteViews)]
struct NoteViewOf(Entity);

/// The views that show an item.
#[derive(Component, Debug)]
#[relationship_target(relationship = NoteViewOf)]
struct NoteViews(Vec<Entity>);

/// A note shown after the label of an item.
///
/// It goes on the item entity next to its [`Item<String>`]. A [`Combo`] button never shows it;
/// a screen reader reads the label followed by the note.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ItemNote {
    /// The text of the note.
    pub text: String,
    /// The color the note is shown in.
    pub tone: NoteTone,
}

impl ItemNote {
    /// A note in the secondary text color.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: NoteTone::Secondary,
        }
    }

    /// A note in the warning text color.
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: NoteTone::Warning,
        }
    }
}

/// The color of an [`ItemNote`], a slot of the theme.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum NoteTone {
    /// `text_secondary`.
    #[default]
    Secondary,
    /// `warning_text`.
    Warning,
}

/// Makes an item a header (`ItemHeader(true)`): a heading row in the list that no one can pick.
///
/// It goes on the item entity next to its [`Item<String>`] when the item is created, so a
/// [`List`] (a [`Combo`] popup too) shows the label as a heading between the runs of items
/// around it. The row is no `ListItem`: arrow keys, Home and End skip it, a click or Enter never
/// picks it, and it never shows as selected or hovered. A [`FilterCombo`] hides it while a query
/// is typed. A list with an [`ItemViewCreator`] shows it the same way and does not run the
/// creator for it. The flag is a field so that one array of items, which has one bundle type,
/// can hold headers and options: `(Item::new("Ordinary"), ItemHeader(false))` next to
/// `(Item::new("Switches"), ItemHeader(true))`. Changing it on an item that already has a row
/// does not change that row. Nothing stops an app from selecting a header in code: the widgets
/// only never pick it.
#[derive(Component, Debug, Copy, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ItemHeader(pub bool);

/// Marks the [`ItemView`] of an [`ItemHeader`] item.
#[derive(Component, Debug, Copy, Clone, Default)]
pub(crate) struct HeaderRow;

/// Marks the text of an [`ItemNote`] inside a default [`ItemView`].
#[derive(Component, Debug, Clone, Copy)]
struct ItemNoteView(NoteTone);

/// The ink of a note of `tone` on a row drawn on `background`, at least 4.5:1.
pub(crate) fn note_ink(colors: &ThemeColors, tone: NoteTone, background: Color) -> Color {
    let own = match tone {
        NoteTone::Secondary => colors.text_secondary,
        NoteTone::Warning => colors.warning_text,
    };
    // A transparent row has no color to measure against.
    if background.alpha() == 0.0 || contrast_ratio(own, background) >= 4.5 {
        own
    } else {
        colors.text_primary
    }
}

/// Keeps the note of a row readable when the row's background changes.
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
    // Touch only the new views and those of changed notes, unless the theme changed.
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
    // Drain the removals the theme branch skipped, or they are read again.
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

/// Shows the new text of a renamed `Item<String>` in its default rows.
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

/// Names each default row to assistive technology: its label, then its note.
fn sync_item_labels(
    mut views: Query<(&ItemView, &mut AccessibilityNode, Has<HeaderRow>), Without<CustomView>>,
    items: Query<&Item<String>>,
    notes: Query<&ItemNote>,
) {
    for (ItemView { item_ref, .. }, mut node, is_header) in &mut views {
        if is_header {
            sync_role(&mut node, Role::Heading);
        }
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

/// Marks the [`ItemLabel`] of a header row, which is themed as a caption.
#[derive(Component, Debug, Clone, Copy)]
struct HeaderLabel;

impl ItemLabel {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_item_labels, Some(context.entity));
    }
}

/// Marks a default [`ItemView`] of a [`List`], colored by the global `handle_mouse_events`.
#[derive(Component, Debug, Copy, Clone, Default)]
struct DefaultRow;

fn handle_mouse_events(
    t: On<MouseEvent>,
    selections: SelectionParam,
    mut item_views: Query<(&ItemView, &mut BackgroundColor), With<DefaultRow>>,
    styles: Query<&CornerStyle>,
    theme: Res<Theme>,
) {
    // A drag can arrive after the pointer left the view, so it must not recolor it.
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

    let hovered = t.event().hovered();
    let pressed = t.event().pressed();
    let style = styles.get(*list_ref).copied().unwrap_or(CornerStyle::None);
    background_color.0 = item_background(theme.colors(), style, is_selected || pressed, hovered);
}

/// The [`ItemView`] a pointer event on `target` belongs to, if it is a custom view's.
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
/// reaching the outer list.
///
/// The `ListBox` finds a row from any ancestor of the original target, so a click on a nested
/// combo would otherwise pick the row.
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
    if !matches!(
        kind,
        MouseKind::Pressed | MouseKind::Released | MouseKind::Cancel
    ) {
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
    background_color.0 = item_background(theme.colors(), style, is_selected, hovered.get());
}

/// Colors a custom view when the pointer enters or leaves it or anything inside it.
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

/// The background of an [`ItemView`] from its state.
pub(crate) const fn item_background(
    colors: &ThemeColors,
    style: CornerStyle,
    selected: bool,
    hovered: bool,
) -> Color {
    if matches!(style, CornerStyle::Separated) {
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

/// The border color of an [`ItemView`].
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
/// `listbox_update_selection` is not registered: it would set [`Selected`] past the rules of the
/// [`Selection`]. `selection_change` sets it. The `ListBox` also sends this for a click on a
/// selected row, which deselects it where the selection allows.
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
    // A nested list can report a row of the list around it.
    if item_view.list_ref != source {
        return;
    }
    selections.update_selection(source, |selection| {
        selection.toggle_selected(item_view.item_ref)
    });
}

/// Toggles the already-selected row the `ListBox` reports as chosen again; a `One` keeps it.
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
/// The `ListBox` stops a first press itself but lets repeats and the release bubble to the
/// window, where directional navigation would move the focus out. Keys from a focused
/// descendant go on: see [`pass_first_presses_from_descendants`].
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
/// The `ListBox` takes any first press that reaches it, so stop it on the child it comes
/// through and send it on from the list's parent.
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
        Has<HeaderRow>,
    )>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let changed: EntityHashSet = q.iter().collect();
    if changed.is_empty() {
        return;
    }
    for (list_ref, children, list_selection, style, is_multi) in lists {
        let changed_selection_ref = match list_selection {
            Selection::Ref(entity) if changed.contains(entity) => *entity,
            _ if changed.contains(&list_ref) => list_ref,
            _ => continue,
        };
        let Some(selection) = selections.get(changed_selection_ref) else {
            continue;
        };
        sync_multi_select(&mut commands, list_ref, is_multi, selection);

        for item_view_id in children.map_or_default(RelationshipTarget::iter) {
            let Ok((item_view, hovered, mut bg, mut border_color, was_selected, is_header)) =
                item_views.get_mut(item_view_id)
            else {
                continue;
            };
            if is_header {
                continue;
            }

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
    headers: Query<&ItemHeader>,
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

    let existing = children.map_or_default(|c| {
        c.iter()
            .filter_map(|c| item_views.get(c).ok())
            .collect::<Vec<_>>()
    });

    let mut commands_run = Vec::new();
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
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
                Node {
                    display: Display::Flex,
                    justify_self: JustifySelf::Stretch,
                    // A horizontal flex row overflows a narrow list instead of squeezing rows.
                    flex_shrink: 0.0,
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
            if headers.get(item_ref).is_ok_and(|header| header.0)
                && let Some(item) = item
            {
                // No `ListItem`: the `ListBox` neither walks to the row nor picks it.
                view.remove::<(ListItem, Selectable)>()
                    .insert(HeaderRow)
                    .with_children(|parent| {
                        parent.spawn((
                            Text(item.data.clone()),
                            ItemLabel,
                            HeaderLabel,
                            Pickable::IGNORE,
                        ));
                    });
                return;
            }
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
                view.insert(DefaultRow).with_children(|parent| {
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

/// The `ItemsChanged` observer that keeps a list or palette in step with its items entity.
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

/// A list inserted after its selection and items starts watching them now.
fn on_insert_list(t: On<Insert<List>>, mut commands: Commands) {
    commands.run_system_cached_with(watch_list, t.event().event_target());
}

/// Starts watching the items of `list_ref`; deferred so two in one frame add one observer.
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

/// Clears the views of the lists whose items no longer resolve, as if their source had none.
///
/// Despawning the source is not reported by `ItemsChanged`, so look when a `Selection` or
/// `ItemsRef` was removed.
fn clear_dangling_lists(
    mut removed_selections: RemovedComponents<Selection>,
    mut removed_refs: RemovedComponents<ItemsRef>,
    lists: Query<(Entity, &ItemsWatch), (With<List>, Without<NoDefaultItems>)>,
    selections: SelectionParam,
    commands: Commands,
) {
    let removed = removed_selections.read().count() + removed_refs.read().count() > 0;
    if removed {
        unwatch_dangling(&lists, &selections, commands, on_items_changed);
    }
}

/// Despawns the observer and drops the [`ItemsWatch`] of each list whose items no longer resolve.
pub(crate) fn unwatch_dangling<M: 'static, F: bevy::ecs::query::QueryFilter>(
    lists: &Query<(Entity, &ItemsWatch), F>,
    selections: &SelectionParam,
    mut commands: Commands,
    on_items_changed: impl IntoSystem<In<ItemsChangedInput>, (), M> + Copy + Send + Sync + 'static,
) {
    for (list, watch) in lists {
        if selections.get_items(list).is_some() {
            continue;
        }
        commands.entity(watch.observer).despawn();
        commands.entity(list).remove::<ItemsWatch>();
        commands.run_system_cached_with(
            on_items_changed,
            ItemsChangedInput {
                list_ref: list,
                items: Arc::from(Vec::<Entity>::new()),
            },
        );
    }
}

/// Makes `on_items_changed` run for `list_ref` whenever its items change, and once now.
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
        return;
    }
    if let Some(old) = watch {
        commands.entity(old.observer).despawn();
    }

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
        Has<HeaderRow>,
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
            let Ok((
                ItemView { item_ref, .. },
                hovered,
                mut node,
                mut bg,
                mut border_color,
                is_header,
            )) = item_views.get_mut(item_view_id)
            else {
                continue;
            };

            // A header is never selected or hovered.
            let is_selected = !is_header && selection.is_selected(*item_ref);
            let hovered = !is_header && hovered.get();

            bg.0 = item_background(theme.colors(), *corner_style, is_selected, hovered);
            border_color.set_all(item_border(theme.colors(), *corner_style, is_selected));

            node.padding = theme.item_padding;
            theme_item_border(&mut node, *corner_style, *layout, i, last, &theme);
        }
    }
}

fn theme_item_labels(
    item_label: In<Option<Entity>>,
    mut item_labels: Query<(&mut TextColor, &mut TextFont, Has<HeaderLabel>), With<ItemLabel>>,
    theme: Res<Theme>,
) {
    let it = if let Some(item_label_ref) = item_label.as_ref() {
        Either::Left(item_labels.get_mut(*item_label_ref).into_iter())
    } else {
        Either::Right(item_labels.iter_mut())
    };

    for (mut text_color, mut text_font, is_header) in it {
        let (ink, size) = if is_header {
            (
                theme.colors().text_secondary,
                Theme::font_size(theme.font_size_caption),
            )
        } else {
            (
                theme.colors().text_primary,
                Theme::font_size(theme.font_size_secondary),
            )
        };
        text_color.set_if_neq(TextColor(ink));
        let font = theme.body_font.clone().into();
        if text_font.font != font {
            text_font.font = font;
        }
        if text_font.font_size != size {
            text_font.font_size = size;
        }
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
            // A transparent border keeps selecting a row from moving its content.
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
