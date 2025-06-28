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
use bevy::ecs::spawn::SpawnRelated;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::OnAdd;
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
use bevy::ui::Node;
use bevy::ui::Overflow;
use bevy::ui::OverflowClipMargin;
use bevy::ui::PositionType;
use bevy::ui::ScrollPosition;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::EntityCommandsExt;
use crate::MouseEvent;
use crate::WidgetId;
use crate::get_anywhere;
use crate::get_or_insert_id;
use crate::select::Item;
use crate::select::ItemOf;
use crate::select::Items;
use crate::select::Selection;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;

pub(crate) struct ListPlugin;

impl Plugin for ListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_scroll_position)
            .add_observer(theme_items)
            .add_observer(theme_lists)
            .add_observer(theme_item_labels)
            .add_observer(on_add_item_of)
            .add_observer(on_selection_change);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_insert = Self::on_add)]
#[require(Name::new("List"))]
pub struct List;

#[derive(Component, Debug, Copy, Clone)]
pub struct ListRef(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
struct ItemLabel;

impl List {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);

        let (items_entity, items) = if let Some((items_entity, existing_items)) =
            get_anywhere::<Items>(&world, &context, id)
        {
            (
                items_entity,
                existing_items
                    .into_iter()
                    .map(|item| {
                        let e = world.entity(item).get::<Item>().unwrap();
                        (item, e.label.clone())
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            let new_items = Items::default();
            let items_entity = world
                .commands()
                .entity(context.entity)
                .insert(new_items.clone())
                .id();
            (items_entity, vec![])
        };

        // world
        //     .commands()
        //     .entity(items_entity)
        //     .observe(|t: Trigger<OnReplace>| {
        //         println!("Local handler for SelectionChanged {t:?}");
        //     });

        world
            .commands()
            .entity(items_entity)
            .insert(ListRef(context.entity));

        assert!(
            world.entity(context.entity).get::<Node>().is_none(),
            "Cannot override Node for List"
        );

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
                        ItemView { item: item_ref },
                        children![(Text(item_label), ItemLabel, Pickable::IGNORE)],
                    ))
                    // TODO register observer once and reuse it for all items
                    .observe_and_transform_mouse_events()
                    .observe(handle_mouse_events);
            }
        }));

        // TODO allow options to set max size and scroll if contents don't fit
        let node = Node {
            position_type: PositionType::Relative,
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::RowDense,
            overflow: Overflow::scroll(),
            overflow_clip_margin: OverflowClipMargin::padding_box(),
            ..default()
        };
        world
            .commands()
            .entity(context.entity)
            .insert((node, children));
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_add = Self::on_add)]
pub(crate) struct ItemView {
    item: Entity,
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

fn on_add_item_of(
    t: Trigger<OnAdd, ItemOf>,
    items: Query<(&Item, &ItemOf)>,
    lists: Query<&ListRef>,
    mut commands: Commands,
) {
    let (item, ItemOf(item_of)) = items.get(t.target()).unwrap();
    println!("Insert item {}", item.label);
    let ListRef(list) = lists.get(*item_of).unwrap();

    commands.entity(*list).with_children(|spawner| {
        spawner
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
                ItemView { item: t.target() },
                children![(Text(item.label.clone()), ItemLabel, Pickable::IGNORE)],
            ))
            .observe_and_transform_mouse_events()
            .observe(handle_mouse_events);
    });
}

#[derive(Component)]
struct IsHovered;

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
            item: target_item_entity,
        },
        mut background_color,
    ) = item_views.get_mut(t.target()).unwrap();
    let ItemOf(items_entity) = item_ofs.get(*target_item_entity).unwrap();
    let selection = selections.get(*items_entity).unwrap();

    let is_selected = selection.items.contains(target_item_entity);

    match (t.event(), is_selected) {
        (MouseEvent::Pressed, _) | (MouseEvent::Over, true) => {
            background_color.0 = theme.colors.selected_hovered_item;
            commands.entity(t.target()).insert_if_new(IsHovered);
        }
        (MouseEvent::Over, false) => {
            background_color.0 = theme.colors.hovered_item;
            commands.entity(t.target()).insert_if_new(IsHovered);
        }
        (MouseEvent::Out, true) => {
            background_color.0 = theme.colors.selected_item;
            commands.entity(t.target()).remove::<IsHovered>();
        }
        (MouseEvent::Out, false) => {
            background_color.0 = theme.colors.background_primary;
            commands.entity(t.target()).remove::<IsHovered>();
        }
        (MouseEvent::Released, _) => {
            if let Some(new_selection) = selection.toggle_selected(*target_item_entity) {
                commands.entity(*items_entity).insert(new_selection);
            }
        }
    }
}

