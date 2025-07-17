use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::color::Color;
use bevy::color::Luminance;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::ecs::system::IntoSystem;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::OnInsert;
use bevy::picking::Pickable;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::Display;
use bevy::ui::JustifyContent;
use bevy::ui::JustifySelf;
use bevy::ui::Node;
use bevy::ui::RelativeCursorPosition;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::utils::default;
use itertools::Either;

use crate::MouseEvent;
use crate::list::CornerStyle;
use crate::list::ItemView;
use crate::list::ItemsChangedInput;
use crate::list::List;
use crate::list::ListLayout;
use crate::list::NoDefaultItems;
use crate::select::Item;
use crate::select::ItemOf;
use crate::select::Items;
use crate::select::ItemsChanged;
use crate::select::ItemsRef;
use crate::select::OnSelectionChange;
use crate::select::Selection;
use crate::select::SelectionRef;
use crate::theme::Theme;

const COLOR_SQUARE_SIZE: f32 = 35.0;

#[derive(Copy, Clone, Debug, Default)]
pub struct ColorPalettePlugin;

impl Plugin for ColorPalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((|| None::<Entity>)
                .pipe(theme_items)
                .run_if(resource_changed::<Theme>),),
        )
        .add_observer(on_add_items_ref)
        .add_observer(on_selection_change);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[require(
    List,
    ListLayout::VerticalGrid { columns: 6 },
    CornerStyle::None,
    Selection::default(),
    NoDefaultItems
)]
pub struct ColorPalette;

// fn on_add_item(
//     t: Trigger<OnAddItem>,
//     items: Query<&Item<Color>>,
//     mut commands: Commands,
//     lists: Query<(), With<ColorPalette>>,
// ) {
//     if !lists.contains(t.target()) {
//         return;
//     }
//     let color_item = items.get(t.event().item_id).unwrap();
//     let list_id = t.target();

//     commands.entity(list_id).with_children(|spawner| {
//         spawner
//             .spawn((
//                 Name::new("ItemView"),
//                 Button,
//                 // Allow scrolling
//                 Pickable {
//                     should_block_lower: false,
//                     ..default()
//                 },
//                 Node {
//                     display: Display::Flex,
//                     justify_self: JustifySelf::Stretch,
//                     // flex_grow: 1.0,
//                     justify_content: JustifyContent::Stretch,
//                     align_items: AlignItems::Stretch,
//                     padding: UiRect::ZERO,
//                     ..default()
//                 },
//                 ItemView {
//                     item_id: t.event().item_id,
//                 },
//             ))
//             .observe(handle_mouse_events)
//             .with_children(|parent| {
//                 parent.spawn((
//                     Node {
//                         border: UiRect::all(Val::Px(1.0)),
//                         width: Val::Px(COLOR_SQUARE_SIZE),
//                         height: Val::Px(COLOR_SQUARE_SIZE),
//                         ..default()
//                     },
//                     BorderRadius::all(Val::Px(5.0)),
//                     BackgroundColor(color_item.data),
//                     Pickable::IGNORE,
//                 ));
//             });
//     });

//     commands.run_system_cached_with(theme_items, Some(list_id));
// }

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
                .map(|c| item_views.get(*c).unwrap())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut items = items.clone();
    for (view_ref, ItemView { item_id }) in existing {
        let len_before = items.len();
        items.retain(|x| x != item_id);
        if len_before == items.len() {
            // a view was found without a corresponding item
            commands.entity(view_ref).despawn();
        }
    }

    for item_id in items {
        let item = items_query.get(item_id).unwrap();
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
                    ItemView { item_id },
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

fn on_add_items_ref(
    t: Trigger<OnInsert, ItemsRef>,
    lists: Query<&ItemsRef, With<ColorPalette>>,
    items: Query<&Items>,
    mut commands: Commands,
) {
    let Ok(ItemsRef(items_ref)) = lists.get(t.target()).copied() else {
        return;
    };
    let list_ref = t.target();

    // register observer for items change event
    commands
        .entity(items_ref)
        .observe(move |t: Trigger<ItemsChanged>, mut commands: Commands| {
            commands.run_system_cached_with(
                on_items_changed,
                ItemsChangedInput {
                    list_ref,
                    items: t.event().0.iter().copied().collect(),
                },
            );
        });

    // update all items now
    let all_items = items.get(items_ref).unwrap().iter().copied().collect();
    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            list_ref,
            items: all_items,
        },
    );
}

fn handle_mouse_events(
    t: Trigger<MouseEvent>,
    selections: Query<&Selection>,
    item_ofs: Query<(&Item<Color>, &ItemOf)>,
    mut item_views: Query<&ItemView>,
    mut commands: Commands,
) {
    let ItemView {
        item_id: target_item_entity,
    } = item_views.get_mut(t.target()).unwrap();
    let (Item { data: item_color }, ItemOf(items_entity)) =
        item_ofs.get(*target_item_entity).unwrap();
    let selection = selections.get(*items_entity).unwrap();

    let is_selected = selection.items.contains(target_item_entity);

    let highlight = match (t.event(), is_selected) {
        (MouseEvent::Pressed, _) | (MouseEvent::Over, true) => Some(HoverState::SelectedHover),
        (MouseEvent::Over, false) => Some(HoverState::Hover),
        (MouseEvent::Out, true) => Some(HoverState::Selected),
        (MouseEvent::Out, false) => Some(HoverState::Unselected),
        (MouseEvent::Released, _) => {
            if let Some(new_selection) = selection.toggle_selected(*target_item_entity) {
                commands.entity(*items_entity).insert(new_selection);
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
    lists: Query<(Option<&Children>, Option<&SelectionRef>), (With<List>, With<ColorPalette>)>,
    selections: Query<&Selection>,
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
        Either::Left([lists.get(*list_id).unwrap()].into_iter())
    } else {
        Either::Right(lists.iter())
    };

    for (children, selection_ref) in it {
        let Some(children) = children else { return };
        if (*children).is_empty() {
            println!("No items to theme");
            return;
        }
        let selection = if let Some(SelectionRef(selection_ref)) = selection_ref {
            selections.get(*selection_ref).unwrap()
        } else {
            &Selection::default()
        };

        for item_view_id in children {
            let (
                children,
                ItemView { item_id },
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

            let hover_state = if selection.is_selected(*item_id) {
                HoverState::Selected
            } else {
                HoverState::Unselected
            };
            let Item { data: item_color } = color_items.get(*item_id).unwrap();
            commands
                .run_system_cached_with(set_highlight, (*item_view_id, *item_color, hover_state));
        }
    }
}

fn on_selection_change(
    t: Trigger<OnSelectionChange>,
    selections: Query<&Selection>,
    lists: Query<&Children, (With<List>, With<ColorPalette>)>,
    mut item_views: Query<(&ItemView, &RelativeCursorPosition)>,
    color_items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    if !lists.contains(t.target()) {
        return;
    }
    let selection = selections.get(t.event().selection_id).unwrap();

    for item_view_id in lists.get(t.target()).unwrap() {
        let (item_view, rcp) = item_views.get_mut(*item_view_id).unwrap();
        let Item { data: item_color } = color_items.get(item_view.item_id).unwrap();
        let hover_state = match (selection.is_selected(item_view.item_id), rcp.mouse_over()) {
            (true, true) => HoverState::SelectedHover,
            (true, false) => HoverState::Selected,
            (false, true) => HoverState::Hover,
            (false, false) => HoverState::Unselected,
        };
        commands.run_system_cached_with(set_highlight, (*item_view_id, *item_color, hover_state));
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
