use bevy::app::App;
use bevy::app::Plugin;
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
use bevy::picking::Pickable;
use bevy::render::view::Visibility;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::FlexDirection;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::RelativeCursorPosition;
use bevy::ui::Val;
use bevy::utils::default;

use crate::MouseEvent;
use crate::color_palette::ColorPalette;
use crate::float::Float;
use crate::push_button::NoDefaultHover;
use crate::push_button::PushButton;
use crate::scroll_area::ScrollArea;
use crate::select::Items;
use crate::select::Selection;
use crate::select::ViewOf;
use crate::theme::Theme;

#[derive(Copy, Clone, Debug, Default)]
pub struct ColorPaletteComboPlugin;

impl Plugin for ColorPaletteComboPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_systems(
        //     Update,
        //     ((|| None::<Entity>)
        //         .pipe(theme_items)
        //         .run_if(resource_changed::<Theme>),),
        // )
        // .add_observer(on_add_item)
        // .add_observer(on_selection_change);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_insert=Self::on_insert)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Selection::default(),
    Items::default(),
    Name::new("ColorPaletteCombo"),
    RelativeCursorPosition::default()
)]
pub struct ColorPaletteCombo;

#[derive(Component, Debug, Copy, Clone)]
struct ColorPaletteComboButton;

impl ColorPaletteCombo {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let mut button_id = Entity::PLACEHOLDER;
        let mut color_palette_id = Entity::PLACEHOLDER;
        world.commands().entity(context.entity).with_children(|x| {
            button_id = x
                .spawn((
                    ColorPaletteComboButton,
                    PushButton::new("color", "\\/"),
                    NoDefaultHover,
                    // allow mouse events to pass through to parent
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
                ))
                .id();
            color_palette_id = x
                .spawn((
                    Float,
                    Node {
                        top: Val::Px(50.0),
                        position_type: PositionType::Absolute,
                        width: Val::Px(800.0),
                        height: Val::Px(200.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    ScrollArea::bundle(
                        move |parent| {
                            parent.spawn((
                                ViewOf(context.entity),
                                Pickable {
                                    should_block_lower: false,
                                    ..default()
                                },
                                ColorPalette,
                            ));
                        },
                        true,
                        false,
                    ),
                    RelativeCursorPosition::default(),
                ))
                .id();
            // x.spawn(Node::default()).with_children(|spawner| {
            //     color_palette_id = spawner
            //         .spawn((
            //             Float,
            //             Node {
            //                 position_type: PositionType::Absolute,
            //                 width: Val::Px(800.0),
            //                 height: Val::Px(200.0),
            //                 ..default()
            //             },
            //             Visibility::Hidden,
            //             ScrollArea::bundle(
            //                 move |parent| {
            //                     parent.spawn((
            //                         ViewOf(context.entity),
            //                         Pickable {
            //                             should_block_lower: false,
            //                             ..default()
            //                         },
            //                         ColorPalette,
            //                     ));
            //                 },
            //                 true,
            //                 false,
            //             ),
            //             RelativeCursorPosition::default(),
            //         ))
            //         .id();
            // });
        });

        world
            .commands()
            .entity(context.entity)
            .insert((
                State::Closed,
                SubRefs {
                    button_id,
                    color_palette_id,
                },
            ))
            .observe(handle_combo_button_mouse_events);
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct SubRefs {
    button_id: Entity,
    color_palette_id: Entity,
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
#[component(immutable, on_insert=Self::on_insert)]
pub enum State {
    Open,
    Closed,
}

impl State {
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            State::Open => State::Closed,
            State::Closed => State::Open,
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let (
            state,
            &SubRefs {
                color_palette_id, ..
            },
        ) = world
            .entity(context.entity)
            .components::<(&State, &SubRefs)>();

        let visibility = match state {
            State::Open => Visibility::Visible,
            State::Closed => Visibility::Hidden,
        };
        world.commands().entity(color_palette_id).insert(visibility);
    }
}

fn handle_combo_button_mouse_events(
    trigger: Trigger<MouseEvent>,
    combos: Query<(&SubRefs, &State), With<ColorPaletteCombo>>,
    mut combo_buttons: Query<
        (&mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let (SubRefs { button_id, .. }, combo_state) = combos.get(trigger.target()).unwrap();
    let (mut background_color, mut border_color) = combo_buttons.get_mut(*button_id).unwrap();

    match (trigger.event(), combo_state) {
        (MouseEvent::Pressed, _) => {
            commands
                .entity(trigger.target())
                .insert(combo_state.inverse());
        }
        (MouseEvent::Over, _) | (_, State::Open) => {
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor(theme.colors.border_highlight);
        }

        (MouseEvent::Out, State::Closed) => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor(theme.colors.border);
        }
        (MouseEvent::Released, State::Closed) => {}
    }
}
