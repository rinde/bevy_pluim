use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::event::EventReader;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::Changed;
use bevy::ecs::query::With;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnRelated;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;
use bevy::picking::Pickable;
use bevy::picking::hover::HoverMap;
use bevy::platform::collections::HashMap;
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
use crate::select::Select;
use crate::select::Selection;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;

pub(crate) struct ListPlugin;

impl Plugin for ListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_selection_change, update_scroll_position))
            .add_observer(theme_items)
            .add_observer(theme_lists)
            .add_observer(theme_item_labels);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_add = Self::on_add)]
pub struct List;

#[derive(Component, Debug, Clone, Copy)]
struct ItemLabel;

impl List {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);

        let items = if let Some(existing_select) = get_anywhere::<Select>(&world, &context, id) {
            existing_select.items.clone()
        } else {
            let new_select = Select { items: Vec::new() };
            world.commands().entity(context.entity).insert(new_select);
            vec![]
        };

        assert!(
            world.entity(context.entity).get::<Node>().is_none(),
            "Cannot override Node for List"
        );

        let children = Children::spawn(SpawnWith(|parent: &mut RelatedSpawner<ChildOf>| {
            for (i, option) in items.into_iter().enumerate() {
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
                        Item { index: i },
                        children![(Text(option), ItemLabel, Pickable::IGNORE)],
                    ))
                    .observe_and_transform_mouse_events()
                    .observe(hover);
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
pub(crate) struct Item {
    index: usize,
}

impl Item {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let Some(parent) = world.entity(context.entity).get::<ChildOf>() else {
            panic!("Item must be a child of an entity");
        };
        let Some(&parent_id) = world.entity(parent.0).get::<WidgetId>() else {
            panic!("Item must be a child of an entity with a WidgetId");
        };
        world.commands().entity(context.entity).insert(parent_id);
    }
}

fn hover(
    t: Trigger<MouseEvent>,
    mut selections: Query<&mut Selection>,
    parents: Query<&ChildOf>,
    mut items: Query<(&Item, &mut BackgroundColor)>,
    theme: Res<Theme>,
) {
    let (item, mut background_color) = items.get_mut(t.target()).unwrap();
    let mut target = t.target();
    // TODO this loop can be avoided by keeping track of the selection entity
    loop {
        if selections.contains(target) {
            break;
        }
        target = parents.get(target).unwrap().0;
    }
    let mut selection = selections.get_mut(target).unwrap();

    match (t.event(), selection.0) {
        (MouseEvent::Pressed, _) => {
            background_color.0 = theme.colors.selected_hovered_item;
        }
        (MouseEvent::Over, Some(x)) if x == item.index => {
            background_color.0 = theme.colors.selected_hovered_item;
        }
        (MouseEvent::Over, _) => {
            background_color.0 = theme.colors.hovered_item;
        }
        (MouseEvent::Out, Some(x)) if x == item.index => {
            background_color.0 = theme.colors.selected_item;
        }
        (MouseEvent::Out, _) => {
            background_color.0 = theme.colors.background_primary;
        }
        (MouseEvent::Released, _) => {
            selection.0 = Some(item.index);
        }
    }
}

fn on_selection_change(
    changed_si_query: Query<(&Selection, &WidgetId), Changed<Selection>>,
    mut all_selectable_items_query: Query<(&Item, &mut BackgroundColor, &WidgetId)>,
    theme: Res<Theme>,
) {
    for (selection, widget_id) in changed_si_query {
        for (item, mut bg, _) in &mut all_selectable_items_query
            .iter_mut()
            .filter(|(_, _, id)| *id == widget_id)
        {
            if selection.0 == Some(item.index) {
                bg.0 = theme.colors.selected_item; // TODO when this is the item that is actively being hovered than this is not the correct color
            } else {
                bg.0 = theme.colors.background_primary;
            }
        }
    }
}

fn theme_items(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut items: Query<(
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
        &WidgetId,
        &Item,
    )>,
    model: Query<(&WidgetId, &Selection, &Select)>,
) {
    #[derive(Debug)]
    struct State {
        selection: Option<usize>,
        num_items: usize,
    }
    let mut map = HashMap::new();
    model.iter().for_each(|(id, selection, select)| {
        map.insert(
            *id,
            State {
                selection: selection.0,
                num_items: select.items.len(),
            },
        );
    });

    for (mut node, mut bg, mut br, id, item) in &mut items {
        let items_state = map.get(id).unwrap_or_else(|| {
            panic!("Found a Item {item:?} {id:?} without a state");
        });
        if items_state.selection == Some(item.index) {
            bg.0 = theme.colors.selected_item;
        } else {
            bg.0 = theme.colors.background_primary;
        }

        let last = items_state.num_items - 1;
        *br = match item.index {
            0 => BorderRadius::top(theme.corner_radius),
            n if n == last => BorderRadius::bottom(theme.corner_radius),
            _ => BorderRadius::ZERO,
        };

        node.padding = theme.item_padding;
    }
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
