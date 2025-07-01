use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::picking::Pickable;
use bevy::render::view::Visibility;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::FlexDirection;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::RelativeCursorPosition;
use bevy::ui::Val;
use bevy::ui::widget::Text;
use bevy::utils::default;
use itertools::Itertools;

use crate::MouseEvent;
use crate::float::Float;
use crate::get_or_insert_id;
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
        app.add_systems(Update, close_combos_on_outside_click)
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
    Name::new("Combo"),
    RelativeCursorPosition::default()
)
]
pub struct Combo;

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboButton"))]
pub struct ComboButton;

#[derive(Component, Debug, Copy, Clone)]
pub struct ComboSubRefs {
    pub button_id: Entity,
    pub list_id: Entity,
}

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ComboList"))]
pub struct ComboList;

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
#[component(immutable, on_insert=Self::on_insert)]
pub enum ComboState {
    Open,
    Closed,
}

impl ComboState {
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            ComboState::Open => ComboState::Closed,
            ComboState::Closed => ComboState::Open,
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let (state, &ComboSubRefs { list_id, .. }) = world
            .entity(context.entity)
            .components::<(&ComboState, &ComboSubRefs)>();

        let visibility = match state {
            ComboState::Open => Visibility::Visible,
            ComboState::Closed => Visibility::Hidden,
        };
        world.commands().entity(list_id).insert(visibility);
    }
}

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
                    // allow mouse events to pass through to parent
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
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
                        RelativeCursorPosition::default(),
                    ))
                    .id();
            });
        });

        world
            .commands()
            .entity(context.entity)
            .insert((ComboState::Closed, ComboSubRefs { button_id, list_id }))
            .observe(handle_combo_button_mouse_events);
    }
}

fn on_selection_change(
    t: Trigger<OnSelectionChange>,
    combos: Query<&ComboSubRefs, With<Combo>>,
    pblr: Query<&PushButtonRefs>,
    selections: Query<&Selection>,
    items: Query<&Item>,
    mut labels: Query<&mut Text, With<PushButtonLabel>>,
) {
    let Ok(ComboSubRefs {
        button_id: button, ..
    }) = combos.get(t.target())
    else {
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
}

/// System that closes the combo when clicking outside of it.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<
        (
            Entity,
            &RelativeCursorPosition,
            &ComboState,
            &ComboSubRefs,
            &Selection,
        ),
        With<Combo>,
    >,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    combo_lists: Query<&RelativeCursorPosition, With<ComboList>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if mouse_button.just_released(MouseButton::Left) {
        for (combo_id, rcp, &combo_state, subs, selection) in combos.iter() {
            if combo_state == ComboState::Open
                && !rcp.mouse_over()
                && !(selection.multi && combo_lists.get(subs.list_id).unwrap().mouse_over())
            {
                let (mut background_color, mut border_color) =
                    combo_buttons.get_mut(subs.button_id).unwrap();

                *background_color = BackgroundColor(theme.colors.background_primary);
                *border_color = BorderColor(theme.colors.border);
                commands.entity(combo_id).insert(ComboState::Closed);
            }
        }
    }
}

fn handle_combo_button_mouse_events(
    trigger: Trigger<MouseEvent>,
    combos: Query<(&ComboSubRefs, &ComboState), With<Combo>>,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let (ComboSubRefs { button_id, .. }, combo_state) = combos.get(trigger.target()).unwrap();
    let (mut background_color, mut border_color) = combo_buttons.get_mut(*button_id).unwrap();

    match (trigger.event(), combo_state) {
        (MouseEvent::Pressed, _) => {
            commands
                .entity(trigger.target())
                .insert(combo_state.inverse());
        }
        (MouseEvent::Over, _) | (_, ComboState::Open) => {
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor(theme.colors.border_highlight);
        }

        (MouseEvent::Out, ComboState::Closed) => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor(theme.colors.border);
        }
        (MouseEvent::Released, ComboState::Closed) => {}
    }
}
