use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::spawn::SpawnRelated;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::ecs::system::IntoSystem;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::World;
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
use bevy::ui::JustifyItems;
use bevy::ui::JustifySelf;
use bevy::ui::Node;
use bevy::ui::Overflow;
use bevy::ui::OverflowClipMargin;
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
use crate::WidgetId;
use crate::get_anywhere;
use crate::get_or_insert_id;
use crate::select::Item;
use crate::select::ItemOf;
use crate::select::Items;
use crate::select::OnAddItem;
use crate::select::OnSelectionChange;
use crate::select::Selection;
use crate::select::ViewOf;
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
            .add_observer(on_add_item)
            .add_observer(on_selection_change);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("List"),
    ListLayout::Vertical,
    CornerStyle::FollowLayout,
    ItemViewFactory(Box::new(ItemLabel::create))
)]
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
}

#[derive(Component)]
pub struct ItemViewFactory(
    pub  Box<
        dyn Fn(&mut RelatedSpawnerCommands<ChildOf>, &World, Entity, &Item) -> Entity + Send + Sync,
    >,
);

impl std::fmt::Debug for ItemViewFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ItemViewFactory").finish()
    }
}

impl List {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);

        let (items_entity, items) = if let Some((items_entity, existing_items)) =
            get_anywhere::<Items>(&world, &context, id)
        {
            let items = existing_items
                .iter()
                .copied()
                .map(|item| {
                    let e = world.entity(item).get::<Item>().unwrap();
                    (item, e.label.clone())
                })
                .collect::<Vec<_>>();

            (items_entity, items)
        } else {
            let new_items = Items::default();
            let items_entity = world
                .commands()
                .entity(context.entity)
                .insert(new_items.clone())
                .id();
            (items_entity, vec![])
        };

        if context.entity != items_entity {
            world
                .commands()
                .entity(context.entity)
                .insert(ViewOf(items_entity));
        }

        assert!(
            world.entity(context.entity).get::<Node>().is_none(),
            "Cannot override Node for List"
        );

        // TODO reuse event handle code
        let children = Children::spawn(SpawnWith(|parent: &mut RelatedSpawner<ChildOf>| {
            for (item_ref, item_label) in items {
                parent
                    .spawn((
                        Button,
                        // Allow scrolling
                        Pickable {
                            should_block_lower: false,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ItemView { item_id: item_ref },
                        children![(Text(item_label), ItemLabel, Pickable::IGNORE)],
                    ))
                    .observe(handle_mouse_events);
            }
        }));

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
            overflow: Overflow::clip(),
            overflow_clip_margin: OverflowClipMargin::padding_box(),
            ..default()
        };
        world
            .commands()
            .entity(context.entity)
            .insert((node, children));

        world
            .commands()
            .run_system_cached_with(theme_lists, Some(context.entity));
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[require(RelativeCursorPosition::default())]
#[component(on_add = Self::on_add)]
pub(crate) struct ItemView {
    item_id: Entity,
}

impl ItemView {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let Some(parent) = world.entity(context.entity).get::<ChildOf>() else {
            panic!("ItemView must be a child of an entity");
        };
        let Some(&parent_id) = world.entity(parent.0).get::<WidgetId>() else {
            panic!("ItemView must be a child of an entity with a WidgetId");
        };
        world.commands().entity(context.entity).insert(parent_id);
    }
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

    fn create(
        parent: &mut RelatedSpawnerCommands<ChildOf>,
        _world: &World,
        _item_ref: Entity,
        item: &Item,
    ) -> Entity {
        parent.spawn((Text(item.label.clone()), ItemLabel)).id()
    }
}

fn on_add_item(
    t: Trigger<OnAddItem>,
    items: Query<&Item>,
    mut commands: Commands,
    lists: Query<&ItemViewFactory, With<List>>,
    world: &World,
) {
    let Ok(factory) = lists.get(t.target()) else {
        return;
    };
    let item = items.get(t.event().item_id).unwrap();
    let list_id = t.target();

    commands.entity(list_id).with_children(|spawner| {
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
                    item_id: t.event().item_id,
                },
                // children![(Text(item.label.clone()), ItemLabel, Pickable::IGNORE)],
            ))
            .observe(handle_mouse_events)
            .with_children(|parent| {
                let id = factory.0(parent, world, t.event().item_id, item);

                parent.commands().entity(id).insert(Pickable::IGNORE);
            });
    });

    commands.run_system_cached_with(theme_items, Some(list_id));
}

fn handle_mouse_events(
    t: Trigger<MouseEvent>,
    selections: Query<&Selection>,
    item_ofs: Query<&ItemOf>,
    mut item_views: Query<(&ItemView, &mut BackgroundColor)>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let (
        ItemView {
            item_id: target_item_entity,
        },
        mut background_color,
    ) = item_views.get_mut(t.target()).unwrap();
    let ItemOf(items_entity) = item_ofs.get(*target_item_entity).unwrap();
    let selection = selections.get(*items_entity).unwrap();

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
                commands.entity(*items_entity).insert(new_selection);
            }
        }
    }
}

fn on_selection_change(
    t: Trigger<OnSelectionChange>,
    selections: Query<&Selection>,
    lists: Query<&Children, With<List>>,
    mut item_views: Query<(&ItemView, &RelativeCursorPosition, &mut BackgroundColor)>,
    theme: Res<Theme>,
) {
    if !lists.contains(t.target()) {
        return;
    }
    let selection = selections.get(t.event().selection_id).unwrap();

    for item_view_id in lists.get(t.target()).unwrap() {
        let (item_view, rcp, mut bg) = item_views.get_mut(*item_view_id).unwrap();

        match (selection.is_selected(item_view.item_id), rcp.mouse_over()) {
            (true, true) => bg.0 = theme.colors.selected_hovered_item,
            (true, false) => bg.0 = theme.colors.selected_item,
            (false, true) => bg.0 = theme.colors.hovered_item,
            (false, false) => bg.0 = theme.colors.background_primary,
        }
    }
}

fn theme_items(
    list_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut item_views: Query<(
        &ItemView,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
        &mut BorderColor,
    )>,
    selections: Query<&Selection>,
    lists: Query<
        (
            &Children,
            Option<&ViewOf>,
            Entity,
            &ListLayout,
            &CornerStyle,
        ),
        With<List>,
    >,
) {
    let mut theme_item = |children: &Children,
                          view_of: Option<&ViewOf>,
                          list_id: Entity,
                          layout: ListLayout,
                          corner_style: CornerStyle| {
        if children.is_empty() {
            return;
        }
        let last = children.len() - 1;
        // either this List is a view of some items defined elsewhere, or the items are
        // define on the List entity.
        let selection_id = view_of.map_or(list_id, |ViewOf(selection_id)| *selection_id);
        let selection = selections.get(selection_id).unwrap();

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
                corner_style,
                layout,
                i,
                last,
                &theme,
            );

            node.padding = theme.item_padding;
        }
    };
    let list_id = list_id.0;

    if let Some(list_id) = list_id {
        let (children, view_of, list_id, layout, corner_style) = lists.get(list_id).unwrap();
        theme_item(children, view_of, list_id, *layout, *corner_style);
    } else {
        for (children, view_of, list_id, layout, corner_style) in lists {
            theme_item(children, view_of, list_id, *layout, *corner_style);
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
            CornerStyle::FollowLayout => UiRect::all(theme.border_thickness),
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
    }
}
