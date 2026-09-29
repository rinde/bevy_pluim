//! `List`: renders the items of a [`Selection`] as views and keeps them in sync.

#![expect(clippy::used_underscore_binding, reason = "Caused by Reflect")]
use std::collections::HashMap;

use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use itertools::Either;

use crate::MouseEvent;
use crate::select::{Item, ItemsChanged, Selection, SelectionParam, SelectionValue};
use crate::theme::Theme;

pub(crate) struct ListPlugin;

impl Plugin for ListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_scroll_position)
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
            .register_type::<ItemView>()
            .register_type::<ListLayout>()
            .register_type::<CornerStyle>();
    }
}

/// Renders the items of a [`Selection`] as views, one per item, and keeps them in sync.
///
/// Use [`ListLayout`] and [`CornerStyle`] to change how they are laid out.
#[derive(Component, Debug, Copy, Clone, Default)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("List"),
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

        // TODO allow options to set max size and scroll if contents don't fit
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
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[require(RelativeCursorPosition::default())]
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
    t: Trigger<MouseEvent>,
    mut selections: SelectionParam,
    mut item_views: Query<(&ItemView, &mut BackgroundColor)>,
    theme: Res<Theme>,
) {
    let Ok((
        ItemView {
            item_ref: target_item_entity,
            list_ref,
        },
        mut background_color,
    )) = item_views.get_mut(t.target())
    else {
        return;
    };
    let Some(selection) = selections.get(*list_ref) else {
        return;
    };
    let is_selected = selection.is_selected(*target_item_entity);

    match (t.event(), is_selected) {
        (MouseEvent::Pressed, _) | (MouseEvent::Over, true) => {
            background_color.0 = theme.colors.selected_hovered_item;
        }
        (MouseEvent::Over, false) => {
            background_color.0 = theme.colors.hovered_item;
        }
        (MouseEvent::Out, true) => {
            background_color.0 = theme.colors.selected_item;
        }
        (MouseEvent::Out, false) => {
            background_color.0 = theme.colors.background_primary;
        }
        (MouseEvent::Released, _) => {
            selections.update_selection(*list_ref, |selection| {
                selection.toggle_selected(*target_item_entity)
            });
        }
    }
}

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    selections: SelectionParam,
    lists: Query<(Entity, Option<&Children>, &Selection), (With<List>, Without<NoDefaultItems>)>,
    mut item_views: Query<(
        &ItemView,
        &RelativeCursorPosition,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    theme: Res<Theme>,
) {
    for changed_selection_ref in q {
        for (list_ref, children, list_selection) in lists {
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

                for item_view_id in children.map(RelationshipTarget::iter).unwrap_or_default() {
                    let Ok((item_view, rcp, mut bg, mut border_color)) =
                        item_views.get_mut(item_view_id)
                    else {
                        continue;
                    };

                    let is_selected = selection.is_selected(item_view.item_ref);
                    match (is_selected, rcp.mouse_over()) {
                        (true, true) => bg.0 = theme.colors.selected_hovered_item,
                        (true, false) => bg.0 = theme.colors.selected_item,
                        (false, true) => bg.0 = theme.colors.hovered_item,
                        (false, false) => bg.0 = theme.colors.background_primary,
                    }

                    if is_selected {
                        border_color.0 = theme.colors.border_highlight;
                    } else {
                        border_color.0 = theme.colors.border;
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
) {
    let ItemsChangedInput { list_ref, items } = &*input;
    let Ok(children) = lists.get(*list_ref) else {
        return;
    };

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
        commands.entity(*list_ref).with_children(|spawner| {
            spawner
                .spawn((
                    Name::new("ItemView"),
                    Button,
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
                ))
                .observe(handle_mouse_events)
                .with_children(|parent| {
                    parent.spawn((Text(item.data.clone()), ItemLabel, Pickable::IGNORE));
                });
        });
    }

    commands.run_system_cached_with(theme_items, Some(*list_ref));
}

fn on_insert_selection(
    t: Trigger<OnInsert, Selection>,
    lists: Query<(), (With<List>, With<Selection>, Without<NoDefaultItems>)>,
    selections: SelectionParam,
    mut commands: Commands,
    mut existing_values_ref_map: Local<HashMap<Entity, Entity>>,
) {
    let list_ref = t.target();
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
        .observe(move |t: Trigger<ItemsChanged>, mut commands: Commands| {
            commands.run_system_cached_with(
                on_items_changed,
                ItemsChangedInput {
                    list_ref,
                    items: t.event().0.iter().collect(),
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
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
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
            let Ok((
                ItemView { item_ref, .. },
                mut node,
                mut bg,
                mut border_radius,
                mut border_color,
            )) = item_views.get_mut(item_view_id)
            else {
                continue;
            };

            let is_selected = selection.is_selected(*item_ref);

            if is_selected {
                bg.0 = theme.colors.selected_item;
                border_color.0 = theme.colors.border_highlight;
            } else {
                bg.0 = theme.colors.background_primary;
            }
            border_color.0 = theme.colors.border;

            theme_item_border(
                &mut node,
                &mut border_radius,
                *corner_style,
                *layout,
                i,
                last,
                &theme,
            );

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
        text_font.font_size = theme.font_size;
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
            &mut BorderRadius,
            &ListLayout,
            &CornerStyle,
        ),
        With<List>,
    >,
) {
    let theme_list = |node: &mut Node,
                      bg: &mut BackgroundColor,
                      bc: &mut BorderColor,
                      br: &mut BorderRadius,
                      layout: ListLayout,
                      corner_style: CornerStyle| {
        node.border = match corner_style {
            CornerStyle::FollowLayout | CornerStyle::None => UiRect::all(theme.border_thickness),
            CornerStyle::Tabs => UiRect::ZERO,
        };
        bg.0 = theme.colors.background_primary;
        bc.0 = theme.colors.border;

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
        *br = radius;
    };

    if let Some(list_id) = *list {
        if let Ok((mut node, mut bg, mut bc, mut br, layout, corner_style)) = lists.get_mut(list_id)
        {
            theme_list(&mut node, &mut bg, &mut bc, &mut br, *layout, *corner_style);
        }
    } else {
        for (mut node, mut bg, mut bc, mut br, layout, corner_style) in &mut lists {
            theme_list(&mut node, &mut bg, &mut bc, &mut br, *layout, *corner_style);
        }
    }
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 21., mouse_wheel_event.y * 21.),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}

const fn theme_item_border(
    node: &mut Node,
    border_radius: &mut BorderRadius,
    corner_style: CornerStyle,
    layout: ListLayout,
    i: usize,
    last: usize,
    theme: &Theme,
) {
    match corner_style {
        CornerStyle::Tabs => {
            *border_radius = match layout {
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
                    *border_radius = match i {
                        0 => BorderRadius::top(theme.corner_radius),
                        n if n == last => BorderRadius::bottom(theme.corner_radius),
                        _ => BorderRadius::ZERO,
                    }
                }
                ListLayout::Horizontal => {
                    *border_radius = match i {
                        0 => BorderRadius::left(theme.corner_radius),
                        n if n == last => BorderRadius::right(theme.corner_radius),
                        _ => BorderRadius::ZERO,
                    }
                }
                ListLayout::VerticalGrid { columns } => {
                    let columns = columns as usize;
                    *border_radius = match i {
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
            *border_radius = BorderRadius::ZERO;
        }
    }
}
