use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::name::Name;
use bevy::ecs::query::Changed;
use bevy::ecs::query::Or;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::prelude::SpawnRelated;
use bevy::render::view::Visibility;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::FlexDirection;
use bevy::ui::Interaction;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::Val;
use bevy::utils::default;

use crate::WidgetId;
use crate::float::Float;
use crate::get_or_insert_id;
use crate::list::ItemView;
use crate::list::List;
use crate::push_button::NoDefaultHover;
use crate::push_button::PushButton;
use crate::select::Items;
use crate::select::Selection;
use crate::select::ViewOf;
use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_press,
                // on_change,
                on_hover,
                close_combos_on_outside_click,
            ),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[component(on_add=Self::on_add)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Selection::default(),
    Items::default(),
    Name::new("Combo")
)
]
pub struct Combo;

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboButton"))]
pub struct ComboButton;

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboList"))]
pub struct ComboList;

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);
        world.commands().entity(context.entity).with_children(|x| {
            x.spawn((
                id,
                PushButton::new("Select..", "\\/"),
                ComboButton,
                NoDefaultHover,
            ));
            x.spawn((
                Node { ..default() },
                children![(
                    id,
                    ViewOf(context.entity),
                    List,
                    Float,
                    Visibility::Hidden,
                    ComboList
                )],
            ));
        });
    }
}

fn on_press(
    combo_buttons: Query<(&Interaction, &WidgetId), (Changed<Interaction>, With<ComboButton>)>,
    mut items: Query<(&WidgetId, &mut Visibility), (With<List>, With<Float>)>,
) {
    for (inter, id) in combo_buttons {
        if let Interaction::Pressed = inter {
            for (item_id, mut visibility) in &mut items {
                if item_id == id {
                    visibility.toggle_visible_hidden();
                    break;
                }
            }
        }
    }
}

// fn on_change(
//     changed_selections: Query<(&Selection, &WidgetId, &Select),
// Changed<Selection>>,     mut labels: Query<(&WidgetId, &mut Text),
// With<PushButtonLabel>>,     mut combo_buttons: Query<(&mut BackgroundColor,
// &mut BorderColor), With<ComboButton>>,     theme: Res<Theme>,
// ) {
//     for (selection, id, items) in changed_selections {
//         for (item_id, mut text) in &mut labels {
//             if item_id == id {
//                 if let Some(selection) = selection.0 {
//                     text.0 = items.items[selection].to_string();
//                 }
//                 break;
//             }
//         }
//     }
//     if !changed_selections.is_empty() {
//         for (mut background_color, mut border_color) in &mut combo_buttons {
//             *background_color =
// BackgroundColor(theme.colors.background_primary);             *border_color =
// BorderColor(theme.colors.border);         }
//     }
// }

/// System that closes the combo when clicking outside of it.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combo_interactions: Query<(&WidgetId, &Interaction), Or<(With<ComboButton>, With<ItemView>)>>,
    mut combo_lists: Query<(&WidgetId, &mut Visibility), With<ComboList>>,
) {
    // TODO fix this, it needs to auto-close when a SINGLE combo is open, and stay
    // open when a MULTI combo is open
    if mouse_button.just_pressed(MouseButton::Left) {
        // Check if click was on any combo button or item
        let clicked_on_combobox = combo_interactions
            .iter()
            .find_map(|(id, &interaction)| (interaction == Interaction::Pressed).then_some(*id));

        for (id, mut visibility) in &mut combo_lists {
            if clicked_on_combobox == Some(*id) {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn on_hover(
    mut combo_buttons: Query<
        (
            &WidgetId,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<ComboButton>),
    >,
    combo_lists: Query<(&WidgetId, &Visibility), With<ComboList>>,
    theme: Res<Theme>,
) {
    for (id, interaction, mut background_color, mut border_color) in &mut combo_buttons {
        let (_, visibility) = combo_lists
            .iter()
            .find(|(list_id, _)| id == *list_id)
            .unwrap();

        match (interaction, matches!(visibility, Visibility::Visible)) {
            (Interaction::Hovered, _) | (_, true) => {
                *background_color = BackgroundColor(theme.colors.background_secondary);
                *border_color = BorderColor(theme.colors.border_highlight);
            }
            (Interaction::None, false) => {
                *background_color = BackgroundColor(theme.colors.background_primary);
                *border_color = BorderColor(theme.colors.border);
            }
            (Interaction::Pressed, false) => {}
        }
    }
}
