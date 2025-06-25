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
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::SpawnRelated;
use bevy::render::view::Visibility;
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
use crate::push_button::PushButton;
use crate::push_button::PushButtonLabel;
use crate::select::Select;
use crate::select::Selection;

#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        // TODO add detection for press outside
        app.add_systems(Update, (on_press, on_change));
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

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let id = get_or_insert_id(&mut world, &context);
        world.commands().entity(context.entity).with_children(|x| {
            x.spawn((id, PushButton::new("Select.."), ComboButton));
            x.spawn((
                Node { ..default() },
                children![(id, List, Float, Visibility::Hidden,)],
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
