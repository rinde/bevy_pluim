//! `List`: renders the items of a [`Selection`] as views and keeps them in sync.

use std::collections::HashMap;

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::{ActiveDescendant, ListBox, ListBoxMultiSelect, ListItem, ValueChange};
use itertools::Either;

use crate::focus::TabStop;
use crate::select::{Item, ItemsChanged, Selection, SelectionParam, SelectionValue};
use crate::theme::{Theme, ThemeColors};
use crate::{MouseEvent, MouseKind};

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
                ),
            )
            .add_observer(on_insert_selection)
            .add_observer(on_value_change)
            .register_type::<ItemView>()
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
}

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

#[derive(Component, Debug, Clone, Copy)]
#[component(on_insert=Self::on_insert)]
struct ItemLabel;

impl ItemLabel {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_item_labels, Some(context.entity));
    }
}

fn handle_mouse_events(
    t: On<MouseEvent>,
    mut selections: SelectionParam,
    mut item_views: Query<(&ItemView, &mut BackgroundColor, Has<Selected>)>,
    theme: Res<Theme>,
) {
    // A click and a drag say nothing new about the pointer being on the view: the over,
    // press, release and out events around them set the colours. A drag in particular can
    // arrive after the pointer left the view, so it must not put it back under the pointer.
    if matches!(t.event().kind, MouseKind::Click(_) | MouseKind::Drag(_)) {
        return;
    }
    let Ok((
        ItemView {
            item_ref: target_item_entity,
            list_ref,
        },
        mut background_color,
        is_marked_selected,
    )) = item_views.get_mut(t.event().event_target())
    else {
        return;
    };
    let Some(selection) = selections.get(*list_ref) else {
        return;
    };
    let is_selected = selection.is_selected(*target_item_entity);

    if matches!(t.event().kind, MouseKind::Released) {
        // The `ListBox` leaves a click on a `Selected` row alone, so deselecting is ours.
        // Selecting is the `ListBox`'s: it sends a `ValueChange`, see `on_value_change`.
        // Both go by the `Selected` the `ListBox` saw, not by the `Selection`: the release
        // can be handled after the `ValueChange` of the same click has changed the
        // `Selection`, and the click would undo itself.
        if is_marked_selected {
            selections.update_selection(*list_ref, |selection| {
                selection.toggle_selected(*target_item_entity)
            });
        }
        return;
    }

    // The pointer is on the view for every event but `Out`; a press looks selected.
    let hovered = !matches!(t.event().kind, MouseKind::Out);
    let pressed = matches!(t.event().kind, MouseKind::Pressed);
    background_color.0 = item_background(&theme.colors, is_selected || pressed, hovered);
}

/// The background of an [`ItemView`] from its state: selected and hovered, selected,
/// hovered, or at rest.
pub(crate) const fn item_background(colors: &ThemeColors, selected: bool, hovered: bool) -> Color {
    match (selected, hovered) {
        (true, true) => colors.selected_hovered_item,
        (true, false) => colors.selected_item,
        (false, true) => colors.hovered_item,
        (false, false) => colors.background_primary,
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
/// A row that already carries [`Selected`] is left alone: the `ListBox` may still send a
/// change for it (a click on any selected row but the first one of a multiple selection),
/// but the release of that click deselects it, see `handle_mouse_events`.
fn on_value_change(
    t: On<ValueChange<Entity>>,
    lists: Query<(), With<List>>,
    item_views: Query<(&ItemView, Has<Selected>)>,
    mut selections: SelectionParam,
) {
    let ValueChange { source, value, .. } = *t.event();
    if lists.get(source).is_err() {
        return;
    }
    let Ok((item_view, is_marked_selected)) = item_views.get(value) else {
        return;
    };
    if is_marked_selected {
        return;
    }
    selections.update_selection(source, |selection| {
        selection.toggle_selected(item_view.item_ref)
    });
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
        for (list_ref, children, list_selection, is_multi) in lists {
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
                    bg.0 = item_background(&theme.colors, is_selected, hovered.get());

                    if is_selected {
                        border_color.set_all(theme.colors.border_highlight);
                    } else {
                        border_color.set_all(theme.colors.border);
                    }
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
    lists: Query<Option<&Children>, (With<List>, Without<NoDefaultItems>)>,
    item_views: Query<(Entity, &ItemView)>,
    selections: SelectionParam,
) {
    let ItemsChangedInput { list_ref, items } = &*input;
    let Ok(children) = lists.get(*list_ref) else {
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
        let Ok(item) = items_query.get(item_ref) else {
            continue;
        };
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
            view.observe(handle_mouse_events).with_children(|parent| {
                parent.spawn((Text(item.data.clone()), ItemLabel, Pickable::IGNORE));
            });
            if is_selected {
                view.insert(Selected);
            }
        });
    }

    commands.run_system_cached_with(theme_items, Some(*list_ref));
}

fn on_insert_selection(
    t: On<Insert, Selection>,
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

            bg.0 = item_background(&theme.colors, is_selected, hovered.get());
            if is_selected {
                border_color.set_all(theme.colors.border_highlight);
            } else {
                border_color.set_all(theme.colors.border);
            }

            theme_item_border(&mut node, *corner_style, *layout, i, last, &theme);

            node.padding = theme.item_padding;
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
        text_font.font_size = FontSize::Px(theme.font_size);
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
            CornerStyle::Tabs => UiRect::ZERO,
        };
        bg.0 = theme.colors.background_primary;
        bc.set_all(theme.colors.border);

        let radius = match corner_style {
            CornerStyle::FollowLayout => BorderRadius::all(theme.corner_radius),
            CornerStyle::Tabs => match layout {
                ListLayout::Vertical | ListLayout::VerticalGrid { .. } => {
                    BorderRadius::left(theme.corner_radius)
                }
                ListLayout::Horizontal => BorderRadius::top(theme.corner_radius),
            },
            CornerStyle::None => BorderRadius::all(Val::Px(0.0)),
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

const fn theme_item_border(
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
                    BorderRadius::left(theme.corner_radius)
                }
                ListLayout::Horizontal => BorderRadius::top(theme.corner_radius),
            };
            node.border = UiRect::all(Val::Px(1.0));
        }
        CornerStyle::FollowLayout => {
            node.border = UiRect::ZERO;
            match layout {
                ListLayout::Vertical => {
                    node.border_radius = match i {
                        0 => BorderRadius::top(theme.corner_radius),
                        n if n == last => BorderRadius::bottom(theme.corner_radius),
                        _ => BorderRadius::ZERO,
                    }
                }
                ListLayout::Horizontal => {
                    node.border_radius = match i {
                        0 => BorderRadius::left(theme.corner_radius),
                        n if n == last => BorderRadius::right(theme.corner_radius),
                        _ => BorderRadius::ZERO,
                    }
                }
                ListLayout::VerticalGrid { columns } => {
                    let columns = columns as usize;
                    node.border_radius = match i {
                        0 => BorderRadius::top_left(theme.corner_radius),
                        n if n == columns - 1 => BorderRadius::top_right(theme.corner_radius),
                        n if n % columns == 0 && last.abs_diff(n) < columns => {
                            BorderRadius::bottom_left(theme.corner_radius)
                        }
                        n if n == last && last.is_multiple_of(columns) => {
                            BorderRadius::bottom_right(theme.corner_radius)
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
    }
}
