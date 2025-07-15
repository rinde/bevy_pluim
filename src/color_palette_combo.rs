use bevy::app::App;
use bevy::app::Plugin;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::Single;
use bevy::ecs::world::DeferredWorld;
use bevy::math::Vec2;
use bevy::picking::Pickable;
use bevy::render::view::Visibility;
use bevy::transform::components::GlobalTransform;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::ComputedNode;
use bevy::ui::FlexDirection;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::RelativeCursorPosition;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::utils::default;
use bevy::window::PrimaryWindow;
use bevy::window::Window;

use crate::MouseEvent;
use crate::color_palette::ColorPalette;
use crate::float::Float;
use crate::list::ListLayout;
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
        let mut float_id = Entity::PLACEHOLDER;
        let list_layout = world.entity(context.entity).get::<ListLayout>().copied();

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
            float_id = x
                .spawn((
                    Float,
                    Node {
                        left: Val::Px(0.0),
                        top: Val::Px(40.0),
                        position_type: PositionType::Absolute,
                        height: Val::Px(250.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
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
                                list_layout.unwrap_or(ListLayout::VerticalGrid { columns: 6 }),
                            ));
                        },
                        true,
                        false,
                    ),
                    RelativeCursorPosition::default(),
                ))
                .id();
        });

        world
            .commands()
            .entity(context.entity)
            .insert((
                State::Closed,
                SubRefs {
                    button_id,
                    float_id,
                },
            ))
            .observe(handle_combo_button_mouse_events);
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct SubRefs {
    button_id: Entity,
    float_id: Entity,
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
        let (state, &SubRefs { float_id, .. }) = world
            .entity(context.entity)
            .components::<(&State, &SubRefs)>();

        let visibility = match state {
            State::Open => Visibility::Visible,
            State::Closed => Visibility::Hidden,
        };
        world.commands().entity(float_id).insert(visibility);
        world
            .commands()
            .run_system_cached_with(position_float, float_id);
    }
}

fn position_float(
    float_id: In<Entity>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut dims: Query<(&mut Node, &ChildOf, &GlobalTransform, &ComputedNode), With<Float>>,
    parent_dims: Query<&ComputedNode, Without<Float>>,
) {
    const MARGIN: f32 = 10.0;
    const BUTTON_FLOAT_DIST: f32 = 3.0;

    let (mut node, ChildOf(parent), global_transform, computed_node) =
        dims.get_mut(*float_id).unwrap();
    let parent_computed_node = parent_dims.get(*parent).unwrap();

    let parent_size = parent_computed_node.size() * parent_computed_node.inverse_scale_factor();

    let size = computed_node.content_size() * computed_node.inverse_scale_factor();
    let translation = Vec2::new(
        global_transform.translation().x,
        global_transform.translation().y,
    ) * computed_node.inverse_scale_factor();
    let max_coord = translation + (size / 2.0);

    // set x position
    let Val::Px(current_x) = node.left else {
        panic!("Unexpected value type for node.left")
    };
    let mut intended_x = -size.x / 2.0 + (parent_size.x / 2.0);
    let delta_x = intended_x - current_x;
    let global_max_x = max_coord.x + delta_x;

    if global_max_x + MARGIN > window.width() {
        let excess_x = global_max_x + MARGIN - window.width();
        intended_x -= excess_x;
    }
    node.left = Val::Px(intended_x);

    // set y position
    let Val::Px(current_y) = node.top else {
        panic!("Unexpected value type for node.top")
    };
    let mut intended_y = parent_size.y + BUTTON_FLOAT_DIST;
    let delta_y = intended_y - current_y;
    let global_max_y = max_coord.y + delta_y;

    if global_max_y + MARGIN > window.height() {
        intended_y = -size.y - BUTTON_FLOAT_DIST;
    }
    node.top = Val::Px(intended_y);
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
