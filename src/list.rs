//! `List`: renders the items of a [`Selection`] as views and keeps them in sync.

use std::collections::HashMap;

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemId;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::{
    ActiveDescendant, Button, ListBox, ListBoxMultiSelect, ListItem, ReselectListRow, ValueChange,
};
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::focus::{AccessibilitySync, TabStop, sync_label};
use crate::select::{Item, ItemsChanged, Selection, SelectionParam, SelectionValue};
use crate::theme::{Theme, ThemeColors};
use crate::{MouseEvent, MouseKind};

/// Padding around a [`CornerStyle::Separated`] list.
const SEPARATED_LIST_PADDING: Val = Val::Px(6.0);
/// Gap between the rows of a [`CornerStyle::Separated`] list.
const SEPARATED_ROW_GAP: Val = Val::Px(2.0);
/// Width of the ring on the selected row of a [`CornerStyle::Separated`] list.
const SEPARATED_RING_WIDTH: Val = Val::Px(1.0);
/// Padding inside a row of a [`CornerStyle::Separated`] list: 10 pixels by 12.
const SEPARATED_ITEM_PADDING: UiRect = UiRect::axes(Val::Px(12.0), Val::Px(10.0));

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
                ),
            )
            .add_systems(PostUpdate, sync_item_labels.in_set(AccessibilitySync))
            .add_observer(on_insert_selection)
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
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("List"),
    ListBox,
    TabStop,
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
pub enum ListLayout {
    /// One item per row.
    Vertical,
    /// One item per column.
    Horizontal,
    /// A grid that fills row by row.
    VerticalGrid {
        /// The number of columns.
        columns: u16,
    },
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
/// A [`Button`] inside a view handles its own pointer: a press on it does not restyle the
/// row and a click on it does not change the selection. Contents that are not buttons pass
/// the pointer on to the row.
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
        // TODO change this assert
        // assert!(
        //     world.entity(context.entity).get::<Node>().is_none(),
        //     "Cannot override Node for List"
        // );

        let Some(layout) = world.entity(context.entity).get::<ListLayout>() else {
            return;
        };
        let grid_auto_flow = match layout {
            ListLayout::Vertical | ListLayout::VerticalGrid { columns: _ } => {
                GridAutoFlow::RowDense
            }
            ListLayout::Horizontal => GridAutoFlow::ColumnDense,
        };
        let grid_template_columns = match layout {
            ListLayout::Vertical | ListLayout::Horizontal => vec![],
            ListLayout::VerticalGrid { columns } => vec![RepeatedGridTrack::auto(*columns)],
        };

        let node = Node {
            position_type: PositionType::Relative,
            display: Display::Grid,
            grid_auto_flow,
            grid_template_columns,
            ..default()
        };
        world.commands().entity(context.entity).insert(node);

        world
            .commands()
            .run_system_cached_with(theme_lists, Some(context.entity));
    }
}

/// One item of a [`List`] (or [`ColorPalette`](crate::color_palette::ColorPalette)).
///
/// It is a [`ListItem`]: it carries [`Selected`] while its item is selected. Its
/// [`Hovered`] says whether the pointer is on it, so that a change of the selection
/// keeps the hover colour of the view under the pointer.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Hovered, ListItem)]
pub struct ItemView {
    /// The item this view shows.
    pub item_ref: Entity,
    /// The list this view belongs to.
    pub list_ref: Entity,
}

