use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::query::Changed;
use bevy::ecs::resource::Resource;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::spawn::SpawnRelated;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::FromWorld;
use bevy::ecs::world::World;
use bevy::text::TextColor;
use bevy::text::TextFont;
use bevy::ui::AlignContent;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::Display;
use bevy::ui::FlexDirection;
use bevy::ui::FlexWrap;
use bevy::ui::Interaction;
use bevy::ui::JustifyContent;
use bevy::ui::JustifyItems;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::theme::Theme;
use crate::theme::on_theme_change;

#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Widgets>()
            .init_resource::<Theme>()
            .add_systems(
                Update,
                (selectable_items_hover, on_selection_change, on_theme_change),
            );
    }
}

#[derive(Resource, Default, Copy, Clone, Debug)]
pub struct Widgets {
    id: u32,
}

impl Widgets {
    pub const fn next_id(&mut self) -> WidgetId {
        let id = WidgetId(self.id);
        self.id += 1;
        id
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct WidgetId(u32);

impl FromWorld for WidgetId {
    fn from_world(world: &mut World) -> Self {
        let mut widgets = world
            .get_resource_mut::<Widgets>()
            .expect("SelectableItems requires Widgets resource");
        widgets.next_id()
    }
}

#[derive(Component, Debug, Clone)]
#[component(on_add = Self::on_add)]
struct SelectableItems {
    items: Vec<String>,
    selected_index: Option<usize>,
}
impl SelectableItems {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        // insert WidgetId if it doesn't exist
        if world.entity(context.entity).get::<WidgetId>().is_none() {
            let mut widgets = world
                .get_resource_mut::<Widgets>()
                .expect("SelectableItems requires Widgets resource");
            let id = widgets.next_id();
            world.commands().entity(context.entity).insert(id);
        }

        let Some(selectable_items) = world.entity(context.entity).get::<SelectableItems>() else {
            panic!("SelectableItems is in an invalid state");
        };

        let theme = world.resource::<Theme>();

        let last = selectable_items.items.len() - 1;
        let children = selectable_items
            .items
            .iter()
            .enumerate()
            .map(move |(idx, option)| {
                let border_radius = match idx {
                    0 => BorderRadius::top(theme.corner_radius),
                    n if n == last => BorderRadius::bottom(theme.corner_radius),
                    _ => BorderRadius::ZERO,
                };
                (
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        ..default()
                    },
                    border_radius,
                    SelectableItem { index: idx },
                    children![(
                        Text(option.clone()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.colors.text_primary),
                    )],
                )
            })
            .collect::<Vec<_>>();
        let children = Children::spawn(SpawnIter(children.into_iter()));

        let styling_bundle = (
            Node {
                // width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::Wrap,
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(1.0)),
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Start,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme.colors.background_primary),
            BorderColor(theme.colors.border),
            BorderRadius::all(theme.corner_radius),
            children,
        );
        world
            .commands()
            .entity(context.entity)
            .insert(styling_bundle);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_add = Self::on_add)]
pub(crate) struct SelectableItem {
    index: usize,
}

impl SelectableItem {
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

#[must_use]
pub fn selectable_items(options: Vec<impl Into<String> + Send + Sync>) -> impl Bundle {
    (SelectableItems {
        items: options.into_iter().map(Into::into).collect(),
        selected_index: None,
    },)
}

fn selectable_items_hover(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut SelectableItem,
            &WidgetId,
        ),
        Changed<Interaction>,
    >,
    mut selectable_items_query: Query<(&mut SelectableItems, &WidgetId)>,
    theme: Res<Theme>,
) {
    for (interaction, mut background_color, item, widget_id) in &mut interaction_query {
        let Some(mut selectable_items) = selectable_items_query
            .iter_mut()
            .find_map(|(sc, id)| (id == widget_id).then_some(sc))
        else {
            panic!("Found a SelectableItem without a SelectableItems");
        };

        match (*interaction, selectable_items.selected_index) {
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
                selectable_items.selected_index = Some(item.index);
            }
        }
    }
}

fn on_selection_change(
    changed_si_query: Query<(&SelectableItems, &WidgetId), Changed<SelectableItems>>,
    mut all_selectable_items_query: Query<(&SelectableItem, &mut BackgroundColor, &WidgetId)>,
    theme: Res<Theme>,
) {
    for (selectable_items, widget_id) in changed_si_query {
        for (item, mut bg, _) in &mut all_selectable_items_query
            .iter_mut()
            .filter(|(_, _, id)| *id == widget_id)
        {
            if selectable_items.selected_index == Some(item.index) {
                *bg = BackgroundColor(theme.colors.hovered_item);
            } else {
                *bg = BackgroundColor(theme.colors.background_primary);
            }
        }
    }
}
