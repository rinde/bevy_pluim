use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::color::Color;
use bevy::color::Srgba;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::name::Name;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::Changed;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::ecs::system::IntoSystem;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::Single;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::OnInsert;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
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
use itertools::Either;

use crate::MouseEvent;
use crate::color_palette::ColorPalette;
use crate::color_push_button;
use crate::color_push_button::ColorPushButton;
use crate::color_push_button::ColorSquare;
use crate::color_push_button::ColorSquareRef;
use crate::float::Float;
use crate::list::ListLayout;
use crate::push_button::NoDefaultHover;
use crate::scroll_area::ScrollArea;
use crate::select::Item;
use crate::select::ItemsRef;
use crate::select::Selection;
use crate::select::SelectionParam;
use crate::theme::Theme;

#[derive(Copy, Clone, Debug, Default)]
pub struct ColorPaletteComboPlugin;

impl Plugin for ColorPaletteComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_floats)
                    .run_if(resource_changed::<Theme>),
                close_combos_on_outside_click,
                on_selection_change,
            ),
        )
        .add_observer(on_add_items_ref);
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[component(on_add=Self::on_insert)]
#[require(
    Node {
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Selection::default(),
    Name::new("ColorPaletteCombo"),
    RelativeCursorPosition::default()
)]
pub struct ColorPaletteCombo;

#[derive(Component, Debug, Copy, Clone)]
struct ColorPaletteComboButton;

#[derive(Component, Debug, Copy, Clone)]
struct ColorPaletteComboFloat;

impl ColorPaletteCombo {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let mut button_id = Entity::PLACEHOLDER;
        let mut float_id = Entity::PLACEHOLDER;
        let list_layout = world.entity(context.entity).get::<ListLayout>().copied();

        let existing_items_ref = world.entity(context.entity).get::<ItemsRef>().copied();
        world.commands().entity(context.entity).with_children(|x| {
            button_id = x
                .spawn((
                    ColorPaletteComboButton,
                    ColorPushButton,
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
                    ColorPaletteComboFloat,
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
                            let bundle = (
                                Pickable {
                                    should_block_lower: false,
                                    ..default()
                                },
                                ColorPalette,
                                list_layout.unwrap_or(ListLayout::VerticalGrid { columns: 6 }),
                            );

                            if let Some(items_ref) = existing_items_ref {
                                parent.spawn((bundle, items_ref));
                            } else {
                                parent.spawn(bundle);
                            }
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
                    button: button_id,
                    float_id,
                },
            ))
            .observe(handle_combo_button_mouse_events);
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct SubRefs {
    button: Entity,
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
        let (state, &sub_refs) = world
            .entity(context.entity)
            .components::<(&State, &SubRefs)>();

        let visibility = match state {
            State::Open => Visibility::Visible,
            State::Closed => Visibility::Hidden,
        };
        world
            .commands()
            .entity(sub_refs.float_id)
            .insert(visibility);
        world
            .commands()
            .run_system_cached_with(position_float, sub_refs);
    }
}

fn position_float(
    sub_refs: In<SubRefs>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut dims: Query<(&mut Node, &GlobalTransform, &ComputedNode), With<ColorPaletteComboFloat>>,
    parent_dims: Query<
        &ComputedNode,
        (
            With<ColorPaletteComboButton>,
            Without<ColorPaletteComboFloat>,
        ),
    >,
) {
    const MARGIN: f32 = 10.0;
    const BUTTON_FLOAT_DIST: f32 = 3.0;

    let (mut node, global_transform, computed_node) = dims.get_mut(sub_refs.float_id).unwrap();
    let button_computed_node = parent_dims.get(sub_refs.button).unwrap();

    let button_size = button_computed_node.size() * button_computed_node.inverse_scale_factor();
    let size = computed_node.content_size() * computed_node.inverse_scale_factor();
    let translation = Vec2::new(
        global_transform.translation().x,
        global_transform.translation().y,
    ) * computed_node.inverse_scale_factor();
    let min_coord = translation - (size / 2.0);
    let max_coord = translation + (size / 2.0);

    let mut intended_x = -size.x / 2.0 + (button_size.x / 2.0);
    let mut intended_y = button_size.y + BUTTON_FLOAT_DIST;

    // set x position
    let Val::Px(current_x) = node.left else {
        panic!("Unexpected value type for node.left")
    };

    let delta_x = intended_x - current_x;
    let global_max_x = max_coord.x + delta_x;
    let global_min_x = min_coord.x + delta_x;

    if global_max_x + MARGIN > window.width() {
        let excess_x = global_max_x + MARGIN - window.width();
        intended_x -= excess_x;
    } else if global_min_x < MARGIN {
        let excess_x = MARGIN - global_min_x;
        intended_x += excess_x;
    }
    node.left = Val::Px(intended_x);

    // set y position
    let Val::Px(current_y) = node.top else {
        panic!("Unexpected value type for node.top")
    };

    let delta_y = intended_y - current_y;
    let global_max_y = max_coord.y + delta_y;

    if global_max_y + MARGIN > window.height() {
        intended_y = -size.y - BUTTON_FLOAT_DIST;
    }
    node.top = Val::Px(intended_y);
}

fn on_selection_change(
    q: Query<Entity, Changed<Selection>>,
    combos: Query<(Entity, &SubRefs), With<ColorPaletteCombo>>,
    color_square_refs: Query<&ColorSquareRef>,
    mut color_squares: Query<&mut BackgroundColor, With<ColorSquare>>,
    selections: SelectionParam,
    items: Query<&Item<Color>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (color_palette_combo_ref, sub_refs) in combos {
            let Some((selection_ref, selection)) = selections.get_with_ref(color_palette_combo_ref)
            else {
                continue;
            };
            if selection_ref != changed_selection_ref {
                continue;
            }

            // TODO make this less brittle
            let scroll_contents_ref = children
                .get(sub_refs.float_id)
                .unwrap()
                .iter()
                .next()
                .unwrap();
            let color_palette_ref = children
                .get(*scroll_contents_ref)
                .unwrap()
                .iter()
                .next()
                .unwrap();

            if selections
                .get_ref(*color_palette_ref)
                .is_none_or(|existing_ref| existing_ref != changed_selection_ref)
            {
                // Selection id has changed -> propagate to the List
                commands
                    .entity(*color_palette_ref)
                    .insert(Selection::reference(changed_selection_ref));
            }

            let ColorSquareRef(color_square_id) = color_square_refs.get(sub_refs.button).unwrap();
            let mut color_square_background = color_squares.get_mut(*color_square_id).unwrap();

            assert!(selection.items.len() <= 1, "Multi selection not supported");
            let color = if let Some(selected) = selection.items.first() {
                items.get(*selected).unwrap().data
            } else {
                Color::Srgba(Srgba::NONE)
            };
            color_square_background.0 = color;
            commands.run_system_cached_with(color_push_button::update_theme, Some(sub_refs.button));
        }
    }
}

