use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::relationship::RelationshipSourceCollection;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::ecs::system::IntoSystem;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::OnInsert;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;
use bevy::picking::Pickable;
use bevy::picking::hover::HoverMap;
use bevy::text::TextColor;
use bevy::text::TextFont;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::Display;
use bevy::ui::GridAutoFlow;
use bevy::ui::JustifyContent;
use bevy::ui::JustifySelf;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::RelativeCursorPosition;
use bevy::ui::RepeatedGridTrack;
use bevy::ui::ScrollPosition;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;
use itertools::Either;

use crate::MouseEvent;
use crate::select::Item;
use crate::select::Items;
use crate::select::ItemsChanged;
use crate::select::ItemsRef;
use crate::select::Selection;
use crate::select::SelectionRef;
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
                ),
            )
            .add_observer(on_insert_selection_ref)
            .add_observer(on_add_items_ref);
    }
}

#[derive(Component, Debug, Copy, Clone, Default)]
#[component(on_insert = Self::on_insert)]
#[require(Name::new("List"), ListLayout::Vertical, CornerStyle::FollowLayout,  Pickable {
    should_block_lower: false,
    ..default()
},)]
pub struct List;

#[derive(Component, Debug, Copy, Clone)]
pub enum ListLayout {
    Vertical,
    Horizontal,
    VerticalGrid { columns: u16 },
}

#[derive(Component, Debug, Copy, Clone)]
pub enum CornerStyle {
    FollowLayout,
    Tabs,
    None,
}

#[derive(Component, Copy, Clone, Default, Debug)]
pub struct NoDefaultItems;

impl List {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        // TODO change this assert
        // assert!(
        //     world.entity(context.entity).get::<Node>().is_none(),
        //     "Cannot override Node for List"
        // );

        let layout = world.entity(context.entity).get::<ListLayout>().unwrap();
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

#[derive(Component, Debug, Copy, Clone)]
#[require(RelativeCursorPosition::default())]
pub struct ItemView {
    pub item_id: Entity,
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
    selections: Query<&Selection>,
    selection_refs: Query<&SelectionRef>,
    mut item_views: Query<(&ItemView, &ChildOf, &mut BackgroundColor)>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let (
        ItemView {
            item_id: target_item_entity,
        },
        ChildOf(list_entity),
        mut background_color,
    ) = item_views.get_mut(t.target()).unwrap();
    let SelectionRef(selection_ref) = selection_refs.get(*list_entity).unwrap();
    let selection = selections.get(*selection_ref).unwrap();
    let is_selected = selection.items.contains(target_item_entity);

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
            if let Some(new_selection) = selection.toggle_selected(*target_item_entity) {
                commands.entity(*selection_ref).insert(new_selection);
            }
        }
    }
}

#[derive(Debug)]
struct SelectionChangeInput {
    selection_ref: Entity,
    list_ref: Entity,
}

fn on_selection_change(
    t: In<SelectionChangeInput>,
    selections: Query<&Selection>,
    lists: Query<&Children, (With<List>, Without<NoDefaultItems>)>,
    mut item_views: Query<(&ItemView, &RelativeCursorPosition, &mut BackgroundColor)>,
    theme: Res<Theme>,
) {
    let SelectionChangeInput {
        selection_ref,
        list_ref,
    } = *t;
    let selection = selections.get(selection_ref).unwrap();

    let Ok(children) = lists.get(list_ref) else {
        return;
    };

    for item_view_id in children {
        let (item_view, rcp, mut bg) = item_views.get_mut(*item_view_id).unwrap();

        match (selection.is_selected(item_view.item_id), rcp.mouse_over()) {
            (true, true) => bg.0 = theme.colors.selected_hovered_item,
            (true, false) => bg.0 = theme.colors.selected_item,
            (false, true) => bg.0 = theme.colors.hovered_item,
            (false, false) => bg.0 = theme.colors.background_primary,
        }
    }
}

fn on_insert_selection_ref(
    t: Trigger<OnInsert, SelectionRef>,
    lists: Query<&SelectionRef, With<List>>,
    mut commands: Commands,
) {
    let Ok(SelectionRef(selection_ref)) = lists.get(t.target()).copied() else {
        return;
    };
    let list_ref = t.target();

    // register observer for selection change event
    commands.entity(selection_ref).observe(
        move |_: Trigger<OnInsert, Selection>, mut commands: Commands| {
            let input = SelectionChangeInput {
                selection_ref,
                list_ref,
            };
            commands.run_system_cached_with(on_selection_change, input);
        },
    );

    // update view now
    commands.run_system_cached_with(
        on_selection_change,
        SelectionChangeInput {
            selection_ref,
            list_ref,
        },
    );
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
                    parent.spawn((Text(item.data.clone()), ItemLabel, Pickable::IGNORE));
                });
        });
    }

    commands.run_system_cached_with(theme_items, Some(*list_ref));
}

fn on_add_items_ref(
    t: Trigger<OnInsert, ItemsRef>,
    lists: Query<&ItemsRef, (With<List>, Without<NoDefaultItems>)>,
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
                    items: t.event().0.iter().collect(),
                },
            );
        });

    // update all items now
    let all_items = items.get(items_ref).unwrap().iter().collect();
    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            list_ref,
            items: all_items,
        },
    );
}

fn theme_items(
    list_id: In<Option<Entity>>,
    lists: Query<
        (
            Option<&Children>,
            Option<&SelectionRef>,
            &ListLayout,
            &CornerStyle,
        ),
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
    selections: Query<&Selection>,
) {
    let iter = if let Some(list_id) = list_id.0 {
        Either::Left([lists.get(list_id).unwrap()].into_iter())
    } else {
        Either::Right(lists.into_iter())
    };

    for (children, selection_ref, layout, corner_style) in iter {
        let Some(children) = children else {
            continue;
        };
        if children.is_empty() {
            continue;
        }
        let last = children.len() - 1;

        let selection = if let Some(SelectionRef(selection_ref)) = selection_ref {
            selections.get(*selection_ref).unwrap()
        } else {
            &Selection::default()
        };

        for (i, item_view_id) in children.iter().enumerate() {
            let (ItemView { item_id }, mut node, mut bg, mut border_radius, mut border_color) =
                item_views.get_mut(*item_view_id).unwrap();

            let is_selected = selection.is_selected(*item_id);

            if is_selected {
                bg.0 = theme.colors.selected_item;
                border_color.0 = theme.colors.border_highlight;
            } else {
                bg.0 = theme.colors.background_primary;
                border_color.0 = theme.colors.border;
            }

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
        Either::Left([item_labels.get_mut(*item_label_ref).unwrap()].into_iter())
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
        let (mut node, mut bg, mut bc, mut br, layout, corner_style) =
            lists.get_mut(list_id).unwrap();
        theme_list(&mut node, &mut bg, &mut bc, &mut br, *layout, *corner_style);
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
                        n if n == last && last % columns == 0 => {
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
