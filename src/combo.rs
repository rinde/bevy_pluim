use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::Changed;
use bevy::ecs::query::Or;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::render::view::Visibility;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::FlexDirection;
use bevy::ui::Interaction;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::Val;
use bevy::ui::widget::Text;
use bevy::utils::default;
use itertools::Itertools;

use crate::WidgetId;
use crate::float::Float;
use crate::get_or_insert_id;
use crate::list::ItemView;
use crate::list::List;
use crate::push_button::NoDefaultHover;
use crate::push_button::PushButton;
use crate::push_button::PushButtonLabel;
use crate::push_button::PushButtonRefs;
use crate::select::Item;
use crate::select::Items;
use crate::select::OnSelectionChange;
use crate::select::Selection;
use crate::select::ViewOf;
use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_press, on_hover, close_combos_on_outside_click))
            .add_observer(on_selection_change);
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
pub struct SubComponentsRef {
    pub button: Entity,
    pub list: Entity,
}

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboList"))]
pub struct ComboList;

const PLACEHOLDER_TEXT: &str = "Select..";

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);
        let mut button_id = Entity::PLACEHOLDER;
        let mut list_id = Entity::PLACEHOLDER;
        world.commands().entity(context.entity).with_children(|x| {
            button_id = x
                .spawn((
                    id,
                    PushButton::new(PLACEHOLDER_TEXT, "\\/"),
                    ComboButton,
                    NoDefaultHover,
                ))
                .id();

            x.spawn(Node::default()).with_children(|spawner| {
                list_id = spawner
                    .spawn((
                        id,
                        ViewOf(context.entity),
                        List,
                        Float,
                        Visibility::Hidden,
                        ComboList,
                    ))
                    .id();
            });
        });

        world
            .commands()
            .entity(context.entity)
            .insert(SubComponentsRef {
                button: button_id,
                list: list_id,
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

fn on_selection_change(
    t: Trigger<OnSelectionChange>,
    combos: Query<&SubComponentsRef, With<Combo>>,
    pblr: Query<&PushButtonRefs>,
    selections: Query<&Selection>,
    items: Query<&Item>,
    theme: Res<Theme>,
    mut lists: Query<&mut Visibility, With<List>>,
    mut labels: Query<&mut Text, With<PushButtonLabel>>,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
) {
    let Ok(SubComponentsRef { button, list }) = combos.get(t.target()) else {
        return;
    };

    let PushButtonRefs { label_id, .. } = pblr.get(*button).unwrap();
    let selection = selections.get(t.event().selection_id).unwrap();

    let mut text = labels.get_mut(*label_id).unwrap();
    let label = if selection.items.is_empty() {
        PLACEHOLDER_TEXT.to_string()
    } else {
        selection
            .items
            .iter()
            .map(|item| &items.get(*item).unwrap().label)
            .join(", ")
    };
    text.0 = label;

    let mut list = lists.get_mut(*list).unwrap();
    if !selection.multi {
        *list = Visibility::Hidden;
        let (mut background_color, mut border_color) = combo_buttons.get_mut(*button).unwrap();
        *background_color = BackgroundColor(theme.colors.background_primary);
        *border_color = BorderColor(theme.colors.border);
    }
}

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