fn on_add_items_ref(
    t: Trigger<OnInsert, ItemsRef>,
    combos: Query<(&ItemsRef, &SubRefs), With<ColorPaletteCombo>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    let Ok((ItemsRef(items_ref), SubRefs { float_id, .. })) = combos.get(t.target()) else {
        return;
    };

    // TODO make this less brittle
    let scroll_contents_ref = children.get(*float_id).unwrap().iter().next().unwrap();
    let color_palette_ref = children
        .get(*scroll_contents_ref)
        .unwrap()
        .iter()
        .next()
        .unwrap();

    // propagate the ItemsRef to the list
    commands
        .entity(*color_palette_ref)
        .insert(ItemsRef(*items_ref));
}

/// System that closes the combo when clicking outside of it.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<(Entity, &RelativeCursorPosition, &State, &SubRefs), With<ColorPaletteCombo>>,
    mut combo_buttons: Query<
        (&mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    selections: SelectionParam,
    floats: Query<&RelativeCursorPosition, With<ColorPaletteComboFloat>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if mouse_button.just_released(MouseButton::Left) {
        for (combo_id, rcp, &combo_state, subs) in combos.iter() {
            let selection = selections.get(combo_id).unwrap();
            if combo_state == State::Open
                && !rcp.mouse_over()
                && !(selection.multi && floats.get(subs.float_id).unwrap().mouse_over())
            {
                let (mut background_color, mut border_color) =
                    combo_buttons.get_mut(subs.button).unwrap();

                *background_color = BackgroundColor(theme.colors.background_primary);
                *border_color = BorderColor(theme.colors.border);
                commands.entity(combo_id).insert(State::Closed);
            }
        }
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
    let (
        SubRefs {
            button: button_id, ..
        },
        combo_state,
    ) = combos.get(trigger.target()).unwrap();
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

fn theme_floats(
    float_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut floats: Query<&mut BorderColor, With<ColorPaletteComboFloat>>,
) {
    let it = if let Some(float_id) = float_id.as_ref() {
        Either::Left([floats.get_mut(*float_id).unwrap()].into_iter())
    } else {
        Either::Right(floats.iter_mut())
    };

    for mut border_color in it {
        *border_color = BorderColor(theme.colors.border_highlight);
    }
}
