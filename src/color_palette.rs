//! `ColorPalette`: a grid of colour swatches over a selection of [`Item<Color>`](crate::select::Item).

use std::collections::HashMap;

use bevy::a11y::AccessibilityNode;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::ListBoxMultiSelect;
use itertools::Either;

use crate::list::{
    CornerStyle, ItemView, ItemsChangedInput, List, ListLayout, NoDefaultItems, sync_multi_select,
    sync_selected,
};
use crate::select::{Item, ItemsChanged, Selection, SelectionParam, SelectionValue};
use crate::tailwind::color_name;
use crate::theme::Theme;
use crate::{MouseEvent, MouseKind};

const COLOR_SQUARE_SIZE: f32 = 35.0;

/// Registers the systems and observers of [`ColorPalette`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug, Default)]
pub struct ColorPalettePlugin;

impl Plugin for ColorPalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_items)
                    .run_if(resource_changed::<Theme>),
                on_selection_change,
                sync_accessible_names,
            ),
        )
        .add_observer(on_insert_selection)
        .register_type::<Item<Color>>();
    }
}

/// A grid of colour swatches for a selection of [`Item<Color>`](crate::select::Item).
///
/// Like a [`List`] it is a [`ListBox`](bevy::ui_widgets::ListBox) and its swatches
/// ([`ItemView`]s) are [`ListItem`](bevy::ui_widgets::ListItem)s, so a swatch of the
/// selection carries [`Selected`]. The [`Selection`] stays the one source of truth:
/// [`Selected`] follows it, see [`List`]. A swatch's accessible name is the name of its
/// colour: its Tailwind name (`blue-500`) or else its hex (`#3b82f6`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
#[require(
    List,
    ListLayout::VerticalGrid { columns: 6 },
    CornerStyle::None,
    NoDefaultItems,
    Name::new("ColorPalette")
)]
pub struct ColorPalette;

fn on_items_changed(
    input: In<ItemsChangedInput>,
    items_query: Query<&Item<Color>>,
    mut commands: Commands,
    lists: Query<Option<&Children>, With<ColorPalette>>,
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
    for (
        view_ref,
        ItemView {
            item_ref: item_id, ..
        },
    ) in existing
    {
        let len_before = items.len();
        items.retain(|x| x != item_id);
        if len_before == items.len() {
            // a view was found without a corresponding item
            commands.entity(view_ref).despawn();
        }
    }

    for item_id in items {
        let Ok(item) = items_query.get(item_id) else {
            continue;
        };
        let is_selected = selection.is_some_and(|selection| selection.is_selected(item_id));
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
                    item_ref: item_id,
                    list_ref: *list_ref,
                },
            ));
            view.observe(handle_mouse_events).with_children(|parent| {
                parent.spawn((
                    Node {
                        border: UiRect::all(Val::Px(1.0)),
                        width: Val::Px(COLOR_SQUARE_SIZE),
                        height: Val::Px(COLOR_SQUARE_SIZE),
                        border_radius: BorderRadius::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(item.data),
                    Pickable::IGNORE,
                ));
            });
            if is_selected {
                view.insert(Selected);
            }
        });
    }

    commands.run_system_cached_with(theme_items, Some(*list_ref));
}

/// Keeps the label of each swatch's [`AccessibilityNode`] at the name of its colour.
fn sync_accessible_names(
    swatches: Query<(Entity, &ItemView)>,
    palettes: Query<(), With<ColorPalette>>,
    items: Query<&Item<Color>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (entity, view) in &swatches {
        if !palettes.contains(view.list_ref) {
            continue;
        }
        let (Ok(Item { data }), Ok(mut node)) = (items.get(view.item_ref), nodes.get_mut(entity))
        else {
            continue;
        };
        let name = color_name(*data);
        if node.label() != Some(name.as_str()) {
            node.set_label(name);
        }
    }
}

