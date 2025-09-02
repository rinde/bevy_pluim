use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use itertools::Either;

use crate::MouseEvent;
use crate::list::{CornerStyle, ItemView, ItemsChangedInput, List, ListLayout, NoDefaultItems};
use crate::select::{Item, ItemsChanged, Selection, SelectionParam, SelectionValue};
use crate::theme::Theme;

const COLOR_SQUARE_SIZE: f32 = 35.0;

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
            ),
        )
        .add_observer(on_insert_selection);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
) {
    let ItemsChangedInput { list_ref, items } = &*input;
    let Ok(children) = lists.get(*list_ref) else {
        return;
    };

    let existing = children
        .map(|c| {
            c.iter()
                .map(|c| item_views.get(c).unwrap())
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
        let item = items_query.get(item_id).unwrap_or_else(|_| {
            panic!("Item<Color> not found with id {item_id}");
        });
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
                        item_ref: item_id,
                        list_ref: *list_ref,
                    },
                ))
                .observe(handle_mouse_events)
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            border: UiRect::all(Val::Px(1.0)),
                            width: Val::Px(COLOR_SQUARE_SIZE),
                            height: Val::Px(COLOR_SQUARE_SIZE),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(5.0)),
                        BackgroundColor(item.data),
                        Pickable::IGNORE,
                    ));
                });
        });
    }

    commands.run_system_cached_with(theme_items, Some(*list_ref));
}

fn on_insert_selection(
    t: Trigger<OnInsert, Selection>,
    color_palettes: Query<&Selection, With<ColorPalette>>,
    selections: SelectionParam,
    mut commands: Commands,
    mut existing_values_ref_map: Local<HashMap<Entity, Entity>>,
) {
    let list_ref = t.target();
    if color_palettes.get(list_ref).is_err() {
        return;
    }
    let (values_ref, all_items) = selections.get_item_values(list_ref).unwrap();
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

fn handle_mouse_events(
    t: Trigger<MouseEvent>,
    selections: SelectionParam,
    color_items: Query<&Item<Color>>,
    mut item_views: Query<&ItemView>,
    mut commands: Commands,
) {
    let ItemView {
        item_ref: target_item_ref,
        list_ref: target_list_ref,
    } = item_views.get_mut(t.target()).unwrap();
    let Item { data: item_color } = color_items.get(*target_item_ref).unwrap();
    let (selection_ref, selection) = selections
        .get_with_ref(*target_list_ref)
        .unwrap_or_else(|| panic!("ColorPalette: No selection found for entity {target_list_ref}"));

    let is_selected = selection.is_selected(*target_item_ref);

    let highlight = match (t.event(), is_selected) {
        (MouseEvent::Pressed, _) | (MouseEvent::Over, true) => Some(HoverState::SelectedHover),
        (MouseEvent::Over, false) => Some(HoverState::Hover),
        (MouseEvent::Out, true) => Some(HoverState::Selected),
        (MouseEvent::Out, false) => Some(HoverState::Unselected),
        (MouseEvent::Released, _) => {
            if let Some(new_selection) = selection.toggle_selected(*target_item_ref) {
                commands
                    .entity(selection_ref)
                    .insert(new_selection.into_component());
            }
            None
        }
    };

    if let Some(hover_state) = highlight {
        commands.run_system_cached_with(set_highlight, (t.target(), *item_color, hover_state));
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
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
    )>,
    mut color_item_views: Query<(&mut Node, &mut BorderRadius), Without<ItemView>>,
    color_items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    let it = if let Some(list_id) = list_id.as_ref() {
        Either::Left([color_palettes.get(*list_id).unwrap()].into_iter())
    } else {
        Either::Right(color_palettes.iter())
    };

    for (color_palette_ref, children) in it {
        let Some(children) = children else { return };
        if (*children).is_empty() {
            continue;
        }
        let selection = if let Some(selection) = selections.get(color_palette_ref) {
            selection
        } else {
            &SelectionValue::default()
        };

        for item_view_id in children {
            let (
                children,
                ItemView { item_ref, .. },
                mut outer_node,
                mut outer_background_color,
                mut outer_border_radius,
            ) = item_views.get_mut(*item_view_id).unwrap();
            *outer_border_radius = BorderRadius::all(Val::Px(5.0));
            outer_node.padding = UiRect::all(Val::Px(0.5));
            outer_node.border = UiRect::all(Val::Px(1.0));
            outer_background_color.0 = theme.colors.background_primary;

            let (mut inner_node, mut inner_border_radius) = color_item_views
                .get_mut(*children.first().unwrap())
                .unwrap();

            inner_node.border = UiRect::all(Val::Px(1.0));
            *inner_border_radius = BorderRadius::all(Val::Px(5.0));

            let hover_state = if selection.is_selected(*item_ref) {
                HoverState::Selected
            } else {
                HoverState::Unselected
            };
            let Item { data: item_color } = color_items.get(*item_ref).unwrap();
            commands
                .run_system_cached_with(set_highlight, (*item_view_id, *item_color, hover_state));
        }
    }
}

fn on_selection_change(
    q: Query<Entity, Changed<Selection>>,
    selections: SelectionParam,
    color_palettes: Query<(Entity, &Children), With<ColorPalette>>,
    mut item_views: Query<(&ItemView, &RelativeCursorPosition)>,
    color_items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (color_palette_ref, children) in color_palettes {
            let Some((selection_ref, selection)) = selections.get_with_ref(color_palette_ref)
            else {
                continue;
            };
            if selection_ref != changed_selection_ref {
                continue;
            }

            for item_view_id in children {
                let (item_view, rcp) = item_views.get_mut(*item_view_id).unwrap();
                let Item { data: item_color } = color_items.get(item_view.item_ref).unwrap();
                let hover_state =
                    match (selection.is_selected(item_view.item_ref), rcp.mouse_over()) {
                        (true, true) => HoverState::SelectedHover,
                        (true, false) => HoverState::Selected,
                        (false, true) => HoverState::Hover,
                        (false, false) => HoverState::Unselected,
                    };
                commands.run_system_cached_with(
                    set_highlight,
                    (*item_view_id, *item_color, hover_state),
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
    let (children, mut outer_border_color) = item_views.get_mut(item_view_ref).unwrap();

    let background_color = theme.colors.background_primary;
    let (inner, outer) = match hover_state {
        HoverState::SelectedHover => highlight_color(item_color, background_color, 0.8),
        HoverState::Selected => highlight_color(item_color, background_color, 0.6),
        HoverState::Hover => highlight_color(item_color, background_color, 0.3),
        HoverState::Unselected if item_color.luminance() < 0.5 => (item_color.lighter(0.1), None),
        HoverState::Unselected => (item_color.darker(0.1), None),
    };

    outer_border_color.0 = if let Some(outer) = outer {
        outer
    } else {
        background_color
    };

    let child = children.first().unwrap();
    let mut color_square_border = color_squares.get_mut(*child).unwrap();
    color_square_border.0 = inner;
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
