use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::children;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
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
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::WidgetId;
use crate::float::Float;
use crate::get_or_insert_id;
use crate::list::Item;
use crate::list::List;
use crate::push_button::NoDefaultHover;
use crate::push_button::PushButton;
use crate::push_button::PushButtonLabel;
use crate::select::Select;
use crate::select::Selection;
use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

// TODO ensure list is visible within window
// TODO allow specifying placeholder text
impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_press, on_change, on_hover, close_combos_on_outside_click),
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
    Select::default(),
)
]
pub struct Combo;

#[derive(Component, Debug, Copy, Clone)]
pub struct ComboButton;

#[derive(Component, Debug, Copy, Clone)]
pub struct ComboList;

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);
        world.commands().entity(context.entity).with_children(|x| {
            x.spawn((id, PushButton::new("Select.."), ComboButton, NoDefaultHover));
            x.spawn((
                Node { ..default() },
                children![(id, List, Float, Visibility::Hidden, ComboList)],
            ));
        });
    }
}

fn on_press(
    combo_buttons: Query<
        (&Interaction, &WidgetId),
        (Changed<Interaction>, Or<(With<ComboButton>, With<Item>)>),
    >,
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

fn on_change(
    changed_selections: Query<(&Selection, &WidgetId, &Select), Changed<Selection>>,
    mut labels: Query<(&WidgetId, &mut Text), With<PushButtonLabel>>,
) {
    for (selection, id, items) in changed_selections {
        for (item_id, mut text) in &mut labels {
            if item_id == id {
                if let Some(selection) = selection.0 {
                    text.0 = items.items[selection].to_string();
                }
                break;
            }
        }
    }
}

/// System that closes the combo when clicking outside of it.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combo_interactions: Query<&Interaction, Or<(With<ComboButton>, With<Item>)>>,
    mut combo_lists: Query<&mut Visibility, With<ComboList>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    // Check if click was on any combo button or item
    let clicked_on_combobox = combo_interactions
        .iter()
        .any(|&interaction| interaction == Interaction::Pressed);

    if !clicked_on_combobox {
        for mut visibility in &mut combo_lists {
            *visibility = Visibility::Hidden;
        }
    }
}

fn on_hover(
    mut query: Query<
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
    for (id, interaction, mut background_color, mut border_color) in &mut query {
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