fn on_insert_selection(
    t: On<Insert<Selection>>,
    color_palettes: Query<&Selection, With<ColorPalette>>,
    selections: SelectionParam,
    mut commands: Commands,
    mut existing_values_ref_map: Local<HashMap<Entity, Entity>>,
) {
    let list_ref = t.event().event_target();
    if color_palettes.get(list_ref).is_err() {
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

fn handle_mouse_events(
    t: On<MouseEvent>,
    selections: SelectionParam,
    color_items: Query<&Item<Color>>,
    item_views: Query<&ItemView>,
    mut commands: Commands,
) {
    let Ok(ItemView {
        item_ref: target_item_ref,
        list_ref: target_list_ref,
    }) = item_views.get(t.event().event_target())
    else {
        return;
    };
    let Ok(Item { data: item_color }) = color_items.get(*target_item_ref) else {
        return;
    };
    let Some(selection) = selections.get(*target_list_ref) else {
        return;
    };

    let is_selected = selection.is_selected(*target_item_ref);

    let highlight = match t.event().kind {
        // A press is a pointer on the swatch, and shows the strongest highlight.
        MouseKind::Pressed => Some(HoverState::SelectedHover),
        MouseKind::Over => Some(HoverState::of(is_selected, true)),
        MouseKind::Out => Some(HoverState::of(is_selected, false)),
        // A click and a drag say nothing new about the pointer being on the swatch: the
        // over, press, release and out events around them set the highlight. A drag in
        // particular can arrive after the pointer left the swatch.
        // A release says nothing either: selecting and deselecting are the `ListBox`'s events,
        // see `list::on_value_change` and `list::on_reselect_row`.
        MouseKind::Click(_) | MouseKind::Drag | MouseKind::Released => None,
    };

    if let Some(hover_state) = highlight {
        commands.run_system_cached_with(
            set_highlight,
            (t.event().event_target(), *item_color, hover_state),
        );
    }
}

fn theme_items(
    list_id: In<Option<Entity>>,
    theme: Res<Theme>,
    color_palettes: Query<(Entity, Option<&Children>), With<ColorPalette>>,
    selections: SelectionParam,
    mut item_views: Query<(
        &Children,
        &ItemView,
        &Hovered,
        &mut Node,
        &mut BackgroundColor,
    )>,
    mut color_item_views: Query<&mut Node, Without<ItemView>>,
    color_items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    let it = list_id.as_ref().map_or_else(
        || Either::Right(color_palettes.iter()),
        |list_id| Either::Left(color_palettes.get(*list_id).into_iter()),
    );
    let no_selection = SelectionValue::default();

    for (color_palette_ref, children) in it {
        let Some(children) = children else { continue };
        if (*children).is_empty() {
            continue;
        }
        let selection = selections.get(color_palette_ref).unwrap_or(&no_selection);

        for item_view_id in children {
            let Ok((
                children,
                ItemView { item_ref, .. },
                hovered,
                mut outer_node,
                mut outer_background_color,
            )) = item_views.get_mut(*item_view_id)
            else {
                continue;
            };
            outer_node.border_radius = BorderRadius::all(Val::Px(5.0));
            outer_node.padding = UiRect::all(Val::Px(0.5));
            outer_node.border = UiRect::all(Val::Px(1.0));
            outer_background_color.0 = theme.colors.background_primary;

            let Some(mut inner_node) = children
                .first()
                .and_then(|child| color_item_views.get_mut(*child).ok())
            else {
                continue;
            };

            inner_node.border = UiRect::all(Val::Px(1.0));
            inner_node.border_radius = BorderRadius::all(Val::Px(5.0));

            let hover_state = HoverState::of(selection.is_selected(*item_ref), hovered.get());
            let Ok(Item { data: item_color }) = color_items.get(*item_ref) else {
                continue;
            };
            commands
                .run_system_cached_with(set_highlight, (*item_view_id, *item_color, hover_state));
        }
    }
}

fn on_selection_change(
    q: Query<Entity, Changed<Selection>>,
    selections: SelectionParam,
    color_palettes: Query<(Entity, Option<&Children>, Has<ListBoxMultiSelect>), With<ColorPalette>>,
    item_views: Query<(&ItemView, &Hovered, Has<Selected>)>,
    color_items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (color_palette_ref, children, is_multi) in color_palettes {
            let Some((selection_ref, selection)) = selections.get_with_ref(color_palette_ref)
            else {
                continue;
            };
            // Its own selection, or the one it refers to.
            if selection_ref != changed_selection_ref && color_palette_ref != changed_selection_ref
            {
                continue;
            }
            sync_multi_select(&mut commands, color_palette_ref, is_multi, selection);

            for item_view_id in children.map(RelationshipTarget::iter).unwrap_or_default() {
                let Ok((item_view, hovered, was_selected)) = item_views.get(item_view_id) else {
                    continue;
                };
                sync_selected(
                    &mut commands,
                    item_view_id,
                    was_selected,
                    selection.is_selected(item_view.item_ref),
                );
                let Ok(Item { data: item_color }) = color_items.get(item_view.item_ref) else {
                    continue;
                };
                let hover_state =
                    HoverState::of(selection.is_selected(item_view.item_ref), hovered.get());
                commands.run_system_cached_with(
                    set_highlight,
                    (item_view_id, *item_color, hover_state),
                );
            }
        }
    }
}

fn set_highlight(
    input: In<(Entity, Color, HoverState)>,
    mut item_views: Query<(&Children, &mut BorderColor), With<ItemView>>,
    mut color_squares: Query<&mut BorderColor, Without<ItemView>>,
    theme: Res<Theme>,
) {
    let (item_view_ref, item_color, hover_state) = *input;
    let Ok((children, mut outer_border_color)) = item_views.get_mut(item_view_ref) else {
        return;
    };

    let background_color = theme.colors.background_primary;
    let (inner, outer) = match hover_state {
        HoverState::SelectedHover => highlight_color(item_color, background_color, 0.8),
        HoverState::Selected => highlight_color(item_color, background_color, 0.6),
        HoverState::Hover => highlight_color(item_color, background_color, 0.3),
        HoverState::Unselected if item_color.luminance() < 0.5 => (item_color.lighter(0.1), None),
        HoverState::Unselected => (item_color.darker(0.1), None),
    };

    outer_border_color.set_all(outer.unwrap_or(background_color));

    if let Some(mut color_square_border) = children
        .first()
        .and_then(|child| color_squares.get_mut(*child).ok())
    {
        color_square_border.set_all(inner);
    }
}

fn highlight_color(
    item_color: Color,
    background_color: Color,
    amount: f32,
) -> (Color, Option<Color>) {
    let background_is_light = background_color.luminance() > 0.5;
    let item_color_is_light = item_color.luminance() > 0.5;

    match (background_is_light, item_color_is_light) {
        (true, true) => (item_color.darker(amount), None),
        (true, false) => (item_color.lighter(amount), Some(item_color.darker(amount))),
        (false, true) => (item_color.darker(amount), Some(item_color.lighter(amount))),
        (false, false) => (item_color.lighter(amount), None),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoverState {
    SelectedHover,
    Selected,
    Unselected,
    Hover,
}

impl HoverState {
    /// The highlight of a swatch from its state, for the hover handler and the theme
    /// system alike so that they cannot disagree.
    const fn of(selected: bool, hovered: bool) -> Self {
        match (selected, hovered) {
            (true, true) => Self::SelectedHover,
            (true, false) => Self::Selected,
            (false, true) => Self::Hover,
            (false, false) => Self::Unselected,
        }
    }
}
