use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::query::Changed;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::spawn::SpawnRelated;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
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

pub(crate) struct SelectableItemsPlugin;

impl Plugin for SelectableItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (selectable_items_hover, on_selection_change));
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
        let selection = world
            .entity(context.entity)
            .get::<Selection>()
            .copied()
            .unwrap_or_else(|| {
                let state = world.try_query_filtered::<(&Selection, &WidgetId), ()>();
                let select = state.and_then(|mut s| {
                    s.iter(&world)
                        .find_map(|(selection, selection_id)| {
                            (selection_id == &id).then_some(selection)
                        })
                        .copied()
                });

                select.unwrap_or_else(|| {
                    let new_selection = Selection(None);
                    world
                        .commands()
                        .entity(context.entity)
                        .insert(new_selection);
                    new_selection
                })
            });

        let Some(selectable_items) = world
            .entity(context.entity)
            .get::<SelectableItems>()
            .cloned()
        else {
            panic!("SelectableItems is in an invalid state");
        };

        let theme = *world.resource::<Theme>();

        assert!(
            world.entity(context.entity).get::<Node>().is_none(),
            "Cannot override Node for SelectableItems"
        );

        let last = selectable_items.items.len() - 1;
        let children = selectable_items
            .items
            .into_iter()
            .enumerate()
            .map(move |(idx, option)| {
                let bg_color = if selection.0 == Some(idx) {
                    BackgroundColor(theme.colors.selected_item)
                } else {
                    BackgroundColor(theme.colors.background_primary)
                };
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
                    bg_color,
                    SelectableItem { index: idx },
                    children![(
                        Text(option),
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

        let mut node = Node {
            position_type: PositionType::Relative,
            border: UiRect::all(theme.border_thickness),
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::RowDense,
            ..default()
        };

        if let Some(float) = world.entity(context.entity).get::<Float>() {
            let float_bundle = float.apply(&mut node, &theme);
            world.commands().entity(context.entity).insert(float_bundle);
        }

        let styling_bundle = (
            node,
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
    mut all_selectable_items_query: Query<(&SelectableItem, &mut BackgroundColor, &WidgetId)>,
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
