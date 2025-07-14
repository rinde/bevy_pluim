use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::color::Color;
use bevy::color::palettes::tailwind::GRAY_300;
use bevy::color::palettes::tailwind::GRAY_700;
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
use bevy::ecs::world::World;
use bevy::picking::Pickable;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::Display;
use bevy::ui::JustifyContent;
use bevy::ui::JustifySelf;
use bevy::ui::Node;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::utils::default;
use itertools::Either;

use crate::MouseEvent;
use crate::list::ItemView;
use crate::list::List;
use crate::list::ListLayout;
use crate::list::NoDefaultItems;
use crate::select::Item;
use crate::select::ItemOf;
use crate::select::OnAddItem;
use crate::select::Selection;
use crate::select::ViewOf;
use crate::theme::Theme;

#[derive(Copy, Clone, Debug, Default)]
pub struct ColorPalettePlugin;

impl Plugin for ColorPalettePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    (|| None::<Entity>)
                        .pipe(theme_items)
                        .run_if(resource_changed::<Theme>),
                    // (|| None::<Entity>)
                    //     .pipe(theme_lists)
                    //     .run_if(resource_changed::<Theme>),
                    // (|| None::<Entity>)
                    //     .pipe(theme_item_labels)
                    //     .run_if(resource_changed::<Theme>),
                ),
            )
            .add_observer(on_add_item)
            // .add_observer(on_selection_change)
            ;
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct MyColor(pub Color);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[require(List,ListLayout::VerticalGrid { columns: 3 }, Selection::default(), NoDefaultItems)]
pub struct ColorPalette;

fn on_add_item(
    t: Trigger<OnAddItem>,
    items: Query<&Item>,
    mut commands: Commands,
    lists: Query<(), With<ColorPalette>>,
    world: &World,
) {
    if !lists.contains(t.target()) {
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
            ))
            .observe(handle_mouse_events)
            .with_children(|parent| {
                let color = world.entity(t.event().item_id).get::<MyColor>().unwrap();
                let id = parent
                    .spawn((
                        Node {
                            border: UiRect::all(Val::Px(1.0)),
                            width: Val::Px(35.0),
                            height: Val::Px(35.0),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(5.0)),
                        BackgroundColor(color.0),
                    ))
                    .id();
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

fn theme_items(
    list_id: In<Option<Entity>>,
    theme: Res<Theme>,
    lists: Query<
        (
            // &ListSystemIds,
            &Children,
            Option<&ViewOf>,
            Entity,
        ),
        (With<List>, With<ColorPalette>),
    >,
    selections: Query<&Selection>,
    mut item_views: Query<(
        &Children,
        &ItemView,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
        &mut BorderColor,
    )>,
    mut color_items: Query<(&mut Node, &mut BorderRadius, &mut BorderColor), Without<ItemView>>,
) {
    let it = if let Some(list_id) = list_id.as_ref() {
        Either::Left([lists.get(*list_id).unwrap()].into_iter())
    } else {
        Either::Right(lists.iter())
    };

    for (children, view_of, list_id) in it {
        if (*children).is_empty() {
            println!("No items to theme");
            return;
        }
        // either this List is a view of some items defined elsewhere, or the items are
        // define on the List entity.
        let selection_id = view_of.map_or(list_id, |ViewOf(selection_id)| *selection_id);
        let selection = selections.get(selection_id).unwrap();

        for item_view_id in children {
            let (children, ItemView { item_id }, mut node, mut bg, mut br, mut _bc) =
                item_views.get_mut(*item_view_id).unwrap();
            *br = BorderRadius::all(Val::Px(5.0));
            let is_selected = selection.is_selected(*item_id);

            if is_selected {
                bg.0 = GRAY_700.into();
            } else {
                bg.0 = theme.colors.background_primary;
            }
            node.padding = UiRect::all(Val::Px(1.0));

            let (mut _color_node, mut border_radius, mut border_color) =
                color_items.get_mut(*children.first().unwrap()).unwrap();

            if is_selected {
                border_color.0 = GRAY_300.into(); //theme.colors.border_highlight;
            } else {
                border_color.0 = theme.colors.background_primary; //theme.colors.border;
            }
            // color_node.border = UiRect::all(theme.border_thickness);
            *border_radius = BorderRadius::all(Val::Px(5.0));
        }
    }
}