/// A note shown after the label of an item.
///
/// It goes on the item entity next to its [`Item<String>`] and shows in the default view of a
/// [`List`] (so also in a [`Combo`](crate::combo::Combo)'s popup), in the caption size and the
/// colour of its [`NoteTone`].
///
/// A [`Combo`](crate::combo::Combo)'s button shows the label of the selected item only, never
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
struct ItemNoteView;

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
    views: Query<(Entity, &ItemView, Option<&Children>), Without<CustomView>>,
    notes: Query<&ItemNote>,
    mut note_texts: Query<(&mut Text, &mut TextColor, &mut TextFont), With<ItemNoteView>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for (view, ItemView { item_ref, .. }, children) in &views {
        let wanted = notes
            .get(*item_ref)
            .ok()
            .filter(|note| !note.text.is_empty());
        let existing = children
            .into_iter()
            .flatten()
            .copied()
            .find(|child| note_texts.contains(*child));
        let color = |note: &ItemNote| match note.tone {
            NoteTone::Secondary => theme.colors.text_secondary,
            NoteTone::Warning => theme.colors.warning_text,
        };
        let font = || TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(theme.font_size_caption),
            ..default()
        };
        match (wanted, existing) {
            (Some(note), Some(child)) => {
                if let Ok((mut text, mut text_color, mut text_font)) = note_texts.get_mut(child) {
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
                        ItemNoteView,
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
    background_color.0 = item_background(&theme.colors, style, is_selected || pressed, hovered);
}

/// The [`ItemView`] a pointer event on `target` belongs to, if `target` is a custom view or
/// lies under one without a nested button in between.
fn custom_view_of(
    target: Entity,
    parents: &Query<&ChildOf>,
    views: &Query<(), With<CustomView>>,
    buttons: &Query<(), With<Button>>,
) -> Option<Entity> {
    let mut current = target;
    loop {
        if views.contains(current) {
            return Some(current);
        }
        if buttons.contains(current) {
            return None;
        }
        current = parents.get(current).ok()?.parent();
    }
}

fn handle_custom_view_mouse_events(
    t: On<MouseEvent>,
    parents: Query<&ChildOf>,
    custom_views: Query<(), With<CustomView>>,
    buttons: Query<(), With<Button>>,
    selections: SelectionParam,
    mut item_views: Query<(&ItemView, &Hovered, &mut BackgroundColor)>,
    styles: Query<&CornerStyle>,
    theme: Res<Theme>,
) {
    let kind = t.event().kind;
    if !matches!(kind, MouseKind::Pressed | MouseKind::Released) {
        return;
    }
    let Some(view) = custom_view_of(t.event().event_target(), &parents, &custom_views, &buttons)
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
        background_color.0 = item_background(&theme.colors, style, true, hovered.get());
        return;
    }
    // Deselecting is the `ListBox`'s event, see `on_reselect_row`.
    background_color.0 = item_background(&theme.colors, style, is_selected, hovered.get());
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
        background_color.0 = item_background(&theme.colors, style, is_selected, hovered.get());
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
const fn item_border(colors: &ThemeColors, style: CornerStyle, selected: bool) -> Color {
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
    for changed_selection_ref in q {
        for (list_ref, children, list_selection, style, is_multi) in lists {
            let is_changed = match list_selection {
                Selection::Ref(entity) => {
                    changed_selection_ref == *entity || changed_selection_ref == list_ref
                }
                Selection::Value(_selection) => changed_selection_ref == list_ref,
            };
            if is_changed {
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
                    bg.0 = item_background(&theme.colors, *style, is_selected, hovered.get());
                    border_color.set_all(item_border(&theme.colors, *style, is_selected));
                }
            }
        }
    }
}

pub(crate) struct ItemsChangedInput {
    pub(crate) list_ref: Entity,
    pub(crate) items: Vec<Entity>,
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
    let mut items = items.clone();
    for (view_ref, ItemView { item_ref, .. }) in existing {
        let len_before = items.len();
        items.retain(|x| x != item_ref);
        if len_before == items.len() {
            // a view was found without a corresponding item
            commands.entity(view_ref).despawn();
        }
    }

    for item_ref in items {
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
                view.insert(CustomView);
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

fn on_insert_selection(
    t: On<Insert<Selection>>,
    lists: Query<(), (With<List>, With<Selection>, Without<NoDefaultItems>)>,
    selections: SelectionParam,
    mut commands: Commands,
    mut existing_values_ref_map: Local<HashMap<Entity, Entity>>,
) {
    let list_ref = t.event().event_target();
    if lists.get(list_ref).is_err() {
        return;
    }

    let Some((values_ref, all_items)) = selections.get_item_values(list_ref) else {
        return;
    };
    if existing_values_ref_map.get(&list_ref) == Some(&values_ref) {
        // no change in items ref
        return;
    }
    existing_values_ref_map.insert(list_ref, values_ref);

    // register observer for items change event
    commands
        .entity(values_ref)
        .observe(move |t: On<ItemsChanged>, mut commands: Commands| {
            commands.run_system_cached_with(
                on_items_changed,
                ItemsChangedInput {
                    list_ref,
                    items: t.event().items.iter().collect(),
                },
            );
        });

    // update all items now
    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            list_ref,
            items: all_items.iter().collect(),
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
        if children.is_empty() {
            continue;
        }
        let last = children.len() - 1;

        let selection = selections.get(list_ref).unwrap_or(&no_selection);

        for (i, item_view_id) in children.iter().enumerate() {
            let Ok((ItemView { item_ref, .. }, hovered, mut node, mut bg, mut border_color)) =
                item_views.get_mut(item_view_id)
            else {
                continue;
            };

            let is_selected = selection.is_selected(*item_ref);

            bg.0 = item_background(&theme.colors, *corner_style, is_selected, hovered.get());
            border_color.set_all(item_border(&theme.colors, *corner_style, is_selected));

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
        text_color.0 = theme.colors.text_primary;
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
            bg.0 = theme.colors.background_primary;
        }
        bc.set_all(theme.colors.border);

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
                    let columns = columns as usize;
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