// TODO evaluate the relationship between Item and ItemView because these look
// ups are error prone // maybe also a relationship between Item and ItemView,
// each Item could have a Views(Vec<Entity>) <-> DataRef(Entity)
fn on_selection_change(
    t: Trigger<OnInsert, Selection>,
    selections: Query<(&Selection, &ListRef)>,
    lists: Query<&Children, With<List>>,
    mut item_views: Query<(&ItemView, Option<&IsHovered>, &mut BackgroundColor)>,
    theme: Res<Theme>,
) {
    let Ok((selection, ListRef(list_ref))) = selections.get(t.target()) else {
        return;
    };

    for item_view_id in lists.get(*list_ref).unwrap() {
        let (item_view, is_hovered, mut bg) = item_views.get_mut(*item_view_id).unwrap();
        match (selection.is_selected(item_view.item), is_hovered.is_some()) {
            (true, true) => bg.0 = theme.colors.selected_hovered_item,
            (true, false) => bg.0 = theme.colors.selected_item,
            (false, true) => bg.0 = theme.colors.hovered_item,
            (false, false) => bg.0 = theme.colors.background_primary,
        }
    }
}

// TODO use SelectionChanged event
// fn on_selection_change(
//     changed_si_query: Query<(&Selection, &WidgetId, &Items),
// Changed<Selection>>,     sfasd: Query<(&List, &Children)>
//     mut all_selectable_items_query: Query<(&ItemView, &mut BackgroundColor,
// &WidgetId)>,     theme: Res<Theme>,
// ) {
// // TODO use query::iter_descendants()
//     for (selection, widget_id) in changed_si_query {
//         for (item, mut bg, _) in &mut all_selectable_items_query
//             .iter_mut()
//             .filter(|(_, _, id)| *id == widget_id)
//         {
//             if selection.0 == Some(item.item) {
//                 bg.0 = theme.colors.selected_item; // TODO when this is the
// item that is actively being hovered than this is not the correct color
//             } else {
//                 bg.0 = theme.colors.background_primary;
//             }
//         }
//     }
// }

fn theme_items(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut item_views: Query<(&mut Node, &mut BackgroundColor, &mut BorderRadius), With<ItemView>>,
    lists: Query<&Children>,
    model: Query<(&Selection, &Items, &ListRef)>,
) {
    for (selection, items, ListRef(list_ref)) in model {
        println!("{selection:?} {items:?}");
        let last = items.len() - 1;

        for (i, (item_view_id, item_id)) in lists
            .iter_descendants::<Children>(*list_ref)
            .zip(items)
            .enumerate()
        {
            println!("{i}");
            let (mut node, mut bg, mut br) = item_views.get_mut(item_view_id).unwrap();

            let is_selected = selection.is_selected(item_id);

            if is_selected {
                bg.0 = theme.colors.selected_item;
            } else {
                bg.0 = theme.colors.background_primary;
            }

            *br = match i {
                0 => BorderRadius::top(theme.corner_radius),
                n if n == last => BorderRadius::bottom(theme.corner_radius),
                _ => BorderRadius::ZERO,
            };

            node.padding = theme.item_padding;
        }
    }

    // #[derive(Debug)]
    // struct State {
    //     selection: Option<usize>,
    //     num_items: usize,
    // }
    // let mut map = HashMap::new();
    // model.iter().for_each(|(id, selection, items)| {
    //     map.insert(
    //         *id,
    //         State {
    //             selection: None, //selection.0,
    //             num_items: items.len(),
    //         },
    //     );
    // });

    // println!("Theme items");
    // for (mut node, mut bg, mut br, id, item) in &mut items {
    //     println!("{item:?}");
    //     let items_state = map.get(id).unwrap_or_else(|| {
    //         panic!("Found a Item {item:?} {id:?} without a state");
    //     });
    //     // if items_state.selection == Some(item.item) {
    //     //     bg.0 = theme.colors.selected_item;
    //     // } else {
    //     bg.0 = theme.colors.background_primary;
    //     // }

    //     let last = items_state.num_items - 1;
    //     // *br = match item.item {
    //     //     0 => BorderRadius::top(theme.corner_radius),
    //     //     n if n == last => BorderRadius::bottom(theme.corner_radius),
    //     //     _ => BorderRadius::ZERO,
    //     // };

    //     node.padding = theme.item_padding;
    // }
}

fn theme_lists(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut lists: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BorderRadius,
        ),
        With<List>,
    >,
) {
    for (mut node, mut bg, mut bc, mut br) in &mut lists {
        node.border = UiRect::all(theme.border_thickness);
        bg.0 = theme.colors.background_primary;
        bc.0 = theme.colors.border;
        *br = BorderRadius::all(theme.corner_radius);
    }
}

fn theme_item_labels(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut selectable_items_container: Query<(&mut TextColor, &mut TextFont), With<ItemLabel>>,
) {
    for (mut text_color, mut text_font) in &mut selectable_items_container {
        text_color.0 = theme.colors.text_primary;
        text_font.font_size = theme.font_size;
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
