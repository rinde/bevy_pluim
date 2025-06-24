use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::Changed;
use bevy::ecs::query::With;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::spawn::SpawnRelated;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashMap;
use bevy::text::TextColor;
use bevy::text::TextFont;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::Display;
use bevy::ui::GridAutoFlow;
use bevy::ui::Interaction;
use bevy::ui::JustifyContent;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::WidgetId;
use crate::Widgets;
use crate::float::Float;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;

pub(crate) struct SelectableItemsPlugin;

impl Plugin for SelectableItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (selectable_items_hover, on_selection_change))
            .add_observer(theme_item)
            .add_observer(theme_selectable_items)
            .add_observer(theme_item_label);
    }
}

#[derive(Component, Debug, Clone)]
#[component(on_add = Self::on_add)]
pub struct SelectableItems {
    items: Vec<String>,
}

impl SelectableItems {
    #[must_use]
    pub fn new(options: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            items: options.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Selection(pub Option<usize>);

#[derive(Component, Debug, Clone, Copy)]
struct ItemLabel;

impl SelectableItems {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        // insert WidgetId if it doesn't exist
        let id = world
            .entity(context.entity)
            .get::<WidgetId>()
            .copied()
            .unwrap_or_else(|| {
                let mut widgets = world
                    .get_resource_mut::<Widgets>()
                    .expect("SelectableItems requires Widgets resource");
                let id = widgets.next_id();
                world.commands().entity(context.entity).insert(id);
                id
            });

        println!("Add SelectableItems component {id:?}");

        // use existing selection if it exists, or query it from any of its parents or
        // default to None
        if world.entity(context.entity).get::<Selection>().is_none() {
            let state = world.try_query_filtered::<&WidgetId, With<Selection>>();
            let has_selection =
                state.is_some_and(|mut s| s.iter(&world).any(|selection_id| (selection_id == &id)));

            if !has_selection {
                world
                    .commands()
                    .entity(context.entity)
                    .insert(Selection(None));
            }
        }

        let Some(selectable_items) = world
            .entity(context.entity)
            .get::<SelectableItems>()
            .cloned()
        else {
            panic!("SelectableItems is in an invalid state");
        };

        assert!(
            world.entity(context.entity).get::<Node>().is_none(),
            "Cannot override Node for SelectableItems"
        );

        let children = selectable_items
            .items
            .into_iter()
            .enumerate()
            .map(move |(idx, option)| {
                (
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Item { index: idx },
                    children![(Text(option), ItemLabel)],
                )
            })
            .collect::<Vec<_>>();
        let children = Children::spawn(SpawnIter(children.into_iter()));

        let mut node = Node {
            position_type: PositionType::Relative,
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::RowDense,
            ..default()
        };
        if let Some(float) = world.entity(context.entity).get::<Float>() {
            float.apply(&mut node);
        }
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
            panic!("SelectableItem must be a child of an entity");
        };
        let Some(&parent_id) = world.entity(parent.0).get::<WidgetId>() else {
            panic!("SelectableItem must be a child of an entity with a WidgetId");
        };
        world.commands().entity(context.entity).insert(parent_id);
    }
}

fn selectable_items_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut Item, &WidgetId),
        Changed<Interaction>,
    >,
    mut selectable_items_query: Query<(&mut Selection, &WidgetId)>,
    theme: Res<Theme>,
) {
    for (interaction, mut background_color, item, widget_id) in &mut interaction_query {
        let Some(mut selection) = selectable_items_query
            .iter_mut()
            .find_map(|(sc, id)| (id == widget_id).then_some(sc))
        else {
            panic!("Found a SelectableItem without a SelectableItems");
        };

        match (*interaction, selection.0) {
            (Interaction::Hovered, Some(x)) if x == item.index => {
                *background_color = BackgroundColor(theme.colors.selected_hovered_item);
            }
            (Interaction::Hovered, _) => {
                *background_color = BackgroundColor(theme.colors.hovered_item);
            }
            (Interaction::None, Some(x)) if x == item.index => {
                *background_color = BackgroundColor(theme.colors.selected_item);
            }
            (Interaction::None, _) => {
                *background_color = BackgroundColor(theme.colors.background_primary);
            }
            (Interaction::Pressed, _) => {
                selection.0 = Some(item.index);
            }
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
                *bg = BackgroundColor(theme.colors.selected_item);
            } else {
                *bg = BackgroundColor(theme.colors.background_primary);
            }
        }
    }
}

fn theme_item(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut items: Query<(
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
        &WidgetId,
        &Item,
    )>,
    containers: Query<(&WidgetId, &Selection, &SelectableItems)>,
) {
    struct State {
        selection: Option<usize>,
        num_items: usize,
    }
    let mut map = HashMap::new();
    containers.iter().for_each(|(id, selection, items)| {
        map.insert(
            *id,
            State {
                selection: selection.0,
                num_items: items.items.len(),
            },
        );
    });

    for (mut node, mut bg, mut br, id, item) in &mut items {
        let items_state = map.get(id).unwrap();
        if items_state.selection == Some(item.index) {
            *bg = BackgroundColor(theme.colors.selected_item);
        } else {
            *bg = BackgroundColor(theme.colors.background_primary);
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

fn theme_selectable_items(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut selectable_items_container: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BorderRadius,
        ),
        With<SelectableItems>,
    >,
) {
    for (mut node, mut bg, mut bc, mut br) in &mut selectable_items_container {
        node.border = UiRect::all(theme.border_thickness);
        *bg = BackgroundColor(theme.colors.background_primary);
        *bc = BorderColor(theme.colors.border);
        *br = BorderRadius::all(theme.corner_radius);
    }
}

fn theme_item_label(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut selectable_items_container: Query<(&mut TextColor, &mut TextFont), With<ItemLabel>>,
) {
    for (mut text_color, mut text_font) in &mut selectable_items_container {
        text_color.0 = theme.colors.text_primary;
        text_font.font_size = theme.font_size;
    }
}
